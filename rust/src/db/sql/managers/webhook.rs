//! Менеджер хранилища данных
//!
//! Автоматически извлечён из mod.rs в рамках декомпозиции

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::models::webhook::{Webhook, UpdateWebhook, WebhookLog};
use crate::error::{Error, Result};
use async_trait::async_trait;

#[async_trait]
impl WebhookManager for SqlStore {
    async fn get_webhook(&self, webhook_id: i64) -> Result<crate::models::webhook::Webhook> {
        self.db.get_webhook(webhook_id).await
    }

    async fn get_webhooks_by_project(&self, project_id: i64) -> Result<Vec<crate::models::webhook::Webhook>> {
        self.db.get_webhooks_by_project(project_id).await
    }

    async fn create_webhook(&self, webhook: crate::models::webhook::Webhook) -> Result<crate::models::webhook::Webhook> {
        self.db.create_webhook(webhook).await
    }

    async fn update_webhook(&self, webhook_id: i64, webhook: crate::models::webhook::UpdateWebhook) -> Result<crate::models::webhook::Webhook> {
        self.db.update_webhook(webhook_id, webhook).await
    }

    async fn delete_webhook(&self, webhook_id: i64) -> Result<()> {
        self.db.delete_webhook(webhook_id).await
    }

    async fn get_webhook_logs(&self, webhook_id: i64) -> Result<Vec<crate::models::webhook::WebhookLog>> {
        self.db.get_webhook_logs(webhook_id).await
    }

    async fn create_webhook_log(&self, log: crate::models::webhook::WebhookLog) -> Result<crate::models::webhook::WebhookLog> {
        self.db.create_webhook_log(log).await
    }
}

