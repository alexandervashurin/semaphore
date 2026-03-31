//! Namespace API handlers
//!
//! Handlers для управления Kubernetes namespaces

use crate::api::handlers::kubernetes::client::KubernetesClusterService;
use crate::api::state::AppState;
use crate::error::{Error, Result};
use axum::{
    extract::{Path, Query, State},
    Json,
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
