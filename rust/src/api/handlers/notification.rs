//! Handlers для Notification Policy API

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::NotificationPolicyManager;
use crate::models::notification::{
    NotificationPolicy, NotificationPolicyCreate, NotificationPolicyUpdate,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

/// GET /api/project/{project_id}/notifications
pub async fn list_notification_policies(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<NotificationPolicy>>, (StatusCode, Json<ErrorResponse>)> {
    let policies = state
        .store
        .get_notification_policies(project_id)
        .await
        .map_err(|e| {
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
    let policy = state
        .store
        .create_notification_policy(project_id, payload)
        .await
        .map_err(|e| {
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
    let policy = state
        .store
        .get_notification_policy(id, project_id)
        .await
        .map_err(|e| {
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
    let policy = state
        .store
        .update_notification_policy(id, project_id, payload)
        .await
        .map_err(|e| {
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
    state
        .store
        .delete_notification_policy(id, project_id)
        .await
        .map_err(|e| {
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
    let policy = state
        .store
        .get_notification_policy(id, project_id)
        .await
        .map_err(|e| {
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
                Json(ErrorResponse::new(format!(
                    "Failed to send test webhook: {}",
                    e
                ))),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::create_app;
    use crate::db::mock::MockStore;
    use axum::body::Body;
    use axum::http::Request;
    use serde_json::json;
    use std::sync::Arc;
    use tower::ServiceExt;

    async fn create_test_app() -> axum::Router {
        let store = Arc::new(MockStore::new());
        create_app(store).await
    }

    #[tokio::test]
    async fn test_list_notifications_returns_empty() {
        let app = create_test_app().await;
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/project/1/notifications")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_notification_empty_name_rejected() {
        let app = create_test_app().await;
        let body = json!({"name": "", "webhook_url": "http://example.com", "trigger": "on_failure"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/project/1/notifications")
                    .header("Content-Type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        // 400 from handler or 422 from axum validation
        assert!(resp.status() == StatusCode::BAD_REQUEST || resp.status() == StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_create_notification_empty_webhook_rejected() {
        let app = create_test_app().await;
        let body = json!({"name": "Test", "webhook_url": "", "trigger": "on_failure"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/project/1/notifications")
                    .header("Content-Type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        // 400 from handler or 422 from axum validation
        assert!(resp.status() == StatusCode::BAD_REQUEST || resp.status() == StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_create_notification_invalid_trigger_rejected() {
        let app = create_test_app().await;
        let body = json!({"name": "Test", "webhook_url": "http://example.com", "trigger": "invalid"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/project/1/notifications")
                    .header("Content-Type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(resp.status() == StatusCode::BAD_REQUEST || resp.status() == StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_create_notification_valid_triggers() {
        let triggers = ["on_failure", "on_success", "on_start", "always"];
        for trigger in &triggers {
            let app = create_test_app().await;
            let body = json!({"name": "Test", "webhook_url": "http://example.com", "trigger": trigger});
            let resp = app
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/project/1/notifications")
                        .header("Content-Type", "application/json")
                        .body(Body::from(body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            // MockStore may return 500 or 201
            assert!(resp.status() != StatusCode::BAD_REQUEST);
        }
    }

    #[tokio::test]
    async fn test_get_notification_not_found() {
        let app = create_test_app().await;
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/api/project/1/notifications/999")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_update_notification_empty_name_rejected() {
        let app = create_test_app().await;
        let body = json!({"name": "", "webhook_url": "http://example.com", "trigger": "on_failure"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/project/1/notifications/1")
                    .header("Content-Type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        // 400 from handler or 422 from axum validation
        assert!(resp.status() == StatusCode::BAD_REQUEST || resp.status() == StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_update_notification_invalid_trigger_rejected() {
        let app = create_test_app().await;
        let body = json!({"name": "Test", "webhook_url": "http://example.com", "trigger": "invalid"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/project/1/notifications/1")
                    .header("Content-Type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(resp.status() == StatusCode::BAD_REQUEST || resp.status() == StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_delete_notification_no_content() {
        let app = create_test_app().await;
        let resp = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/api/project/1/notifications/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // MockStore may return 204 or 404
        assert!(resp.status() == StatusCode::NO_CONTENT || resp.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_test_notification_not_found() {
        let app = create_test_app().await;
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/project/1/notifications/999/test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
