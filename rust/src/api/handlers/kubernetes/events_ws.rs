//! Kubernetes Events WebSocket Streaming
//!
//! Real-time стриминг событий Kubernetes через WebSocket

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use k8s_openapi::api::core::v1::Event;
use kube::{
    api::{Api, ListParams, WatchEvent, WatchParams},
    Client,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::api::state::AppState;
use crate::error::Result;

/// Параметры WebSocket подключения
#[derive(Debug, Deserialize)]
pub struct EventStreamQuery {
    pub namespace: Option<String>,
    pub types: Option<String>, // фильтр по типам: "Normal,Warning"
}

/// Сообщение WebSocket для событий
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventStreamMessage {
    /// Новое событие
    Event {
        name: String,
        namespace: String,
        type_: String,
        reason: String,
        message: String,
        count: i32,
        first_seen: Option<String>,
        last_seen: Option<String>,
        involved_object: EventInvolvedObject,
    },
    /// Ошибка
    Error { message: String },
    /// Подтверждение подключения
    Connected { namespace: String, count: usize },
    /// Heartbeat для проверки соединения
    Heartbeat,
}

/// Краткая информация об объекте
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInvolvedObject {
    pub kind: String,
    pub name: String,
    pub api_version: Option<String>,
    pub uid: Option<String>,
}

/// Обработчик WebSocket подключения для стриминга событий
pub async fn events_websocket(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Query(query): Query<EventStreamQuery>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_event_stream(socket, state, namespace, query))
}

/// Обработка потока событий
async fn handle_event_stream(
    socket: WebSocket,
    state: Arc<AppState>,
    namespace: String,
    query: EventStreamQuery,
) {
    let (mut sender, mut receiver) = socket.split();

    // Отправляем подтверждение подключения
    let connected_msg = EventStreamMessage::Connected {
        namespace: namespace.clone(),
        count: 0,
    };

    if let Ok(json) = serde_json::to_string(&connected_msg) {
        if let Err(e) = sender.send(Message::Text(json.into())).await {
            warn!("Ошибка отправки сообщения подключения: {}", e);
            return;
        }
    }

    // Получаем Kubernetes клиент
    let client = match state.kubernetes_client() {
        Ok(c) => c,
        Err(e) => {
            let error_msg = EventStreamMessage::Error {
                message: format!("Failed to get Kubernetes client: {}", e),
            };
            if let Ok(json) = serde_json::to_string(&error_msg) {
                let _ = sender.send(Message::Text(json.into())).await;
            }
            return;
        }
    };

    // Создаем API для событий
    let api: Api<Event> = Api::namespaced(client.raw().clone(), &namespace);

    // Настраиваем параметры watch
    let mut watch_params = ListParams::default()
        .timeout(300); // 5 минут таймаут

    // Добавляем фильтр по типам если указан
    if let Some(types) = &query.types {
        let type_selectors: Vec<String> = types
            .split(',')
            .map(|t| format!("type={}", t.trim()))
            .collect();
        if !type_selectors.is_empty() {
            watch_params.field_selector = Some(type_selectors.join(","));
        }
    }

    info!("Starting event watch for namespace: {}", namespace);

    // Запускаем watch цикл
    loop {
        tokio::select! {
            // Обработка входящих сообщений от клиента (ping/pong/close)
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Ping(data))) => {
                        if let Err(e) = sender.send(Message::Pong(data)).await {
                            warn!("Ошибка отправки Pong: {}", e);
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!("WebSocket connection closed for namespace: {}", namespace);
                        break;
                    }
                    Some(Ok(Message::Text(text))) => {
                        // Обрабатываем текстовые сообщения (например, heartbeat запросы)
                        if text == "heartbeat" {
                            if let Ok(json) = serde_json::to_string(&EventStreamMessage::Heartbeat) {
                                let _ = sender.send(Message::Text(json.into())).await;
                            }
                        }
                    }
                    Some(Err(e)) => {
                        warn!("Ошибка получения сообщения: {}", e);
                        break;
                    }
                    _ => {}
                }
            }

            // Watch за событиями Kubernetes
            watch_result = watch_events(&api, &watch_params, &mut sender) => {
                match watch_result {
                    Ok(should_continue) => {
                        if !should_continue {
                            break;
                        }
                        // Переподключаемся после завершения watch (reconnect)
                        info!("Reconnecting event watch for namespace: {}", namespace);
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }
                    Err(e) => {
                        error!("Ошибка watch событий: {}", e);
                        let error_msg = EventStreamMessage::Error {
                            message: format!("Watch error: {}", e),
                        };
                        if let Ok(json) = serde_json::to_string(&error_msg) {
                            let _ = sender.send(Message::Text(json.into())).await;
                        }
                        // Пробуем переподключиться
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
            }
        }
    }
}

/// Watch за событиями Kubernetes
async fn watch_events(
    api: &Api<Event>,
    params: &ListParams,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> Result<bool> {
    // Конвертируем ListParams в WatchParams
    let watch_params = WatchParams {
        field_selector: params.field_selector.clone(),
        label_selector: params.label_selector.clone(),
        timeout: Some(300),
        bookmarks: true,
        send_initial_events: false,
    };

    let stream = api.watch(&watch_params, "0").await.map_err(|e| {
        crate::error::Error::Kubernetes(format!("Failed to start watch: {}", e))
    })?;

    tokio::pin!(stream);

    let mut event_count = 0;

    while let Some(result) = stream.next().await {
        match result {
            Ok(watch_event) => {
                match watch_event {
                    WatchEvent::Added(event)
                    | WatchEvent::Modified(event)
                    | WatchEvent::Deleted(event) => {
                        // Конвертируем в наш формат и отправляем
                        if let Some(msg) = convert_event(&event) {
                            if let Ok(json) = serde_json::to_string(&msg) {
                                if let Err(e) = sender.send(Message::Text(json.into())).await {
                                    warn!("Ошибка отправки события: {}", e);
                                    return Ok(false); // Закрываем соединение
                                }
                                event_count += 1;
                            }
                        }
                    }
                    WatchEvent::Bookmark(_) => {
                        // Игнорируем bookmarks
                    }
                    WatchEvent::Error(e) => {
                        error!("Kubernetes watch error: {:?}", e);
                        return Err(crate::error::Error::Kubernetes(format!(
                            "Watch error: {:?}",
                            e
                        )));
                    }
                }
            }
            Err(e) => {
                error!("Ошибка получения события: {}", e);
                return Err(crate::error::Error::Kubernetes(format!(
                    "Stream error: {}",
                    e
                )));
            }
        }
    }

    info!("Watch stream completed. Events sent: {}", event_count);
    Ok(event_count > 0) // Продолжаем если были события
}

/// Конвертируем Kubernetes Event в наш формат
fn convert_event(event: &Event) -> Option<EventStreamMessage> {
    let involved = &event.involved_object;

    Some(EventStreamMessage::Event {
        name: event.metadata.name.clone()?,
        namespace: event.metadata.namespace.clone()?,
        type_: event.type_.clone().unwrap_or_default(),
        reason: event.reason.clone().unwrap_or_default(),
        message: event.message.clone().unwrap_or_default(),
        count: event.count.unwrap_or(1),
        first_seen: event.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        last_seen: event.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        involved_object: EventInvolvedObject {
            kind: involved.kind.clone()?,
            name: involved.name.clone()?,
            api_version: involved.api_version.clone(),
            uid: involved.uid.clone(),
        },
    })
}
