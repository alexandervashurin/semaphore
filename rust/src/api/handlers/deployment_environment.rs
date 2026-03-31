//! Deployment Environment Handlers (FI-GL-1 — GitLab Environments)
//!
//! Реестр окружений деплоя: production/staging/dev/review
//! История деплоев, статусы, URL живого окружения.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use std::sync::Arc;

use crate::api::extractors::AuthUser;
use crate::api::state::AppState;
use crate::db::store::DeploymentEnvironmentManager;
use crate::models::{DeploymentEnvironmentCreate, DeploymentEnvironmentUpdate};

/// GET /api/project/{project_id}/deploy-environments
pub async fn list_deploy_environments(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    _auth: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let envs = state
        .store
        .get_deployment_environments(project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(json!(envs)))
}

/// GET /api/project/{project_id}/deploy-environments/{id}
pub async fn get_deploy_environment(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
    _auth: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let env = state
        .store
        .get_deployment_environment(id, project_id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))))?;
    Ok(Json(json!(env)))
}

/// POST /api/project/{project_id}/deploy-environments
pub async fn create_deploy_environment(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    _auth: AuthUser,
    Json(payload): Json<DeploymentEnvironmentCreate>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let env = state
        .store
        .create_deployment_environment(project_id, payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok((StatusCode::CREATED, Json(json!(env))))
}

/// PUT /api/project/{project_id}/deploy-environments/{id}
pub async fn update_deploy_environment(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
    _auth: AuthUser,
    Json(payload): Json<DeploymentEnvironmentUpdate>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let env = state
        .store
        .update_deployment_environment(id, project_id, payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(json!(env)))
}

/// DELETE /api/project/{project_id}/deploy-environments/{id}
pub async fn delete_deploy_environment(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
    _auth: AuthUser,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    state
        .store
        .delete_deployment_environment(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/project/{project_id}/deploy-environments/{id}/history
pub async fn get_deploy_history(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
    _auth: AuthUser,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let history = state
        .store
        .get_deployment_history(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(json!(history)))
}
