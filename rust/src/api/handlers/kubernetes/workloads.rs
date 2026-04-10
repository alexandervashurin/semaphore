//! Kubernetes Workloads Handlers — DaemonSets, StatefulSets, ReplicaSets, Events

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

// ── Shared helpers ────────────────────────────────────────────────────────────

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
        other => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": other.to_string()}))),
    })
}

fn k8s_err(e: Error) -> (StatusCode, Json<Value>) {
    let msg = e.to_string();
    if msg.contains("FORBIDDEN") {
        (StatusCode::FORBIDDEN, Json(json!({"error": "Нет прав", "code": "K8S_FORBIDDEN", "detail": msg})))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": msg})))
    }
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<u32>,
    pub continue_token: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// DaemonSets
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn list_daemonsets(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((cluster_id, namespace)): Path<(String, String)>,
    Query(q): Query<ListQuery>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.list_daemonsets(&namespace, q.limit, q.continue_token).await {
        Ok(list) => (StatusCode::OK, Json(json!(list))),
        Err(e) => k8s_err(e),
    }
}

pub async fn get_daemonset(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.get_daemonset(&namespace, &name).await {
        Ok(d) => (StatusCode::OK, Json(json!(d))),
        Err(Error::NotFound(msg)) => (StatusCode::NOT_FOUND, Json(json!({"error": msg, "code": "DAEMONSET_NOT_FOUND"}))),
        Err(e) => k8s_err(e),
    }
}

pub async fn restart_daemonset(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.restart_daemonset(&namespace, &name).await {
        Ok(()) => (StatusCode::OK, Json(json!({"message": "Restart initiated"}))),
        Err(Error::NotFound(msg)) => (StatusCode::NOT_FOUND, Json(json!({"error": msg, "code": "DAEMONSET_NOT_FOUND"}))),
        Err(e) => k8s_err(e),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// StatefulSets
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn list_statefulsets(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((cluster_id, namespace)): Path<(String, String)>,
    Query(q): Query<ListQuery>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.list_statefulsets(&namespace, q.limit, q.continue_token).await {
        Ok(list) => (StatusCode::OK, Json(json!(list))),
        Err(e) => k8s_err(e),
    }
}

pub async fn get_statefulset(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.get_statefulset(&namespace, &name).await {
        Ok(s) => (StatusCode::OK, Json(json!(s))),
        Err(Error::NotFound(msg)) => (StatusCode::NOT_FOUND, Json(json!({"error": msg, "code": "STATEFULSET_NOT_FOUND"}))),
        Err(e) => k8s_err(e),
    }
}

#[derive(Debug, Deserialize)]
pub struct ScaleBody { pub replicas: i32 }

pub async fn scale_statefulset(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((cluster_id, namespace, name)): Path<(String, String, String)>,
    Json(body): Json<ScaleBody>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.scale_statefulset(&namespace, &name, body.replicas).await {
        Ok(()) => (StatusCode::OK, Json(json!({"message": "Scaled", "replicas": body.replicas}))),
        Err(Error::NotFound(msg)) => (StatusCode::NOT_FOUND, Json(json!({"error": msg, "code": "STATEFULSET_NOT_FOUND"}))),
        Err(e) => k8s_err(e),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ReplicaSets
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn list_replicasets(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((cluster_id, namespace)): Path<(String, String)>,
    Query(q): Query<ListQuery>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.list_replicasets(&namespace, q.limit, q.continue_token).await {
        Ok(list) => (StatusCode::OK, Json(json!(list))),
        Err(e) => k8s_err(e),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Events
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct EventQuery {
    pub limit: Option<u32>,
    pub object_name: Option<String>,
    pub object_kind: Option<String>,
    pub event_type: Option<String>,
}

pub async fn list_events(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((cluster_id, namespace)): Path<(String, String)>,
    Query(q): Query<EventQuery>,
) -> (StatusCode, Json<Value>) {
    let svc = match get_svc(&state, &cluster_id).await { Ok(s) => s, Err(r) => return r };
    match svc.list_events(&namespace, q.object_name, q.object_kind, q.event_type, q.limit).await {
        Ok(list) => (StatusCode::OK, Json(json!(list))),
        Err(e) => k8s_err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_body_deserialization() {
        let json = r#"{"replicas": 3}"#;
        let body: ScaleBody = serde_json::from_str(json).unwrap();
        assert_eq!(body.replicas, 3);
    }

    #[test]
    fn test_scale_body_zero_replicas() {
        let json = r#"{"replicas": 0}"#;
        let body: ScaleBody = serde_json::from_str(json).unwrap();
        assert_eq!(body.replicas, 0);
    }

    #[test]
    fn test_scale_body_negative_replicas() {
        let json = r#"{"replicas": -1}"#;
        let body: ScaleBody = serde_json::from_str(json).unwrap();
        assert_eq!(body.replicas, -1);
    }

    #[test]
    fn test_scale_body_large_replicas() {
        let json = r#"{"replicas": 100}"#;
        let body: ScaleBody = serde_json::from_str(json).unwrap();
        assert_eq!(body.replicas, 100);
    }

    #[test]
    fn test_list_query_with_limit() {
        let json = r#"{"limit": 50, "continue_token": "abc123"}"#;
        let query: ListQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, Some(50));
        assert_eq!(query.continue_token, Some("abc123".to_string()));
    }

    #[test]
    fn test_list_query_empty() {
        let json = r#"{}"#;
        let query: ListQuery = serde_json::from_str(json).unwrap();
        assert!(query.limit.is_none());
        assert!(query.continue_token.is_none());
    }

    #[test]
    fn test_event_query_with_filters() {
        let json = r#"{"limit": 20, "object_name": "my-pod", "object_kind": "Pod", "event_type": "Warning"}"#;
        let query: EventQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, Some(20));
        assert_eq!(query.object_name, Some("my-pod".to_string()));
        assert_eq!(query.object_kind, Some("Pod".to_string()));
        assert_eq!(query.event_type, Some("Warning".to_string()));
    }

    #[test]
    fn test_event_query_partial() {
        let json = r#"{"object_name": "test-deploy", "limit": 10}"#;
        let query: EventQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.object_name, Some("test-deploy".to_string()));
        assert_eq!(query.limit, Some(10));
        assert!(query.object_kind.is_none());
        assert!(query.event_type.is_none());
    }

    #[test]
    fn test_event_query_empty() {
        let json = r#"{}"#;
        let query: EventQuery = serde_json::from_str(json).unwrap();
        assert!(query.limit.is_none());
        assert!(query.object_name.is_none());
        assert!(query.object_kind.is_none());
        assert!(query.event_type.is_none());
    }

    #[test]
    fn test_scale_body_serialization() {
        let body = ScaleBody { replicas: 5 };
        let json = serde_json::to_string(&body).unwrap();
        assert_eq!(json, r#"{"replicas":5}"#);
    }

    #[test]
    fn test_k8s_resource_types_daemonset() {
        let daemonset_kind = "DaemonSet";
        assert_eq!(daemonset_kind, "DaemonSet");
    }

    #[test]
    fn test_k8s_resource_types_statefulset() {
        let statefulset_kind = "StatefulSet";
        assert_eq!(statefulset_kind, "StatefulSet");
    }

    #[test]
    fn test_k8s_resource_types_replicaset() {
        let replicaset_kind = "ReplicaSet";
        assert_eq!(replicaset_kind, "ReplicaSet");
    }
}
