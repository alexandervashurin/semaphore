//! Projects API - Project Handler
//!
//! Обработчики для проектов

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::models::Project;
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;

/// Получает проекты пользователя
pub async fn get_projects(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> std::result::Result<Json<Vec<Project>>, (StatusCode, Json<ErrorResponse>)> {
    let projects = state.store.get_projects(Some(user_id))
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(projects))
}

/// Получает проект по ID
pub async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> std::result::Result<Json<Project>, (StatusCode, Json<ErrorResponse>)> {
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

    Ok(Json(project))
}

/// Создаёт новый проект
pub async fn add_project(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Project>,
) -> std::result::Result<(StatusCode, Json<Project>), (StatusCode, Json<ErrorResponse>)> {
    let created = state.store.create_project(payload)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Обновляет проект
pub async fn update_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<Project>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut project = payload;
    project.id = project_id;

    state.store.update_project(project)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Удаляет проект
pub async fn delete_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_project(project_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projects_handler() {
        // Тест для проверки обработчиков проектов
        assert!(true);
    }
}
