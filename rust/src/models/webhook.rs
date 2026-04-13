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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_type_serialization() {
        assert_eq!(serde_json::to_string(&WebhookType::Generic).unwrap(), "\"generic\"");
        assert_eq!(serde_json::to_string(&WebhookType::Slack).unwrap(), "\"slack\"");
        assert_eq!(serde_json::to_string(&WebhookType::Teams).unwrap(), "\"teams\"");
        assert_eq!(serde_json::to_string(&WebhookType::Discord).unwrap(), "\"discord\"");
        assert_eq!(serde_json::to_string(&WebhookType::Telegram).unwrap(), "\"telegram\"");
    }

    #[test]
    fn test_webhook_serialization() {
        let webhook = Webhook {
            id: 1,
            project_id: Some(10),
            name: "Slack Notifications".to_string(),
            r#type: WebhookType::Slack,
            url: "https://hooks.slack.com/xxx".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: serde_json::json!(["task_completed", "task_failed"]),
            retry_count: 3,
            timeout_secs: 30,
            created: Utc::now(),
            updated: Utc::now(),
        };
        let json = serde_json::to_string(&webhook).unwrap();
        assert!(json.contains("\"name\":\"Slack Notifications\""));
        assert!(json.contains("\"type\":\"slack\""));
        assert!(json.contains("\"active\":true"));
    }

    #[test]
    fn test_create_webhook_serialization() {
        let create = CreateWebhook {
            project_id: None,
            name: "New Webhook".to_string(),
            r#type: WebhookType::Generic,
            url: "https://example.com/webhook".to_string(),
            secret: Some("secret".to_string()),
            headers: None,
            active: true,
            events: vec!["task_completed".to_string()],
            retry_count: 5,
            timeout_secs: 60,
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"name\":\"New Webhook\""));
        assert!(json.contains("\"events\":[\"task_completed\"]"));
    }

    #[test]
    fn test_update_webhook_skip_nulls() {
        let update = UpdateWebhook {
            name: Some("Updated Name".to_string()),
            r#type: None,
            url: None,
            secret: None,
            headers: None,
            active: Some(false),
            events: None,
            retry_count: None,
            timeout_secs: None,
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"name\":\"Updated Name\""));
        assert!(json.contains("\"active\":false"));
        // UpdateWebhook derives Default but not skip_serializing_if on all fields
        // So url:None serializes as "url":null
        assert!(json.contains("\"url\":null"));
    }

    #[test]
    fn test_webhook_log_serialization() {
        let log = WebhookLog {
            id: 1,
            webhook_id: 10,
            event_type: "task_completed".to_string(),
            status_code: Some(200),
            success: true,
            error: None,
            attempts: 1,
            payload: Some(serde_json::json!({"task_id": 100})),
            response: Some(serde_json::json!({"status": "ok"})),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&log).unwrap();
        assert!(json.contains("\"event_type\":\"task_completed\""));
        assert!(json.contains("\"status_code\":200"));
        assert!(json.contains("\"success\":true"));
    }

    #[test]
    fn test_webhook_type_clone() {
        let wt = WebhookType::Custom;
        let cloned = wt.clone();
        assert_eq!(cloned, wt);
    }

    #[test]
    fn test_webhook_type_equality() {
        assert_eq!(WebhookType::Slack, WebhookType::Slack);
        assert_ne!(WebhookType::Slack, WebhookType::Teams);
    }

    #[test]
    fn test_webhook_type_all_serialization() {
        let types = [WebhookType::Generic, WebhookType::Slack, WebhookType::Teams,
                     WebhookType::Discord, WebhookType::Telegram, WebhookType::Custom];
        for t in &types {
            let json = serde_json::to_string(t).unwrap();
            assert!(json.starts_with('"') && json.ends_with('"'));
        }
    }

    #[test]
    fn test_create_webhook_clone() {
        let create = CreateWebhook {
            project_id: Some(1), name: "Clone Webhook".to_string(),
            r#type: WebhookType::Generic, url: "https://example.com".to_string(),
            secret: None, headers: None, active: true, events: vec![],
            retry_count: 3, timeout_secs: 30,
        };
        let cloned = create.clone();
        assert_eq!(cloned.name, create.name);
        assert_eq!(cloned.r#type, create.r#type);
    }

    #[test]
    fn test_update_webhook_clone() {
        let update = UpdateWebhook {
            name: Some("Clone Update".to_string()), r#type: None, url: None,
            secret: None, headers: None, active: Some(true), events: None,
            retry_count: None, timeout_secs: None,
        };
        let cloned = update.clone();
        assert_eq!(cloned.name, update.name);
        assert_eq!(cloned.active, update.active);
    }

    #[test]
    fn test_webhook_log_clone() {
        let log = WebhookLog {
            id: 1, webhook_id: 10, event_type: "task_failed".to_string(),
            status_code: Some(500), success: false, error: Some("Error".to_string()),
            attempts: 3, payload: None, response: None, created: Utc::now(),
        };
        let cloned = log.clone();
        assert_eq!(cloned.event_type, log.event_type);
        assert_eq!(cloned.status_code, log.status_code);
    }

    #[test]
    fn test_test_webhook_serialization() {
        let test = TestWebhook {
            url: "https://test.com/hook".to_string(),
            r#type: WebhookType::Slack,
            secret: Some("test_secret".to_string()),
            headers: Some(serde_json::json!({"Authorization": "Bearer token"})),
        };
        let json = serde_json::to_string(&test).unwrap();
        assert!(json.contains("\"url\":\"https://test.com/hook\""));
        assert!(json.contains("\"type\":\"slack\""));
    }

    #[test]
    fn test_webhook_deserialization() {
        let json = r#"{"id":5,"project_id":10,"name":"Deser Webhook","type":"discord","url":"https://discord.com/webhook","secret":null,"headers":null,"active":true,"events":["task_completed"],"retry_count":3,"timeout_secs":30,"created":"2024-01-01T00:00:00Z","updated":"2024-01-01T00:00:00Z"}"#;
        let webhook: Webhook = serde_json::from_str(json).unwrap();
        assert_eq!(webhook.id, 5);
        assert_eq!(webhook.name, "Deser Webhook");
        assert_eq!(webhook.r#type, WebhookType::Discord);
    }

    #[test]
    fn test_webhook_type_debug() {
        let wt = WebhookType::Generic;
        let debug_str = format!("{:?}", wt);
        assert!(debug_str.contains("Generic"));
    }

    #[test]
    fn test_webhook_unicode_name() {
        let webhook = Webhook {
            id: 1, project_id: Some(1), name: "Вебхук".to_string(),
            r#type: WebhookType::Generic, url: "https://example.com".to_string(),
            secret: None, headers: None, active: true, events: serde_json::json!([]),
            retry_count: 3, timeout_secs: 30, created: Utc::now(), updated: Utc::now(),
        };
        let json = serde_json::to_string(&webhook).unwrap();
        let restored: Webhook = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, "Вебхук");
    }
}
