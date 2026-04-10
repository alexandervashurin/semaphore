//! Handlers для Playbook API

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::PlaybookManager;
use crate::models::playbook::{Playbook, PlaybookCreate, PlaybookUpdate};
use crate::models::playbook_run::PlaybookRunRequest;
use crate::services::playbook_run_service::PlaybookRunService;
use crate::services::playbook_sync_service::PlaybookSyncService;
use crate::validators::PlaybookValidator;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

/// GET /api/project/{project_id}/playbooks
pub async fn get_project_playbooks(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<Playbook>>, (StatusCode, Json<ErrorResponse>)> {
    let playbooks = state.store.get_playbooks(project_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(Json(playbooks))
}

/// POST /api/project/{project_id}/playbooks
pub async fn create_playbook(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<PlaybookCreate>,
) -> Result<(StatusCode, Json<Playbook>), (StatusCode, Json<ErrorResponse>)> {
    // Валидация playbook
    if let Err(e) = PlaybookValidator::validate(&payload.content, &payload.playbook_type) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(format!("Ошибка валидации: {}", e))),
        ));
    }

    let playbook = state
        .store
        .create_playbook(project_id, payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok((StatusCode::CREATED, Json(playbook)))
}

/// GET /api/project/{project_id}/playbooks/{id}
pub async fn get_playbook(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<Json<Playbook>, (StatusCode, Json<ErrorResponse>)> {
    let playbook = state
        .store
        .get_playbook(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(Json(playbook))
}

/// PUT /api/project/{project_id}/playbooks/{id}
pub async fn update_playbook(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
    Json(payload): Json<PlaybookUpdate>,
) -> Result<Json<Playbook>, (StatusCode, Json<ErrorResponse>)> {
    // Валидация playbook
    // Для обновления предполагаем, что тип не меняется (берем из БД)
    // Упрощенная валидация - только YAML синтаксис
    if let Err(e) = PlaybookValidator::check_yaml_syntax(&payload.content) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(format!("Ошибка YAML синтаксиса: {}", e))),
        ));
    }

    let playbook = state
        .store
        .update_playbook(id, project_id, payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(Json(playbook))
}

