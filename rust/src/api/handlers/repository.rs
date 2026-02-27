//! Repository Handlers
//!
//! Обработчики запросов для управления репозиториями

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::models::Repository;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;

/// Получить список репозиториев проекта
///
/// GET /api/projects/:project_id/repositories
pub async fn get_repositories(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<Repository>>, (StatusCode, Json<ErrorResponse>)> {
    let repositories = state.store.get_repositories(project_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(repositories))
}

/// Создать репозиторий
///
/// POST /api/projects/:project_id/repositories
pub async fn create_repository(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<RepositoryCreatePayload>,
) -> Result<(StatusCode, Json<Repository>), (StatusCode, Json<ErrorResponse>)> {
    let repository = Repository::new(
        project_id,
        payload.name,
        payload.git_url,
    );

    let created = state.store.create_repository(repository)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Получить репозиторий по ID
///
/// GET /api/projects/:project_id/repositories/:repository_id
pub async fn get_repository(
    State(state): State<Arc<AppState>>,
    Path((project_id, repository_id)): Path<(i32, i32)>,
) -> Result<Json<Repository>, (StatusCode, Json<ErrorResponse>)> {
    let repository = state.store.get_repository(project_id, repository_id)
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

    Ok(Json(repository))
}

/// Обновить репозиторий
///
/// PUT /api/projects/:project_id/repositories/:repository_id
pub async fn update_repository(
    State(state): State<Arc<AppState>>,
    Path((project_id, repository_id)): Path<(i32, i32)>,
    Json(payload): Json<RepositoryUpdatePayload>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut repository = state.store.get_repository(project_id, repository_id)
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
        repository.name = name;
    }
    if let Some(git_url) = payload.git_url {
        repository.git_url = git_url;
    }

    state.store.update_repository(repository)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Удалить репозиторий
///
/// DELETE /api/projects/:project_id/repositories/:repository_id
pub async fn delete_repository(
    State(state): State<Arc<AppState>>,
    Path((project_id, repository_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_repository(project_id, repository_id)
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

/// Payload для создания репозитория
#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryCreatePayload {
    pub name: String,
    pub git_url: String,
}

/// Payload для обновления репозитория
#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_url: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_create_payload_deserialize() {
        let json = r#"{
            "name": "My Repo",
            "git_url": "https://github.com/user/repo.git"
        }"#;
        let payload: RepositoryCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "My Repo");
        assert_eq!(payload.git_url, "https://github.com/user/repo.git");
    }

    #[test]
    fn test_repository_update_payload_deserialize_all_fields() {
        let json = r#"{
            "name": "Updated Repo",
            "git_url": "https://github.com/user/new-repo.git"
        }"#;
        let payload: RepositoryUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, Some("Updated Repo".to_string()));
        assert_eq!(payload.git_url, Some("https://github.com/user/new-repo.git".to_string()));
    }

    #[test]
    fn test_repository_update_payload_deserialize_partial() {
        let json = r#"{"git_url": "https://new.url.git"}"#;
        let payload: RepositoryUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, None);
        assert_eq!(payload.git_url, Some("https://new.url.git".to_string()));
    }
}
