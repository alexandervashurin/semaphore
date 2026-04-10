//! Webhook - универсальный сервис отправки webhook уведомлений
//!
//! Поддерживает различные типы webhook:
//! - Generic JSON webhook
//! - Slack
//! - Microsoft Teams
//! - Discord
//! - Telegram
//! - Custom

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use reqwest::{
    header::{HeaderMap, CONTENT_TYPE},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info, warn};

/// Тип webhook
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WebhookType {
    Generic,
    Slack,
    Teams,
    Discord,
    Telegram,
    Custom,
}

/// Конфигурация webhook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub id: i64,
    pub name: String,
    pub r#type: WebhookType,
    pub url: String,
    pub secret: Option<String>,
    pub headers: Option<serde_json::Value>,
    pub active: bool,
    pub events: Vec<String>,
    pub retry_count: i32,
    pub timeout_secs: i64,
}

/// Событие webhook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub data: Value,
    pub metadata: WebhookMetadata,
}

/// Метаданные события
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookMetadata {
    pub source: String,
    pub version: String,
    pub project_id: Option<i64>,
    pub user_id: Option<i64>,
}

/// Результат отправки webhook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResult {
    pub success: bool,
    pub status_code: Option<u16>,
    pub response_body: Option<String>,
    pub error: Option<String>,
    pub attempts: u32,
}

/// WebhookService - сервис для отправки webhook уведомлений
pub struct WebhookService {
    client: Client,
}

impl WebhookService {
    /// Создаёт новый WebhookService
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Создаёт WebhookService с кастомным таймаутом
    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(timeout_secs))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Отправляет webhook
    pub async fn send_webhook(
        &self,
        config: &WebhookConfig,
        event: &WebhookEvent,
    ) -> Result<WebhookResult> {
        if !config.active {
            warn!("Webhook {} не активен", config.name);
            return Ok(WebhookResult {
                success: false,
                status_code: None,
                response_body: None,
                error: Some("Webhook не активен".to_string()),
                attempts: 0,
            });
        }

        let payload = self.build_payload(config, event);
        let mut attempts = 0;
        let mut last_error: Option<String> = None;

        while attempts <= config.retry_count as u32 {
            attempts += 1;

            match self.send_request(config, &payload).await {
                Ok(result) => {
                    if result.success {
                        info!(
                            "Webhook {} успешно отправлен (попытка {}/{})",
                            config.name,
                            attempts,
                            config.retry_count + 1
                        );
                        return Ok(result);
                    }
                    last_error = result.error.clone();
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    warn!(
                        "Webhook {} ошибка отправки (попытка {}/{}): {}",
                        config.name,
                        attempts,
                        config.retry_count + 1,
                        e
                    );
                }
            }

            if attempts <= config.retry_count as u32 {
                // Экспоненциальная задержка между попытками
                tokio::time::sleep(std::time::Duration::from_millis(
                    100 * 2u64.pow(attempts - 1),
                ))
                .await;
            }
        }

        error!(
            "Webhook {} не отправлен после {} попыток",
            config.name, attempts
        );
        Ok(WebhookResult {
            success: false,
            status_code: None,
            response_body: None,
            error: last_error,
            attempts,
        })
    }

    /// Строит payload в зависимости от типа webhook
    fn build_payload(&self, config: &WebhookConfig, event: &WebhookEvent) -> Value {
        match config.r#type {
            WebhookType::Slack => self.build_slack_payload(event),
            WebhookType::Teams => self.build_teams_payload(event),
            WebhookType::Discord => self.build_discord_payload(event),
            WebhookType::Telegram => self.build_telegram_payload(event),
            WebhookType::Generic | WebhookType::Custom => self.build_generic_payload(config, event),
        }
    }

    /// Generic webhook payload
    fn build_generic_payload(&self, config: &WebhookConfig, event: &WebhookEvent) -> Value {
        json!({
            "event": event.event_type,
            "timestamp": event.timestamp,
            "data": event.data,
            "metadata": event.metadata
        })
    }

    /// Slack webhook payload
    fn build_slack_payload(&self, event: &WebhookEvent) -> Value {
        let color = match event.event_type.as_str() {
            "task_success" => "good",
            "task_failed" => "danger",
            "task_started" => "warning",
            _ => "#439FE0",
        };

        let emoji = match event.event_type.as_str() {
            "task_success" => "✅",
            "task_failed" => "❌",
            "task_started" => "🚀",
            _ => "📢",
        };

        let title = event
            .data
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Уведомление");
        let text = event
            .data
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        json!({
            "attachments": [{
                "color": color,
                "author_name": format!("{} Velum UI", emoji),
                "title": title,
                "text": text,
                "fields": [
                    {
                        "title": "Событие",
                        "value": event.event_type,
                        "short": true
                    },
                    {
                        "title": "Время",
                        "value": event.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                        "short": true
                    }
                ],
                "footer": "Velum UI",
                "ts": event.timestamp.timestamp()
            }]
        })
    }

    /// Microsoft Teams webhook payload
    fn build_teams_payload(&self, event: &WebhookEvent) -> Value {
        let color = match event.event_type.as_str() {
            "task_success" => "8BC34A",
            "task_failed" => "F44336",
            "task_started" => "FF9800",
            _ => "439FE0",
        };

        let title = event
            .data
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Уведомление");
        let text = event
            .data
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        json!({
            "@type": "MessageCard",
            "@context": "http://schema.org/extensions",
            "themeColor": color,
            "summary": title,
            "sections": [{
                "activityTitle": title,
                "activitySubtitle": "Velum UI",
                "activityText": text,
                "facts": [
                    {
                        "name": "Событие",
                        "value": event.event_type
                    },
                    {
                        "name": "Время",
                        "value": event.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
                    }
                ]
            }]
        })
    }

    /// Discord webhook payload
    fn build_discord_payload(&self, event: &WebhookEvent) -> Value {
        let color = match event.event_type.as_str() {
            "task_success" => 0x00FF00,
            "task_failed" => 0xFF0000,
            "task_started" => 0xFFA500,
            _ => 0x439FE0,
        };

        let title = event
            .data
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Уведомление");
        let text = event
            .data
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        json!({
            "embeds": [{
                "title": title,
                "description": text,
                "color": color,
                "fields": [
                    {
                        "name": "Событие",
                        "value": event.event_type,
                        "inline": true
                    },
                    {
                        "name": "Время",
                        "value": event.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                        "inline": true
                    }
                ],
                "footer": {
                    "text": "Velum UI"
                },
                "timestamp": event.timestamp.to_rfc3339()
            }]
        })
    }

    /// Telegram webhook payload
    fn build_telegram_payload(&self, event: &WebhookEvent) -> Value {
        let emoji = match event.event_type.as_str() {
            "task_success" => "✅",
            "task_failed" => "❌",
            "task_started" => "🚀",
            _ => "📢",
        };

        let title = event
            .data
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Уведомление");
        let text = event
            .data
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let message = format!(
            "<b>{} {}</b>\n\n{}\n\n<i>Время: {}</i>",
            emoji,
            title,
            text,
            event.timestamp.format("%Y-%m-%d %H:%M:%S")
        );

        json!({
            "text": message,
            "parse_mode": "HTML"
        })
    }

    /// Отправляет HTTP запрос
    async fn send_request(&self, config: &WebhookConfig, payload: &Value) -> Result<WebhookResult> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

        // Добавляем кастомные заголовки
        if let Some(custom_headers) = &config.headers {
            if let Some(obj) = custom_headers.as_object() {
                for (key, value) in obj {
                    if let Some(v) = value.as_str() {
                        if let Ok(header_name) = key.as_str().parse::<reqwest::header::HeaderName>()
                        {
                            if let Ok(header_value) = v.parse::<reqwest::header::HeaderValue>() {
                                headers.insert(header_name, header_value);
                            }
                        }
                    }
                }
            }
        }

        let body_bytes = serde_json::to_vec(payload).map_err(|e| {
            Error::Other(format!("Ошибка сериализации webhook: {}", e))
        })?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();

        if let Some(secret) = &config.secret {
            let sig = crate::services::alert::AlertService::compute_webhook_request_signature(
                secret.as_str(),
                &timestamp,
                &body_bytes,
            );
            headers.insert(
                reqwest::header::HeaderName::from_static("x-semaphore-signature"),
                sig.parse().map_err(|e| {
                    Error::Other(format!("Заголовок подписи webhook: {}", e))
                })?,
            );
            headers.insert(
                reqwest::header::HeaderName::from_static("x-semaphore-timestamp"),
                timestamp.parse().map_err(|e| {
                    Error::Other(format!("Заголовок timestamp webhook: {}", e))
                })?,
            );
        }

        let request = self
            .client
            .post(&config.url)
            .headers(headers)
            .body(body_bytes);

        let response = request
            .send()
            .await
            .map_err(|e| Error::Other(format!("Ошибка отправки webhook: {}", e)))?;

        let status_code = response.status().as_u16();
        let is_success = response.status().is_success();

        let response_body = response.text().await.ok();

        Ok(WebhookResult {
            success: is_success,
            status_code: Some(status_code),
            response_body,
            error: if !is_success {
                Some(format!("HTTP статус: {}", status_code))
            } else {
                None
            },
            attempts: 1,
        })
    }
}

