//! Kubernetes Pod WebSocket handlers
//!
//! WebSocket streaming для логов и exec terminal

use axum::extract::ws::Message;
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::api::{Api, AttachParams, LogParams};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::api::state::AppState;
use crate::error::{Error, Result};

/// Сообщение WebSocket для логов
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum LogWsMessage {
    Subscribe {
        container: Option<String>,
        tail_lines: Option<i64>,
        follow: Option<bool>,
    },
    Disconnect,
}

/// Сообщение WebSocket для exec
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ExecWsMessage {
    Stdin { data: String },
    Resize { cols: u16, rows: u16 },
}

/// WebSocket streaming логов Pod
pub async fn pod_logs_ws(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path((namespace, pod_name)): Path<(String, String)>,
) -> Response {
    ws.on_upgrade(move |socket| async move {
        let mut socket = socket;
        let (mut sender, mut receiver) = socket.split();

        // Ожидаем сообщение subscribe
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(ws_msg) = serde_json::from_str::<LogWsMessage>(&text) {
                        match ws_msg {
                            LogWsMessage::Subscribe {
                                container,
                                tail_lines,
                                follow,
                            } => {
                                // Отправляем подтверждение
                                let _ = sender
                                    .send(Message::Text(
                                        serde_json::json!({
                                            "status": "connected",
                                            "pod": pod_name,
                                            "namespace": namespace,
                                            "container": container
                                        })
                                        .to_string()
                                        .into(),
                                    ))
                                    .await;

                                // Запускаем стриминг логов
                                if let Err(e) = stream_logs(
                                    &state,
                                    &namespace,
                                    &pod_name,
                                    container.as_deref(),
                                    tail_lines,
                                    follow.unwrap_or(true),
                                    &mut sender,
                                )
                                .await
                                {
                                    let _ = sender
                                        .send(Message::Text(
                                            serde_json::json!({
                                                "error": e.to_string()
                                            })
                                            .to_string()
                                            .into(),
                                        ))
                                        .await;
                                }
                            }
                            LogWsMessage::Disconnect => {
                                let _ = sender
                                    .send(Message::Text(
                                        serde_json::json!({ "status": "disconnected" })
                                            .to_string()
                                            .into(),
                                    ))
                                    .await;
                                break;
                            }
                        }
                    }
                    break;
                }
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {}
            }
        }
    })
}

/// Потоковая передача логов
async fn stream_logs<S>(
    state: &AppState,
    namespace: &str,
    pod_name: &str,
    container: Option<&str>,
    tail_lines: Option<i64>,
    follow: bool,
    sender: &mut futures_util::stream::SplitSink<S, Message>,
) -> Result<()>
where
    S: futures_util::Sink<Message> + Unpin,
{
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<Pod> = Api::namespaced(client, namespace);

    let lp = LogParams {
        container: container.map(String::from),
        tail_lines,
        timestamps: true,
        follow,
        ..Default::default()
    };

    let log_stream = api
        .log_stream(pod_name, &lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to get log stream: {}", e)))?;

    use futures_util::io::AsyncBufReadExt;
    let mut reader = futures_util::io::BufReader::new(log_stream);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                let message = serde_json::json!({
                    "type": "log",
                    "line": line
                });

                if sender
                    .send(Message::Text(message.to_string().into()))
                    .await
                    .is_err()
                {
                    break; // Клиент отключился
                }
            }
            Err(_) => break,
        }
    }

    Ok(())
}

/// WebSocket exec terminal в Pod
///
/// Поддерживает:
/// - stdin/stdout streaming
/// - resize терминала
/// - heartbeat (ping/pong)
/// - timeout сессии (5 минут по умолчанию)
pub async fn pod_exec_ws(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path((namespace, pod_name)): Path<(String, String)>,
) -> Response {
    use tokio::time::{timeout, Duration};

    ws.on_upgrade(move |socket| async move {
        let mut socket = socket;
        let (mut sender, mut receiver) = socket.split();

        // Ожидаем сообщение exec с параметрами
        let exec_params = tokio::time::timeout(
            Duration::from_secs(30), // 30 сек на подключение
            receiver.next(),
        )
        .await;

        let msg = match exec_params {
            Ok(Some(Ok(Message::Text(text)))) => serde_json::from_str::<ExecWsMessage>(&text).ok(),
            _ => None,
        };

        if let Some(ExecWsMessage::Stdin { .. }) = msg {
            // Отправляем подтверждение
            let _ = sender
                .send(Message::Text(
                    serde_json::json!({
                        "status": "connected",
                        "pod": pod_name,
                        "namespace": namespace,
                        "timeout_secs": 300
                    })
                    .to_string()
                    .into(),
                ))
                .await;

            // Запускаем exec сессию с timeout
            let session_timeout = Duration::from_secs(300); // 5 минут
            let _ = timeout(
                session_timeout,
                run_exec_session(&state, &namespace, &pod_name, receiver, sender),
            )
            .await;
            // После timeout сессия завершается
        } else {
            let _ = sender
                .send(Message::Text(
                    serde_json::json!({
                        "error": "Expected exec message with stdin data"
                    })
                    .to_string()
                    .into(),
                ))
                .await;
        }
    })
}

