//! Менеджер хранилища данных
//!
//! Автоматически извлечён из mod.rs в рамках декомпозиции

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::webhook::{UpdateWebhook, Webhook, WebhookLog};
use async_trait::async_trait;

#[async_trait]
impl WebhookManager for SqlStore {
    async fn get_webhook(&self, webhook_id: i64) -> Result<crate::models::webhook::Webhook> {
        self.db.get_webhook(webhook_id).await
    }

    async fn get_webhooks_by_project(
        &self,
        project_id: i64,
    ) -> Result<Vec<crate::models::webhook::Webhook>> {
        self.db.get_webhooks_by_project(project_id).await
    }

    async fn create_webhook(
        &self,
        webhook: crate::models::webhook::Webhook,
    ) -> Result<crate::models::webhook::Webhook> {
        self.db.create_webhook(webhook).await
    }

    async fn update_webhook(
        &self,
        webhook_id: i64,
        webhook: crate::models::webhook::UpdateWebhook,
    ) -> Result<crate::models::webhook::Webhook> {
        self.db.update_webhook(webhook_id, webhook).await
    }

    async fn delete_webhook(&self, webhook_id: i64) -> Result<()> {
        self.db.delete_webhook(webhook_id).await
    }

    async fn get_webhook_logs(
        &self,
        webhook_id: i64,
    ) -> Result<Vec<crate::models::webhook::WebhookLog>> {
        self.db.get_webhook_logs(webhook_id).await
    }

    async fn create_webhook_log(
        &self,
        log: crate::models::webhook::WebhookLog,
    ) -> Result<crate::models::webhook::WebhookLog> {
        self.db.create_webhook_log(log).await
    }
}

#[cfg(test)]
mod tests {
    use crate::models::webhook::{CreateWebhook, UpdateWebhook, Webhook, WebhookLog, WebhookType};
    use chrono::Utc;

    #[test]
    fn test_webhook_type_serialization() {
        assert_eq!(
            serde_json::to_string(&WebhookType::Generic).unwrap(),
            "\"generic\""
        );
        assert_eq!(
            serde_json::to_string(&WebhookType::Slack).unwrap(),
            "\"slack\""
        );
        assert_eq!(
            serde_json::to_string(&WebhookType::Teams).unwrap(),
            "\"teams\""
        );
        assert_eq!(
            serde_json::to_string(&WebhookType::Discord).unwrap(),
            "\"discord\""
        );
        assert_eq!(
            serde_json::to_string(&WebhookType::Telegram).unwrap(),
            "\"telegram\""
        );
        assert_eq!(
            serde_json::to_string(&WebhookType::Custom).unwrap(),
            "\"custom\""
        );
    }

    #[test]
    fn test_webhook_serialization() {
        let webhook = Webhook {
            id: 1,
            project_id: Some(10),
            name: "Slack Hook".to_string(),
            r#type: WebhookType::Slack,
            url: "https://hooks.slack.com/xxx".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: serde_json::json!(["task_completed"]),
            retry_count: 3,
            timeout_secs: 30,
            created: Utc::now(),
            updated: Utc::now(),
        };
        let json = serde_json::to_string(&webhook).unwrap();
        assert!(json.contains("\"name\":\"Slack Hook\""));
        assert!(json.contains("\"type\":\"slack\""));
    }

    #[test]
    fn test_create_webhook_serialization() {
        let create = CreateWebhook {
            project_id: None,
            name: "New Hook".to_string(),
            r#type: WebhookType::Generic,
            url: "https://example.com".to_string(),
            secret: Some("secret".to_string()),
            headers: None,
            active: true,
            events: vec!["task_completed".to_string()],
            retry_count: 5,
            timeout_secs: 60,
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"name\":\"New Hook\""));
        assert!(json.contains("\"events\":[\"task_completed\"]"));
    }

    #[test]
    fn test_update_webhook_default() {
        let update = UpdateWebhook::default();
        assert!(update.name.is_none());
        assert!(update.active.is_none());
        assert!(update.url.is_none());
    }

    #[test]
    fn test_update_webhook_serialization() {
        let update = UpdateWebhook {
            name: Some("Updated".to_string()),
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
        assert!(json.contains("\"name\":\"Updated\""));
        assert!(json.contains("\"active\":false"));
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
            response: Some(serde_json::json!({"ok": true})),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&log).unwrap();
        assert!(json.contains("\"event_type\":\"task_completed\""));
        assert!(json.contains("\"status_code\":200"));
        assert!(json.contains("\"success\":true"));
    }

    #[test]
    fn test_webhook_type_equality() {
        assert_eq!(WebhookType::Slack, WebhookType::Slack);
        assert_ne!(WebhookType::Slack, WebhookType::Teams);
    }

    #[test]
    fn test_create_webhook_clone() {
        let create = CreateWebhook {
            project_id: Some(1),
            name: "Clone".to_string(),
            r#type: WebhookType::Discord,
            url: "https://discord.com/webhook".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec!["test".to_string()],
            retry_count: 1,
            timeout_secs: 10,
        };
        let cloned = create.clone();
        assert_eq!(cloned.name, create.name);
        assert_eq!(cloned.r#type, create.r#type);
    }

    #[test]
    fn test_webhook_log_clone() {
        let log = WebhookLog {
            id: 1,
            webhook_id: 1,
            event_type: "test".to_string(),
            status_code: None,
            success: false,
            error: Some("error".to_string()),
            attempts: 3,
            payload: None,
            response: None,
            created: Utc::now(),
        };
        let cloned = log.clone();
        assert_eq!(cloned.event_type, log.event_type);
        assert_eq!(cloned.attempts, log.attempts);
    }
}