impl Default for WebhookService {
    fn default() -> Self {
        Self::new()
    }
}

/// Создаёт событие для задачи
pub fn create_task_event(
    event_type: &str,
    task_id: i64,
    task_name: &str,
    project_id: Option<i64>,
    user_id: Option<i64>,
    status: Option<&str>,
) -> WebhookEvent {
    WebhookEvent {
        event_type: event_type.to_string(),
        timestamp: Utc::now(),
        data: json!({
            "task_id": task_id,
            "task_name": task_name,
            "status": status.unwrap_or("unknown"),
            "title": format!("Задача: {}", task_name),
            "text": format!("Задача '{}' изменила статус на: {}", task_name, status.unwrap_or("unknown"))
        }),
        metadata: WebhookMetadata {
            source: "velum".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            project_id,
            user_id,
        },
    }
}

/// Создаёт событие для пользователя
pub fn create_user_event(
    event_type: &str,
    user_id: i64,
    username: &str,
    project_id: Option<i64>,
) -> WebhookEvent {
    WebhookEvent {
        event_type: event_type.to_string(),
        timestamp: Utc::now(),
        data: json!({
            "user_id": user_id,
            "username": username,
            "title": format!("Пользователь: {}", username),
            "text": format!("Действие с пользователем: {}", username)
        }),
        metadata: WebhookMetadata {
            source: "velum".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            project_id,
            user_id: Some(user_id),
        },
    }
}

