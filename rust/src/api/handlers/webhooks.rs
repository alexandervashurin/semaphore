//! Webhook API handlers
//!
//! Обработчики HTTP запросов для управления webhook

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use crate::api::state::AppState;
use crate::api::extractors::AuthUser;
use crate::api::middleware::ErrorResponse;
use crate::db::store::ProjectStore;
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
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(ErrorResponse::new("Webhook not found"))))?;

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
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(ErrorResponse::new("Webhook not found"))))?;

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
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(ErrorResponse::new("Webhook not found"))))?;

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
    Json(payload): Option<Json<TestWebhook>>,
) -> std::result::Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let webhook = state.store.get_webhook(webhook_id).await
        .map_err(|e| {
            tracing::error!("Failed to get webhook: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(e.to_string())))
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(ErrorResponse::new("Webhook not found"))))?;

    if webhook.project_id != Some(project_id) {
        return Err((StatusCode::FORBIDDEN, Json(ErrorResponse::new("Webhook does not belong to project"))));
    }

    use crate::services::webhook::WebhookService;
    
    let test_url = payload.as_ref().map(|p| p.url.clone()).unwrap_or_else(|| webhook.url.clone());
    let test_type = payload.as_ref().map(|p| p.r#type.clone()).unwrap_or_else(|| webhook.r#type.clone());

    let result = WebhookService::send_test(&test_url, &test_type).await;

    match result {
        Ok(_) => Ok(Json(serde_json::json!({"success": true, "message": "Webhook test successful"}))),
        Err(e) => Ok(Json(serde_json::json!({"success": false, "error": e.to_string()}))),
    }
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
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(ErrorResponse::new("Webhook not found"))))?;

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
