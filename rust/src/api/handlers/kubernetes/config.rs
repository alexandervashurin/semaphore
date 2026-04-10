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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_query_deserialize_default() {
        let q: ListQuery = serde_json::from_str("{}").unwrap();
        assert!(q.limit.is_none());
        assert!(q.continue_token.is_none());
    }

    #[test]
    fn test_list_query_deserialize_with_values() {
        let q: ListQuery = serde_json::from_str(r#"{"limit": 50, "continue_token": "abc"}"#).unwrap();
        assert_eq!(q.limit, Some(50));
        assert_eq!(q.continue_token, Some("abc".to_string()));
    }

    #[test]
    fn test_list_query_debug_format() {
        let q = ListQuery { limit: Some(10), continue_token: None };
        let debug_str = format!("{:?}", q);
        assert!(debug_str.contains("ListQuery"));
        assert!(debug_str.contains("10"));
    }

    #[test]
    fn test_update_config_map_body_deserialize() {
        let body: UpdateConfigMapBody = serde_json::from_str(
            r#"{"data": {"key1": "val1", "key2": "val2"}}"#
        ).unwrap();
        assert_eq!(body.data.get("key1"), Some(&"val1".to_string()));
        assert_eq!(body.data.get("key2"), Some(&"val2".to_string()));
    }

    #[test]
    fn test_update_config_map_body_empty_data() {
        let body: UpdateConfigMapBody = serde_json::from_str(
            r#"{"data": {}}"#
        ).unwrap();
        assert!(body.data.is_empty());
    }

    #[test]
    fn test_secret_get_query_default() {
        let q: SecretGetQuery = serde_json::from_str("{}").unwrap();
        assert!(q.reveal.is_none());
    }

    #[test]
    fn test_secret_get_query_reveal_true() {
        let q: SecretGetQuery = serde_json::from_str(r#"{"reveal": true}"#).unwrap();
        assert_eq!(q.reveal, Some(true));
    }

    #[test]
    fn test_secret_get_query_reveal_false() {
        let q: SecretGetQuery = serde_json::from_str(r#"{"reveal": false}"#).unwrap();
        assert_eq!(q.reveal, Some(false));
    }

    #[test]
    fn test_k8s_error_json_forbidden() {
        let err = Error::Forbidden("access denied".to_string());
        let (status, Json(val)) = k8s_err(err);
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(val["code"], "K8S_FORBIDDEN");
        assert_eq!(val["error"], "Нет прав");
    }

    #[test]
    fn test_k8s_error_json_not_found() {
        let err = Error::NotFound("item not found".to_string());
        let (status, Json(val)) = k8s_err(err);
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(val["error"], "item not found");
    }

    #[test]
    fn test_k8s_error_json_generic() {
        let err = Error::Kubernetes("some k8s error".to_string());
        let (status, Json(val)) = k8s_err(err);
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(val["error"], "some k8s error");
    }

    #[test]
    fn test_btree_map_ordering() {
        let mut data = BTreeMap::new();
        data.insert("z_key".to_string(), "z_val".to_string());
        data.insert("a_key".to_string(), "a_val".to_string());
        let body = UpdateConfigMapBody { data };
        let serialized = serde_json::to_string(&body).unwrap();
        assert!(serialized.find("a_key") < serialized.find("z_key"));
    }
}
