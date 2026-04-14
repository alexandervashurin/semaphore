//! Webhook SQL реализация
//!
//! Полная реализация CRUD операций для webhook

use super::SqlDb;
use crate::error::{Error, Result};
use crate::models::webhook::{UpdateWebhook, Webhook, WebhookLog, WebhookType};
use chrono::Utc;
use sqlx::{FromRow, Row};

// Helper types для SQLx
#[derive(Debug, Clone, FromRow)]
struct WebhookRow {
    id: i64,
    project_id: Option<i64>,
    name: String,
    #[sqlx(rename = "type")]
    webhook_type: String,
    url: String,
    secret: Option<String>,
    headers: Option<serde_json::Value>,
    active: bool,
    events: serde_json::Value,
    retry_count: i32,
    timeout_secs: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, FromRow)]
struct WebhookLogRow {
    id: i64,
    webhook_id: i64,
    event_type: String,
    status_code: Option<i32>,
    success: bool,
    error: Option<String>,
    attempts: i32,
    payload: Option<serde_json::Value>,
    response: Option<serde_json::Value>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl SqlDb {
    /// Получает webhook по ID
    pub async fn get_webhook(&self, webhook_id: i64) -> Result<Webhook> {
        let row = sqlx::query_as::<_, WebhookRow>("SELECT * FROM webhook WHERE id = $1")
            .bind(webhook_id)
            .fetch_optional(
                self.get_postgres_pool()
                    .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?,
            )
            .await
            .map_err(Error::Database)?;

        let row =
            row.ok_or_else(|| Error::NotFound(format!("Webhook {} not found", webhook_id)))?;
        Ok(self.row_to_webhook(row))
    }

    /// Получает webhook проекта
    pub async fn get_webhooks_by_project(&self, project_id: i64) -> Result<Vec<Webhook>> {
        let rows = sqlx::query_as::<_, WebhookRow>(
            "SELECT * FROM webhook WHERE project_id = $1 ORDER BY created_at DESC",
        )
        .bind(project_id)
        .fetch_all(
            self.get_postgres_pool()
                .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?,
        )
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(|r| self.row_to_webhook(r)).collect())
    }

