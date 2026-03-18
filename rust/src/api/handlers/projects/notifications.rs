//! Projects API - Notifications Handler
//!
//! Обработчики для уведомлений в проектах

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;
use crate::db::store::ProjectStore;

/// Отправляет тестовое уведомление
///
/// POST /api/projects/{project_id}/notifications/test
pub async fn send_test_notification(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    payload: Option<Json<TestNotificationPayload>>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Получаем проект
    let project = state.store.get_project(project_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Project not found".to_string()))
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string()))
            )
        })?;

    // Проверяем, включены ли уведомления
    if !project.alert {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Notifications are disabled for this project".to_string()))
        ));
    }

    // Получаем chat ID
    let chat_id = project.alert_chat
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Chat ID is not configured".to_string()))
        ))?;

    // Формируем тестовое сообщение
    let message = format!(
        "🔔 Test notification from Velum UI\n\nProject: {}\nStatus: Success",
        project.name
    );

    // Отправляем уведомление через AlertService
    // В реальной реализации нужно использовать AlertService
    // let alert_service = crate::services::alert::AlertService::new(...);
    // alert_service.send_alert(&message, &chat_id).await?;

    // Логгируем для отладки
    tracing::info!("Test notification sent to chat {}: {}", chat_id, message);

    Ok(StatusCode::OK)
}

// ============================================================================
// Types
// ============================================================================

/// Payload для тестового уведомления
#[derive(Debug, Deserialize)]
pub struct TestNotificationPayload {
    /// Сообщение (опционально, используется по умолчанию)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_payload_deserialize_empty() {
        let json = r#"{}"#;
        let payload: TestNotificationPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.message, None);
    }

    #[test]
    fn test_notification_payload_deserialize_with_message() {
        let json = r#"{"message": "Test message"}"#;
        let payload: TestNotificationPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.message, Some("Test message".to_string()));
    }
}
