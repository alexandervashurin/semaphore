//! Environment Handlers
//!
//! Обработчики запросов для управления окружениями

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::models::Environment;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;

/// Получить список окружений проекта
///
/// GET /api/projects/:project_id/environments
pub async fn get_environments(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<Environment>>, (StatusCode, Json<ErrorResponse>)> {
    let environments = state.store.get_environments(project_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

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
    let environment = Environment::new(
        project_id,
        payload.name,
        payload.json,
    );

    let created = state.store.create_environment(environment)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Получить окружение по ID
///
/// GET /api/projects/:project_id/environments/:environment_id
pub async fn get_environment(
    State(state): State<Arc<AppState>>,
    Path((project_id, environment_id)): Path<(i32, i32)>,
) -> Result<Json<Environment>, (StatusCode, Json<ErrorResponse>)> {
    let environment = state.store.get_environment(project_id, environment_id)
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
    let mut environment = state.store.get_environment(project_id, environment_id)
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

    state.store.update_environment(environment)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Удалить окружение
///
/// DELETE /api/projects/:project_id/environments/:environment_id
pub async fn delete_environment(
    State(state): State<Arc<AppState>>,
    Path((project_id, environment_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_environment(project_id, environment_id)
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
        assert_eq!(payload.json, Some("{\"DB_HOST\": \"staging.db\"}".to_string()));
    }

    #[test]
    fn test_environment_update_payload_deserialize_partial() {
        let json = r#"{"json": "{\"NEW_VAR\": \"value\"}"}"#;
        let payload: EnvironmentUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, None);
        assert_eq!(payload.json, Some("{\"NEW_VAR\": \"value\"}".to_string()));
    }
}
