//! Kubernetes Pods Handlers — /api/kubernetes/clusters/{cluster_id}/namespaces/{ns}/pods/...
//!
//! Включает:
//! - pod_exec   — WebSocket прокси к kubectl exec  (GET .../exec?command=...&container=...)
//! - pod_portforward — WebSocket прокси к port-forward (GET .../portforward?port=8080)
//!
//! list_pods / get_pod / delete_pod / pod_logs уже реализованы в workloads_k8s.rs

use axum::{
    extract::{Path, Query, State, ws::{WebSocket, WebSocketUpgrade, Message}},
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use futures::{StreamExt, SinkExt};
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, AttachParams};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::api::extractors::AuthUser;
use crate::api::state::AppState;

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
        Err(e) => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error": e.to_string()}))).into_response(),
    };

    let command: Vec<String> = q.command
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
                let _ = ws.send(Message::Text(
                    format!("{{\"error\":\"{}\"}}", e).into()
                )).await;
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
            let _ = ws_tx.send(Message::Text("{\"error\":\"no stdin\"}".into())).await;
            return;
        }
    };

    let mut pod_stdout = match attached.stdout() {
        Some(s) => s,
        None => {
            let _ = ws_tx.send(Message::Text("{\"error\":\"no stdout\"}".into())).await;
            return;
        }
    };

    let stdin_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            match msg {
                Message::Binary(data) => {
                    if pod_stdin.write_all(&data).await.is_err() { break; }
                }
                Message::Text(text) => {
                    if pod_stdin.write_all(text.as_bytes()).await.is_err() { break; }
                }
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
                    if ws_tx.send(Message::Binary(buf[..n].to_vec().into())).await.is_err() {
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
    let kube_client = match state.kubernetes_client() {
        Ok(c) => c,
        Err(e) => return (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error": e.to_string()}))).into_response(),
    };

    ws.on_upgrade(move |socket| async move {
        let pods: Api<Pod> = Api::namespaced(kube_client.raw().clone(), &namespace);
        match pods.portforward(&name, &[q.port]).await {
            Ok(mut pf) => {
                match pf.take_stream(q.port) {
                    Some(stream) => handle_portforward_socket(socket, stream).await,
                    None => {
                        let mut ws = socket;
                        let _ = ws.send(Message::Text(
                            format!("{{\"error\":\"port {} not available\"}}", q.port).into()
                        )).await;
                    }
                }
            }
            Err(e) => {
                let mut ws = socket;
                let _ = ws.send(Message::Text(
                    format!("{{\"error\":\"{}\"}}", e).into()
                )).await;
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
                Message::Binary(data) => {
                    if pod_tx.write_all(&data).await.is_err() { break; }
                }
                Message::Text(text) => {
                    if pod_tx.write_all(text.as_bytes()).await.is_err() { break; }
                }
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
                    if ws_tx.send(Message::Binary(buf[..n].to_vec().into())).await.is_err() {
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
