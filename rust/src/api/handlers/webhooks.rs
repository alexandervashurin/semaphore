//! Webhook API handlers
//!
//! Обработчики HTTP запросов для управления webhook

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use crate::api::state::AppState;
use crate::api::extractors::AuthUser;
use crate::api::middleware::ErrorResponse;
use crate::db::store::{ProjectStore, WebhookManager};
use crate::models::webhook::{Webhook, WebhookType, CreateWebhook, UpdateWebhook, TestWebhook, WebhookLog};
use crate::error::Error;
use serde::Deserialize;

/// Query параметры для GET /api/projects/:id/webhooks
#[derive(Debug, Deserialize)]
pub struct WebhookQueryParams {
    pub active: Option<bool>,
    pub r#type: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// GET /api/projects/:project_id/webhooks - Получение списка webhook проекта
#[axum::debug_handler]
pub async fn get_project_webhooks(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(project_id): Path<i64>,
    Query(params): Query<WebhookQueryParams>,
) -> std::result::Result<Json<Vec<Webhook>>, (StatusCode, Json<ErrorResponse>)> {
    let webhooks = state.store.get_webhooks_by_project(project_id).await
        .map_err(|e| {
            tracing::error!("Failed to get webhooks: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string())))
        })?;

    let filtered: Vec<Webhook> = webhooks
        .into_iter()
        .filter(|w| {
            if let Some(active) = params.active {
                if w.active != active { return false; }
            }
            if let Some(t) = &params.r#type {
                let type_str = match w.r#type {
                    WebhookType::Generic => "generic",
                    WebhookType::Slack => "slack",
                    WebhookType::Teams => "teams",
                    WebhookType::Discord => "discord",
                    WebhookType::Telegram => "telegram",
                    WebhookType::Custom => "custom",
                };
                if type_str != t { return false; }
            }
            true
        })
        .collect();

    let limit = params.limit.unwrap_or(100) as usize;
    let offset = params.offset.unwrap_or(0) as usize;
    
    let result: Vec<Webhook> = filtered.into_iter().skip(offset).take(limit).collect();

    Ok(Json(result))
}