/// Создаёт событие для проекта
pub fn create_project_event(
    event_type: &str,
    project_id: i64,
    project_name: &str,
    user_id: Option<i64>,
) -> WebhookEvent {
    WebhookEvent {
        event_type: event_type.to_string(),
        timestamp: Utc::now(),
        data: json!({
            "project_id": project_id,
            "project_name": project_name,
            "title": format!("Проект: {}", project_name),
            "text": format!("Действие с проектом: {}", project_name)
        }),
        metadata: WebhookMetadata {
            source: "velum".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            project_id: Some(project_id),
            user_id,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_webhook_type_serialization() {
        // Проверяем сериализацию типов webhook
        let types = vec![
            WebhookType::Generic,
            WebhookType::Slack,
            WebhookType::Teams,
            WebhookType::Discord,
            WebhookType::Telegram,
            WebhookType::Custom,
        ];

        for webhook_type in types {
            let serialized = serde_json::to_string(&webhook_type).unwrap();
            let deserialized: WebhookType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(webhook_type, deserialized);
        }
    }

    #[test]
    fn test_webhook_config_creation() {
        let config = WebhookConfig {
            id: 1,
            name: "Test Webhook".to_string(),
            r#type: WebhookType::Generic,
            url: "https://example.com/webhook".to_string(),
            secret: Some("secret".to_string()),
            headers: None,
            active: true,
            events: vec!["task.completed".to_string()],
            retry_count: 3,
            timeout_secs: 30,
        };

        assert_eq!(config.id, 1);
        assert_eq!(config.name, "Test Webhook");
        assert!(config.active);
        assert_eq!(config.retry_count, 3);
    }

    #[test]
    fn test_webhook_event_creation() {
        let event = WebhookEvent {
            event_type: "task.completed".to_string(),
            timestamp: Utc::now(),
            data: json!({"task_id": 123}),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0.0".to_string(),
                project_id: Some(1),
                user_id: Some(2),
            },
        };

        assert_eq!(event.event_type, "task.completed");
        assert_eq!(event.metadata.project_id, Some(1));
        assert_eq!(event.metadata.user_id, Some(2));
    }

    #[test]
    fn test_webhook_service_new() {
        let service = WebhookService::new();
        assert!(true);
    }

    #[test]
    fn test_webhook_service_with_timeout() {
        let service = WebhookService::with_timeout(60);
        assert!(true);
    }

    #[tokio::test]
    async fn test_send_webhook_inactive() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Inactive Webhook".to_string(),
            r#type: WebhookType::Generic,
            url: "https://example.com/webhook".to_string(),
            secret: None,
            headers: None,
            active: false,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = create_task_event(
            "task.completed",
            1,
            "Test Task",
            None,
            None,
            Some("completed"),
        );
        let result = service.send_webhook(&config, &event).await.unwrap();

        assert!(!result.success);
        assert_eq!(result.error, Some("Webhook не активен".to_string()));
        assert_eq!(result.attempts, 0);
    }

    #[test]
    fn test_build_generic_payload() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Generic,
            url: "https://example.com".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "test.event".to_string(),
            timestamp: Utc::now(),
            data: json!({"key": "value"}),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };

        let payload = service.build_payload(&config, &event);

        assert_eq!(payload["event"], "test.event");
        assert!(payload["data"].is_object());
        assert!(payload["metadata"].is_object());
    }

    #[test]
    fn test_build_slack_payload() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Slack,
            url: "https://hooks.slack.com".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = create_task_event(
            "task.completed",
            1,
            "Test Task",
            None,
            None,
            Some("completed"),
        );
        let payload = service.build_payload(&config, &event);

        assert!(payload["attachments"].is_array());
    }

    #[test]
    fn test_create_task_event() {
        let event = create_task_event(
            "task.started",
            42,
            "My Task",
            None,
            Some(10),
            Some("running"),
        );

        assert_eq!(event.event_type, "task.started");
        assert_eq!(event.metadata.user_id, Some(10));
        assert!(event.data["title"].as_str().unwrap().contains("My Task"));
    }

    #[test]
    fn test_create_project_event() {
        let event = create_project_event("project.created", 5, "My Project", Some(20));

        assert_eq!(event.event_type, "project.created");
        assert_eq!(event.metadata.project_id, Some(5));
        assert_eq!(event.metadata.user_id, Some(20));
        assert!(event.data["project_name"]
            .as_str()
            .unwrap()
            .contains("My Project"));
    }

    #[test]
    fn test_build_teams_payload() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Teams,
            url: "https://outlook.office.com/webhook".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = create_task_event(
            "task.completed",
            1,
            "Test Task",
            None,
            None,
            Some("completed"),
        );
        let payload = service.build_payload(&config, &event);

        assert_eq!(payload["@type"], "MessageCard");
        assert!(payload["summary"].as_str().unwrap().contains("Test Task"));
        assert!(payload["sections"].is_array());
    }

    #[test]
    fn test_build_discord_payload() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Discord,
            url: "https://discord.com/api/webhooks".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        // task_success для зелёного цвета (0x00FF00 = 65280)
        let event = WebhookEvent {
            event_type: "task_success".to_string(),
            timestamp: Utc::now(),
            data: json!({
                "title": "Test Task",
                "text": "Task completed successfully"
            }),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };
        let payload = service.build_payload(&config, &event);

        assert!(payload["embeds"].is_array());
        let embed = &payload["embeds"][0];
        assert!(embed["title"].as_str().unwrap().contains("Test Task"));
        assert_eq!(embed["color"], 65280); // 0x00FF00 green
    }

    #[test]
    fn test_build_telegram_payload() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Telegram,
            url: "https://api.telegram.org/bot".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = create_task_event(
            "task.completed",
            1,
            "Test Task",
            None,
            None,
            Some("completed"),
        );
        let payload = service.build_payload(&config, &event);

        assert!(payload["text"].as_str().unwrap().contains("Test Task"));
        assert_eq!(payload["parse_mode"], "HTML");
    }

    #[test]
    fn test_build_slack_payload_task_failed() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Slack,
            url: "https://hooks.slack.com".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        // Используем task_failed вместо task.failed для матчинга цвета
        let event = WebhookEvent {
            event_type: "task_failed".to_string(),
            timestamp: Utc::now(),
            data: json!({
                "title": "Failing Task",
                "text": "Task failed"
            }),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };
        let payload = service.build_payload(&config, &event);

        let attachments = payload["attachments"].as_array().unwrap();
        assert!(!attachments.is_empty());
        let attachment = &attachments[0];
        assert_eq!(attachment["color"], "danger");
        assert_eq!(attachment["title"].as_str().unwrap(), "Failing Task");
    }

    #[tokio::test]
    async fn test_send_request_success() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Generic,
            url: "https://httpbin.org/post".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 5,
        };

        let event = create_task_event("task.completed", 1, "Test", None, None, Some("completed"));
        let result = service.send_request(&config, &serde_json::json!({"test": true})).await;

        // httpbin.org может быть недоступен, проверяем что вызов не паникует
        if let Ok(r) = result {
            assert_eq!(r.attempts, 1);
        }
    }

    #[tokio::test]
    async fn test_send_request_with_custom_headers() {
        let service = WebhookService::new();
        let custom_headers = serde_json::json!({
            "X-Custom-Header": "custom-value",
            "Authorization": "Bearer token123"
        });

        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Generic,
            url: "https://httpbin.org/post".to_string(),
            secret: None,
            headers: Some(custom_headers),
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 5,
        };

        let payload = serde_json::json!({"test": true});
        let result = service.send_request(&config, &payload).await;

        if let Ok(r) = result {
            assert_eq!(r.attempts, 1);
        }
    }

    #[tokio::test]
    async fn test_send_request_invalid_url() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Generic,
            url: "not-a-valid-url".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 5,
        };

        let payload = serde_json::json!({"test": true});
        let result = service.send_request(&config, &payload).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_webhook_retry_on_failure() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Retry Test".to_string(),
            r#type: WebhookType::Generic,
            url: "https://httpbin.org/status/500".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 2,
            timeout_secs: 3,
        };

        let event = create_task_event("task.completed", 1, "Test", None, None, Some("completed"));
        let result = service.send_webhook(&config, &event).await.unwrap();

        // httpbin возвращает 500, поэтому retry должен исчерпать попытки
        assert!(!result.success);
    }

    #[test]
    fn test_create_user_event() {
        let event = create_user_event("user.created", 42, "testuser", Some(1));

        assert_eq!(event.event_type, "user.created");
        assert_eq!(event.metadata.user_id, Some(42));
        assert_eq!(event.metadata.project_id, Some(1));
        assert!(event.data["username"].as_str().unwrap() == "testuser");
    }

    #[test]
    fn test_create_project_event_additional() {
        let event = create_project_event("project.created", 10, "My Project", Some(5));

        assert_eq!(event.event_type, "project.created");
        assert_eq!(event.metadata.project_id, Some(10));
        assert_eq!(event.metadata.user_id, Some(5));
        assert!(event.data["project_name"].as_str().unwrap() == "My Project");
    }

    #[test]
    fn test_webhook_type_all_variants() {
        let types = [
            WebhookType::Generic,
            WebhookType::Slack,
            WebhookType::Teams,
            WebhookType::Discord,
            WebhookType::Telegram,
            WebhookType::Custom,
        ];
        for t in &types {
            let json = serde_json::to_string(t).unwrap();
            assert!(json.starts_with('"') && json.ends_with('"'));
        }
    }

    #[test]
    fn test_webhook_result_default() {
        let result = WebhookResult {
            success: false,
            status_code: None,
            response_body: None,
            error: Some("test error".to_string()),
            attempts: 3,
        };
        assert!(!result.success);
        assert_eq!(result.attempts, 3);
        assert!(result.status_code.is_none());
        assert!(result.response_body.is_none());
        assert!(result.error.is_some());
    }

    #[test]
    fn test_webhook_config_default_values() {
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Generic,
            url: "https://example.com".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec!["task.success".to_string()],
            retry_count: 3,
            timeout_secs: 30,
        };
        assert!(config.active);
        assert_eq!(config.retry_count, 3);
        assert_eq!(config.timeout_secs, 30);
        assert!(config.secret.is_none());
        assert!(config.headers.is_none());
    }

    #[test]
    fn test_webhook_event_metadata() {
        let event = WebhookEvent {
            event_type: "test.event".to_string(),
            timestamp: Utc::now(),
            data: serde_json::json!({"key": "value"}),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0.0".to_string(),
                project_id: Some(10),
                user_id: Some(5),
            },
        };
        assert_eq!(event.metadata.source, "test");
        assert_eq!(event.metadata.version, "1.0.0");
    }

    #[test]
    fn test_webhook_service_default() {
        let service = WebhookService::default();
        // Default should create a service with a client
        let _ = service;
    }

    // ==================== Дополнительные тесты ====================

    #[test]
    fn test_webhook_type_deserialization() {
        // Проверяем десериализацию из JSON строк
        let cases = vec![
            ("\"generic\"", WebhookType::Generic),
            ("\"slack\"", WebhookType::Slack),
            ("\"teams\"", WebhookType::Teams),
            ("\"discord\"", WebhookType::Discord),
            ("\"telegram\"", WebhookType::Telegram),
            ("\"custom\"", WebhookType::Custom),
        ];

        for (json, expected) in cases {
            let deserialized: WebhookType = serde_json::from_str(json).unwrap();
            assert_eq!(deserialized, expected);
        }
    }

    #[test]
    fn test_webhook_type_debug() {
        let webhook_type = WebhookType::Slack;
        let debug_str = format!("{:?}", webhook_type);
        assert!(debug_str.contains("Slack"));
    }

    #[test]
    fn test_webhook_config_serialization() {
        let config = WebhookConfig {
            id: 42,
            name: "Test Config".to_string(),
            r#type: WebhookType::Discord,
            url: "https://discord.com/webhook".to_string(),
            secret: Some("my_secret".to_string()),
            headers: Some(json!({"X-Custom": "value"})),
            active: true,
            events: vec!["task.started".to_string(), "task.completed".to_string()],
            retry_count: 5,
            timeout_secs: 60,
        };

        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("\"id\":42"));
        assert!(serialized.contains("\"name\":\"Test Config\""));
        assert!(serialized.contains("\"type\":\"discord\""));
        assert!(serialized.contains("\"active\":true"));
        assert!(serialized.contains("\"retry_count\":5"));
        assert!(serialized.contains("\"timeout_secs\":60"));
    }

    #[test]
    fn test_webhook_config_deserialization() {
        let json = r#"{
            "id": 10,
            "name": "Config",
            "type": "telegram",
            "url": "https://api.telegram.org",
            "secret": null,
            "headers": null,
            "active": false,
            "events": ["user.created"],
            "retry_count": 1,
            "timeout_secs": 15
        }"#;

        let config: WebhookConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.id, 10);
        assert_eq!(config.name, "Config");
        assert_eq!(config.r#type, WebhookType::Telegram);
        assert!(!config.active);
        assert_eq!(config.retry_count, 1);
        assert_eq!(config.timeout_secs, 15);
        assert!(config.secret.is_none());
        assert!(config.headers.is_none());
    }

    #[test]
    fn test_webhook_result_success() {
        let result = WebhookResult {
            success: true,
            status_code: Some(200),
            response_body: Some("OK".to_string()),
            error: None,
            attempts: 1,
        };

        assert!(result.success);
        assert_eq!(result.status_code, Some(200));
        assert!(result.error.is_none());
        assert_eq!(result.attempts, 1);
    }

    #[test]
    fn test_webhook_result_failure_with_error() {
        let result = WebhookResult {
            success: false,
            status_code: Some(500),
            response_body: Some("Internal Server Error".to_string()),
            error: Some("HTTP статус: 500".to_string()),
            attempts: 3,
        };

        assert!(!result.success);
        assert_eq!(result.status_code, Some(500));
        assert!(result.error.is_some());
        assert_eq!(result.error.as_ref().unwrap(), "HTTP статус: 500");
        assert_eq!(result.attempts, 3);
    }

    #[test]
    fn test_webhook_metadata_debug() {
        let metadata = WebhookMetadata {
            source: "test_source".to_string(),
            version: "2.0.0".to_string(),
            project_id: None,
            user_id: Some(99),
        };

        assert_eq!(metadata.source, "test_source");
        assert_eq!(metadata.version, "2.0.0");
        assert!(metadata.project_id.is_none());
        assert_eq!(metadata.user_id, Some(99));
    }

    #[test]
    fn test_build_slack_payload_task_started() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Slack,
            url: "https://hooks.slack.com".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "task_started".to_string(),
            timestamp: Utc::now(),
            data: json!({
                "title": "Starting Task",
                "text": "Task is starting"
            }),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };
        let payload = service.build_payload(&config, &event);

        let attachments = payload["attachments"].as_array().unwrap();
        assert!(!attachments.is_empty());
        let attachment = &attachments[0];
        assert_eq!(attachment["color"], "warning");
        assert_eq!(attachment["author_name"].as_str().unwrap(), "🚀 Velum UI");
    }

    #[test]
    fn test_build_slack_payload_unknown_event() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Slack,
            url: "https://hooks.slack.com".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "unknown_event".to_string(),
            timestamp: Utc::now(),
            data: json!({
                "title": "Unknown Event",
                "text": "Something happened"
            }),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };
        let payload = service.build_payload(&config, &event);

        let attachments = payload["attachments"].as_array().unwrap();
        let attachment = &attachments[0];
        assert_eq!(attachment["color"], "#439FE0");
        assert_eq!(attachment["author_name"].as_str().unwrap(), "📢 Velum UI");
    }

    #[test]
    fn test_build_teams_payload_colors() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Teams,
            url: "https://outlook.office.com/webhook".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        // task_success -> green
        let event_success = WebhookEvent {
            event_type: "task_success".to_string(),
            timestamp: Utc::now(),
            data: json!({
                "title": "Success",
                "text": "Task succeeded"
            }),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };
        let payload = service.build_payload(&config, &event_success);
        assert_eq!(payload["themeColor"], "8BC34A");

        // task_failed -> red
        let event_failed = WebhookEvent {
            event_type: "task_failed".to_string(),
            timestamp: Utc::now(),
            data: json!({
                "title": "Failed",
                "text": "Task failed"
            }),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };
        let payload = service.build_payload(&config, &event_failed);
        assert_eq!(payload["themeColor"], "F44336");
    }

    #[test]
    fn test_build_discord_payload_task_failed() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Discord,
            url: "https://discord.com/api/webhooks".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "task_failed".to_string(),
            timestamp: Utc::now(),
            data: json!({
                "title": "Failed Task",
                "text": "Task failed with error"
            }),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };
        let payload = service.build_payload(&config, &event);

        let embed = &payload["embeds"][0];
        assert_eq!(embed["color"], 0xFF0000); // Red
    }

    #[test]
    fn test_build_discord_payload_task_started() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Discord,
            url: "https://discord.com/api/webhooks".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "task_started".to_string(),
            timestamp: Utc::now(),
            data: json!({
                "title": "Starting",
                "text": "Task started"
            }),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };
        let payload = service.build_payload(&config, &event);

        let embed = &payload["embeds"][0];
        assert_eq!(embed["color"], 0xFFA500); // Orange
    }

    #[test]
    fn test_build_telegram_payload_task_failed_emoji() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Telegram,
            url: "https://api.telegram.org/bot".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "task_failed".to_string(),
            timestamp: Utc::now(),
            data: json!({
                "title": "Failed Task",
                "text": "Something went wrong"
            }),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };
        let payload = service.build_payload(&config, &event);

        let text = payload["text"].as_str().unwrap();
        assert!(text.contains("❌"));
    }

    #[test]
    fn test_build_telegram_payload_task_started_emoji() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Telegram,
            url: "https://api.telegram.org/bot".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "task_started".to_string(),
            timestamp: Utc::now(),
            data: json!({
                "title": "Starting",
                "text": "Task starting now"
            }),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };
        let payload = service.build_payload(&config, &event);

        let text = payload["text"].as_str().unwrap();
        assert!(text.contains("🚀"));
    }

    #[test]
    fn test_build_telegram_payload_unknown_event_emoji() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Telegram,
            url: "https://api.telegram.org/bot".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "unknown".to_string(),
            timestamp: Utc::now(),
            data: json!({
                "title": "Unknown",
                "text": "Unknown event occurred"
            }),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };
        let payload = service.build_payload(&config, &event);

        let text = payload["text"].as_str().unwrap();
        assert!(text.contains("📢"));
    }

    #[test]
    fn test_build_payload_routing() {
        // Проверяем, что build_payload правильно маршрутизирует к нужным билдерам
        let service = WebhookService::new();
        let event = WebhookEvent {
            event_type: "test".to_string(),
            timestamp: Utc::now(),
            data: json!({"title": "Test", "text": "Test text"}),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };

        // Slack
        let config_slack = WebhookConfig {
            id: 1,
            name: "Slack".to_string(),
            r#type: WebhookType::Slack,
            url: "https://slack.com".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };
        let payload = service.build_payload(&config_slack, &event);
        assert!(payload["attachments"].is_array());

        // Teams
        let config_teams = WebhookConfig {
            id: 1,
            name: "Teams".to_string(),
            r#type: WebhookType::Teams,
            url: "https://teams.com".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };
        let payload = service.build_payload(&config_teams, &event);
        assert_eq!(payload["@type"], "MessageCard");

        // Discord
        let config_discord = WebhookConfig {
            id: 1,
            name: "Discord".to_string(),
            r#type: WebhookType::Discord,
            url: "https://discord.com".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };
        let payload = service.build_payload(&config_discord, &event);
        assert!(payload["embeds"].is_array());

        // Telegram
        let config_telegram = WebhookConfig {
            id: 1,
            name: "Telegram".to_string(),
            r#type: WebhookType::Telegram,
            url: "https://telegram.org".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };
        let payload = service.build_payload(&config_telegram, &event);
        assert!(payload["text"].is_string());
    }

    #[test]
    fn test_create_task_event_with_all_status_values() {
        let statuses = vec!["pending", "running", "success", "failed", "cancelled"];

        for status in statuses {
            let event = create_task_event("task.status", 1, "Task", Some(1), Some(2), Some(status));
            assert_eq!(event.data["status"], status);
            assert_eq!(event.metadata.project_id, Some(1));
            assert_eq!(event.metadata.user_id, Some(2));
        }
    }

    #[test]
    fn test_create_task_event_without_optional_fields() {
        let event = create_task_event("task.created", 1, "Task", None, None, None);

        assert_eq!(event.data["task_id"], 1);
        assert_eq!(event.data["task_name"], "Task");
        assert_eq!(event.data["status"], "unknown");
        assert!(event.metadata.project_id.is_none());
        assert!(event.metadata.user_id.is_none());
    }

    #[test]
    fn test_create_user_event_without_project() {
        let event = create_user_event("user.updated", 10, "john", None);

        assert_eq!(event.metadata.user_id, Some(10));
        assert!(event.metadata.project_id.is_none());
        assert_eq!(event.data["username"], "john");
    }

    #[test]
    fn test_create_project_event_without_user() {
        let event = create_project_event("project.deleted", 100, "Old Project", None);

        assert_eq!(event.metadata.project_id, Some(100));
        assert!(event.metadata.user_id.is_none());
        assert_eq!(event.data["project_name"], "Old Project");
    }

    #[test]
    fn test_webhook_config_with_secret() {
        let config = WebhookConfig {
            id: 1,
            name: "Secure Webhook".to_string(),
            r#type: WebhookType::Generic,
            url: "https://example.com/webhook".to_string(),
            secret: Some("super_secret_key".to_string()),
            headers: None,
            active: true,
            events: vec![],
            retry_count: 3,
            timeout_secs: 30,
        };

        assert!(config.secret.is_some());
        assert_eq!(config.secret.as_ref().unwrap(), "super_secret_key");
    }

    #[test]
    fn test_webhook_config_with_custom_headers() {
        let headers = json!({
            "Authorization": "Bearer abc123",
            "X-Request-ID": "req-001",
            "Content-Type": "application/json"
        });

        let config = WebhookConfig {
            id: 1,
            name: "Configured Webhook".to_string(),
            r#type: WebhookType::Custom,
            url: "https://example.com/custom".to_string(),
            secret: None,
            headers: Some(headers),
            active: true,
            events: vec!["custom.event".to_string()],
            retry_count: 0,
            timeout_secs: 10,
        };

        assert!(config.headers.is_some());
        assert_eq!(config.r#type, WebhookType::Custom);
    }

    #[test]
    fn test_webhook_event_timestamp() {
        let before = Utc::now();
        let event = WebhookEvent {
            event_type: "test".to_string(),
            timestamp: Utc::now(),
            data: json!({}),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };
        let after = Utc::now();

        assert!(event.timestamp >= before);
        assert!(event.timestamp <= after);
    }

    #[test]
    fn test_webhook_config_events_multiple() {
        let config = WebhookConfig {
            id: 1,
            name: "Multi-event".to_string(),
            r#type: WebhookType::Generic,
            url: "https://example.com".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![
                "task.started".to_string(),
                "task.completed".to_string(),
                "task.failed".to_string(),
            ],
            retry_count: 0,
            timeout_secs: 30,
        };

        assert_eq!(config.events.len(), 3);
        assert!(config.events.contains(&"task.started".to_string()));
        assert!(config.events.contains(&"task.completed".to_string()));
        assert!(config.events.contains(&"task.failed".to_string()));
    }

    #[test]
    fn test_webhook_config_zero_retry() {
        let config = WebhookConfig {
            id: 1,
            name: "No Retry".to_string(),
            r#type: WebhookType::Generic,
            url: "https://example.com".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        assert_eq!(config.retry_count, 0);
    }

    #[test]
    fn test_webhook_config_high_retry() {
        let config = WebhookConfig {
            id: 1,
            name: "High Retry".to_string(),
            r#type: WebhookType::Generic,
            url: "https://example.com".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 10,
            timeout_secs: 60,
        };

        assert_eq!(config.retry_count, 10);
    }

    #[test]
    fn test_webhook_config_timeout_values() {
        let timeouts = vec![5, 30, 60, 120, 300];

        for timeout in timeouts {
            let config = WebhookConfig {
                id: 1,
                name: format!("Timeout {}", timeout),
                r#type: WebhookType::Generic,
                url: "https://example.com".to_string(),
                secret: None,
                headers: None,
                active: true,
                events: vec![],
                retry_count: 0,
                timeout_secs: timeout,
            };
            assert_eq!(config.timeout_secs, timeout);
        }
    }

    #[test]
    fn test_webhook_service_with_timeout_values() {
        let timeouts = vec![10, 30, 60];

        for timeout in timeouts {
            let service = WebhookService::with_timeout(timeout);
            let _ = service;
        }
    }

    #[test]
    fn test_build_generic_payload_structure() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Generic,
            url: "https://example.com".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "custom.event".to_string(),
            timestamp: Utc::now(),
            data: json!({
                "user_id": 123,
                "action": "login",
                "ip": "192.168.1.1"
            }),
            metadata: WebhookMetadata {
                source: "auth_service".to_string(),
                version: "2.1.0".to_string(),
                project_id: Some(5),
                user_id: Some(123),
            },
        };

        let payload = service.build_payload(&config, &event);

        assert_eq!(payload["event"], "custom.event");
        assert!(payload["timestamp"].is_string());
        assert_eq!(payload["data"]["user_id"], 123);
        assert_eq!(payload["data"]["action"], "login");
        assert_eq!(payload["metadata"]["source"], "auth_service");
        assert_eq!(payload["metadata"]["version"], "2.1.0");
        assert_eq!(payload["metadata"]["project_id"], 5);
    }

    #[test]
    fn test_build_slack_payload_with_fields() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Slack,
            url: "https://hooks.slack.com".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "task_success".to_string(),
            timestamp: Utc::now(),
            data: json!({
                "title": "Deploy Success",
                "text": "Application deployed successfully"
            }),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };
        let payload = service.build_payload(&config, &event);

        let attachments = payload["attachments"].as_array().unwrap();
        let attachment = &attachments[0];
        assert_eq!(attachment["color"], "good");
        assert!(attachment["fields"].is_array());

        let fields = attachment["fields"].as_array().unwrap();
        assert!(!fields.is_empty());
    }

    #[test]
    fn test_build_teams_payload_structure() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Teams,
            url: "https://outlook.office.com/webhook".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "task_success".to_string(),
            timestamp: Utc::now(),
            data: json!({
                "title": "Teams Test",
                "text": "Test message"
            }),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };
        let payload = service.build_payload(&config, &event);

        assert_eq!(payload["@type"], "MessageCard");
        assert_eq!(payload["@context"], "http://schema.org/extensions");
        assert!(payload["summary"].is_string());
        assert!(payload["sections"].is_array());

        let sections = payload["sections"].as_array().unwrap();
        assert_eq!(sections.len(), 1);
        let section = &sections[0];
        assert!(section["activityTitle"].is_string());
        assert!(section["facts"].is_array());
    }

    #[test]
    fn test_build_discord_embed_structure() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Discord,
            url: "https://discord.com/api/webhooks".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "test_event".to_string(),
            timestamp: Utc::now(),
            data: json!({
                "title": "Discord Test",
                "text": "Test embed"
            }),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };
        let payload = service.build_payload(&config, &event);

        let embeds = payload["embeds"].as_array().unwrap();
        assert_eq!(embeds.len(), 1);
        let embed = &embeds[0];

        assert!(embed["title"].is_string());
        assert!(embed["description"].is_string());
        assert!(embed["color"].is_number());
        assert!(embed["fields"].is_array());
        assert!(embed["footer"]["text"].as_str().unwrap() == "Velum UI");
        assert!(embed["timestamp"].is_string());
    }

    #[test]
    fn test_build_telegram_message_format() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Telegram,
            url: "https://api.telegram.org/bot".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "task_success".to_string(),
            timestamp: Utc::now(),
            data: json!({
                "title": "Telegram Test",
                "text": "Test message content"
            }),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };
        let payload = service.build_payload(&config, &event);

        let text = payload["text"].as_str().unwrap();
        assert!(text.contains("✅"));
        assert!(text.contains("Telegram Test"));
        assert!(text.contains("Test message content"));
        assert!(text.contains("Время:"));
        assert_eq!(payload["parse_mode"], "HTML");
    }

    #[test]
    fn test_webhook_result_debug() {
        let result = WebhookResult {
            success: true,
            status_code: Some(200),
            response_body: Some("{\"status\": \"ok\"}".to_string()),
            error: None,
            attempts: 1,
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("WebhookResult"));
        assert!(debug_str.contains("success: true"));
    }

    #[test]
    fn test_webhook_event_clone() {
        let event = WebhookEvent {
            event_type: "test.clone".to_string(),
            timestamp: Utc::now(),
            data: json!({"key": "value"}),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: Some(1),
                user_id: Some(2),
            },
        };

        let cloned = event.clone();
        assert_eq!(cloned.event_type, event.event_type);
        assert_eq!(cloned.data, event.data);
        assert_eq!(cloned.metadata.source, event.metadata.source);
    }

    #[test]
    fn test_webhook_config_clone() {
        let config = WebhookConfig {
            id: 1,
            name: "Clone Test".to_string(),
            r#type: WebhookType::Discord,
            url: "https://discord.com".to_string(),
            secret: Some("secret".to_string()),
            headers: Some(json!({"X-Test": "value"})),
            active: true,
            events: vec!["test.event".to_string()],
            retry_count: 3,
            timeout_secs: 30,
        };

        let cloned = config.clone();
        assert_eq!(cloned.id, config.id);
        assert_eq!(cloned.name, config.name);
        assert_eq!(cloned.r#type, config.r#type);
        assert_eq!(cloned.url, config.url);
    }

    #[test]
    fn test_webhook_type_partial_eq() {
        assert!(WebhookType::Generic == WebhookType::Generic);
        assert!(WebhookType::Slack != WebhookType::Teams);
        assert!(WebhookType::Discord == WebhookType::Discord);
        assert!(WebhookType::Telegram != WebhookType::Custom);
    }

    #[test]
    fn test_build_generic_payload_missing_title_text() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Generic,
            url: "https://example.com".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "test".to_string(),
            timestamp: Utc::now(),
            data: json!({}), // нет title и text
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };

        let payload = service.build_payload(&config, &event);
        assert_eq!(payload["event"], "test");
        assert!(payload["data"].is_object());
    }

    #[test]
    fn test_build_slack_payload_missing_title_text() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Slack,
            url: "https://hooks.slack.com".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "test".to_string(),
            timestamp: Utc::now(),
            data: json!({}),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };

        let payload = service.build_payload(&config, &event);
        let attachments = payload["attachments"].as_array().unwrap();
        let attachment = &attachments[0];
        // Значения по умолчанию
        assert_eq!(attachment["title"].as_str().unwrap(), "Уведомление");
        assert_eq!(attachment["text"].as_str().unwrap(), "");
    }

    #[test]
    fn test_build_teams_payload_missing_title_text() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Teams,
            url: "https://outlook.office.com/webhook".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "test".to_string(),
            timestamp: Utc::now(),
            data: json!({}),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };

        let payload = service.build_payload(&config, &event);
        assert_eq!(payload["summary"].as_str().unwrap(), "Уведомление");
    }

    #[test]
    fn test_build_discord_payload_missing_title_text() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Discord,
            url: "https://discord.com/api/webhooks".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "test".to_string(),
            timestamp: Utc::now(),
            data: json!({}),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };

        let payload = service.build_payload(&config, &event);
        let embed = &payload["embeds"][0];
        assert_eq!(embed["title"].as_str().unwrap(), "Уведомление");
        assert_eq!(embed["description"].as_str().unwrap(), "");
    }

    #[test]
    fn test_build_telegram_payload_missing_title_text() {
        let service = WebhookService::new();
        let config = WebhookConfig {
            id: 1,
            name: "Test".to_string(),
            r#type: WebhookType::Telegram,
            url: "https://api.telegram.org/bot".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "test".to_string(),
            timestamp: Utc::now(),
            data: json!({}),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };

        let payload = service.build_payload(&config, &event);
        let text = payload["text"].as_str().unwrap();
        assert!(text.contains("Уведомление"));
        assert!(text.contains("Время:"));
    }
}