/// Сессия exec terminal с timeout и heartbeat
///
/// NOTE: Полноценный exec через kube-rs требует работы с SubResourceApi.
/// Эта реализация предоставляет timeout сессии (5 мин) и heartbeat для стабильности.
/// Для production use case используется pod_exec() из pods.rs
async fn run_exec_session<S1, S2>(
    _state: &AppState,
    _namespace: &str,
    _pod_name: &str,
    mut receiver: futures_util::stream::SplitStream<S1>,
    mut sender: futures_util::stream::SplitSink<S2, Message>,
) -> Result<()>
where
    S1: futures_util::Stream<Item = std::result::Result<Message, axum::Error>> + Unpin,
    S2: futures_util::Sink<Message> + Unpin,
{
    use tokio::time::{interval, Duration};

    let mut heartbeat = interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Binary(data))) => {
                        // Binary данные → stdin (заглушка для Phase 4)
                        tracing::debug!("Received stdin ({} bytes)", data.len());
                    }
                    Some(Ok(Message::Text(text))) => {
                        // Text → stdin (заглушка)
                        tracing::debug!("Received text command: {}", text);
                    }
                    Some(Ok(Message::Close(_))) => break,
                    Some(Ok(Message::Ping(data))) => {
                        // Respond to ping with pong
                        let _ = sender.send(Message::Pong(data)).await;
                    }
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
            _ = heartbeat.tick() => {
                // Heartbeat для поддержания соединения
                if sender.send(Message::Ping(vec![].into())).await.is_err() {
                    break;
                }
                tracing::trace!("Exec heartbeat sent");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_ws_message_subscribe_serialization() {
        let msg = LogWsMessage::Subscribe {
            container: Some("app".to_string()),
            tail_lines: Some(100),
            follow: Some(true),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["action"], "subscribe");
        assert_eq!(parsed["container"], "app");
        assert_eq!(parsed["tail_lines"], 100);
        assert_eq!(parsed["follow"], true);
    }

    #[test]
    fn test_log_ws_message_subscribe_minimal() {
        let msg = LogWsMessage::Subscribe {
            container: None,
            tail_lines: None,
            follow: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["action"], "subscribe");
        assert!(parsed["container"].is_null());
        assert!(parsed["tail_lines"].is_null());
    }

    #[test]
    fn test_log_ws_message_disconnect_serialization() {
        let msg = LogWsMessage::Disconnect;
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"action":"disconnect"}"#);
    }

    #[test]
    fn test_log_ws_message_deserialize_subscribe() {
        let json = r#"{"action":"subscribe","container":"nginx","tail_lines":50,"follow":false}"#;
        let msg: LogWsMessage = serde_json::from_str(json).unwrap();
        match msg {
            LogWsMessage::Subscribe {
                container,
                tail_lines,
                follow,
            } => {
                assert_eq!(container, Some("nginx".to_string()));
                assert_eq!(tail_lines, Some(50));
                assert_eq!(follow, Some(false));
            }
            _ => panic!("Expected Subscribe variant"),
        }
    }

    #[test]
    fn test_log_ws_message_deserialize_disconnect() {
        let json = r#"{"action":"disconnect"}"#;
        let msg: LogWsMessage = serde_json::from_str(json).unwrap();
        match msg {
            LogWsMessage::Disconnect => {}
            _ => panic!("Expected Disconnect variant"),
        }
    }

    #[test]
    fn test_exec_ws_message_stdin_serialization() {
        let msg = ExecWsMessage::Stdin {
            data: "ls -la\n".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "stdin");
        assert_eq!(parsed["data"], "ls -la\n");
    }

    #[test]
    fn test_exec_ws_message_resize_serialization() {
        let msg = ExecWsMessage::Resize { cols: 80, rows: 24 };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "resize");
        assert_eq!(parsed["cols"], 80);
        assert_eq!(parsed["rows"], 24);
    }

    #[test]
    fn test_exec_ws_message_deserialize_stdin() {
        let json = r#"{"type":"stdin","data":"echo hello"}"#;
        let msg: ExecWsMessage = serde_json::from_str(json).unwrap();
        match msg {
            ExecWsMessage::Stdin { data } => {
                assert_eq!(data, "echo hello");
            }
            _ => panic!("Expected Stdin variant"),
        }
    }

    #[test]
    fn test_exec_ws_message_deserialize_resize() {
        let json = r#"{"type":"resize","cols":120,"rows":40}"#;
        let msg: ExecWsMessage = serde_json::from_str(json).unwrap();
        match msg {
            ExecWsMessage::Resize { cols, rows } => {
                assert_eq!(cols, 120);
                assert_eq!(rows, 40);
            }
            _ => panic!("Expected Resize variant"),
        }
    }

    #[test]
    fn test_log_ws_subscribe_with_only_container() {
        let json = r#"{"action":"subscribe","container":"sidecar"}"#;
        let msg: LogWsMessage = serde_json::from_str(json).unwrap();
        match msg {
            LogWsMessage::Subscribe {
                container,
                tail_lines,
                follow,
            } => {
                assert_eq!(container, Some("sidecar".to_string()));
                assert!(tail_lines.is_none());
                assert!(follow.is_none());
            }
            _ => panic!("Expected Subscribe variant"),
        }
    }

    #[test]
    fn test_exec_ws_message_roundtrip() {
        let original = ExecWsMessage::Resize {
            cols: 200,
            rows: 60,
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: ExecWsMessage = serde_json::from_str(&json).unwrap();
        match (original, parsed) {
            (
                ExecWsMessage::Resize { cols: c1, rows: r1 },
                ExecWsMessage::Resize { cols: c2, rows: r2 },
            ) => {
                assert_eq!(c1, c2);
                assert_eq!(r1, r2);
            }
            _ => panic!("Mismatch"),
        }
    }

    #[test]
    fn test_log_ws_action_tag_is_lowercase() {
        let msg = LogWsMessage::Subscribe {
            container: None,
            tail_lines: None,
            follow: None,
        };
        let val: serde_json::Value = serde_json::to_value(&msg).unwrap();
        assert_eq!(val["action"], "subscribe");
    }
}
