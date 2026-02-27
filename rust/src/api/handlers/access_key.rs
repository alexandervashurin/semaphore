//! Access Key Handlers
//!
//! Обработчики запросов для управления ключами доступа

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::models::AccessKey;
use crate::models::access_key::AccessKeyType;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;

/// Получить список ключей доступа проекта
///
/// GET /api/projects/:project_id/keys
pub async fn get_access_keys(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<AccessKey>>, (StatusCode, Json<ErrorResponse>)> {
    let keys = state.store.get_access_keys(project_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(keys))
}

/// Создать ключ доступа
///
/// POST /api/projects/:project_id/keys
pub async fn create_access_key(
    State(state): State<Arc<AppState>>,
    Path(_project_id): Path<i32>,
    Json(payload): Json<AccessKeyCreatePayload>,
) -> Result<(StatusCode, Json<AccessKey>), (StatusCode, Json<ErrorResponse>)> {
    let key = AccessKey::new(
        payload.name,
        payload.key_type,
    );

    let created = state.store.create_access_key(key)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Получить ключ доступа по ID
///
/// GET /api/projects/:project_id/keys/:key_id
pub async fn get_access_key(
    State(state): State<Arc<AppState>>,
    Path((project_id, key_id)): Path<(i32, i32)>,
) -> Result<Json<AccessKey>, (StatusCode, Json<ErrorResponse>)> {
    let key = state.store.get_access_key(project_id, key_id)
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

    Ok(Json(key))
}

/// Обновить ключ доступа
///
/// PUT /api/projects/:project_id/keys/:key_id
pub async fn update_access_key(
    State(state): State<Arc<AppState>>,
    Path((project_id, key_id)): Path<(i32, i32)>,
    Json(payload): Json<AccessKeyUpdatePayload>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut key = state.store.get_access_key(project_id, key_id)
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

    if let Some(name) = payload.name {
        key.name = name;
    }

    state.store.update_access_key(key)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Удалить ключ доступа
///
/// DELETE /api/projects/:project_id/keys/:key_id
pub async fn delete_access_key(
    State(state): State<Arc<AppState>>,
    Path((project_id, key_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_access_key(project_id, key_id)
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

/// Payload для создания ключа доступа
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessKeyCreatePayload {
    pub name: String,
    #[serde(rename = "type")]
    pub key_type: AccessKeyType,
}

/// Payload для обновления ключа доступа
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessKeyUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_key_create_payload_deserialize_ssh() {
        let json = r#"{
            "name": "SSH Key",
            "type": "ssh"
        }"#;
        let payload: AccessKeyCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "SSH Key");
        assert_eq!(payload.key_type, AccessKeyType::SSH);
    }

    #[test]
    fn test_access_key_create_payload_deserialize_login_password() {
        let json = r#"{
            "name": "Login Password",
            "type": "login_password"
        }"#;
        let payload: AccessKeyCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "Login Password");
        assert_eq!(payload.key_type, AccessKeyType::LoginPassword);
    }

    #[test]
    fn test_access_key_update_payload_deserialize() {
        let json = r#"{"name": "Updated Key"}"#;
        let payload: AccessKeyUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, Some("Updated Key".to_string()));
    }

    #[test]
    fn test_access_key_update_payload_deserialize_empty() {
        let json = r#"{}"#;
        let payload: AccessKeyUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, None);
    }
}
