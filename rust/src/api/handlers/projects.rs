//! Projects Handlers
//!
//! Обработчики запросов для управления проектами

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::models::Project;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;

/// Получить список проектов
///
/// GET /api/projects
pub async fn get_projects(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Project>>, (StatusCode, Json<ErrorResponse>)> {
    let projects = state.store.get_projects(None)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(projects))
}

/// Создать проект
///
/// POST /api/projects
pub async fn create_project(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ProjectCreatePayload>,
) -> Result<(StatusCode, Json<Project>), (StatusCode, Json<ErrorResponse>)> {
    let project = Project::new(payload.name);

    let created = state.store.create_project(project)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Получить проект по ID
///
/// GET /api/projects/:id
pub async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Project>, (StatusCode, Json<ErrorResponse>)> {
    let project = state.store.get_project(project_id)
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

    Ok(Json(project))
}

/// Обновить проект
///
/// PUT /api/projects/:id
pub async fn update_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<ProjectUpdatePayload>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut project = state.store.get_project(project_id)
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
        project.name = name;
    }
    if let Some(alert) = payload.alert {
        project.alert = Some(alert);
    }

    state.store.update_project(project)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Удалить проект
///
/// DELETE /api/projects/:id
pub async fn delete_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_project(project_id)
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

/// Payload для создания проекта
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectCreatePayload {
    pub name: String,
}

/// Payload для обновления проекта
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert: Option<bool>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_create_payload_deserialize() {
        let json = r#"{"name": "Test Project"}"#;
        let payload: ProjectCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "Test Project");
    }

    #[test]
    fn test_project_update_payload_deserialize_all_fields() {
        let json = r#"{
            "name": "Updated Project",
            "alert": true
        }"#;
        let payload: ProjectUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, Some("Updated Project".to_string()));
        assert_eq!(payload.alert, Some(true));
    }

    #[test]
    fn test_project_update_payload_deserialize_partial() {
        let json = r#"{"alert": false}"#;
        let payload: ProjectUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, None);
        assert_eq!(payload.alert, Some(false));
    }

    #[test]
    fn test_project_update_payload_deserialize_empty() {
        let json = r#"{}"#;
        let payload: ProjectUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, None);
        assert_eq!(payload.alert, None);
    }
}
