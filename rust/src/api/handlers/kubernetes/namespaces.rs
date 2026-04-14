//! Namespace API handlers
//!
//! Handlers для управления Kubernetes namespaces

use crate::api::handlers::kubernetes::client::KubernetesClusterService;
use crate::api::state::AppState;
use crate::error::{Error, Result};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use k8s_openapi::api::core::v1::{LimitRange, Namespace, ResourceQuota};
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

use super::types::NamespaceSummary;

/// Query параметры для list namespaces
#[derive(Debug, Deserialize)]
pub struct ListNamespacesQuery {
    pub label_selector: Option<String>,
    pub limit: Option<i32>,
}

/// Payload для создания namespace
#[derive(Debug, Deserialize)]
pub struct CreateNamespacePayload {
    pub name: String,
    pub labels: Option<BTreeMap<String, String>>,
    pub annotations: Option<BTreeMap<String, String>>,
}

/// Payload для обновления namespace
#[derive(Debug, Deserialize)]
pub struct UpdateNamespacePayload {
    pub labels: Option<BTreeMap<String, String>>,
    pub annotations: Option<BTreeMap<String, String>>,
}

/// Список namespace'ов
/// GET /api/kubernetes/namespaces
pub async fn list_namespaces(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListNamespacesQuery>,
) -> Result<Json<Vec<NamespaceSummary>>> {
    let client = state.kubernetes_client()?;
    let service = KubernetesClusterService::new(client);

    let namespaces = service.list_namespaces().await?;

    let summaries = namespaces
        .iter()
        .map(|ns| NamespaceSummary {
            name: ns
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            uid: ns
                .get("uid")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            status: ns
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            created_at: ns
                .get("created_at")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            labels: ns
                .get("labels")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default(),
            annotations: ns
                .get("annotations")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default(),
            pods_count: None,
            services_count: None,
            deployments_count: None,
        })
        .collect();

    Ok(Json(summaries))
}

/// Детали namespace
/// GET /api/kubernetes/namespaces/{name}
pub async fn get_namespace(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let service = KubernetesClusterService::new(client);

    let ns = service.get_namespace(&name).await?;
    Ok(Json(ns))
}

/// Создать namespace
/// POST /api/kubernetes/namespaces
pub async fn create_namespace(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateNamespacePayload>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let service = KubernetesClusterService::new(client);

    let created = service
        .create_namespace(&payload.name, payload.labels.clone())
        .await?;
    Ok(Json(created))
}

/// Обновить namespace
/// PUT /api/kubernetes/namespaces/{name}
pub async fn update_namespace(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<UpdateNamespacePayload>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Namespace> = client.api_all();

    // Получаем текущий namespace
    let mut ns = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    // Обновляем labels и annotations
    if let Some(labels) = payload.labels {
        ns.metadata.labels = Some(labels);
    }

    if let Some(annotations) = payload.annotations {
        ns.metadata.annotations = Some(annotations);
    }

    let updated = api
        .replace(&name, &Default::default(), &ns)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(serde_json::json!(updated)))
}

/// Удалить namespace
/// DELETE /api/kubernetes/namespaces/{name}
pub async fn delete_namespace(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let service = KubernetesClusterService::new(client);

    service.delete_namespace(&name).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Namespace {} deleted", name)
    })))
}

/// Получить ResourceQuota namespace
/// GET /api/kubernetes/namespaces/{name}/quota
pub async fn get_namespace_quota(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Vec<ResourceQuota>>> {
    let client = state.kubernetes_client()?;
    let api: Api<ResourceQuota> = client.api::<ResourceQuota>(Some(&name));

    let quotas = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(quotas.items))
}

