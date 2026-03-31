//! Task Snapshot & Rollback Handlers

use crate::api::extractors::AuthUser;
use crate::api::state::AppState;
use crate::db::store::{SnapshotManager, TaskManager};
use crate::models::snapshot::{RollbackRequest, TaskSnapshotCreate};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct SnapshotQuery {
    pub template_id: Option<i32>,
    pub limit: Option<i64>,
}

/// GET /api/project/{project_id}/snapshots
pub async fn list_snapshots(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(project_id): Path<i32>,
    Query(q): Query<SnapshotQuery>,
) -> impl IntoResponse {
    let store = state.store.store();
    let limit = q.limit.unwrap_or(50).min(200);
    match store.get_snapshots(project_id, q.template_id, limit).await {
        Ok(list) => (StatusCode::OK, Json(json!(list))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/project/{project_id}/snapshots (manual snapshot creation)
pub async fn create_snapshot(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(project_id): Path<i32>,
    Json(body): Json<TaskSnapshotCreate>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.create_snapshot(project_id, body).await {
        Ok(s) => (StatusCode::CREATED, Json(json!(s))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// DELETE /api/project/{project_id}/snapshots/{id}
pub async fn delete_snapshot(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((project_id, id)): Path<(i32, i32)>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.delete_snapshot(id, project_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/project/{project_id}/snapshots/{id}/rollback
/// Создаёт новую задачу с параметрами из снапшота
pub async fn rollback_snapshot(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((project_id, id)): Path<(i32, i32)>,
    Json(body): Json<RollbackRequest>,
) -> impl IntoResponse {
    let store = state.store.store();

    // Get the snapshot
    let snap = match store.get_snapshot(id, project_id).await {
        Ok(s) => s,
        Err(e) => {
            return (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response()
        }
    };

    // Build task params from snapshot
    let rollback_msg = body
        .message
        .unwrap_or_else(|| format!("Rollback to snapshot #{} (task #{})", id, snap.task_id));

    let mut task = crate::models::Task {
        id: 0,
        template_id: snap.template_id,
        project_id,
        status: crate::services::task_logger::TaskStatus::Waiting,
        playbook: None,
        secret: None,
        arguments: snap.arguments.clone(),
        git_branch: snap.git_branch.clone(),
        user_id: Some(auth.user_id),
        integration_id: None,
        schedule_id: None,
        created: chrono::Utc::now(),
        start: None,
        end: None,
        message: Some(rollback_msg),
        commit_hash: None,
        commit_message: None,
        build_task_id: None,
        version: None,
        inventory_id: snap.inventory_id,
        repository_id: None,
        environment_id: snap.environment_id,
        params: None,
        environment: None,
    };

    match store.create_task(task).await {
        Ok(t) => (
            StatusCode::CREATED,
            Json(json!({
                "message": "Rollback task created",
                "task_id": t.id,
                "snapshot_id": id,
                "from_task": snap.task_id
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/project/{project_id}/tasks/{task_id}/snapshot
/// Создаёт снапшот из конкретной задачи (вызывается после успешного завершения)
pub async fn snapshot_from_task(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((project_id, task_id)): Path<(i32, i32)>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let store = state.store.store();

    // Get the task
    let task = match store.get_task(task_id, project_id).await {
        Ok(t) => t,
        Err(e) => {
            return (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response()
        }
    };

    let label = body
        .get("label")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let payload = TaskSnapshotCreate {
        template_id: task.template_id,
        task_id,
        git_branch: task.git_branch.clone(),
        git_commit: task.commit_hash.clone(),
        arguments: task.arguments.clone(),
        inventory_id: task.inventory_id,
        environment_id: task.environment_id,
        message: task.message.clone(),
        label,
    };

    match store.create_snapshot(project_id, payload).await {
        Ok(s) => (StatusCode::CREATED, Json(json!(s))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
