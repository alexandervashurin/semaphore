//! Environment Handlers
//!
//! Обработчики запросов для управления окружениями

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::EnvironmentManager;
use crate::error::Error;
use crate::models::Environment;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Получить список окружений проекта
///
/// GET /api/projects/:project_id/environments
pub async fn get_environments(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<Environment>>, (StatusCode, Json<ErrorResponse>)> {
    let environments = state
        .store
        .get_environments(project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(Json(environments))
}

/// Создать окружение
///
/// POST /api/projects/:project_id/environments
pub async fn create_environment(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<EnvironmentCreatePayload>,
) -> Result<(StatusCode, Json<Environment>), (StatusCode, Json<ErrorResponse>)> {
    let environment = Environment::new(project_id, payload.name, payload.json);

    let created = state
        .store
        .create_environment(environment)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Получить окружение по ID
///
/// GET /api/projects/:project_id/environments/:environment_id
pub async fn get_environment(
    State(state): State<Arc<AppState>>,
    Path((project_id, environment_id)): Path<(i32, i32)>,
) -> Result<Json<Environment>, (StatusCode, Json<ErrorResponse>)> {
    let environment = state
        .store
        .get_environment(project_id, environment_id)
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

    Ok(Json(environment))
}

/// Обновить окружение
///
/// PUT /api/projects/:project_id/environments/:environment_id
pub async fn update_environment(
    State(state): State<Arc<AppState>>,
    Path((project_id, environment_id)): Path<(i32, i32)>,
    Json(payload): Json<EnvironmentUpdatePayload>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut environment = state
        .store
        .get_environment(project_id, environment_id)
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
        environment.name = name;
    }
    if let Some(json) = payload.json {
        environment.json = json;
    }

    state
        .store
        .update_environment(environment)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(StatusCode::OK)
}

/// Удалить окружение
///
/// DELETE /api/projects/:project_id/environments/:environment_id
pub async fn delete_environment(
    State(state): State<Arc<AppState>>,
    Path((project_id, environment_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .delete_environment(project_id, environment_id)
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

/// Payload для создания окружения
#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentCreatePayload {
    pub name: String,
    pub json: String,
}

/// Payload для обновления окружения
#[derive(Debug, Serialize, Deserialize)]
pub struct EnvironmentUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_create_payload_deserialize() {
        let json = r#"{
            "name": "Production",
            "json": "{\"DB_HOST\": \"prod.db\"}"
        }"#;
        let payload: EnvironmentCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "Production");
        assert_eq!(payload.json, "{\"DB_HOST\": \"prod.db\"}");
    }

    #[test]
    fn test_environment_update_payload_deserialize_all_fields() {
        let json = r#"{
            "name": "Staging",
            "json": "{\"DB_HOST\": \"staging.db\"}"
        }"#;
        let payload: EnvironmentUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, Some("Staging".to_string()));
        assert_eq!(
            payload.json,
            Some("{\"DB_HOST\": \"staging.db\"}".to_string())
        );
    }

    #[test]
    fn test_environment_update_payload_deserialize_partial() {
        let json = r#"{"json": "{\"NEW_VAR\": \"value\"}"}"#;
        let payload: EnvironmentUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, None);
        assert_eq!(payload.json, Some("{\"NEW_VAR\": \"value\"}".to_string()));
    }

    #[test]
    fn test_environment_create_payload_roundtrip() {
        let original = EnvironmentCreatePayload {
            name: "Roundtrip".to_string(),
            json: "{\"KEY\": \"value\"}".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: EnvironmentCreatePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, original.name);
        assert_eq!(restored.json, original.json);
    }

    #[test]
    fn test_environment_update_payload_roundtrip() {
        let original = EnvironmentUpdatePayload {
            name: Some("Updated".to_string()),
            json: Some("{\"DB\": \"updated\"}".to_string()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: EnvironmentUpdatePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, original.name);
        assert_eq!(restored.json, original.json);
    }

    #[test]
    fn test_environment_update_payload_all_null() {
        let json = r#"{"name": null, "json": null}"#;
        let payload: EnvironmentUpdatePayload = serde_json::from_str(json).unwrap();
        assert!(payload.name.is_none());
        assert!(payload.json.is_none());
    }

    #[test]
    fn test_environment_update_payload_empty() {
        let json = r#"{}"#;
        let payload: EnvironmentUpdatePayload = serde_json::from_str(json).unwrap();
        assert!(payload.name.is_none());
        assert!(payload.json.is_none());
    }

    #[test]
    fn test_environment_create_payload_empty_fields() {
        let json = r#"{"name": "", "json": ""}"#;
        let payload: EnvironmentCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "");
        assert_eq!(payload.json, "");
    }

    #[test]
    fn test_environment_create_payload_complex_json() {
        let json = r#"{
            "name": "Complex Env",
            "json": "{\"DB_HOST\": \"db.example.com\", \"DB_PORT\": 5432, \"FEATURE_FLAGS\": {\"dark_mode\": true}}"
        }"#;
        let payload: EnvironmentCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "Complex Env");
        assert!(payload.json.contains("DB_HOST"));
    }

    #[test]
    fn test_environment_create_payload_unicode() {
        let json = r#"{
            "name": "Окружение Продакшн",
            "json": "{\"Хост\": \"бд.example.com\"}"
        }"#;
        let payload: EnvironmentCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "Окружение Продакшн");
    }

    #[test]
    fn test_environment_create_payload_debug() {
        let payload = EnvironmentCreatePayload {
            name: "Debug".to_string(),
            json: "{}".to_string(),
        };
        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("EnvironmentCreatePayload"));
        assert!(debug_str.contains("Debug"));
    }

    #[test]
    fn test_environment_update_payload_debug() {
        let payload = EnvironmentUpdatePayload {
            name: Some("Debug".to_string()),
            json: None,
        };
        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("EnvironmentUpdatePayload"));
    }

    #[test]
    fn test_environment_create_payload_clone_independence() {
        // EnvironmentCreatePayload doesn't derive Clone
        let json = r#"{"name": "Original", "json": "{}"}"#;
        let p1: EnvironmentCreatePayload = serde_json::from_str(json).unwrap();
        let p2: EnvironmentCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(p1.name, p2.name);
    }

    #[test]
    fn test_environment_update_payload_clone_independence() {
        // EnvironmentUpdatePayload doesn't derive Clone
        let json = r#"{"name": "Original", "json": "{}"}"#;
        let p1: EnvironmentUpdatePayload = serde_json::from_str(json).unwrap();
        let p2: EnvironmentUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(p1.name, p2.name);
    }

    #[test]
    fn test_environment_update_payload_single_field_name() {
        let json = r#"{"name": "Renamed"}"#;
        let payload: EnvironmentUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, Some("Renamed".to_string()));
        assert!(payload.json.is_none());
    }

    #[test]
    fn test_environment_update_payload_single_field_json() {
        let json = r#"{"json": "{\"A\": 1}"}"#;
        let payload: EnvironmentUpdatePayload = serde_json::from_str(json).unwrap();
        assert!(payload.name.is_none());
        assert_eq!(payload.json, Some("{\"A\": 1}".to_string()));
    }

    #[test]
    fn test_environment_create_payload_json_with_escaped_quotes() {
        let json = r#"{
            "name": "Escaped",
            "json": "{\"message\": \"Hello \\\"World\\\"\"}"
        }"#;
        let payload: EnvironmentCreatePayload = serde_json::from_str(json).unwrap();
        assert!(payload.json.contains("Hello"));
    }

    #[test]
    fn test_environment_create_with_empty_json_object() {
        let payload = EnvironmentCreatePayload {
            name: "Empty JSON".to_string(),
            json: "{}".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        let restored: EnvironmentCreatePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.json, "{}");
    }

    #[test]
    fn test_environment_update_payload_newline_in_json() {
        let payload = EnvironmentUpdatePayload {
            name: None,
            json: Some("{\"line1\": \"value1\",\n\"line2\": \"value2\"}".to_string()),
        };
        let json = serde_json::to_string(&payload).unwrap();
        let restored: EnvironmentUpdatePayload = serde_json::from_str(&json).unwrap();
        assert!(restored.json.as_ref().unwrap().contains("line1"));
    }
}
