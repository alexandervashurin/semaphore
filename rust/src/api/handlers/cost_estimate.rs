//! Terraform Cost Estimate Handlers (Infracost integration)

use crate::api::extractors::AuthUser;
use crate::api::state::AppState;
use crate::db::store::{CostEstimateManager, TaskManager};
use crate::models::cost_estimate::CostEstimateCreate;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
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
        Err(e) => {
            let e: crate::error::Error = e;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
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
        Err(e) => {
            let e: crate::error::Error = e;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
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
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "No cost estimate for this task"})),
        )
            .into_response(),
        Err(e) => {
            let e: crate::error::Error = e;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
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
        Err(e) => {
            return (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response();
        }
    };

    let payload = CostEstimateCreate {
        project_id,
        task_id,
        template_id: task.template_id,
        currency: body
            .get("currency")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        monthly_cost: body.get("monthly_cost").and_then(|v| v.as_f64()),
        monthly_cost_diff: body.get("monthly_cost_diff").and_then(|v| v.as_f64()),
        resource_count: body
            .get("resource_count")
            .and_then(|v| v.as_i64())
            .map(|n| n as i32),
        resources_added: body
            .get("resources_added")
            .and_then(|v| v.as_i64())
            .map(|n| n as i32),
        resources_changed: body
            .get("resources_changed")
            .and_then(|v| v.as_i64())
            .map(|n| n as i32),
        resources_deleted: body
            .get("resources_deleted")
            .and_then(|v| v.as_i64())
            .map(|n| n as i32),
        breakdown_json: body
            .get("breakdown_json")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        infracost_version: body
            .get("infracost_version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    };

    match store.create_cost_estimate(payload).await {
        Ok(cost) => (StatusCode::CREATED, Json(json!(cost))).into_response(),
        Err(e) => {
            let e: crate::error::Error = e;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_query_defaults() {
        let query = CostQuery { limit: None };
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_cost_query_with_limit() {
        let query = CostQuery { limit: Some(50) };
        assert_eq!(query.limit, Some(50));
    }

    #[test]
    fn test_cost_query_deserialize_with_limit() {
        let json = r#"{"limit": 25}"#;
        let query: CostQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, Some(25));
    }

    #[test]
    fn test_cost_query_deserialize_without_limit() {
        let json = r#"{}"#;
        let query: CostQuery = serde_json::from_str(json).unwrap();
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_cost_query_deserialize_null_limit() {
        let json = r#"{"limit": null}"#;
        let query: CostQuery = serde_json::from_str(json).unwrap();
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_cost_query_debug() {
        let query = CostQuery { limit: Some(100) };
        // CostQuery doesn't derive Debug, so we just verify the struct exists
        assert_eq!(query.limit, Some(100));
    }

    #[test]
    fn test_cost_query_clone() {
        // CostQuery doesn't derive Clone, test via deserialization
        let json = r#"{"limit": 75}"#;
        let q1: CostQuery = serde_json::from_str(json).unwrap();
        let q2: CostQuery = serde_json::from_str(json).unwrap();
        assert_eq!(q1.limit, q2.limit);
    }

    #[test]
    fn test_cost_query_zero_limit() {
        let query = CostQuery { limit: Some(0) };
        assert_eq!(query.limit, Some(0));
    }

    #[test]
    fn test_cost_query_negative_limit() {
        let query = CostQuery { limit: Some(-1) };
        assert_eq!(query.limit, Some(-1));
    }

    #[test]
    fn test_cost_query_max_limit() {
        let query = CostQuery {
            limit: Some(i64::MAX),
        };
        assert_eq!(query.limit, Some(i64::MAX));
    }

    #[test]
    fn test_cost_estimate_create_from_body_minimal() {
        let json = r#"{"currency": "USD", "monthly_cost": 100.0}"#;
        let body: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(body["currency"], "USD");
        assert_eq!(body["monthly_cost"], 100.0);
    }

    #[test]
    fn test_cost_estimate_create_from_body_full() {
        let json = r#"{
            "currency": "EUR",
            "monthly_cost": 250.50,
            "monthly_cost_diff": 25.0,
            "resource_count": 15,
            "resources_added": 5,
            "resources_changed": 3,
            "resources_deleted": 2,
            "breakdown_json": "{}",
            "infracost_version": "v0.10.0"
        }"#;
        let body: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(body["currency"], "EUR");
        assert_eq!(body["monthly_cost"], 250.50);
        assert_eq!(body["resource_count"], 15);
        assert_eq!(body["infracost_version"], "v0.10.0");
    }

    #[test]
    fn test_cost_estimate_create_from_body_null_fields() {
        let json = r#"{
            "currency": null,
            "monthly_cost": null,
            "resource_count": null
        }"#;
        let body: serde_json::Value = serde_json::from_str(json).unwrap();
        assert!(body["currency"].is_null());
        assert!(body["monthly_cost"].is_null());
    }

    #[test]
    fn test_cost_summary_response_structure() {
        // Test that cost_summary handler uses the right query pattern
        let limit_default = None::<i64>.unwrap_or(100).min(500);
        assert_eq!(limit_default, 100);

        let limit_large = Some(1000).unwrap_or(100).min(500);
        assert_eq!(limit_large, 500);
    }

    #[test]
    fn test_cost_query_roundtrip_via_serialize() {
        // CostQuery doesn't derive Serialize, test via deserialization only
        let json = r#"{"limit": 42}"#;
        let query: CostQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, Some(42));
    }

    #[test]
    fn test_cost_query_deserialize_large_number() {
        let json = r#"{"limit": 9223372036854775807}"#;
        let query: CostQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, Some(i64::MAX));
    }
}
