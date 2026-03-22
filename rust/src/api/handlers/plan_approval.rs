//! Plan Approval Handlers (Phase 2)

use crate::api::extractors::AuthUser;
use crate::api::state::AppState;
use crate::db::store::{PlanApprovalManager, TaskManager};
use crate::models::PlanReviewPayload;
use crate::services::task_logger::TaskStatus;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;

/// GET /api/project/{pid}/terraform/plans
/// List pending plans for a project
pub async fn list_pending_plans(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(project_id): Path<i32>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.list_pending_plans(project_id).await {
        Ok(plans) => (StatusCode::OK, Json(json!(plans))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/project/{pid}/tasks/{tid}/plan
/// Get the terraform plan for a specific task
pub async fn get_task_plan(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.get_plan_by_task(project_id, task_id).await {
        Ok(Some(plan)) => (StatusCode::OK, Json(json!(plan))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "No plan found for this task"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

/// POST /api/project/{pid}/terraform/plans/{plan_id}/approve
pub async fn approve_plan(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((project_id, plan_id)): Path<(i32, i64)>,
    Json(body): Json<PlanReviewPayload>,
) -> impl IntoResponse {
    let store = state.store.store();

    // Get plan to find task_id before approving
    let plan = match store.get_plan_by_task(project_id, plan_id as i32).await {
        Ok(Some(p)) => p,
        _ => {
            // Try direct fetch by plan_id from pending plans
            match store.list_pending_plans(project_id).await {
                Ok(plans) => {
                    match plans.into_iter().find(|p| p.id == plan_id) {
                        Some(p) => p,
                        None => return (StatusCode::NOT_FOUND, Json(json!({"error": "Plan not found"}))).into_response(),
                    }
                }
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
            }
        }
    };

    let task_id = plan.task_id;

    if let Err(e) = store.approve_plan(plan_id, auth.user_id, body.comment).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response();
    }

    // Set task back to WaitingRun so it can be re-queued
    let _ = store.update_task_status(project_id, task_id, TaskStatus::Running).await;

    // Trigger task execution
    if let Ok(task) = store.get_task(project_id, task_id).await {
        let store_arc = state.store.as_arc();
        tokio::spawn(crate::services::task_execution::execute_task(store_arc, task));
    }

    StatusCode::OK.into_response()
}

/// POST /api/project/{pid}/terraform/plans/{plan_id}/reject
pub async fn reject_plan(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((project_id, plan_id)): Path<(i32, i64)>,
    Json(body): Json<PlanReviewPayload>,
) -> impl IntoResponse {
    let store = state.store.store();

    // Find the task_id from plan
    let task_id = {
        let plans = store.list_pending_plans(project_id).await.unwrap_or_default();
        plans.into_iter().find(|p| p.id == plan_id).map(|p| p.task_id)
    };

    if let Err(e) = store.reject_plan(plan_id, auth.user_id, body.comment).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response();
    }

    // Set task status to Stopped
    if let Some(tid) = task_id {
        let _ = store.update_task_status(project_id, tid, TaskStatus::Stopped).await;
    }

    StatusCode::OK.into_response()
}
