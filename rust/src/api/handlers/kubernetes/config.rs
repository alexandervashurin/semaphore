//! Kubernetes Config Handlers — ConfigMaps, Secrets

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::Arc;
use crate::api::state::AppState;
use crate::api::extractors::AuthUser;
use crate::error::Error;

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
    } else if msg.contains("not found") {
        (StatusCode::NOT_FOUND, Json(json!({"error": msg})))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": msg})))
    }
}

#[derive(Debug, Deserialize)]
pub struct ListQuery { pub limit: Option<u32>, pub continue_token: Option<String> }

// ═══════════════════════════════════════════════════════════════════════════════
// ConfigMaps
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn list_configmaps(
    State(state): State<Arc<AppState>>, _auth: AuthUser,
    Path((cluster_id, namespace)): Path<(String, String)>,
    Query(q): Query<ListQuery>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.list_configmaps(&namespace, q.limit, q.continue_token).await {
        Ok(list) => (StatusCode::OK, Json(json!(list))),
        Err(e) => k8s_err(e),
    }
}

pub async fn get_configmap(
    State(state): State<Arc<AppState>>, _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.get_configmap(&namespace, &name).await {
        Ok(cm) => (StatusCode::OK, Json(json!(cm))),
        Err(Error::NotFound(msg)) => (StatusCode::NOT_FOUND, Json(json!({"error": msg, "code": "CONFIGMAP_NOT_FOUND"}))),
        Err(e) => k8s_err(e),
    }
}

pub async fn delete_configmap(
    State(state): State<Arc<AppState>>, _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.delete_configmap(&namespace, &name).await {
        Ok(()) => (StatusCode::OK, Json(json!({"message": "ConfigMap deleted"}))),
        Err(Error::NotFound(msg)) => (StatusCode::NOT_FOUND, Json(json!({"error": msg, "code": "CONFIGMAP_NOT_FOUND"}))),
        Err(e) => k8s_err(e),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigMapBody { pub data: BTreeMap<String, String> }

pub async fn update_configmap(
    State(state): State<Arc<AppState>>, _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
    Json(body): Json<UpdateConfigMapBody>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.update_configmap(&namespace, &name, body.data).await {
        Ok(cm) => (StatusCode::OK, Json(json!(cm))),
        Err(Error::NotFound(msg)) => (StatusCode::NOT_FOUND, Json(json!({"error": msg, "code": "CONFIGMAP_NOT_FOUND"}))),
        Err(e) => k8s_err(e),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Secrets  — значения НИКОГДА не возвращаются без явного reveal=true
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn list_secrets(
    State(state): State<Arc<AppState>>, _auth: AuthUser,
    Path((cluster_id, namespace)): Path<(String, String)>,
    Query(q): Query<ListQuery>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.list_secrets(&namespace, q.limit, q.continue_token).await {
        Ok(list) => (StatusCode::OK, Json(json!(list))),
        Err(e) => k8s_err(e),
    }
}

#[derive(Debug, Deserialize)]
pub struct SecretGetQuery { pub reveal: Option<bool> }

pub async fn get_secret(
    State(state): State<Arc<AppState>>, _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
    Query(q): Query<SecretGetQuery>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    // Получаем сырой Secret через отдельный метод reveal
    match svc.get_secret_raw(&namespace, &name, q.reveal.unwrap_or(false)).await {
        Ok(s) => (StatusCode::OK, Json(json!(s))),
        Err(Error::NotFound(msg)) => (StatusCode::NOT_FOUND, Json(json!({"error": msg, "code": "SECRET_NOT_FOUND"}))),
        Err(e) => k8s_err(e),
    }
}

pub async fn delete_secret(
    State(state): State<Arc<AppState>>, _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.delete_secret(&namespace, &name).await {
        Ok(()) => (StatusCode::OK, Json(json!({"message": "Secret deleted"}))),
        Err(Error::NotFound(msg)) => (StatusCode::NOT_FOUND, Json(json!({"error": msg, "code": "SECRET_NOT_FOUND"}))),
        Err(e) => k8s_err(e),
    }
}
