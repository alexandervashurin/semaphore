//! Kubernetes Deployments Handlers
//! /api/kubernetes/clusters/{cluster_id}/namespaces/{ns}/deployments/...

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::api::extractors::AuthUser;
use crate::error::Error;

// ── GET /api/kubernetes/clusters/:id/namespaces/:ns/deployments ──────────────

#[derive(Debug, Deserialize)]
pub struct DeploymentListQuery {
    pub limit: Option<u32>,
    pub continue_token: Option<String>,
}

pub async fn list_deployments(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((cluster_id, namespace)): Path<(String, String)>,
    Query(q): Query<DeploymentListQuery>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    match svc.list_deployments(&namespace, q.limit, q.continue_token).await {
        Ok(list) => (StatusCode::OK, Json(json!(list))),
        Err(e) => k8s_err(e),
    }
}

// ── GET /api/kubernetes/clusters/:id/namespaces/:ns/deployments/:name ────────

pub async fn get_deployment(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    match svc.get_deployment(&namespace, &name).await {
        Ok(d) => (StatusCode::OK, Json(json!(d))),
        Err(Error::NotFound(msg)) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": msg, "code": "DEPLOYMENT_NOT_FOUND"})),
        ),
        Err(e) => k8s_err(e),
    }
}

// ── POST /api/kubernetes/clusters/:id/namespaces/:ns/deployments/:name/scale ─

#[derive(Debug, Deserialize)]
pub struct ScaleBody {
    pub replicas: i32,
}

pub async fn scale_deployment(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
    Json(body): Json<ScaleBody>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    match svc.scale_deployment(&namespace, &name, body.replicas).await {
        Ok(()) => (StatusCode::OK, Json(json!({"message": "Scaled", "replicas": body.replicas}))),
        Err(Error::NotFound(msg)) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": msg, "code": "DEPLOYMENT_NOT_FOUND"})),
        ),
        Err(e) => k8s_err(e),
    }
}

// ── POST /api/kubernetes/clusters/:id/namespaces/:ns/deployments/:name/restart

pub async fn restart_deployment(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    match svc.restart_deployment(&namespace, &name).await {
        Ok(()) => (StatusCode::OK, Json(json!({"message": "Restart initiated"}))),
        Err(Error::NotFound(msg)) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": msg, "code": "DEPLOYMENT_NOT_FOUND"})),
        ),
        Err(e) => k8s_err(e),
    }
}

// ─── helpers ─────────────────────────────────────────────────────────────────

async fn get_svc(
    state: &Arc<AppState>,
    cluster_id: &str,
) -> Result<Arc<crate::kubernetes::KubernetesClusterService>, (StatusCode, Json<Value>)> {
    let mgr = match &state.k8s {
        Some(m) => m.clone(),
        None => return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Kubernetes не сконфигурирован", "code": "K8S_NOT_CONFIGURED"})),
        )),
    };
    mgr.get(cluster_id).await.map_err(|e| match e {
        Error::NotFound(msg) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": msg, "code": "CLUSTER_NOT_FOUND"})),
        ),
        other => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": other.to_string()})),
        ),
    })
}

fn k8s_err(e: Error) -> (StatusCode, Json<Value>) {
    let msg = e.to_string();
    if msg.contains("FORBIDDEN") {
        (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Нет прав", "code": "K8S_FORBIDDEN", "detail": msg})),
        )
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": msg})))
    }
}
