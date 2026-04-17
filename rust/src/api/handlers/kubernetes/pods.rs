//! Kubernetes Pods Handlers — /api/kubernetes/clusters/{cluster_id}/namespaces/{ns}/pods/...
//!
//! Включает:
//! - pod_exec   — WebSocket прокси к kubectl exec  (GET .../exec?command=...&container=...)
//! - pod_portforward — WebSocket прокси к port-forward (GET .../portforward?port=8080)
//!
//! list_pods / get_pod / delete_pod / pod_logs уже реализованы в workloads_k8s.rs

use crate::api::extractors::AuthUser;
use crate::api::state::AppState;
use axum::{
    Json,
    extract::{
        Path, Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, AttachParams};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// ──────────────────────────────────────────────────────────────────────────────
// GET /api/kubernetes/clusters/:cluster_id/namespaces/:namespace/pods/:name/exec
//
// Upgrades to WebSocket.
//   client → binary/text frames   → pod stdin
//   pod stdout                     → binary frames → client
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PodExecQuery {
    /// Shell command to run, space-separated (default: /bin/sh)
    pub command: Option<String>,
    /// Container name (optional, uses first container if omitted)
    pub container: Option<String>,
}

pub async fn pod_exec(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((_cluster_id, namespace, name)): Path<(String, String, String)>,
    Query(q): Query<PodExecQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let kube_client = match state.kubernetes_client() {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    let command: Vec<String> = q
        .command
        .unwrap_or_else(|| "/bin/sh".to_string())
        .split_whitespace()
        .map(String::from)
        .collect();

    ws.on_upgrade(move |socket| async move {
        let pods: Api<Pod> = Api::namespaced(kube_client.raw().clone(), &namespace);
        let ap = AttachParams {
            container: q.container,
            stdin: true,
            stdout: true,
            stderr: false,
            tty: true,
            ..AttachParams::default()
        };

        match pods.exec(&name, command, &ap).await {
            Ok(mut attached) => handle_exec_socket(socket, &mut attached).await,
            Err(e) => {
                let mut ws = socket;
                let _ = ws
                    .send(Message::Text(format!("{{\"error\":\"{}\"}}", e).into()))
                    .await;
                let _ = ws.close().await;
            }
        }
    })
}

async fn handle_exec_socket(socket: WebSocket, attached: &mut kube::api::AttachedProcess) {
    let (mut ws_tx, mut ws_rx) = socket.split();

    let mut pod_stdin = match attached.stdin() {
        Some(s) => s,
        None => {
            let _ = ws_tx
                .send(Message::Text("{\"error\":\"no stdin\"}".into()))
                .await;
            return;
        }
    };

    let mut pod_stdout = match attached.stdout() {
        Some(s) => s,
        None => {
            let _ = ws_tx
                .send(Message::Text("{\"error\":\"no stdout\"}".into()))
                .await;
            return;
        }
    };

    let stdin_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            match msg {
                Message::Binary(data) if pod_stdin.write_all(&data).await.is_err() => break,
                Message::Text(text) if pod_stdin.write_all(text.as_bytes()).await.is_err() => break,
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    let stdout_task = tokio::spawn(async move {
        let mut buf = vec![0u8; 4096];
        loop {
            match pod_stdout.read(&mut buf).await {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if ws_tx
                        .send(Message::Binary(buf[..n].to_vec().into()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = stdin_task => {}
        _ = stdout_task => {}
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// GET /api/kubernetes/clusters/:cluster_id/namespaces/:namespace/pods/:name/portforward
//
// Upgrades to WebSocket. Bidirectional TCP byte stream proxy.
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PodPortForwardQuery {
    /// Target port inside the pod (e.g. 8080)
    pub port: u16,
}

pub async fn pod_portforward(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((_cluster_id, namespace, name)): Path<(String, String, String)>,
    Query(q): Query<PodPortForwardQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    use tokio::time::{Duration, timeout};

    let kube_client = match state.kubernetes_client() {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    ws.on_upgrade(move |socket| async move {
        let pods: Api<Pod> = Api::namespaced(kube_client.raw().clone(), &namespace);

        // Connection timeout: 30 секунд
        let pf_result = timeout(Duration::from_secs(30), pods.portforward(&name, &[q.port])).await;

        match pf_result {
            Ok(Ok(mut pf)) => {
                match pf.take_stream(q.port) {
                    Some(stream) => {
                        // Session timeout: 10 минут для port-forward
                        let _ = timeout(
                            Duration::from_secs(600),
                            handle_portforward_socket(socket, stream),
                        )
                        .await;
                    }
                    None => {
                        let mut ws = socket;
                        let _ = ws
                            .send(Message::Text(
                                format!("{{\"error\":\"port {} not available\"}}", q.port).into(),
                            ))
                            .await;
                    }
                }
            }
            Ok(Err(e)) => {
                let mut ws = socket;
                let _ = ws
                    .send(Message::Text(format!("{{\"error\":\"{}\"}}", e).into()))
                    .await;
            }
            Err(_) => {
                let mut ws = socket;
                let _ = ws
                    .send(Message::Text(
                        "{\"error\":\"Connection timeout (30s)\"}".into(),
                    ))
                    .await;
            }
        }
    })
}

async fn handle_portforward_socket<S>(socket: WebSocket, stream: S)
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (mut pod_rx, mut pod_tx) = tokio::io::split(stream);

    let ws_to_pod = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            match msg {
                Message::Binary(data) if pod_tx.write_all(&data).await.is_err() => break,
                Message::Text(text) if pod_tx.write_all(text.as_bytes()).await.is_err() => break,
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    let pod_to_ws = tokio::spawn(async move {
        let mut buf = vec![0u8; 8192];
        loop {
            match pod_rx.read(&mut buf).await {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if ws_tx
                        .send(Message::Binary(buf[..n].to_vec().into()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = ws_to_pod => {}
        _ = pod_to_ws => {}
    }
}

// Re-export pod CRUD functions from workloads_k8s
pub use super::workloads_k8s::{PodLogsQuery, delete_pod, evict_pod, get_pod, list_pods, pod_logs};

#[cfg(test)]
mod tests {
    use super::*;

    // ── PodExecQuery deserialization ──

    #[test]
    fn test_pod_exec_query_default_command() {
        let json = r#"{}"#;
        let q: PodExecQuery = serde_json::from_str(json).unwrap();
        assert_eq!(q.command, None);
        assert_eq!(q.container, None);
    }

    #[test]
    fn test_pod_exec_query_with_command() {
        let json = r#"{"command": "/bin/bash -l", "container": "app"}"#;
        let q: PodExecQuery = serde_json::from_str(json).unwrap();
        assert_eq!(q.command, Some("/bin/bash -l".to_string()));
        assert_eq!(q.container, Some("app".to_string()));
    }

    #[test]
    fn test_pod_exec_query_command_splitting() {
        let json = r#"{"command": "ls -la /tmp"}"#;
        let q: PodExecQuery = serde_json::from_str(json).unwrap();
        let parts: Vec<String> = q
            .command
            .unwrap()
            .split_whitespace()
            .map(String::from)
            .collect();
        assert_eq!(parts, vec!["ls", "-la", "/tmp"]);
    }

    #[test]
    fn test_pod_exec_default_command_is_sh() {
        let q = PodExecQuery {
            command: None,
            container: None,
        };
        let default_cmd = q.command.unwrap_or_else(|| "/bin/sh".to_string());
        assert_eq!(default_cmd, "/bin/sh");
    }

    // ── PodPortForwardQuery deserialization ──

    #[test]
    fn test_pod_portforward_query_port_8080() {
        let json = r#"{"port": 8080}"#;
        let q: PodPortForwardQuery = serde_json::from_str(json).unwrap();
        assert_eq!(q.port, 8080);
    }

    #[test]
    fn test_pod_portforward_query_port_80() {
        let json = r#"{"port": 80}"#;
        let q: PodPortForwardQuery = serde_json::from_str(json).unwrap();
        assert_eq!(q.port, 80);
    }

    #[test]
    fn test_pod_portforward_query_port_range() {
        let json = r#"{"port": 3000}"#;
        let q: PodPortForwardQuery = serde_json::from_str(json).unwrap();
        assert_eq!(q.port, 3000);
        assert!(q.port > 0);
    }

    #[test]
    fn test_pod_portforward_query_max_port() {
        let json = r#"{"port": 65535}"#;
        let q: PodPortForwardQuery = serde_json::from_str(json).unwrap();
        assert_eq!(q.port, 65535);
    }

    #[test]
    fn test_pod_portforward_query_min_port() {
        let json = r#"{"port": 1}"#;
        let q: PodPortForwardQuery = serde_json::from_str(json).unwrap();
        assert_eq!(q.port, 1);
    }

    // ── PodLogsQuery (re-exported) ──

    #[test]
    fn test_pod_logs_query_with_container() {
        let json = r#"{"container": "sidecar", "tail_lines": 200, "previous": false}"#;
        let q: PodLogsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(q.container, Some("sidecar".to_string()));
        assert_eq!(q.tail_lines, Some(200));
        assert_eq!(q.previous, Some(false));
    }

    #[test]
    fn test_pod_logs_query_minimal() {
        let json = r#"{}"#;
        let q: PodLogsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(q.container, None);
        assert_eq!(q.tail_lines, None);
        assert_eq!(q.previous, None);
    }

    #[test]
    fn test_pod_logs_query_with_tail_lines() {
        let json = r#"{"tail_lines": 1000}"#;
        let q: PodLogsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(q.tail_lines, Some(1000));
    }

    // ── AttachParams configuration ──

    #[test]
    fn test_attach_params_defaults() {
        let ap = AttachParams {
            container: None,
            stdin: true,
            stdout: true,
            stderr: false,
            tty: true,
            ..AttachParams::default()
        };
        assert!(ap.stdin);
        assert!(ap.stdout);
        assert!(!ap.stderr);
        assert!(ap.tty);
        assert!(ap.container.is_none());
    }

    // ── Command parsing ──

    #[test]
    fn test_command_split_single_word() {
        let cmd = "/bin/sh";
        let parts: Vec<String> = cmd.split_whitespace().map(String::from).collect();
        assert_eq!(parts, vec!["/bin/sh"]);
    }

    #[test]
    fn test_command_split_multiple_words() {
        let cmd = "python -c 'print(1)'";
        let parts: Vec<String> = cmd.split_whitespace().map(String::from).collect();
        assert_eq!(parts, vec!["python", "-c", "'print(1)'"]);
    }

    #[test]
    fn test_command_empty_string() {
        let cmd = "";
        let parts: Vec<String> = cmd.split_whitespace().map(String::from).collect();
        assert!(parts.is_empty());
    }

    // ── PodPath tuple ──

    #[test]
    fn test_path_tuple_structure() {
        let path: (String, String, String) = (
            "cluster-1".to_string(),
            "default".to_string(),
            "my-pod".to_string(),
        );
        let (cluster_id, namespace, name) = path;
        assert_eq!(cluster_id, "cluster-1");
        assert_eq!(namespace, "default");
        assert_eq!(name, "my-pod");
    }

    // ── WebSocket Message types ──

    #[test]
    fn test_message_binary_variant() {
        let data = vec![1u8, 2, 3, 4];
        let msg = Message::Binary(data.clone().into());
        match msg {
            Message::Binary(bytes) => assert_eq!(bytes.to_vec(), data),
            _ => panic!("Expected Binary message"),
        }
    }

    #[test]
    fn test_message_text_variant() {
        let msg = Message::Text("hello".into());
        match msg {
            Message::Text(text) => assert_eq!(text, "hello"),
            _ => panic!("Expected Text message"),
        }
    }

    #[test]
    fn test_message_close_variant() {
        let msg = Message::Close(None);
        match msg {
            Message::Close(_) => {} // expected
            _ => panic!("Expected Close message"),
        }
    }
}
