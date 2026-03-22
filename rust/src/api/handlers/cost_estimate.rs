//! Terraform Cost Estimate Handlers (Infracost integration)

use crate::api::extractors::AuthUser;
use crate::api::state::AppState;
use crate::db::store::{CostEstimateManager, TaskManager};
use crate::models::cost_estimate::CostEstimateCreate;
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
pub struct CostQuery {
    pub limit: Option<i64>,
}

/// GET /api/project/{project_id}/costs
/// List cost estimates for a project (most recent first)
pub async fn list_cost_estimates(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(project_id): Path<i32>,
    Query(q): Query<CostQuery>,
) -> impl IntoResponse {
    let store = state.store.store();
    let limit = q.limit.unwrap_or(100).min(500);
    match store.get_cost_estimates(project_id, limit).await {
        Ok(list) => (StatusCode::OK, Json(json!(list))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/project/{project_id}/costs/summary
/// Aggregated cost summary per template
pub async fn cost_summary(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(project_id): Path<i32>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.get_cost_summaries(project_id).await {
        Ok(list) => (StatusCode::OK, Json(json!(list))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/project/{project_id}/tasks/{task_id}/cost
/// Get cost estimate for a specific task
pub async fn get_task_cost(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((project_id, task_id)): Path<(i32, i32)>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.get_cost_estimate_for_task(project_id, task_id).await {
        Ok(Some(cost)) => (StatusCode::OK, Json(json!(cost))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "No cost estimate for this task"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

/// POST /api/project/{project_id}/tasks/{task_id}/cost
/// Store a cost estimate (called after terraform plan completes)
pub async fn create_task_cost(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((project_id, task_id)): Path<(i32, i32)>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let store = state.store.store();

    // First get the task to extract template_id
    let task = match store.get_task(task_id, project_id).await {
        Ok(t) => t,
        Err(e) => return (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response(),
    };

    let payload = CostEstimateCreate {
        project_id,
        task_id,
        template_id: task.template_id,
        currency: body.get("currency").and_then(|v| v.as_str()).map(|s| s.to_string()),
        monthly_cost: body.get("monthly_cost").and_then(|v| v.as_f64()),
        monthly_cost_diff: body.get("monthly_cost_diff").and_then(|v| v.as_f64()),
        resource_count: body.get("resource_count").and_then(|v| v.as_i64()).map(|n| n as i32),
        resources_added: body.get("resources_added").and_then(|v| v.as_i64()).map(|n| n as i32),
        resources_changed: body.get("resources_changed").and_then(|v| v.as_i64()).map(|n| n as i32),
        resources_deleted: body.get("resources_deleted").and_then(|v| v.as_i64()).map(|n| n as i32),
        breakdown_json: body.get("breakdown_json").and_then(|v| v.as_str()).map(|s| s.to_string()),
        infracost_version: body.get("infracost_version").and_then(|v| v.as_str()).map(|s| s.to_string()),
    };

    match store.create_cost_estimate(payload).await {
        Ok(cost) => (StatusCode::CREATED, Json(json!(cost))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}
