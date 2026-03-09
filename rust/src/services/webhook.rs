//! Webhook - универсальный сервис отправки webhook уведомлений
//!
//! Поддерживает различные типы webhook:
//! - Generic JSON webhook
//! - Slack
//! - Microsoft Teams
//! - Discord
//! - Telegram
//! - Custom

use reqwest::{Client, header::{HeaderMap, CONTENT_TYPE, AUTHORIZATION}};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use chrono::{DateTime, Utc};
use tracing::{info, warn, error};
use crate::error::{Error, Result};

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
                        info!("Webhook {} успешно отправлен (попытка {}/{})", 
                              config.name, attempts, config.retry_count + 1);
                        return Ok(result);
                    }
                    last_error = result.error.clone();
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    warn!("Webhook {} ошибка отправки (попытка {}/{}): {}", 
                          config.name, attempts, config.retry_count + 1, e);
                }
            }

            if attempts <= config.retry_count as u32 {
                // Экспоненциальная задержка между попытками
                tokio::time::sleep(
                    std::time::Duration::from_millis(100 * 2u64.pow(attempts - 1))
                ).await;
            }
        }

        error!("Webhook {} не отправлен после {} попыток", config.name, attempts);
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

        let title = event.data.get("title").and_then(|v| v.as_str()).unwrap_or("Уведомление");
        let text = event.data.get("text").and_then(|v| v.as_str()).unwrap_or("");

        json!({
            "attachments": [{
                "color": color,
                "author_name": format!("{} Semaphore UI", emoji),
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
                "footer": "Semaphore UI",
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

        let title = event.data.get("title").and_then(|v| v.as_str()).unwrap_or("Уведомление");
        let text = event.data.get("text").and_then(|v| v.as_str()).unwrap_or("");

        json!({
            "@type": "MessageCard",
            "@context": "http://schema.org/extensions",
            "themeColor": color,
            "summary": title,
            "sections": [{
                "activityTitle": title,
                "activitySubtitle": "Semaphore UI",
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

        let title = event.data.get("title").and_then(|v| v.as_str()).unwrap_or("Уведомление");
        let text = event.data.get("text").and_then(|v| v.as_str()).unwrap_or("");

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
                    "text": "Semaphore UI"
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

        let title = event.data.get("title").and_then(|v| v.as_str()).unwrap_or("Уведомление");
        let text = event.data.get("text").and_then(|v| v.as_str()).unwrap_or("");

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
                        if let Ok(header_name) = key.as_str().parse::<reqwest::header::HeaderName>() {
                            if let Ok(header_value) = v.parse::<reqwest::header::HeaderValue>() {
                                headers.insert(header_name, header_value);
                            }
                        }
                    }
                }
            }
        }

        // Добавляем секрет в заголовок (если указан)
        if let Some(secret) = &config.secret {
            headers.insert(
                AUTHORIZATION,
                format!("Bearer {}", secret).parse().unwrap()
            );
        }

        let request = self.client.post(&config.url)
            .headers(headers)
            .json(payload);

        let response = request.send().await.map_err(|e| {
            Error::Other(format!("Ошибка отправки webhook: {}", e))
        })?;

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

/// Helper функции для создания событий

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
            source: "semaphore-ui".to_string(),
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
            source: "semaphore-ui".to_string(),
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
            source: "semaphore-ui".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            project_id: Some(project_id),
            user_id,
        },
    }
}