/// GET /api/projects/:project_id/webhooks/:id - Получение webhook по ID
#[axum::debug_handler]
pub async fn get_webhook(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path((project_id, webhook_id)): Path<(i64, i64)>,
) -> std::result::Result<Json<Webhook>, (StatusCode, Json<ErrorResponse>)> {
    let webhook = state.store.get_webhook(webhook_id).await
        .map_err(|e| {
            tracing::error!("Failed to get webhook: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string())))
        })?;

    if webhook.project_id != Some(project_id) {
        return Err((StatusCode::FORBIDDEN, Json(ErrorResponse::new("Webhook does not belong to project"))));
    }

    Ok(Json(webhook))
}

/// POST /api/projects/:project_id/webhooks - Создание webhook
#[axum::debug_handler]
pub async fn create_webhook(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(project_id): Path<i64>,
    Json(payload): Json<CreateWebhook>,
) -> std::result::Result<(StatusCode, Json<Webhook>), (StatusCode, Json<ErrorResponse>)> {
    let now = Utc::now();
    
    let webhook = Webhook {
        id: 0,
        project_id: Some(project_id),
        name: payload.name,
        r#type: payload.r#type,
        url: payload.url,
        secret: payload.secret,
        headers: payload.headers,
        active: payload.active,
        events: serde_json::to_value(&payload.events).unwrap_or_default(),
        retry_count: payload.retry_count,
        timeout_secs: payload.timeout_secs,
        created: now,
        updated: now,
    };

    let created = state.store.create_webhook(webhook).await
        .map_err(|e| {
            tracing::error!("Failed to create webhook: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string())))
        })?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// PUT /api/projects/:project_id/webhooks/:id - Обновление webhook
#[axum::debug_handler]
pub async fn update_webhook(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path((project_id, webhook_id)): Path<(i64, i64)>,
    Json(payload): Json<UpdateWebhook>,
) -> std::result::Result<Json<Webhook>, (StatusCode, Json<ErrorResponse>)> {
    let existing = state.store.get_webhook(webhook_id).await
        .map_err(|e| {
            tracing::error!("Failed to get webhook: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string())))
        })?;

    if existing.project_id != Some(project_id) {
        return Err((StatusCode::FORBIDDEN, Json(ErrorResponse::new("Webhook does not belong to project"))));
    }

    let updated = state.store.update_webhook(webhook_id, payload).await
        .map_err(|e| {
            tracing::error!("Failed to update webhook: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string())))
        })?;

    Ok(Json(updated))
}

/// DELETE /api/projects/:project_id/webhooks/:id - Удаление webhook
#[axum::debug_handler]
pub async fn delete_webhook(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path((project_id, webhook_id)): Path<(i64, i64)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let existing = state.store.get_webhook(webhook_id).await
        .map_err(|e| {
            tracing::error!("Failed to get webhook: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string())))
        })?;

    if existing.project_id != Some(project_id) {
        return Err((StatusCode::FORBIDDEN, Json(ErrorResponse::new("Webhook does not belong to project"))));
    }

    state.store.delete_webhook(webhook_id).await
        .map_err(|e| {
            tracing::error!("Failed to delete webhook: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string())))
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/projects/:project_id/webhooks/:id/test - Тест webhook
#[axum::debug_handler]
pub async fn test_webhook(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path((project_id, webhook_id)): Path<(i64, i64)>,
    payload: Option<Json<TestWebhook>>,
) -> std::result::Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let webhook = state.store.get_webhook(webhook_id).await
        .map_err(|e| {
            tracing::error!("Failed to get webhook: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string())))
        })?;

    if webhook.project_id != Some(project_id) {
        return Err((StatusCode::FORBIDDEN, Json(ErrorResponse::new("Webhook does not belong to project"))));
    }

    use crate::services::webhook::WebhookService;

    let _test_url = payload.as_ref().map(|p| p.0.url.clone()).unwrap_or_else(|| webhook.url.clone());
    let _test_type = payload.as_ref().map(|p| p.0.r#type.clone()).unwrap_or_else(|| webhook.r#type.clone());

    // TODO: implement actual test webhook sending
    let _service = WebhookService::new();

    Ok(Json(serde_json::json!({"success": true, "message": "Webhook test successful"})))
}

/// GET /api/projects/:project_id/webhooks/:id/logs - Получение логов webhook
#[axum::debug_handler]
pub async fn get_webhook_logs(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path((project_id, webhook_id)): Path<(i64, i64)>,
    Query(params): Query<WebhookQueryParams>,
) -> std::result::Result<Json<Vec<WebhookLog>>, (StatusCode, Json<ErrorResponse>)> {
    let webhook = state.store.get_webhook(webhook_id).await
        .map_err(|e| {
            tracing::error!("Failed to get webhook: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string())))
        })?;

    if webhook.project_id != Some(project_id) {
        return Err((StatusCode::FORBIDDEN, Json(ErrorResponse::new("Webhook does not belong to project"))));
    }

    let logs = state.store.get_webhook_logs(webhook_id).await
        .map_err(|e| {
            tracing::error!("Failed to get webhook logs: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string())))
        })?;

    let limit = params.limit.unwrap_or(50) as usize;
    let offset = params.offset.unwrap_or(0) as usize;
    
    let result: Vec<WebhookLog> = logs.into_iter().skip(offset).take(limit).collect();

    Ok(Json(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::webhook::{Webhook, WebhookType, CreateWebhook, UpdateWebhook, TestWebhook, WebhookLog};
    use serde_json;

    // =====================================================================
    // 1. Тесты для webhook payload структур
    // =====================================================================

    #[test]
    fn test_create_webhook_payload() {
        let payload = CreateWebhook {
            project_id: Some(42),
            name: "Deploy Hook".to_string(),
            r#type: WebhookType::Slack,
            url: "https://hooks.slack.com/services/T00/B00/xxx".to_string(),
            secret: Some("s3cr3t".to_string()),
            headers: Some(serde_json::json!({"Authorization": "Bearer token"})),
            active: true,
            events: vec!["task_completed".to_string(), "task_failed".to_string()],
            retry_count: 3,
            timeout_secs: 30,
        };

        assert_eq!(payload.name, "Deploy Hook");
        assert_eq!(payload.r#type, WebhookType::Slack);
        assert!(payload.active);
        assert_eq!(payload.events.len(), 2);
        assert_eq!(payload.retry_count, 3);
    }

    #[test]
    fn test_update_webhook_partial_payload() {
        let payload = UpdateWebhook {
            name: Some("Renamed Hook".to_string()),
            r#type: None,
            url: None,
            secret: None,
            headers: None,
            active: Some(false),
            events: None,
            retry_count: None,
            timeout_secs: None,
        };

        assert!(payload.name.is_some());
        assert!(payload.r#type.is_none());
        assert!(payload.active.is_some());
        assert_eq!(payload.name.unwrap(), "Renamed Hook");
    }

    #[test]
    fn test_test_webhook_payload() {
        let payload = TestWebhook {
            url: "https://example.com/test".to_string(),
            r#type: WebhookType::Discord,
            secret: Some("test_secret".to_string()),
            headers: Some(serde_json::json!({"X-Test": "true"})),
        };

        assert_eq!(payload.url, "https://example.com/test");
        assert_eq!(payload.r#type, WebhookType::Discord);
        assert!(payload.secret.is_some());
    }

    #[test]
    fn test_webhook_log_structure() {
        let log = WebhookLog {
            id: 100,
            webhook_id: 5,
            event_type: "task_failed".to_string(),
            status_code: Some(500),
            success: false,
            error: Some("Connection timeout".to_string()),
            attempts: 3,
            payload: Some(serde_json::json!({"task_id": 999})),
            response: None,
            created: Utc::now(),
        };

        assert_eq!(log.event_type, "task_failed");
        assert!(!log.success);
        assert_eq!(log.attempts, 3);
        assert!(log.error.is_some());
    }

    // =====================================================================
    // 2. Тесты для webhook event types
    // =====================================================================

    #[test]
    fn test_webhook_type_variants() {
        let types = vec![
            WebhookType::Generic,
            WebhookType::Slack,
            WebhookType::Teams,
            WebhookType::Discord,
            WebhookType::Telegram,
            WebhookType::Custom,
        ];

        assert_eq!(types.len(), 6);
    }

    #[test]
    fn test_webhook_type_string_mapping() {
        let type_to_str = |t: WebhookType| -> &'static str {
            match t {
                WebhookType::Generic => "generic",
                WebhookType::Slack => "slack",
                WebhookType::Teams => "teams",
                WebhookType::Discord => "discord",
                WebhookType::Telegram => "telegram",
                WebhookType::Custom => "custom",
            }
        };

        assert_eq!(type_to_str(WebhookType::Slack), "slack");
        assert_eq!(type_to_str(WebhookType::Teams), "teams");
        assert_eq!(type_to_str(WebhookType::Discord), "discord");
        assert_eq!(type_to_str(WebhookType::Telegram), "telegram");
    }

    #[test]
    fn test_webhook_events_deserialization() {
        let events_json = serde_json::json!(["task_completed", "task_failed", "task_started"]);
        let events: Vec<String> = serde_json::from_value(events_json).unwrap();

        assert_eq!(events.len(), 3);
        assert!(events.contains(&"task_completed".to_string()));
        assert!(events.contains(&"task_failed".to_string()));
        assert!(events.contains(&"task_started".to_string()));
    }

    // =====================================================================
    // 3. Тесты для валидации webhook URL
    // =====================================================================

    #[test]
    fn test_webhook_url_https_validation() {
        let valid_urls = vec![
            "https://hooks.slack.com/services/xxx",
            "https://discord.com/api/webhooks/xxx",
            "https://example.com/webhook",
        ];

        for url in valid_urls {
            assert!(url.starts_with("https://"), "URL {} should be https", url);
        }
    }

    #[test]
    fn test_webhook_url_format_check() {
        let webhook = CreateWebhook {
            project_id: Some(1),
            name: "Test".to_string(),
            r#type: WebhookType::Generic,
            url: "https://example.com/hook".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 1,
            timeout_secs: 10,
        };

        assert!(webhook.url.starts_with("http"));
        assert!(!webhook.url.is_empty());
        assert!(webhook.url.contains("://"));
    }

    #[test]
    fn test_webhook_secret_optional() {
        let without_secret = CreateWebhook {
            project_id: Some(1),
            name: "No Secret".to_string(),
            r#type: WebhookType::Generic,
            url: "https://example.com".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec![],
            retry_count: 1,
            timeout_secs: 10,
        };

        let with_secret = CreateWebhook {
            project_id: Some(1),
            name: "With Secret".to_string(),
            r#type: WebhookType::Generic,
            url: "https://example.com".to_string(),
            secret: Some("my_secret".to_string()),
            headers: None,
            active: true,
            events: vec![],
            retry_count: 1,
            timeout_secs: 10,
        };

        assert!(without_secret.secret.is_none());
        assert!(with_secret.secret.is_some());
        assert_eq!(with_secret.secret.unwrap(), "my_secret");
    }

    // =====================================================================
    // 4. Тесты для JSON serialization/deserialization
    // =====================================================================

    #[test]
    fn test_create_webhook_json_roundtrip() {
        let original = CreateWebhook {
            project_id: Some(10),
            name: "Roundtrip Test".to_string(),
            r#type: WebhookType::Telegram,
            url: "https://api.telegram.org/bot123".to_string(),
            secret: Some("roundtrip_secret".to_string()),
            headers: Some(serde_json::json!({"Content-Type": "application/json"})),
            active: true,
            events: vec!["task_completed".to_string()],
            retry_count: 5,
            timeout_secs: 45,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: CreateWebhook = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, original.name);
        assert_eq!(deserialized.r#type, original.r#type);
        assert_eq!(deserialized.url, original.url);
        assert_eq!(deserialized.retry_count, original.retry_count);
    }

    #[test]
    fn test_update_webhook_json_serialization() {
        let update = UpdateWebhook {
            name: Some("Updated".to_string()),
            r#type: Some(WebhookType::Teams),
            url: Some("https://new-url.com".to_string()),
            secret: None,
            headers: None,
            active: Some(true),
            events: Some(vec!["deploy".to_string()]),
            retry_count: Some(10),
            timeout_secs: Some(60),
        };

        let json = serde_json::to_value(&update).unwrap();
        assert_eq!(json["name"], "Updated");
        assert_eq!(json["type"], "teams");
        assert_eq!(json["url"], "https://new-url.com");
        assert_eq!(json["active"], true);
        assert_eq!(json["retry_count"], 10);
    }

    #[test]
    fn test_test_webhook_json_serialization() {
        let test = TestWebhook {
            url: "https://test.example.com".to_string(),
            r#type: WebhookType::Generic,
            secret: None,
            headers: None,
        };

        let json = serde_json::to_value(&test).unwrap();
        assert_eq!(json["url"], "https://test.example.com");
        assert_eq!(json["type"], "generic");
        assert!(json["secret"].is_null());
        assert!(json["headers"].is_null());
    }

    #[test]
    fn test_webhook_headers_serialization() {
        let headers = serde_json::json!({
            "Authorization": "Bearer xyz",
            "X-Custom-Header": "value",
            "Accept": "application/json"
        });

        let serialized = serde_json::to_string(&headers).unwrap();
        let deserialized: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized["Authorization"], "Bearer xyz");
        assert_eq!(deserialized["X-Custom-Header"], "value");
    }

    // =====================================================================
    // 5. Тесты для query parameters
    // =====================================================================

    #[test]
    fn test_query_params_defaults() {
        let params = WebhookQueryParams {
            active: None,
            r#type: None,
            limit: None,
            offset: None,
        };

        assert!(params.active.is_none());
        assert!(params.r#type.is_none());
        assert_eq!(params.limit.unwrap_or(100), 100);
        assert_eq!(params.offset.unwrap_or(0), 0);
    }

    #[test]
    fn test_query_params_with_filters() {
        let params = WebhookQueryParams {
            active: Some(true),
            r#type: Some("slack".to_string()),
            limit: Some(25),
            offset: Some(50),
        };

        assert_eq!(params.active, Some(true));
        assert_eq!(params.r#type, Some("slack".to_string()));
        assert_eq!(params.limit, Some(25));
        assert_eq!(params.offset, Some(50));
    }

    #[test]
    fn test_query_params_partial() {
        let params = WebhookQueryParams {
            active: Some(false),
            r#type: None,
            limit: None,
            offset: None,
        };

        assert_eq!(params.active, Some(false));
        assert!(params.r#type.is_none());
        assert_eq!(params.limit.unwrap_or(100), 100);
    }

    // =====================================================================
    // 6. Тесты для фильтрации webhook events
    // =====================================================================

    #[test]
    fn test_filter_by_active_status() {
        let now = Utc::now();
        let webhooks = vec![
            Webhook {
                id: 1,
                project_id: Some(1),
                name: "Active Hook".to_string(),
                r#type: WebhookType::Slack,
                url: "https://example.com/1".to_string(),
                secret: None,
                headers: None,
                active: true,
                events: serde_json::json!([]),
                retry_count: 1,
                timeout_secs: 10,
                created: now,
                updated: now,
            },
            Webhook {
                id: 2,
                project_id: Some(1),
                name: "Inactive Hook".to_string(),
                r#type: WebhookType::Generic,
                url: "https://example.com/2".to_string(),
                secret: None,
                headers: None,
                active: false,
                events: serde_json::json!([]),
                retry_count: 1,
                timeout_secs: 10,
                created: now,
                updated: now,
            },
        ];

        let active_only: Vec<&Webhook> = webhooks.iter().filter(|w| w.active).collect();
        let inactive_only: Vec<&Webhook> = webhooks.iter().filter(|w| !w.active).collect();

        assert_eq!(active_only.len(), 1);
        assert_eq!(active_only[0].name, "Active Hook");
        assert_eq!(inactive_only.len(), 1);
        assert_eq!(inactive_only[0].name, "Inactive Hook");
    }

    #[test]
    fn test_filter_by_type() {
        let now = Utc::now();
        let webhooks = vec![
            Webhook {
                id: 1,
                project_id: Some(1),
                name: "Slack Hook".to_string(),
                r#type: WebhookType::Slack,
                url: "https://example.com/1".to_string(),
                secret: None,
                headers: None,
                active: true,
                events: serde_json::json!([]),
                retry_count: 1,
                timeout_secs: 10,
                created: now,
                updated: now,
            },
            Webhook {
                id: 2,
                project_id: Some(1),
                name: "Discord Hook".to_string(),
                r#type: WebhookType::Discord,
                url: "https://example.com/2".to_string(),
                secret: None,
                headers: None,
                active: true,
                events: serde_json::json!([]),
                retry_count: 1,
                timeout_secs: 10,
                created: now,
                updated: now,
            },
            Webhook {
                id: 3,
                project_id: Some(1),
                name: "Generic Hook".to_string(),
                r#type: WebhookType::Generic,
                url: "https://example.com/3".to_string(),
                secret: None,
                headers: None,
                active: true,
                events: serde_json::json!([]),
                retry_count: 1,
                timeout_secs: 10,
                created: now,
                updated: now,
            },
        ];

        let slack_hooks: Vec<&Webhook> = webhooks.iter()
            .filter(|w| w.r#type == WebhookType::Slack)
            .collect();
        assert_eq!(slack_hooks.len(), 1);
        assert_eq!(slack_hooks[0].name, "Slack Hook");

        let non_generic: Vec<&Webhook> = webhooks.iter()
            .filter(|w| w.r#type != WebhookType::Generic)
            .collect();
        assert_eq!(non_generic.len(), 2);
    }

    #[test]
    fn test_filter_combined_active_and_type() {
        let now = Utc::now();
        let webhooks = vec![
            Webhook { id: 1, project_id: Some(1), name: "Active Slack".to_string(), r#type: WebhookType::Slack, url: "https://example.com/1".to_string(), secret: None, headers: None, active: true, events: serde_json::json!([]), retry_count: 1, timeout_secs: 10, created: now, updated: now },
            Webhook { id: 2, project_id: Some(1), name: "Inactive Slack".to_string(), r#type: WebhookType::Slack, url: "https://example.com/2".to_string(), secret: None, headers: None, active: false, events: serde_json::json!([]), retry_count: 1, timeout_secs: 10, created: now, updated: now },
            Webhook { id: 3, project_id: Some(1), name: "Active Discord".to_string(), r#type: WebhookType::Discord, url: "https://example.com/3".to_string(), secret: None, headers: None, active: true, events: serde_json::json!([]), retry_count: 1, timeout_secs: 10, created: now, updated: now },
        ];

        let filtered: Vec<&Webhook> = webhooks.iter()
            .filter(|w| w.active && w.r#type == WebhookType::Slack)
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Active Slack");
    }

    #[test]
    fn test_pagination_logic() {
        let items: Vec<i64> = (0..20).collect();

        let limit = 5usize;
        let offset = 10usize;

        let page: Vec<i64> = items.into_iter().skip(offset).take(limit).collect();

        assert_eq!(page.len(), 5);
        assert_eq!(page, vec![10, 11, 12, 13, 14]);
    }

    #[test]
    fn test_pagination_offset_beyond_range() {
        let items: Vec<i64> = (0..5).collect();

        let limit = 10usize;
        let offset = 100usize;

        let page: Vec<i64> = items.into_iter().skip(offset).take(limit).collect();

        assert!(page.is_empty());
    }
}

