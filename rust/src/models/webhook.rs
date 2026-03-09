//! Webhook модель

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Тип webhook (публичный для моделей)
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

/// Webhook для уведомлений
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Webhook {
    pub id: i64,
    pub project_id: Option<i64>,
    pub name: String,
    pub r#type: WebhookType,
    pub url: String,
    pub secret: Option<String>,
    pub headers: Option<serde_json::Value>,
    pub active: bool,
    pub events: serde_json::Value,
    pub retry_count: i32,
    pub timeout_secs: i64,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

/// Создание webhook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhook {
    pub project_id: Option<i64>,
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

/// Обновление webhook
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateWebhook {
    pub name: Option<String>,
    pub r#type: Option<WebhookType>,
    pub url: Option<String>,
    pub secret: Option<String>,
    pub headers: Option<serde_json::Value>,
    pub active: Option<bool>,
    pub events: Option<Vec<String>>,
    pub retry_count: Option<i32>,
    pub timeout_secs: Option<i64>,
}

/// Тест webhook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestWebhook {
    pub url: String,
    pub r#type: WebhookType,
    pub secret: Option<String>,
    pub headers: Option<serde_json::Value>,
}

/// История webhook
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebhookLog {
    pub id: i64,
    pub webhook_id: i64,
    pub event_type: String,
    pub status_code: Option<i32>,
    pub success: bool,
    pub error: Option<String>,
    pub attempts: i32,
    pub payload: Option<serde_json::Value>,
    pub response: Option<serde_json::Value>,
    pub created: DateTime<Utc>,
}
