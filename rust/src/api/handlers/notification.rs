//! Handlers для Notification Policy API

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::api::middleware::ErrorResponse;
use crate::db::store::NotificationPolicyManager;
use crate::models::notification::{
    NotificationPolicy, NotificationPolicyCreate, NotificationPolicyUpdate,
};

/// GET /api/project/{project_id}/notifications
pub async fn list_notification_policies(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<NotificationPolicy>>, (StatusCode, Json<ErrorResponse>)> {
    let policies = state.store.get_notification_policies(project_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;
    Ok(Json(policies))
}

/// POST /api/project/{project_id}/notifications
pub async fn create_notification_policy(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<NotificationPolicyCreate>,
) -> Result<(StatusCode, Json<NotificationPolicy>), (StatusCode, Json<ErrorResponse>)> {
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Policy name is required".to_string())),
        ));
    }
    if payload.webhook_url.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Webhook URL is required".to_string())),
        ));
    }
    let valid_triggers = ["on_failure", "on_success", "on_start", "always"];
    if !valid_triggers.contains(&payload.trigger.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(format!(
                "Invalid trigger '{}'. Must be one of: on_failure, on_success, on_start, always",
                payload.trigger
            ))),
        ));
    }
    let policy = state.store.create_notification_policy(project_id, payload).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;
    Ok((StatusCode::CREATED, Json(policy)))
}

/// GET /api/project/{project_id}/notifications/{id}
pub async fn get_notification_policy(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<Json<NotificationPolicy>, (StatusCode, Json<ErrorResponse>)> {
    let policy = state.store.get_notification_policy(id, project_id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;
    Ok(Json(policy))
}

/// PUT /api/project/{project_id}/notifications/{id}
pub async fn update_notification_policy(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
    Json(payload): Json<NotificationPolicyUpdate>,
) -> Result<Json<NotificationPolicy>, (StatusCode, Json<ErrorResponse>)> {
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Policy name is required".to_string())),
        ));
    }
    if payload.webhook_url.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Webhook URL is required".to_string())),
        ));
    }
    let valid_triggers = ["on_failure", "on_success", "on_start", "always"];
    if !valid_triggers.contains(&payload.trigger.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(format!(
                "Invalid trigger '{}'. Must be one of: on_failure, on_success, on_start, always",
                payload.trigger
            ))),
        ));
    }
    let policy = state.store.update_notification_policy(id, project_id, payload).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;
    Ok(Json(policy))
}

/// DELETE /api/project/{project_id}/notifications/{id}
pub async fn delete_notification_policy(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_notification_policy(id, project_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/project/{project_id}/notifications/{id}/test
pub async fn test_notification_policy(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let policy = state.store.get_notification_policy(id, project_id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    let test_payload = serde_json::json!({
        "text": format!("[Semaphore] Test notification from policy: {}", policy.name),
        "event": "test",
        "policy_id": policy.id,
        "project_id": policy.project_id,
    });

    let client = reqwest::Client::new();
    client
        .post(&policy.webhook_url)
        .json(&test_payload)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse::new(format!("Failed to send test webhook: {}", e))),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}