/// DELETE /api/project/{project_id}/playbooks/{id}
pub async fn delete_playbook(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .delete_playbook(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/project/{project_id}/playbooks/{id}/sync
/// Синхронизировать playbook из Git репозитория
pub async fn sync_playbook(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<Json<Playbook>, (StatusCode, Json<ErrorResponse>)> {
    let playbook = PlaybookSyncService::sync_from_repository(id, project_id, &state.store)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(Json(playbook))
}

/// GET /api/project/{project_id}/playbooks/{id}/preview
/// Предварительный просмотр содержимого playbook из Git
pub async fn preview_playbook(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<Json<String>, (StatusCode, Json<ErrorResponse>)> {
    let content = PlaybookSyncService::preview_from_repository(id, project_id, &state.store)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(Json(content))
}

/// POST /api/project/{project_id}/playbooks/{id}/run
/// Запустить playbook
pub async fn run_playbook(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
    auth_user: crate::api::extractors::AuthUser,
    Json(payload): Json<PlaybookRunRequest>,
) -> Result<
    (
        StatusCode,
        Json<crate::models::playbook_run::PlaybookRunResult>,
    ),
    (StatusCode, Json<ErrorResponse>),
> {
    let result = PlaybookRunService::run_playbook(id, project_id, auth_user.user_id, payload, &state.store)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok((StatusCode::ACCEPTED, Json(result)))
}

#[cfg(test)]
mod tests {
    use crate::models::playbook::{Playbook, PlaybookCreate, PlaybookUpdate};
    use crate::models::playbook_run::{PlaybookRunRequest, PlaybookRunResult};
    use chrono::Utc;

    #[test]
    fn test_playbook_serialization_full() {
        let playbook = Playbook {
            id: 1,
            project_id: 10,
            name: "deploy.yml".to_string(),
            content: "---\n- hosts: all".to_string(),
            description: Some("Deploy playbook".to_string()),
            playbook_type: "ansible".to_string(),
            repository_id: Some(5),
            created: Utc::now(),
            updated: Utc::now(),
        };
        let json = serde_json::to_string(&playbook).unwrap();
        assert!(json.contains("\"name\":\"deploy.yml\""));
        assert!(json.contains("\"playbook_type\":\"ansible\""));
        assert!(json.contains("\"description\":\"Deploy playbook\""));
    }

    #[test]
    fn test_playbook_serialization_null_fields() {
        let playbook = Playbook {
            id: 2,
            project_id: 10,
            name: "simple.sh".to_string(),
            content: "#!/bin/bash".to_string(),
            description: None,
            playbook_type: "shell".to_string(),
            repository_id: None,
            created: Utc::now(),
            updated: Utc::now(),
        };
        let json = serde_json::to_string(&playbook).unwrap();
        assert!(!json.contains("\"description\":"));
        assert!(!json.contains("\"repository_id\":"));
    }

    #[test]
    fn test_playbook_create_serialization() {
        let create = PlaybookCreate {
            name: "new.yml".to_string(),
            content: "---".to_string(),
            description: Some("New playbook".to_string()),
            playbook_type: "terraform".to_string(),
            repository_id: Some(3),
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"name\":\"new.yml\""));
        assert!(json.contains("\"playbook_type\":\"terraform\""));
    }

    #[test]
    fn test_playbook_create_deserialization() {
        let json = r#"{"name":"test.yml","content":"---","description":null,"playbook_type":"ansible","repository_id":null}"#;
        let create: PlaybookCreate = serde_json::from_str(json).unwrap();
        assert_eq!(create.name, "test.yml");
        assert_eq!(create.playbook_type, "ansible");
    }

    #[test]
    fn test_playbook_update_serialization() {
        let update = PlaybookUpdate {
            name: "updated.yml".to_string(),
            content: "---\nupdated".to_string(),
            description: None,
            playbook_type: "ansible".to_string(),
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"name\":\"updated.yml\""));
    }

    #[test]
    fn test_playbook_update_partial_fields() {
        let update = PlaybookUpdate {
            name: "rename.yml".to_string(),
            content: "---".to_string(),
            description: Some("Renamed".to_string()),
            playbook_type: "ansible".to_string(),
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"description\":\"Renamed\""));
        let deserialized: PlaybookUpdate = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.description, Some("Renamed".to_string()));
    }

    #[test]
    fn test_playbook_run_request_serialization() {
        let req = PlaybookRunRequest::new()
            .with_inventory(1)
            .with_environment(2);
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"inventory_id\":1"));
        assert!(json.contains("\"environment_id\":2"));
    }

    #[test]
    fn test_playbook_run_request_with_extra_vars() {
        let req = PlaybookRunRequest::new()
            .with_extra_vars(serde_json::json!({"app": "web", "env": "prod"}));
        let json = serde_json::to_string(&req).unwrap();
        let parsed: PlaybookRunRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.extra_vars.unwrap()["app"], "web");
    }

    #[test]
    fn test_playbook_run_request_with_tags() {
        let req = PlaybookRunRequest::new()
            .with_tags(vec!["deploy".to_string(), "web".to_string()])
            .with_limit("web_servers".to_string());
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"tags\":[\"deploy\",\"web\"]"));
        assert!(json.contains("\"limit\":\"web_servers\""));
    }

    #[test]
    fn test_playbook_run_result_serialization() {
        let result = PlaybookRunResult {
            task_id: 100,
            template_id: 5,
            status: "waiting".to_string(),
            message: "Task queued".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"task_id\":100"));
        assert!(json.contains("\"message\":\"Task queued\""));
    }

    #[test]
    fn test_playbook_run_result_deserialization() {
        let json = r#"{"task_id":42,"template_id":10,"status":"running","message":"started"}"#;
        let result: PlaybookRunResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.task_id, 42);
        assert_eq!(result.status, "running");
    }

    #[test]
    fn test_playbook_run_request_validate_null_extra_vars() {
        let req = PlaybookRunRequest::new().with_extra_vars(serde_json::json!(null));
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_playbook_run_request_validate_array_extra_vars() {
        let req = PlaybookRunRequest::new().with_extra_vars(serde_json::json!(["a", "b"]));
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_playbook_clone() {
        let playbook = Playbook {
            id: 1,
            project_id: 10,
            name: "clone.yml".to_string(),
            content: "---".to_string(),
            description: None,
            playbook_type: "ansible".to_string(),
            repository_id: None,
            created: Utc::now(),
            updated: Utc::now(),
        };
        let cloned = playbook.clone();
        assert_eq!(cloned.name, playbook.name);
        assert_eq!(cloned.content, playbook.content);
    }
}
