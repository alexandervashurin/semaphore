//! Kubernetes Networking Handlers — Services, Ingress

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

// ── shared helpers ────────────────────────────────────────────────────────────

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
        Error::NotFound(msg) => (StatusCode::NOT_FOUND, Json(json!({"error": msg, "code": "CLUSTER_NOT_FOUND"}))),
        other => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": other.to_string()}))),
    })
}

fn k8s_err(e: Error) -> (StatusCode, Json<Value>) {
    let msg = e.to_string();
    if msg.contains("FORBIDDEN") {
        (StatusCode::FORBIDDEN, Json(json!({"error": "Нет прав", "code": "K8S_FORBIDDEN", "detail": msg})))
    } else if msg.starts_with("NotFound") || msg.contains("not found") {
        (StatusCode::NOT_FOUND, Json(json!({"error": msg})))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": msg})))
    }
}

#[derive(Debug, Deserialize)]
pub struct ListQuery { pub limit: Option<u32>, pub continue_token: Option<String> }

// ═══════════════════════════════════════════════════════════════════════════════
// Services
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn list_services(
    State(state): State<Arc<AppState>>, _auth: AuthUser,
    Path((cluster_id, namespace)): Path<(String, String)>,
    Query(q): Query<ListQuery>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.list_services(&namespace, q.limit, q.continue_token).await {
        Ok(list) => (StatusCode::OK, Json(json!(list))),
        Err(e) => k8s_err(e),
    }
}

pub async fn get_service(
    State(state): State<Arc<AppState>>, _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.get_service(&namespace, &name).await {
        Ok(s) => (StatusCode::OK, Json(json!(s))),
        Err(Error::NotFound(msg)) => (StatusCode::NOT_FOUND, Json(json!({"error": msg, "code": "SERVICE_NOT_FOUND"}))),
        Err(e) => k8s_err(e),
    }
}

pub async fn delete_service(
    State(state): State<Arc<AppState>>, _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.delete_service(&namespace, &name).await {
        Ok(()) => (StatusCode::OK, Json(json!({"message": "Service deleted"}))),
        Err(Error::NotFound(msg)) => (StatusCode::NOT_FOUND, Json(json!({"error": msg, "code": "SERVICE_NOT_FOUND"}))),
        Err(e) => k8s_err(e),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Ingress
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn list_ingresses(
    State(state): State<Arc<AppState>>, _auth: AuthUser,
    Path((cluster_id, namespace)): Path<(String, String)>,
    Query(q): Query<ListQuery>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.list_ingresses(&namespace, q.limit, q.continue_token).await {
        Ok(list) => (StatusCode::OK, Json(json!(list))),
        Err(e) => k8s_err(e),
    }
}

pub async fn get_ingress(
    State(state): State<Arc<AppState>>, _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.get_ingress(&namespace, &name).await {
        Ok(ing) => (StatusCode::OK, Json(json!(ing))),
        Err(Error::NotFound(msg)) => (StatusCode::NOT_FOUND, Json(json!({"error": msg, "code": "INGRESS_NOT_FOUND"}))),
        Err(e) => k8s_err(e),
    }
}

pub async fn delete_ingress(
    State(state): State<Arc<AppState>>, _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.delete_ingress(&namespace, &name).await {
        Ok(()) => (StatusCode::OK, Json(json!({"message": "Ingress deleted"}))),
        Err(Error::NotFound(msg)) => (StatusCode::NOT_FOUND, Json(json!({"error": msg, "code": "INGRESS_NOT_FOUND"}))),
        Err(e) => k8s_err(e),
    }
}
