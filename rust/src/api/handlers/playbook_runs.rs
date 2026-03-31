//! Handlers для истории запусков Playbook

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::PlaybookRunManager;
use crate::models::playbook_run_history::{PlaybookRun, PlaybookRunFilter, PlaybookRunStats};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

/// GET /api/project/{project_id}/playbook-runs
/// Получить список запусков playbook
pub async fn get_playbook_runs(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Query(params): Query<PlaybookRunFilterQuery>,
) -> Result<Json<Vec<PlaybookRun>>, (StatusCode, Json<ErrorResponse>)> {
    let filter = PlaybookRunFilter {
        project_id: Some(project_id),
        playbook_id: params.playbook_id,
        status: params.status,
        user_id: params.user_id,
        date_from: None,
        date_to: None,
        limit: params.limit,
        offset: params.offset,
    };

    let runs = state.store.get_playbook_runs(filter).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    Ok(Json(runs))
}

/// GET /api/project/{project_id}/playbook-runs/{id}
/// Получить запуск playbook по ID
pub async fn get_playbook_run(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<Json<PlaybookRun>, (StatusCode, Json<ErrorResponse>)> {
    let run = state
        .store
        .get_playbook_run(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(Json(run))
}

/// GET /api/project/{project_id}/playbooks/{playbook_id}/runs/stats
/// Получить статистику запусков playbook
pub async fn get_playbook_run_stats(
    State(state): State<Arc<AppState>>,
    Path((project_id, playbook_id)): Path<(i32, i32)>,
) -> Result<Json<PlaybookRunStats>, (StatusCode, Json<ErrorResponse>)> {
    let stats = state
        .store
        .get_playbook_run_stats(playbook_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(Json(stats))
}

/// DELETE /api/project/{project_id}/playbook-runs/{id}
pub async fn delete_playbook_run(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .delete_playbook_run(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Параметры запроса для фильтрации
#[derive(Debug, Deserialize)]
pub struct PlaybookRunFilterQuery {
    pub playbook_id: Option<i32>,
    pub status: Option<crate::models::playbook_run_history::PlaybookRunStatus>,
    pub user_id: Option<i32>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
