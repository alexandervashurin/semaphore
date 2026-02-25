//! WebSocket модуль для real-time обновлений
//!
//! Предоставляет инфраструктуру для:
//! - Подключения WebSocket клиентов
//! - Трансляции логов задач в реальном времени
//! - Уведомлений об изменении статуса задач

use axum::{
    extract::State,
    response::IntoResponse,
    http::StatusCode,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use chrono::{DateTime, Utc};
use tracing::warn;

use crate::api::state::AppState;

/// Сообщение WebSocket
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum WsMessage {
    /// Лог задачи
    Log {
        task_id: i32,
        output: String,
        time: DateTime<Utc>,
    },
    /// Статус задачи
    Status {
        task_id: i32,
        status: String,
        time: DateTime<Utc>,
    },
    /// Ошибка
    Error {
        message: String,
    },
    /// Ping для проверки соединения
    Ping,
    /// Pong ответ
    Pong,
}

/// Менеджер WebSocket подключений
pub struct WebSocketManager {
    /// Канал для широковещательной рассылки сообщений
    broadcaster: broadcast::Sender<WsMessage>,
    /// Подключения по task_id
    #[allow(dead_code)]
    connections: Arc<RwLock<HashMap<i32, Vec<broadcast::Receiver<WsMessage>>>>>,
}

impl WebSocketManager {
    /// Создаёт новый менеджер подключений
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            broadcaster: tx,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Создаёт новый канал для подписки на обновления
    pub fn subscribe(&self) -> broadcast::Receiver<WsMessage> {
        self.broadcaster.subscribe()
    }

    /// Отправляет сообщение всем подключенным клиентам
    pub fn broadcast(&self, message: WsMessage) -> Result<usize, broadcast::error::SendError<WsMessage>> {
        self.broadcaster.send(message)
    }

    /// Отправляет лог задачи
    pub fn send_log(&self, task_id: i32, output: String, time: DateTime<Utc>) {
        let message = WsMessage::Log { task_id, output, time };
        if let Err(e) = self.broadcast(message) {
            warn!("Ошибка отправки лога через WebSocket: {}", e);
        }
    }

    /// Отправляет статус задачи
    pub fn send_status(&self, task_id: i32, status: String, time: DateTime<Utc>) {
        let message = WsMessage::Status { task_id, status, time };
        if let Err(e) = self.broadcast(message) {
            warn!("Ошибка отправки статуса через WebSocket: {}", e);
        }
    }

    /// Отправляет ошибку
    pub fn send_error(&self, message: String) {
        let ws_msg = WsMessage::Error { message };
        if let Err(e) = self.broadcast(ws_msg) {
            warn!("Ошибка отправки ошибки через WebSocket: {}", e);
        }
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Обработчик WebSocket подключений (заглушка)
/// 
/// TODO: Реализовать полноценный WebSocket на tokio-tungstenite
pub async fn websocket_handler(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Временная заглушка - возвращаем ошибку 501 Not Implemented
    (
        StatusCode::NOT_IMPLEMENTED,
        "WebSocket будет реализован в следующей версии"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_log_serialization() {
        let msg = WsMessage::Log {
            task_id: 1,
            output: "Test log".to_string(),
            time: Utc::now(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"log\""));
        assert!(json.contains("\"task_id\":1"));
    }

    #[test]
    fn test_ws_message_status_serialization() {
        let msg = WsMessage::Status {
            task_id: 2,
            status: "running".to_string(),
            time: Utc::now(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"status\""));
        assert!(json.contains("\"status\":\"running\""));
    }

    #[test]
    fn test_ws_message_error_serialization() {
        let msg = WsMessage::Error {
            message: "Test error".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"error\""));
        assert!(json.contains("\"message\":\"Test error\""));
    }

    #[test]
    fn test_websocket_manager_broadcast() {
        let manager = WebSocketManager::new();
        
        // Подписываемся
        let mut rx = manager.subscribe();
        
        // Отправляем сообщение
        manager.send_log(1, "Test".to_string(), Utc::now());
        
        // Получаем сообщение
        let msg = rx.try_recv();
        assert!(msg.is_ok());
        
        match msg.unwrap() {
            WsMessage::Log { task_id, output, .. } => {
                assert_eq!(task_id, 1);
                assert_eq!(output, "Test");
            }
            _ => panic!("Ожидалось сообщение Log"),
        }
    }

    #[test]
    fn test_websocket_manager_multiple_subscribers() {
        let manager = WebSocketManager::new();
        
        // Несколько подписчиков
        let mut rx1 = manager.subscribe();
        let mut rx2 = manager.subscribe();
        
        // Отправляем сообщение
        manager.send_status(2, "success".to_string(), Utc::now());
        
        // Оба получают сообщение
        let msg1 = rx1.try_recv();
        let msg2 = rx2.try_recv();
        
        assert!(msg1.is_ok());
        assert!(msg2.is_ok());
        
        match msg1.unwrap() {
            WsMessage::Status { status, .. } => {
                assert_eq!(status, "success");
            }
            _ => panic!("Ожидалось сообщение Status"),
        }
    }
}