/// Получить LimitRange namespace
/// GET /api/kubernetes/namespaces/{name}/limits
pub async fn get_namespace_limits(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Vec<LimitRange>>> {
    let client = state.kubernetes_client()?;
    let api: Api<LimitRange> = client.api::<LimitRange>(Some(&name));

    let limits = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(limits.items))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_list_namespaces_query_deserialization() {
        let json = r#"{"label_selector":"env=prod","limit":20}"#;
        let query: ListNamespacesQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.label_selector, Some("env=prod".to_string()));
        assert_eq!(query.limit, Some(20));
    }

    #[test]
    fn test_list_namespaces_query_all_optional() {
        let json = r#"{}"#;
        let query: ListNamespacesQuery = serde_json::from_str(json).unwrap();
        assert!(query.label_selector.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_create_namespace_payload_deserialization() {
        let json = r#"{"name":"my-namespace","labels":{"env":"prod"},"annotations":{"description":"test ns"}}"#;
        let payload: CreateNamespacePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "my-namespace");
        assert!(payload.labels.is_some());
        assert!(payload.annotations.is_some());
    }

    #[test]
    fn test_create_namespace_payload_minimal() {
        let json = r#"{"name":"simple-ns"}"#;
        let payload: CreateNamespacePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "simple-ns");
        assert!(payload.labels.is_none());
        assert!(payload.annotations.is_none());
    }

    #[test]
    fn test_create_namespace_payload_labels_map() {
        let json = r#"{"name":"test","labels":{"team":"infra","tier":"backend"}}"#;
        let payload: CreateNamespacePayload = serde_json::from_str(json).unwrap();
        let labels = payload.labels.unwrap();
        assert_eq!(labels.get("team").unwrap(), "infra");
        assert_eq!(labels.get("tier").unwrap(), "backend");
    }

    #[test]
    fn test_update_namespace_payload_deserialization() {
        let json = r#"{"labels":{"app":"web"},"annotations":{"updated":"true"}}"#;
        let payload: UpdateNamespacePayload = serde_json::from_str(json).unwrap();
        let labels = payload.labels.unwrap();
        assert_eq!(labels.get("app").unwrap(), "web");
        let annotations = payload.annotations.unwrap();
        assert_eq!(annotations.get("updated").unwrap(), "true");
    }

    #[test]
    fn test_update_namespace_payload_partial() {
        let json = r#"{"labels":{"role":"worker"}}"#;
        let payload: UpdateNamespacePayload = serde_json::from_str(json).unwrap();
        assert!(payload.labels.is_some());
        assert!(payload.annotations.is_none());
    }

    #[test]
    fn test_update_namespace_payload_both_none() {
        let json = r#"{}"#;
        let payload: UpdateNamespacePayload = serde_json::from_str(json).unwrap();
        assert!(payload.labels.is_none());
        assert!(payload.annotations.is_none());
    }

    #[test]
    fn test_namespace_summary_serialization() {
        let mut labels = HashMap::new();
        labels.insert("env".to_string(), "staging".to_string());
        let summary = NamespaceSummary {
            name: "staging-ns".to_string(),
            uid: "abc-123".to_string(),
            status: "Active".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            labels,
            annotations: HashMap::new(),
            pods_count: Some(10),
            services_count: Some(5),
            deployments_count: Some(3),
        };
        let value = serde_json::to_value(&summary).unwrap();
        assert_eq!(value["name"], "staging-ns");
        assert_eq!(value["uid"], "abc-123");
        assert_eq!(value["status"], "Active");
        assert_eq!(value["labels"]["env"], "staging");
        assert_eq!(value["pods_count"], 10);
        assert_eq!(value["services_count"], 5);
        assert_eq!(value["deployments_count"], 3);
    }

    #[test]
    fn test_namespace_summary_none_counts() {
        let summary = NamespaceSummary {
            name: "test".to_string(),
            uid: "uid".to_string(),
            status: "Unknown".to_string(),
            created_at: "unknown".to_string(),
            labels: HashMap::new(),
            annotations: HashMap::new(),
            pods_count: None,
            services_count: None,
            deployments_count: None,
        };
        let value = serde_json::to_value(&summary).unwrap();
        assert!(value["pods_count"].is_null());
        assert!(value["services_count"].is_null());
        assert!(value["deployments_count"].is_null());
    }

    #[test]
    fn test_k8s_resource_types() {
        let _ns_type = std::any::type_name::<Namespace>();
        let _quota_type = std::any::type_name::<ResourceQuota>();
        let _limit_type = std::any::type_name::<LimitRange>();
        assert!(_ns_type.contains("Namespace"));
        assert!(_quota_type.contains("ResourceQuota"));
        assert!(_limit_type.contains("LimitRange"));
    }

    #[test]
    fn test_k8s_namespace_api_resource() {
        let ns = Namespace {
            metadata: kube::api::ObjectMeta {
                name: Some("test-ns".to_string()),
                ..Default::default()
            },
            spec: None,
            status: None,
        };
        assert_eq!(ns.metadata.name, Some("test-ns".to_string()));
    }

    #[test]
    fn test_k8s_resource_quota_structure() {
        let quota = ResourceQuota {
            metadata: kube::api::ObjectMeta {
                name: Some("compute-resources".to_string()),
                namespace: Some("default".to_string()),
                ..Default::default()
            },
            spec: None,
            status: None,
        };
        assert_eq!(quota.metadata.name, Some("compute-resources".to_string()));
        assert_eq!(quota.metadata.namespace, Some("default".to_string()));
    }

    #[test]
    fn test_k8s_limit_range_structure() {
        let lr = LimitRange {
            metadata: kube::api::ObjectMeta {
                name: Some("default-limits".to_string()),
                namespace: Some("production".to_string()),
                ..Default::default()
            },
            spec: None,
        };
        assert_eq!(lr.metadata.name, Some("default-limits".to_string()));
        assert_eq!(lr.metadata.namespace, Some("production".to_string()));
    }
}
