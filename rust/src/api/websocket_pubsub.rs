//! WebSocket Pub/Sub via Redis
//!
//! Этот модуль предоставляет масштабируемую WebSocket инфраструктуру
//! с использованием Redis pub/sub для координации между несколькими инстансами.

use redis::Client;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::api::websocket::WsMessage;

/// Конфигурация Redis Pub/Sub
#[derive(Debug, Clone)]
pub struct RedisPubSubConfig {
    /// URL подключения к Redis
    pub redis_url: String,
    /// Канал для pub/sub
    pub channel: String,
    /// Включить pub/sub
    pub enabled: bool,
}

impl Default for RedisPubSubConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            channel: "velum:websocket".to_string(),
            enabled: false,
        }
    }
}

/// Redis Pub/Sub менеджер для WebSocket
pub struct WebSocketPubSub {
    config: RedisPubSubConfig,
    client: Option<Client>,
    broadcaster: broadcast::Sender<WsMessage>,
}

impl WebSocketPubSub {
    /// Создаёт новый WebSocket Pub/Sub
    pub fn new(config: RedisPubSubConfig) -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            config,
            client: None,
            broadcaster: tx,
        }
    }

    /// Инициализирует соединение с Redis
    pub fn initialize(&mut self) -> Result<(), String> {
        if !self.config.enabled {
            info!("Redis WebSocket Pub/Sub disabled");
            return Ok(());
        }

        match Client::open(self.config.redis_url.as_str()) {
            Ok(client) => {
                self.client = Some(client);
                info!(
                    "Redis WebSocket Pub/Sub initialized (channel: {})",
                    self.config.channel
                );
                Ok(())
            }
            Err(e) => {
                error!("Failed to connect to Redis for WebSocket Pub/Sub: {}", e);
                Err(format!("Redis connection failed: {}", e))
            }
        }
    }

    /// Публикует сообщение в Redis канал
    pub fn publish(&self, message: &WsMessage) -> Result<(), String> {
        if !self.config.enabled {
            // Fallback to local broadcast
            let _ = self.broadcaster.send(message.clone());
            return Ok(());
        }

        let json = serde_json::to_string(message).map_err(|e| e.to_string())?;

        if let Some(ref client) = self.client {
            let mut conn = client.get_connection().map_err(|e| e.to_string())?;

            redis::cmd("PUBLISH")
                .arg(&self.config.channel)
                .arg(&json)
                .query::<()>(&mut conn)
                .map_err(|e| e.to_string())?;

            debug!(
                "Published message to Redis channel: {}",
                self.config.channel
            );
        }

        Ok(())
    }

    /// Запускает слушателя Redis Pub/Sub (blocking)
    pub fn run_listener(self: Arc<Self>) {
        if !self.config.enabled {
            return;
        }

        info!("Starting Redis WebSocket Pub/Sub listener");

        std::thread::spawn(move || {
            loop {
                if let Some(ref client) = self.client {
                    match client.get_connection() {
                        Ok(mut conn) => {
                            info!(
                                "Connected to Redis for Pub/Sub (channel: {})",
                                self.config.channel
                            );

                            let mut pubsub = conn.as_pubsub();

                            match pubsub.subscribe(&self.config.channel) {
                                Ok(()) => {
                                    info!(
                                        "Successfully subscribed to Redis channel: {}",
                                        self.config.channel
                                    );

                                    // Слушаем сообщения
                                    loop {
                                        match pubsub.get_message() {
                                            Ok(msg) => {
                                                if let Ok(json) = msg.get_payload::<String>() {
                                                    match serde_json::from_str::<WsMessage>(&json) {
                                                        Ok(message) => {
                                                            let _ = self.broadcaster.send(message);
                                                        }
                                                        Err(e) => {
                                                            warn!(
                                                                "Failed to parse WebSocket message: {}",
                                                                e
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                error!(
                                                    "Failed to get Redis Pub/Sub message: {}",
                                                    e
                                                );
                                                break;
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to subscribe to Redis channel: {}", e);
                                    std::thread::sleep(std::time::Duration::from_secs(5));
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to connect to Redis for Pub/Sub: {}", e);
                            std::thread::sleep(std::time::Duration::from_secs(5));
                        }
                    }
                } else {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            }
        });
    }

    /// Отправляет сообщение через локальный broadcast
    pub fn broadcast(
        &self,
        message: WsMessage,
    ) -> Result<usize, broadcast::error::SendError<WsMessage>> {
        self.broadcaster.send(message)
    }

    /// Создаёт подписку на локальный broadcast
    pub fn subscribe_local(&self) -> broadcast::Receiver<WsMessage> {
        self.broadcaster.subscribe()
    }
}

impl Default for WebSocketPubSub {
    fn default() -> Self {
        Self::new(RedisPubSubConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_pubsub_config_default() {
        let config = RedisPubSubConfig::default();
        assert_eq!(config.redis_url, "redis://localhost:6379");
        assert_eq!(config.channel, "velum:websocket");
        assert!(!config.enabled);
    }

    #[test]
    fn test_redis_pubsub_config_custom() {
        let config = RedisPubSubConfig {
            redis_url: "redis://myhost:6379".to_string(),
            channel: "custom:channel".to_string(),
            enabled: true,
        };
        assert_eq!(config.redis_url, "redis://myhost:6379");
        assert_eq!(config.channel, "custom:channel");
        assert!(config.enabled);
    }

    #[test]
    fn test_redis_pubsub_config_debug() {
        let config = RedisPubSubConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("RedisPubSubConfig"));
        assert!(debug_str.contains("redis://localhost:6379"));
    }

    #[test]
    fn test_redis_pubsub_config_clone() {
        let config = RedisPubSubConfig {
            redis_url: "redis://clone".to_string(),
            channel: "ch".to_string(),
            enabled: true,
        };
        let cloned = config.clone();
        assert_eq!(cloned.redis_url, config.redis_url);
        assert_eq!(cloned.enabled, config.enabled);
    }

    #[test]
    fn test_websocket_pubsub_new() {
        let config = RedisPubSubConfig {
            redis_url: "redis://test".to_string(),
            channel: "test:ch".to_string(),
            enabled: false,
        };
        let pubsub = WebSocketPubSub::new(config);
        // client is None until initialized
        assert!(pubsub.client.is_none());
    }

    #[test]
    fn test_websocket_pubsub_default() {
        let pubsub = WebSocketPubSub::default();
        assert!(pubsub.client.is_none());
    }

    #[test]
    fn test_websocket_pubsub_initialize_disabled() {
        let config = RedisPubSubConfig {
            enabled: false,
            ..Default::default()
        };
        let mut pubsub = WebSocketPubSub::new(config);
        // Should succeed without Redis connection
        assert!(pubsub.initialize().is_ok());
    }

    #[test]
    fn test_websocket_pubsub_initialize_invalid_redis() {
        let config = RedisPubSubConfig {
            redis_url: "invalid://not-redis".to_string(),
            channel: "test:ch".to_string(),
            enabled: true,
        };
        let mut pubsub = WebSocketPubSub::new(config);
        // Should fail with invalid URL
        assert!(pubsub.initialize().is_err());
    }

    #[test]
    fn test_websocket_pubsub_publish_disabled() {
        let pubsub = WebSocketPubSub::default();
        let msg = crate::api::websocket::WsMessage::Ping;
        // Should succeed in disabled mode (falls back to local broadcast)
        assert!(pubsub.publish(&msg).is_ok());
    }

    #[test]
    fn test_websocket_pubsub_subscribe_local() {
        let pubsub = WebSocketPubSub::default();
        let receiver = pubsub.subscribe_local();
        // Should return a valid receiver
        assert!(!receiver.is_closed());
    }

    #[test]
    fn test_websocket_pubsub_broadcast() {
        let pubsub = WebSocketPubSub::default();
        // Create a receiver first to keep the channel open
        let mut _rx = pubsub.subscribe_local();
        let msg = crate::api::websocket::WsMessage::Ping;
        let result = pubsub.broadcast(msg);
        // Should succeed when at least one receiver exists
        assert!(result.is_ok());
    }

    #[test]
    fn test_redis_pubsub_config_url_variants() {
        let urls = vec![
            "redis://localhost:6379",
            "redis://:password@host:6379/0",
            "redis+sentinel://host1:26379,host2:26379/mymaster/0",
        ];
        for url in urls {
            let config = RedisPubSubConfig {
                redis_url: url.to_string(),
                ..Default::default()
            };
            assert_eq!(config.redis_url, url);
        }
    }
}
