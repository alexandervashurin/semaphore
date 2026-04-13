//! Repository Handlers
//!
//! Обработчики запросов для управления репозиториями

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::RepositoryManager;
use crate::error::Error;
use crate::models::Repository;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Получить список репозиториев проекта
///
/// GET /api/projects/:project_id/repositories
pub async fn get_repositories(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<Repository>>, (StatusCode, Json<ErrorResponse>)> {
    let repositories = state
        .store
        .get_repositories(project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

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
    let repository = Repository::new(project_id, payload.name, payload.git_url);

    let created = state
        .store
        .create_repository(repository)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Получить репозиторий по ID
///
/// GET /api/projects/:project_id/repositories/:repository_id
pub async fn get_repository(
    State(state): State<Arc<AppState>>,
    Path((project_id, repository_id)): Path<(i32, i32)>,
) -> Result<Json<Repository>, (StatusCode, Json<ErrorResponse>)> {
    let repository = state
        .store
        .get_repository(project_id, repository_id)
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
    let mut repository = state
        .store
        .get_repository(project_id, repository_id)
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

    state
        .store
        .update_repository(repository)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(StatusCode::OK)
}

/// Удалить репозиторий
///
/// DELETE /api/projects/:project_id/repositories/:repository_id
pub async fn delete_repository(
    State(state): State<Arc<AppState>>,
    Path((project_id, repository_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .delete_repository(project_id, repository_id)
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
        assert_eq!(
            payload.git_url,
            Some("https://github.com/user/new-repo.git".to_string())
        );
    }

    #[test]
    fn test_repository_update_payload_deserialize_partial() {
        let json = r#"{"git_url": "https://new.url.git"}"#;
        let payload: RepositoryUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, None);
        assert_eq!(payload.git_url, Some("https://new.url.git".to_string()));
    }

    #[test]
    fn test_repository_create_payload_roundtrip() {
        let original = RepositoryCreatePayload {
            name: "Roundtrip".to_string(),
            git_url: "https://github.com/user/round.git".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: RepositoryCreatePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, original.name);
        assert_eq!(restored.git_url, original.git_url);
    }

    #[test]
    fn test_repository_update_payload_roundtrip() {
        let original = RepositoryUpdatePayload {
            name: Some("Updated".to_string()),
            git_url: Some("https://github.com/user/updated.git".to_string()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: RepositoryUpdatePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, original.name);
        assert_eq!(restored.git_url, original.git_url);
    }

    #[test]
    fn test_repository_update_payload_all_null() {
        let json = r#"{"name": null, "git_url": null}"#;
        let payload: RepositoryUpdatePayload = serde_json::from_str(json).unwrap();
        assert!(payload.name.is_none());
        assert!(payload.git_url.is_none());
    }

    #[test]
    fn test_repository_update_payload_empty() {
        let json = r#"{}"#;
        let payload: RepositoryUpdatePayload = serde_json::from_str(json).unwrap();
        assert!(payload.name.is_none());
        assert!(payload.git_url.is_none());
    }

    #[test]
    fn test_repository_create_payload_gitlab_url() {
        let json = r#"{
            "name": "GitLab Repo",
            "git_url": "https://gitlab.example.com/org/project.git"
        }"#;
        let payload: RepositoryCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "GitLab Repo");
        assert_eq!(payload.git_url, "https://gitlab.example.com/org/project.git");
    }

    #[test]
    fn test_repository_create_payload_ssh_url() {
        let json = r#"{
            "name": "SSH Repo",
            "git_url": "git@github.com:user/repo.git"
        }"#;
        let payload: RepositoryCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.git_url, "git@github.com:user/repo.git");
    }

    #[test]
    fn test_repository_create_payload_empty_name() {
        let json = r#"{"name": "", "git_url": "https://example.com/repo.git"}"#;
        let payload: RepositoryCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "");
    }

    #[test]
    fn test_repository_create_payload_unicode() {
        let json = r#"{
            "name": "Репозиторий",
            "git_url": "https://github.com/user/репо.git"
        }"#;
        let payload: RepositoryCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "Репозиторий");
    }

    #[test]
    fn test_repository_create_payload_debug() {
        let payload = RepositoryCreatePayload {
            name: "Debug Repo".to_string(),
            git_url: "https://example.com/debug.git".to_string(),
        };
        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("RepositoryCreatePayload"));
        assert!(debug_str.contains("Debug Repo"));
    }

    #[test]
    fn test_repository_update_payload_debug() {
        let payload = RepositoryUpdatePayload {
            name: Some("Debug".to_string()),
            git_url: None,
        };
        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("RepositoryUpdatePayload"));
    }

    #[test]
    fn test_repository_create_payload_clone_independence() {
        // RepositoryCreatePayload doesn't derive Clone
        let json = r#"{"name": "Original", "git_url": "https://example.com/orig.git"}"#;
        let p1: RepositoryCreatePayload = serde_json::from_str(json).unwrap();
        let p2: RepositoryCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(p1.name, p2.name);
    }

    #[test]
    fn test_repository_update_payload_clone_independence() {
        // RepositoryUpdatePayload doesn't derive Clone
        let json = r#"{"name": "Original", "git_url": "https://example.com/orig.git"}"#;
        let p1: RepositoryUpdatePayload = serde_json::from_str(json).unwrap();
        let p2: RepositoryUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(p1.name, p2.name);
        assert_eq!(p1.git_url, p2.git_url);
    }

    #[test]
    fn test_repository_update_payload_single_name() {
        let json = r#"{"name": "Only Name"}"#;
        let payload: RepositoryUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, Some("Only Name".to_string()));
        assert!(payload.git_url.is_none());
    }

    #[test]
    fn test_repository_update_payload_single_git_url() {
        let json = r#"{"git_url": "https://only.url.git"}"#;
        let payload: RepositoryUpdatePayload = serde_json::from_str(json).unwrap();
        assert!(payload.name.is_none());
        assert_eq!(payload.git_url, Some("https://only.url.git".to_string()));
    }

    #[test]
    fn test_repository_create_payload_with_subgroup() {
        let json = r#"{
            "name": "Subgroup Repo",
            "git_url": "https://gitlab.com/org/team/subgroup/project.git"
        }"#;
        let payload: RepositoryCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.git_url, "https://gitlab.com/org/team/subgroup/project.git");
    }

    #[test]
    fn test_repository_update_payload_special_chars() {
        let json = r#"{
            "name": "Repo with special chars & < > \" '",
            "git_url": "https://example.com/repo%20with%20spaces.git"
        }"#;
        let payload: RepositoryUpdatePayload = serde_json::from_str(json).unwrap();
        assert!(payload.name.as_ref().unwrap().contains("special chars"));
        assert!(payload.git_url.as_ref().unwrap().contains("%20"));
    }
}
