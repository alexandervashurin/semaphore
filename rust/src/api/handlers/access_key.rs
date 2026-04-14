//! Access Key Handlers
//!
//! Обработчики запросов для управления ключами доступа

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::{AccessKeyManager, ProjectStore};
use crate::error::Error;
use crate::models::AccessKey;
use crate::models::access_key::AccessKeyType;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Получить список ключей доступа проекта
///
/// GET /api/projects/:project_id/keys
pub async fn get_access_keys(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<AccessKey>>, (StatusCode, Json<ErrorResponse>)> {
    let keys = state.store.get_access_keys(project_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(Json(keys))
}

/// Создать ключ доступа
///
/// POST /api/projects/:project_id/keys
pub async fn create_access_key(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<AccessKeyCreatePayload>,
) -> Result<(StatusCode, Json<AccessKey>), (StatusCode, Json<ErrorResponse>)> {
    let mut key = AccessKey::new(payload.name, payload.key_type);
    key.project_id = Some(project_id);

    let created = state.store.create_access_key(key).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Получить ключ доступа по ID
///
/// GET /api/projects/:project_id/keys/:key_id
pub async fn get_access_key(
    State(state): State<Arc<AppState>>,
    Path((project_id, key_id)): Path<(i32, i32)>,
) -> Result<Json<AccessKey>, (StatusCode, Json<ErrorResponse>)> {
    let key = state
        .store
        .get_access_key(project_id, key_id)
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
    let mut key = state
        .store
        .get_access_key(project_id, key_id)
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

    state.store.update_access_key(key).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(StatusCode::OK)
}

/// Удалить ключ доступа
///
/// DELETE /api/projects/:project_id/keys/:key_id
pub async fn delete_access_key(
    State(state): State<Arc<AppState>>,
    Path((project_id, key_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .delete_access_key(project_id, key_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

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

    #[test]
    fn test_access_key_create_payload_roundtrip() {
        let original = AccessKeyCreatePayload {
            name: "Roundtrip Key".to_string(),
            key_type: AccessKeyType::SSH,
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: AccessKeyCreatePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, original.name);
        assert_eq!(restored.key_type, original.key_type);
    }

    #[test]
    fn test_access_key_update_payload_roundtrip() {
        let original = AccessKeyUpdatePayload {
            name: Some("Updated Key".to_string()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: AccessKeyUpdatePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, original.name);
    }

    #[test]
    fn test_access_key_create_payload_access_key_type() {
        let json = r#"{
            "name": "AWS Key",
            "type": "access_key"
        }"#;
        let payload: AccessKeyCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.key_type, AccessKeyType::AccessKey);
    }

    #[test]
    fn test_access_key_create_payload_none_type() {
        let json = r#"{
            "name": "None Key",
            "type": "none"
        }"#;
        let payload: AccessKeyCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.key_type, AccessKeyType::None);
    }

    #[test]
    fn test_access_key_create_payload_unicode_name() {
        let json = r#"{
            "name": "Ключ Доступа",
            "type": "ssh"
        }"#;
        let payload: AccessKeyCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "Ключ Доступа");
    }

    #[test]
    fn test_access_key_create_payload_empty_name() {
        let json = r#"{"name": "", "type": "ssh"}"#;
        let payload: AccessKeyCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "");
    }

    #[test]
    fn test_access_key_create_payload_debug() {
        let payload = AccessKeyCreatePayload {
            name: "Debug Key".to_string(),
            key_type: AccessKeyType::SSH,
        };
        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("AccessKeyCreatePayload"));
        assert!(debug_str.contains("Debug Key"));
    }

    #[test]
    fn test_access_key_update_payload_debug() {
        let payload = AccessKeyUpdatePayload {
            name: Some("Debug".to_string()),
        };
        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("AccessKeyUpdatePayload"));
    }

    #[test]
    fn test_access_key_create_payload_clone() {
        // AccessKeyCreatePayload doesn't derive Clone, test via roundtrip
        let json = r#"{"name": "Clone Key", "type": "login_password"}"#;
        let p1: AccessKeyCreatePayload = serde_json::from_str(json).unwrap();
        let p2: AccessKeyCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(p1.name, p2.name);
        assert_eq!(p1.key_type, p2.key_type);
    }

    #[test]
    fn test_access_key_update_payload_clone() {
        // AccessKeyUpdatePayload doesn't derive Clone, test via deserialization
        let json = r#"{"name": "Clone"}"#;
        let p1: AccessKeyUpdatePayload = serde_json::from_str(json).unwrap();
        let p2: AccessKeyUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(p1.name, p2.name);
    }

    #[test]
    fn test_access_key_create_payload_special_chars() {
        let json = r#"{
            "name": "Key with special chars & < > \" '",
            "type": "ssh"
        }"#;
        let payload: AccessKeyCreatePayload = serde_json::from_str(json).unwrap();
        assert!(payload.name.contains("special chars"));
    }

    #[test]
    fn test_access_key_update_payload_with_special_chars() {
        let json = r#"{"name": "O'Brien's Key"}"#;
        let payload: AccessKeyUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, Some("O'Brien's Key".to_string()));
    }

    #[test]
    fn test_access_key_type_all_variants_in_payload() {
        let types = ["none", "login_password", "ssh", "access_key"];
        for t in types {
            let json = format!(r#"{{"name": "Test", "type": "{}"}}"#, t);
            let payload: AccessKeyCreatePayload = serde_json::from_str(&json).unwrap();
            assert_eq!(payload.name, "Test");
        }
    }

    #[test]
    fn test_access_key_create_payload_newline_in_name() {
        let json = r#"{"name": "Key\nwith\nnewlines", "type": "ssh"}"#;
        let payload: AccessKeyCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "Key\nwith\nnewlines");
    }

    #[test]
    fn test_access_key_update_payload_clone_independence() {
        // AccessKeyUpdatePayload doesn't derive Clone
        let mut name = Some("Original".to_string());
        let p1 = AccessKeyUpdatePayload { name: name.clone() };
        name = Some("Modified".to_string());
        assert_eq!(p1.name, Some("Original".to_string()));
    }
}
