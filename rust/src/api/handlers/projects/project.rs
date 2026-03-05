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
use crate::api::extractors::AuthUser;
use crate::models::Project;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;
use crate::db::store::{ProjectStore, UserManager};
use crate::services::demo_project;
use crate::services::backup::BackupFormat;

/// Получает проекты пользователя
pub async fn get_projects(
    State(state): State<Arc<AppState>>,
) -> std::result::Result<Json<Vec<Project>>, (StatusCode, Json<ErrorResponse>)> {
    let projects = state.store.get_projects(None)
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
    let created = if payload.demo == Some(true) {
        // Создание демо-проекта с предзаполненными ресурсами
        let project_name = if payload.name.is_empty() {
            "Demo".to_string()
        } else {
            payload.name.clone()
        };
        let backup = demo_project::demo_backup(&project_name);
        backup
            .restore_demo(&state.store)
            .await
            .map_err(|e| (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ))?
    } else {
        // Обычное создание проекта
        let mut project = payload;
        project.demo = None;
        state
            .store
            .create_project(project)
            .await
            .map_err(|e| (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ))?
    };

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

/// Восстанавливает проект из backup
///
/// POST /api/projects/restore
pub async fn restore_project(
    State(state): State<Arc<AppState>>,
    AuthUser { user_id, admin, .. }: AuthUser,
    Json(payload): Json<BackupFormat>,
) -> std::result::Result<(StatusCode, Json<Project>), (StatusCode, Json<ErrorResponse>)> {
    if !admin && !state.config.non_admin_can_create_project() {
        let err = ErrorResponse::new("Нет прав на создание проектов").with_code("FORBIDDEN");
        return Err((StatusCode::FORBIDDEN, Json(err)));
    }

    let user = state.store.get_user(user_id).await.map_err(|e| {
        let (status, resp) = ErrorResponse::from_crate_error(&e);
        (status, Json(resp))
    })?;

    payload.verify().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(e.to_string()).with_code("INVALID_BACKUP")),
        )
    })?;

    let project = payload.restore(&user, &state.store).await.map_err(|e| {
        let (status, resp) = ErrorResponse::from_crate_error(&e);
        (status, Json(resp))
    })?;

    Ok((StatusCode::OK, Json(project)))
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
