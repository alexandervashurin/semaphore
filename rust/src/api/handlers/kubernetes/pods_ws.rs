//! Kubernetes Pod WebSocket handlers
//!
//! WebSocket streaming для логов и exec terminal

use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
};
use axum::extract::ws::Message;
use futures_util::{SinkExt, StreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, LogParams},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
                                        serde_json::json!({ "status": "disconnected" }).to_string().into(),
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
pub async fn pod_exec_ws(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path((namespace, pod_name)): Path<(String, String)>,
) -> Response {
    ws.on_upgrade(move |socket| async move {
        let mut socket = socket;
        let (mut sender, mut receiver) = socket.split();

        // Ожидаем сообщение exec
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(exec_msg) = serde_json::from_str::<ExecWsMessage>(&text) {
                        if let ExecWsMessage::Stdin { .. } = exec_msg {
                            // Отправляем подтверждение
                            let _ = sender
                                .send(Message::Text(
                                    serde_json::json!({
                                        "status": "connected",
                                        "pod": pod_name,
                                        "namespace": namespace
                                    })
                                    .to_string()
                                    .into(),
                                ))
                                .await;

                            // Запускаем exec сессию
                            if let Err(e) = run_exec_session(&state, &namespace, &pod_name, &mut receiver, &mut sender).await {
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

/// Сессия exec terminal
async fn run_exec_session<S1, S2>(
    state: &AppState,
    namespace: &str,
    pod_name: &str,
    receiver: &mut futures_util::stream::SplitStream<S1>,
    sender: &mut futures_util::stream::SplitSink<S2, Message>,
) -> Result<()>
where
    S1: futures_util::Stream<Item = std::result::Result<Message, axum::Error>> + Unpin,
    S2: futures_util::Sink<Message> + Unpin,
{
    // NOTE: Полноценная exec сессия требует использования kube::api::SubResourceApi
    // Это заглушка для демонстрации функциональности
    use tokio::time::{Duration, interval};

    let mut tick = interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            Some(msg) = receiver.next() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(exec_msg) = serde_json::from_str::<ExecWsMessage>(&text) {
                            match exec_msg {
                                ExecWsMessage::Stdin { data } => {
                                    // Отправляем stdin в pod (заглушка)
                                    tracing::debug!("Received stdin: {}", data);
                                }
                                ExecWsMessage::Resize { cols, rows } => {
                                    tracing::debug!("Terminal resized: {}x{}", cols, rows);
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Err(_) => break,
                    _ => {}
                }
            }
            _ = tick.tick() => {
                // Heartbeat
                if sender.send(Message::Ping(vec![].into())).await.is_err() {
                    break;
                }
            }
        }
    }

    Ok(())
}
