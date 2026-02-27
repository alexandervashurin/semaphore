//! Users Handlers
//!
//! Обработчики запросов для управления пользователями

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::Deserialize;
use crate::api::state::AppState;
use crate::models::User;
use crate::db::store::RetrieveQueryParams;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;

/// Получить список пользователей
///
/// GET /api/users
pub async fn get_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<User>>, (StatusCode, Json<ErrorResponse>)> {
    let users = state.store.get_users(RetrieveQueryParams::default())
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(users))
}

/// Получить пользователя по ID
///
/// GET /api/users/:id
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
    let user = state.store.get_user(user_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;

    Ok(Json(user))
}

/// Обновить пользователя
///
/// PUT /api/users/:id
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
    Json(payload): Json<UserUpdatePayload>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut user = state.store.get_user(user_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;

    if let Some(username) = payload.username {
        user.username = username;
    }
    if let Some(name) = payload.name {
        user.name = name;
    }
    if let Some(email) = payload.email {
        user.email = email;
    }

    state.store.update_user(user)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Удалить пользователя
///
/// DELETE /api/users/:id
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_user(user_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Types
// ============================================================================

/// Payload для обновления пользователя
#[derive(Debug, Deserialize)]
pub struct UserUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_update_payload_deserialize_all_fields() {
        let json = r#"{
            "username": "newuser",
            "name": "New Name",
            "email": "new@example.com"
        }"#;
        
        let payload: UserUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.username, Some("newuser".to_string()));
        assert_eq!(payload.name, Some("New Name".to_string()));
        assert_eq!(payload.email, Some("new@example.com".to_string()));
    }

    #[test]
    fn test_user_update_payload_deserialize_partial() {
        let json = r#"{
            "email": "new@example.com"
        }"#;
        
        let payload: UserUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.username, None);
        assert_eq!(payload.name, None);
        assert_eq!(payload.email, Some("new@example.com".to_string()));
    }

    #[test]
    fn test_user_update_payload_deserialize_empty() {
        let json = r#"{}"#;
        
        let payload: UserUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.username, None);
        assert_eq!(payload.name, None);
        assert_eq!(payload.email, None);
    }
}
