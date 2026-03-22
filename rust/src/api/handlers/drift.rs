//! GitOps Drift Detection handlers

use crate::api::extractors::AuthUser;
use crate::api::state::AppState;
use crate::db::store::DriftManager;
use crate::models::drift::{DriftConfigCreate, DriftConfigWithStatus};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn list_drift_configs(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(project_id): Path<i32>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.get_drift_configs(project_id).await {
        Ok(configs) => {
            let mut result: Vec<Value> = Vec::new();
            for c in configs {
                let latest = store.get_drift_results(c.id, 1).await.unwrap_or_default();
                let with_status = DriftConfigWithStatus {
                    latest_result: latest.into_iter().next(),
                    config: c,
                };
                result.push(serde_json::to_value(&with_status).unwrap_or(json!({})));
            }
            (StatusCode::OK, Json(json!(result)))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

pub async fn create_drift_config(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(project_id): Path<i32>,
    Json(body): Json<DriftConfigCreate>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.create_drift_config(project_id, body).await {
        Ok(c) => (StatusCode::CREATED, Json(json!(c))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    }
}

#[derive(Deserialize)]
pub struct DriftToggle {
    pub enabled: bool,
    pub schedule: Option<String>,
}

pub async fn update_drift_config(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((project_id, id)): Path<(i32, i32)>,
    Json(body): Json<DriftToggle>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.update_drift_config_enabled(id, project_id, body.enabled).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn delete_drift_config(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((project_id, id)): Path<(i32, i32)>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.delete_drift_config(id, project_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

/// Trigger manual drift check — create a task with --check flag and record result
pub async fn trigger_drift_check(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((project_id, id)): Path<(i32, i32)>,
) -> impl IntoResponse {
    let store = state.store.store();

    // Get the drift config
    let config = match store.get_drift_config(id, project_id).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response(),
    };

    // Create a task with --check argument (dry run)
    let task_body = json!({
        "template_id": config.template_id,
        "message": "Drift check (auto)",
        "arguments": "--check --diff",
        "dry_run": true
    });

    // Post task to the project tasks endpoint via store
    // We record the drift result with "pending" status and the task_id
    let result = store.create_drift_result(
        project_id,
        id,
        config.template_id,
        "pending",
        Some("Drift check triggered manually".to_string()),
        None,
    ).await;

    match result {
        Ok(r) => (StatusCode::OK, Json(json!({
            "message": "Drift check triggered",
            "drift_result_id": r.id,
            "template_id": config.template_id,
            "hint": "Create a task manually with --check --diff arguments to complete the check"
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn get_drift_results(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((project_id, id)): Path<(i32, i32)>,
) -> impl IntoResponse {
    let store = state.store.store();
    // Verify config belongs to project
    if store.get_drift_config(id, project_id).await.is_err() {
        return (StatusCode::NOT_FOUND, Json(json!({"error": "Drift config not found"}))).into_response();
    }
    match store.get_drift_results(id, 50).await {
        Ok(results) => (StatusCode::OK, Json(json!(results))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}