    /// Создаёт webhook
    pub async fn create_webhook(&self, mut webhook: Webhook) -> Result<Webhook> {
        let now = Utc::now();
        let type_str = self.webhook_type_to_string(&webhook.r#type);

        let id = sqlx::query_scalar::<_, i64>(
                    "INSERT INTO webhook (project_id, name, type, url, secret, headers, active, events, retry_count, timeout_secs, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING id"
                )
                .bind(webhook.project_id)
                .bind(&webhook.name)
                .bind(&type_str)
                .bind(&webhook.url)
                .bind(&webhook.secret)
                .bind(&webhook.headers)
                .bind(webhook.active)
                .bind(&webhook.events)
                .bind(webhook.retry_count)
                .bind(webhook.timeout_secs)
                .bind(now)
                .bind(now)
                .fetch_one(self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

        webhook.id = id;
        webhook.created = now;
        webhook.updated = now;
        Ok(webhook)
    }

    /// Обновляет webhook
    pub async fn update_webhook(&self, webhook_id: i64, webhook: UpdateWebhook) -> Result<Webhook> {
        let now = Utc::now();
        let mut current = self.get_webhook(webhook_id).await?;

        // Обновляем поля
        if let Some(name) = webhook.name {
            current.name = name;
        }
        if let Some(r#type) = webhook.r#type {
            current.r#type = r#type;
        }
        if let Some(url) = webhook.url {
            current.url = url;
        }
        if let Some(secret) = webhook.secret {
            current.secret = Some(secret);
        }
        if let Some(headers) = webhook.headers {
            current.headers = Some(headers);
        }
        if let Some(active) = webhook.active {
            current.active = active;
        }
        if let Some(events) = webhook.events {
            current.events = serde_json::to_value(&events).unwrap_or_default();
        }
        if let Some(retry_count) = webhook.retry_count {
            current.retry_count = retry_count;
        }
        if let Some(timeout_secs) = webhook.timeout_secs {
            current.timeout_secs = timeout_secs;
        }
        current.updated = now;

        let type_str = self.webhook_type_to_string(&current.r#type);

        sqlx::query(
                    "UPDATE webhook SET name=$1, type=$2, url=$3, secret=$4, headers=$5, active=$6, events=$7, retry_count=$8, timeout_secs=$9, updated_at=$10 WHERE id=$11"
                )
                .bind(&current.name).bind(&type_str).bind(&current.url)
                .bind(&current.secret).bind(&current.headers)
                .bind(current.active).bind(&current.events)
                .bind(current.retry_count).bind(current.timeout_secs)
                .bind(now).bind(webhook_id)
                .execute(self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
        Ok(current)
    }

    /// Удаляет webhook
    pub async fn delete_webhook(&self, webhook_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM webhook WHERE id = $1")
            .bind(webhook_id)
            .execute(
                self.get_postgres_pool()
                    .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?,
            )
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    /// Получает логи webhook
    pub async fn get_webhook_logs(&self, webhook_id: i64) -> Result<Vec<WebhookLog>> {
        let rows = sqlx::query_as::<_, WebhookLogRow>(
            "SELECT * FROM webhook_log WHERE webhook_id = $1 ORDER BY created_at DESC",
        )
        .bind(webhook_id)
        .fetch_all(
            self.get_postgres_pool()
                .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?,
        )
        .await
        .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|r| self.row_to_webhook_log(r))
            .collect())
    }

    /// Создаёт лог webhook
    pub async fn create_webhook_log(&self, mut log: WebhookLog) -> Result<WebhookLog> {
        let now = Utc::now();

        let id = sqlx::query_scalar::<_, i64>(
                    "INSERT INTO webhook_log (webhook_id, event_type, status_code, success, error, attempts, payload, response, created_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id"
                )
                .bind(log.webhook_id)
                .bind(&log.event_type)
                .bind(log.status_code)
                .bind(log.success)
                .bind(&log.error)
                .bind(log.attempts)
                .bind(&log.payload)
                .bind(&log.response)
                .bind(now)
                .fetch_one(self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

        log.id = id;
        log.created = now;
        Ok(log)
    }

    // === Helper методы ===

    fn row_to_webhook(&self, row: WebhookRow) -> Webhook {
        Webhook {
            id: row.id,
            project_id: row.project_id,
            name: row.name,
            r#type: self.string_to_webhook_type(&row.webhook_type),
            url: row.url,
            secret: row.secret,
            headers: row.headers,
            active: row.active,
            events: row.events,
            retry_count: row.retry_count,
            timeout_secs: row.timeout_secs,
            created: row.created_at,
            updated: row.updated_at,
        }
    }

    fn row_to_webhook_log(&self, row: WebhookLogRow) -> WebhookLog {
        WebhookLog {
            id: row.id,
            webhook_id: row.webhook_id,
            event_type: row.event_type,
            status_code: row.status_code,
            success: row.success,
            error: row.error,
            attempts: row.attempts,
            payload: row.payload,
            response: row.response,
            created: row.created_at,
        }
    }

    fn webhook_type_to_string(&self, t: &WebhookType) -> String {
        match t {
            WebhookType::Generic => "generic".to_string(),
            WebhookType::Slack => "slack".to_string(),
            WebhookType::Teams => "teams".to_string(),
            WebhookType::Discord => "discord".to_string(),
            WebhookType::Telegram => "telegram".to_string(),
            WebhookType::Custom => "custom".to_string(),
        }
    }

    fn string_to_webhook_type(&self, s: &str) -> WebhookType {
        match s {
            "slack" => WebhookType::Slack,
            "teams" => WebhookType::Teams,
            "discord" => WebhookType::Discord,
            "telegram" => WebhookType::Telegram,
            "custom" => WebhookType::Custom,
            _ => WebhookType::Generic,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_row_struct_fields() {
        let row = WebhookRow {
            id: 1,
            project_id: Some(10),
            name: "test".to_string(),
            webhook_type: "slack".to_string(),
            url: "https://example.com".to_string(),
            secret: Some("secret".to_string()),
            headers: Some(serde_json::json!({"Content-Type": "application/json"})),
            active: true,
            events: serde_json::json!(["task_completed"]),
            retry_count: 3,
            timeout_secs: 30,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(row.id, 1);
        assert_eq!(row.webhook_type, "slack");
        assert!(row.active);
    }

    #[test]
    fn test_webhook_log_row_struct_fields() {
        let row = WebhookLogRow {
            id: 1,
            webhook_id: 10,
            event_type: "task_completed".to_string(),
            status_code: Some(200),
            success: true,
            error: None,
            attempts: 1,
            payload: Some(serde_json::json!({"task_id": 5})),
            response: Some(serde_json::json!({"ok": true})),
            created_at: Utc::now(),
        };
        assert_eq!(row.id, 1);
        assert_eq!(row.webhook_id, 10);
        assert!(row.success);
    }

    #[test]
    fn test_webhook_type_to_string_all_variants() {
        let db = SqlDb::new();
        assert_eq!(db.webhook_type_to_string(&WebhookType::Generic), "generic");
        assert_eq!(db.webhook_type_to_string(&WebhookType::Slack), "slack");
        assert_eq!(db.webhook_type_to_string(&WebhookType::Teams), "teams");
        assert_eq!(db.webhook_type_to_string(&WebhookType::Discord), "discord");
        assert_eq!(
            db.webhook_type_to_string(&WebhookType::Telegram),
            "telegram"
        );
        assert_eq!(db.webhook_type_to_string(&WebhookType::Custom), "custom");
    }

    #[test]
    fn test_string_to_webhook_type_all_variants() {
        let db = SqlDb::new();
        assert_eq!(db.string_to_webhook_type("generic"), WebhookType::Generic);
        assert_eq!(db.string_to_webhook_type("slack"), WebhookType::Slack);
        assert_eq!(db.string_to_webhook_type("teams"), WebhookType::Teams);
        assert_eq!(db.string_to_webhook_type("discord"), WebhookType::Discord);
        assert_eq!(db.string_to_webhook_type("telegram"), WebhookType::Telegram);
        assert_eq!(db.string_to_webhook_type("custom"), WebhookType::Custom);
    }

    #[test]
    fn test_string_to_webhook_type_unknown() {
        let db = SqlDb::new();
        assert_eq!(db.string_to_webhook_type("unknown"), WebhookType::Generic);
        assert_eq!(db.string_to_webhook_type(""), WebhookType::Generic);
    }

    #[test]
    fn test_webhook_row_clone() {
        let row = WebhookRow {
            id: 1,
            project_id: None,
            name: "test".to_string(),
            webhook_type: "generic".to_string(),
            url: "https://example.com".to_string(),
            secret: None,
            headers: None,
            active: false,
            events: serde_json::json!([]),
            retry_count: 0,
            timeout_secs: 30,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let cloned = row.clone();
        assert_eq!(cloned.id, row.id);
        assert_eq!(cloned.name, row.name);
    }

    #[test]
    fn test_webhook_log_row_clone() {
        let row = WebhookLogRow {
            id: 1,
            webhook_id: 5,
            event_type: "task_failed".to_string(),
            status_code: Some(500),
            success: false,
            error: Some("connection refused".to_string()),
            attempts: 3,
            payload: None,
            response: None,
            created_at: Utc::now(),
        };
        let cloned = row.clone();
        assert_eq!(cloned.id, row.id);
        assert_eq!(cloned.success, row.success);
    }

    #[test]
    fn test_webhook_type_serialization() {
        let t = WebhookType::Slack;
        let json = serde_json::to_string(&t).unwrap();
        assert!(json.contains("slack"));
    }

    #[test]
    fn test_webhook_type_deserialization() {
        let json = "\"discord\"";
        let t: WebhookType = serde_json::from_str(json).unwrap();
        assert_eq!(t, WebhookType::Discord);
    }

    #[test]
    fn test_webhook_type_equality() {
        assert_eq!(WebhookType::Generic, WebhookType::Generic);
        assert_ne!(WebhookType::Slack, WebhookType::Teams);
    }

    #[test]
    fn test_update_webhook_struct_all_fields() {
        let update = UpdateWebhook {
            name: Some("new name".to_string()),
            r#type: Some(WebhookType::Telegram),
            url: Some("https://new.url".to_string()),
            secret: Some("new secret".to_string()),
            headers: Some(serde_json::json!({})),
            active: Some(true),
            events: Some(vec!["task_started".to_string()]),
            retry_count: Some(5),
            timeout_secs: Some(60),
        };
        assert!(update.name.is_some());
        assert!(update.active.is_some());
        assert_eq!(update.retry_count, Some(5));
    }

    #[test]
    fn test_update_webhook_default() {
        let update = UpdateWebhook::default();
        assert!(update.name.is_none());
        assert!(update.r#type.is_none());
        assert!(update.url.is_none());
        assert!(update.secret.is_none());
        assert!(update.headers.is_none());
        assert!(update.active.is_none());
        assert!(update.events.is_none());
        assert!(update.retry_count.is_none());
        assert!(update.timeout_secs.is_none());
    }

    #[test]
    fn test_webhook_log_struct() {
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
        assert!(log.success);
        assert_eq!(log.attempts, 1);
        assert!(log.error.is_none());
    }
}
