//! Task Structured Output Handlers (FI-PUL-1 — Pulumi Outputs)
//!
//! Именованные key-value выходы задачи.
//! Позволяют передавать output одного шаблона как input другого.
//! Парсятся из stdout по маркеру: VELUM_OUTPUT: {"key":"value"}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use std::sync::Arc;

use crate::api::extractors::AuthUser;
use crate::api::state::AppState;
use crate::db::store::StructuredOutputManager;
use crate::models::{TaskStructuredOutputBatch, TaskStructuredOutputCreate};

/// GET /api/project/{project_id}/tasks/{task_id}/outputs
pub async fn get_task_outputs(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
    _auth: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let outputs = state
        .store
        .get_task_structured_outputs(task_id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(json!(outputs)))
}

/// GET /api/project/{project_id}/tasks/{task_id}/outputs/map
/// Возвращает outputs как плоский map {key: value} — для использования в extra_vars
pub async fn get_task_outputs_map(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
    _auth: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let map = state
        .store
        .get_task_outputs_map(task_id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(json!(map)))
}

/// POST /api/project/{project_id}/tasks/{task_id}/outputs
pub async fn create_task_output(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
    _auth: AuthUser,
    Json(payload): Json<TaskStructuredOutputCreate>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let output = state
        .store
        .create_task_structured_output(task_id, project_id, payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok((StatusCode::CREATED, Json(json!(output))))
}

/// POST /api/project/{project_id}/tasks/{task_id}/outputs/batch
/// Batch-запись нескольких outputs за раз
pub async fn create_task_outputs_batch(
    State(state): State<Arc<AppState>>,
    Path((project_id, task_id)): Path<(i32, i32)>,
    _auth: AuthUser,
    Json(payload): Json<TaskStructuredOutputBatch>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    state
        .store
        .create_task_structured_outputs_batch(task_id, project_id, payload.outputs)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/project/{project_id}/templates/{template_id}/last-outputs
/// Outputs последней успешной задачи шаблона — для ссылок между шаблонами
pub async fn get_template_last_outputs(
    State(state): State<Arc<AppState>>,
    Path((project_id, template_id)): Path<(i32, i32)>,
    _auth: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let map = state
        .store
        .get_template_last_outputs(template_id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(json!(map)))
}
