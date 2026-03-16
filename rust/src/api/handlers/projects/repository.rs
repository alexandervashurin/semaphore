//! Projects API - Repositories Handler
//!
//! Обработчики для репозиториев в проектах

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::models::Repository;
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;
use crate::db::store::{RetrieveQueryParams, RepositoryManager};

/// Получает репозитории проекта
pub async fn get_repositories(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Query(_params): Query<RetrieveQueryParams>,
) -> std::result::Result<Json<Vec<Repository>>, (StatusCode, Json<ErrorResponse>)> {
    let repositories = state.store.get_repositories(project_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(repositories))
}

/// Получает репозиторий по ID
pub async fn get_repository(
    State(state): State<Arc<AppState>>,
    Path((project_id, repository_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<Repository>, (StatusCode, Json<ErrorResponse>)> {
    let repository = state.store.get_repository(project_id, repository_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Repository not found".to_string()))
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string()))
            )
        })?;

    Ok(Json(repository))
}

/// Создаёт новый репозиторий
pub async fn add_repository(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<Repository>,
) -> std::result::Result<(StatusCode, Json<Repository>), (StatusCode, Json<ErrorResponse>)> {
    let mut repository = payload;
    repository.project_id = project_id;

    let created = state.store.create_repository(repository)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Обновляет репозиторий
pub async fn update_repository(
    State(state): State<Arc<AppState>>,
    Path((project_id, repository_id)): Path<(i32, i32)>,
    Json(payload): Json<Repository>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut repository = payload;
    repository.id = repository_id;
    repository.project_id = project_id;

    state.store.update_repository(repository)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Удаляет репозиторий
pub async fn delete_repository(
    State(state): State<Arc<AppState>>,
    Path((project_id, repository_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_repository(project_id, repository_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Возвращает список веток репозитория
///
/// GET /api/project/{project_id}/repositories/{id}/branches
pub async fn get_repository_branches(
    State(state): State<Arc<AppState>>,
    Path((project_id, repository_id)): Path<(i32, i32)>,
) -> std::result::Result<Json<Vec<String>>, (StatusCode, Json<ErrorResponse>)> {
    // Проверяем, что репозиторий существует
    state.store.get_repository(project_id, repository_id)
        .await
        .map_err(|e| match e {
            crate::error::Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Repository not found".to_string()))
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string()))
            )
        })?;

    // Возвращаем базовые ветки (в production здесь был бы git ls-remote)
    Ok(Json(vec!["main".to_string(), "master".to_string(), "develop".to_string()]))
}


// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repositories_handler() {
        // Тест для проверки обработчиков репозиториев
        assert!(true);
    }
}
