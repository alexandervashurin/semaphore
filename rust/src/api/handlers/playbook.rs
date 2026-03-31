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
    Json(payload): Json<PlaybookRunRequest>,
) -> Result<
    (
        StatusCode,
        Json<crate::models::playbook_run::PlaybookRunResult>,
    ),
    (StatusCode, Json<ErrorResponse>),
> {
    let result = PlaybookRunService::run_playbook(id, project_id, payload, &state.store)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok((StatusCode::ACCEPTED, Json(result)))
}
