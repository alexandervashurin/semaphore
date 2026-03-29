//! Kubernetes Cluster Handlers — /api/kubernetes/clusters/{cluster_id}/...
//!
//! Фаза 1: cluster info, namespaces

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::api::state::AppState;
use crate::api::extractors::AuthUser;
use crate::kubernetes::service::NamespaceList;
use crate::error::Error;

// ──────────────────────────────────────────────────────────────────────────────
// GET /api/kubernetes/clusters
// ──────────────────────────────────────────────────────────────────────────────

/// Список доступных кластеров
///
/// GET /api/kubernetes/clusters
pub async fn list_clusters(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> (StatusCode, Json<Value>) {
    match &state.k8s {
        None => (
            StatusCode::OK,
            Json(json!({"clusters": [], "message": "Kubernetes не сконфигурирован"})),
        ),
        Some(mgr) => {
            let clusters = mgr.list_clusters();
            (StatusCode::OK, Json(json!({"clusters": clusters})))
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// GET /api/kubernetes/clusters/:cluster_id/info
// ──────────────────────────────────────────────────────────────────────────────

/// Информация о кластере (версия, доступность)
///
/// GET /api/kubernetes/clusters/{cluster_id}/info
pub async fn cluster_info(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(cluster_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    let mgr = match &state.k8s {
        Some(m) => m.clone(),
        None => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Kubernetes не сконфигурирован", "code": "K8S_NOT_CONFIGURED"})),
        ),
    };

    let svc = match mgr.get(&cluster_id).await {
        Ok(s) => s,
        Err(Error::NotFound(msg)) => return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": msg, "code": "CLUSTER_NOT_FOUND"})),
        ),
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    };

    let info = svc.cluster_info().await;

    // Статус ответа зависит от состояния кластера
    let http_status = if info.reachable {
        StatusCode::OK
    } else if info.status == "unauthorized" {
        StatusCode::UNAUTHORIZED
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (http_status, Json(json!(info)))
}

// ──────────────────────────────────────────────────────────────────────────────
// GET /api/kubernetes/clusters/:cluster_id/namespaces
// ──────────────────────────────────────────────────────────────────────────────

/// Query params для списка namespace'ов
#[derive(Debug, Deserialize)]
pub struct NamespaceListQuery {
    /// Макс. кол-во записей (1..500, default 100)
    pub limit: Option<u32>,
    /// Continue token для пагинации
    pub continue_token: Option<String>,
}

/// Список namespace'ов кластера
///
/// GET /api/kubernetes/clusters/{cluster_id}/namespaces
pub async fn list_namespaces(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(cluster_id): Path<String>,
    Query(q): Query<NamespaceListQuery>,
) -> (StatusCode, Json<Value>) {
    let mgr = match &state.k8s {
        Some(m) => m.clone(),
        None => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Kubernetes не сконфигурирован", "code": "K8S_NOT_CONFIGURED"})),
        ),
    };

    let svc = match mgr.get(&cluster_id).await {
        Ok(s) => s,
        Err(Error::NotFound(msg)) => return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": msg, "code": "CLUSTER_NOT_FOUND"})),
        ),
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    };

    match svc.list_namespaces(q.limit, q.continue_token).await {
        Ok(list) => (StatusCode::OK, Json(json!(list))),
        Err(e) => {
            let msg = e.to_string();
            // Прокидываем 403 от apiserver как 403 ответ
            if msg.contains("FORBIDDEN") {
                (
                    StatusCode::FORBIDDEN,
                    Json(json!({"error": "Нет прав на просмотр namespace'ов", "code": "K8S_FORBIDDEN", "detail": msg})),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": msg})),
                )
            }
        }
    }
}
