//! Kubernetes Events WebSocket Streaming
//!
//! Real-time стриминг событий Kubernetes через WebSocket

use axum::{
    extract::{
        Path, Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use k8s_openapi::api::core::v1::Event;
use kube::api::{Api, ListParams, WatchEvent, WatchParams};
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
        involved_object: Box<EventInvolvedObject>,
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

/// Обработчик WebSocket подключения для стриминга событий (namespace-scoped)
pub async fn events_websocket(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Query(query): Query<EventStreamQuery>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_event_stream(socket, state, false, query, Some(namespace)))
}

/// Обработчик WebSocket подключения для стриминга событий — кластерный (все namespace'ы, K-01)
pub async fn cluster_events_websocket(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(query): Query<EventStreamQuery>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_event_stream(socket, state, true, query, None))
}

/// Обработка потока событий (namespace или весь кластер).
async fn handle_event_stream(
    socket: WebSocket,
    state: Arc<AppState>,
    cluster_wide: bool,
    query: EventStreamQuery,
    namespace: Option<String>,
) {
    let (mut sender, mut receiver) = socket.split();

    let connected_label = if cluster_wide {
        "*".to_string()
    } else {
        namespace.clone().unwrap_or_default()
    };

    // Отправляем подтверждение подключения
    let connected_msg = EventStreamMessage::Connected {
        namespace: connected_label.clone(),
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

    // Создаем API для событий (namespaced или кластерный watch)
    let api: Api<Event> = if cluster_wide {
        Api::all(client.raw().clone())
    } else {
        let ns = namespace.as_deref().unwrap_or("");
        Api::namespaced(client.raw().clone(), ns)
    };

    // Настраиваем параметры watch
    let mut watch_params = ListParams::default().timeout(300); // 5 минут таймаут

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

    if cluster_wide {
        info!("Starting cluster-wide event watch (all namespaces)");
    } else {
        info!(
            "Starting event watch for namespace: {}",
            namespace.as_deref().unwrap_or("")
        );
    }

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
                        if cluster_wide {
                            info!("WebSocket connection closed (cluster-wide events)");
                        } else {
                            info!(
                                "WebSocket connection closed for namespace: {}",
                                namespace.as_deref().unwrap_or("")
                            );
                        }
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
                        if cluster_wide {
                            info!("Reconnecting cluster-wide event watch");
                        } else {
                            info!(
                                "Reconnecting event watch for namespace: {}",
                                namespace.as_deref().unwrap_or("")
                            );
                        }
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

    let stream = api
        .watch(&watch_params, "0")
        .await
        .map_err(|e| crate::error::Error::Kubernetes(format!("Failed to start watch: {}", e)))?;

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
        first_seen: event.first_timestamp.as_ref().map(|t| t.0.to_string()),
        last_seen: event.last_timestamp.as_ref().map(|t| t.0.to_string()),
        involved_object: Box::new(EventInvolvedObject {
            kind: involved.kind.clone()?,
            name: involved.name.clone()?,
            api_version: involved.api_version.clone(),
            uid: involved.uid.clone(),
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_stream_message_connected_serialization() {
        let msg = EventStreamMessage::Connected {
            namespace: "default".to_string(),
            count: 5,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"connected\""));
        assert!(json.contains("\"namespace\":\"default\""));
        assert!(json.contains("\"count\":5"));
    }

    #[test]
    fn test_event_stream_message_event_serialization() {
        let msg = EventStreamMessage::Event {
            name: "my-event".to_string(),
            namespace: "kube-system".to_string(),
            type_: "Warning".to_string(),
            reason: "FailedScheduling".to_string(),
            message: "No nodes available".to_string(),
            count: 3,
            first_seen: Some("2024-01-01T00:00:00Z".to_string()),
            last_seen: Some("2024-01-01T00:05:00Z".to_string()),
            involved_object: Box::new(EventInvolvedObject {
                kind: "Pod".to_string(),
                name: "my-pod".to_string(),
                api_version: Some("v1".to_string()),
                uid: Some("uid-123".to_string()),
            }),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["type"], "event");
        assert_eq!(parsed["name"], "my-event");
        assert_eq!(parsed["reason"], "FailedScheduling");
        assert_eq!(parsed["involved_object"]["kind"], "Pod");
    }

    #[test]
    fn test_event_stream_message_error_serialization() {
        let msg = EventStreamMessage::Error {
            message: "Connection failed".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"error\""));
        assert!(json.contains("Connection failed"));
    }

    #[test]
    fn test_event_stream_message_heartbeat_serialization() {
        let msg = EventStreamMessage::Heartbeat;
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"type":"heartbeat"}"#);
    }

    #[test]
    fn test_event_stream_message_deserialization() {
        let json = r#"{"type":"connected","namespace":"default","count":0}"#;
        let msg: EventStreamMessage = serde_json::from_str(json).unwrap();
        match msg {
            EventStreamMessage::Connected { namespace, count } => {
                assert_eq!(namespace, "default");
                assert_eq!(count, 0);
            }
            _ => panic!("Expected Connected variant"),
        }
    }

    #[test]
    fn test_event_involved_object_serialization() {
        let obj = EventInvolvedObject {
            kind: "Deployment".to_string(),
            name: "my-deploy".to_string(),
            api_version: Some("apps/v1".to_string()),
            uid: Some("uid-456".to_string()),
        };
        let json = serde_json::to_string(&obj).unwrap();
        let parsed: EventInvolvedObject = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.kind, "Deployment");
        assert_eq!(parsed.name, "my-deploy");
        assert_eq!(parsed.api_version, Some("apps/v1".to_string()));
    }

    #[test]
    fn test_event_stream_query_types_parsing() {
        let json = r#"{"namespace":"default","types":"Normal,Warning"}"#;
        let query: EventStreamQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.namespace, Some("default".to_string()));
        assert_eq!(query.types, Some("Normal,Warning".to_string()));
    }

    #[test]
    fn test_event_stream_query_optional_fields() {
        let json = r#"{}"#;
        let query: EventStreamQuery = serde_json::from_str(json).unwrap();
        assert!(query.namespace.is_none());
        assert!(query.types.is_none());
    }

    #[test]
    fn test_event_message_event_type_tag_correct() {
        let msg = EventStreamMessage::Event {
            name: "e1".to_string(),
            namespace: "ns".to_string(),
            type_: "Normal".to_string(),
            reason: "Scheduled".to_string(),
            message: "msg".to_string(),
            count: 1,
            first_seen: None,
            last_seen: None,
            involved_object: Box::new(EventInvolvedObject {
                kind: "Pod".to_string(),
                name: "p".to_string(),
                api_version: None,
                uid: None,
            }),
        };
        let val: serde_json::Value = serde_json::to_value(&msg).unwrap();
        assert_eq!(val["type"], "event");
    }

    #[test]
    fn test_event_message_error_type_tag_correct() {
        let msg = EventStreamMessage::Error {
            message: "err".to_string(),
        };
        let val: serde_json::Value = serde_json::to_value(&msg).unwrap();
        assert_eq!(val["type"], "error");
        assert_eq!(val["message"], "err");
    }

    #[test]
    fn test_event_stream_types_filter_parsing() {
        let types_str = "Normal,Warning";
        let selectors: Vec<String> = types_str
            .split(',')
            .map(|t| format!("type={}", t.trim()))
            .collect();
        assert_eq!(selectors.len(), 2);
        assert_eq!(selectors[0], "type=Normal");
        assert_eq!(selectors[1], "type=Warning");
    }

    #[test]
    fn test_event_involved_object_minimal() {
        let obj = EventInvolvedObject {
            kind: "Node".to_string(),
            name: "node-1".to_string(),
            api_version: None,
            uid: None,
        };
        let json = serde_json::to_string(&obj).unwrap();
        let parsed: EventInvolvedObject = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.kind, "Node");
        assert!(parsed.api_version.is_none());
    }
}
