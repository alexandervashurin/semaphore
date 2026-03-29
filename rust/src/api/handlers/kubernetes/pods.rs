//! Kubernetes Pods Handlers — /api/kubernetes/clusters/{cluster_id}/namespaces/{ns}/pods/...

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

// ──────────────────────────────────────────────────────────────────────────────
// GET /api/kubernetes/clusters/:cluster_id/namespaces/:namespace/pods
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PodListQuery {
    pub limit: Option<u32>,
    pub continue_token: Option<String>,
    pub label_selector: Option<String>,
    pub field_selector: Option<String>,
}

pub async fn list_pods(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((cluster_id, namespace)): Path<(String, String)>,
    Query(q): Query<PodListQuery>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await {
        Ok(s) => s,
        Err(r) => return r,
    };

    match svc.list_pods(&namespace, q.limit, q.continue_token, q.label_selector, q.field_selector).await {
        Ok(list) => (StatusCode::OK, Json(json!(list))),
        Err(e) => k8s_err(e),
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// GET /api/kubernetes/clusters/:cluster_id/namespaces/:namespace/pods/:name
// ──────────────────────────────────────────────────────────────────────────────

pub async fn get_pod(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await {
        Ok(s) => s,
        Err(r) => return r,
    };

    match svc.get_pod(&namespace, &name).await {
        Ok(pod) => (StatusCode::OK, Json(json!(pod))),
        Err(Error::NotFound(msg)) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": msg, "code": "POD_NOT_FOUND"})),
        ),
        Err(e) => k8s_err(e),
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// DELETE /api/kubernetes/clusters/:cluster_id/namespaces/:namespace/pods/:name
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DeletePodQuery {
    pub grace_period_seconds: Option<i64>,
}

pub async fn delete_pod(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
    Query(q): Query<DeletePodQuery>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await {
        Ok(s) => s,
        Err(r) => return r,
    };

    match svc.delete_pod(&namespace, &name, q.grace_period_seconds).await {
        Ok(()) => (StatusCode::OK, Json(json!({"message": "Pod deleted"}))),
        Err(Error::NotFound(msg)) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": msg, "code": "POD_NOT_FOUND"})),
        ),
        Err(e) => k8s_err(e),
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// GET /api/kubernetes/clusters/:cluster_id/namespaces/:namespace/pods/:name/logs
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PodLogsQuery {
    pub container: Option<String>,
    pub tail_lines: Option<i64>,
    pub since_seconds: Option<i64>,
    pub previous: Option<bool>,
}

pub async fn pod_logs(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
    Query(q): Query<PodLogsQuery>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await {
        Ok(s) => s,
        Err(r) => return r,
    };

    match svc.pod_logs(
        &namespace,
        &name,
        q.container,
        q.tail_lines,
        q.since_seconds,
        q.previous.unwrap_or(false),
    ).await {
        Ok(logs) => (StatusCode::OK, Json(json!({"logs": logs}))),
        Err(Error::NotFound(msg)) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": msg, "code": "POD_NOT_FOUND"})),
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
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": msg})),
        )
    }
}
