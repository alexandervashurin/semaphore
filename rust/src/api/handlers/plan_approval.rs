//! Plan Approval Handlers (Phase 2)

use crate::api::extractors::AuthUser;
use crate::api::state::AppState;
use crate::db::store::{PlanApprovalManager, RetrieveQueryParams, TaskManager, UserManager};
use crate::models::PlanReviewPayload;
use crate::models::ProjectUserRole;
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
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
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
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "No plan found for this task"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
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

    // Global admins bypass project-level role check
    if !auth.admin {
        // Require Manager or Owner role to approve plans
        match state
            .store
            .get_project_users(project_id, RetrieveQueryParams::default())
            .await
        {
            Ok(users) => {
                let role = users
                    .into_iter()
                    .find(|u| u.user_id == auth.user_id)
                    .map(|u| u.role)
                    .unwrap_or(ProjectUserRole::None);
                match role {
                    ProjectUserRole::Owner | ProjectUserRole::Manager => {}
                    _ => {
                        return (
                            StatusCode::FORBIDDEN,
                            Json(json!({"error": "Manager or Owner role required"})),
                        )
                            .into_response()
                    }
                }
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()})),
                )
                    .into_response()
            }
        }
    }

    // Get plan by plan_id from pending plans list
    let plan = match store.list_pending_plans(project_id).await {
        Ok(plans) => match plans.into_iter().find(|p| p.id == plan_id) {
            Some(p) => p,
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Plan not found"})),
                )
                    .into_response()
            }
        },
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    };

    let task_id = plan.task_id;

    if let Err(e) = store
        .approve_plan(plan_id, auth.user_id, body.comment)
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response();
    }

    // Set task back to WaitingRun so it can be re-queued
    let _ = store
        .update_task_status(project_id, task_id, TaskStatus::Running)
        .await;

    // Trigger task execution
    if let Ok(task) = store.get_task(project_id, task_id).await {
        let store_arc = state.store.as_arc();
        tokio::spawn(crate::services::task_execution::execute_task(
            store_arc, task,
        ));
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

    // Global admins bypass project-level role check
    if !auth.admin {
        // Require Manager or Owner role to reject plans
        match state
            .store
            .get_project_users(project_id, RetrieveQueryParams::default())
            .await
        {
            Ok(users) => {
                let role = users
                    .into_iter()
                    .find(|u| u.user_id == auth.user_id)
                    .map(|u| u.role)
                    .unwrap_or(ProjectUserRole::None);
                match role {
                    ProjectUserRole::Owner | ProjectUserRole::Manager => {}
                    _ => {
                        return (
                            StatusCode::FORBIDDEN,
                            Json(json!({"error": "Manager or Owner role required"})),
                        )
                            .into_response()
                    }
                }
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()})),
                )
                    .into_response()
            }
        }
    }

    // Find the task_id from plan
    let task_id = {
        let plans = store
            .list_pending_plans(project_id)
            .await
            .unwrap_or_default();
        plans
            .into_iter()
            .find(|p| p.id == plan_id)
            .map(|p| p.task_id)
    };

    if let Err(e) = store.reject_plan(plan_id, auth.user_id, body.comment).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response();
    }

    // Set task status to Stopped
    if let Some(tid) = task_id {
        let _ = store
            .update_task_status(project_id, tid, TaskStatus::Stopped)
            .await;
    }

    StatusCode::OK.into_response()
}
