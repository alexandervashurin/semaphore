//! Kubernetes StatefulSet API handlers
//!
//! Управление StatefulSet: list, get, delete, scale

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::DateTime;
use chrono::Utc;
use k8s_openapi::api::apps::v1::StatefulSet;
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

// ============================================================================
// Types
// ============================================================================

/// Краткая информация о StatefulSet
#[derive(Debug, Serialize)]
pub struct StatefulSetSummary {
    pub name: String,
    pub namespace: String,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub current_replicas: i32,
    pub updated_replicas: i32,
    pub age: String,
}

/// Детальная информация о StatefulSet
#[derive(Debug, Serialize)]
pub struct StatefulSetDetail {
    pub name: String,
    pub namespace: String,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub current_replicas: i32,
    pub updated_replicas: i32,
    pub selector: BTreeMap<String, String>,
    pub template_labels: BTreeMap<String, String>,
    pub containers: Vec<ContainerInfo>,
    pub volume_claim_templates: Vec<VolumeClaimTemplate>,
    pub service_name: String,
    pub update_strategy: String,
    pub conditions: Vec<StatefulSetCondition>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Информация о контейнере
#[derive(Debug, Serialize)]
pub struct ContainerInfo {
    pub name: String,
    pub image: Option<String>,
}

/// Шаблон PVC
#[derive(Debug, Serialize)]
pub struct VolumeClaimTemplate {
    pub name: String,
    pub access_modes: Vec<String>,
    pub storage_class: Option<String>,
    pub storage_size: Option<String>,
}

/// Условие StatefulSet
#[derive(Debug, Serialize)]
pub struct StatefulSetCondition {
    #[serde(rename = "type")]
    pub condition_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
}

/// Query параметры для списка StatefulSets
#[derive(Debug, Deserialize)]
pub struct StatefulSetListQuery {
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
    pub limit: Option<u32>,
}

/// Payload для scale операции StatefulSet
#[derive(Debug, Deserialize)]
pub struct ScaleStatefulSetPayload {
    pub replicas: i32,
}

/// Ответ на операцию StatefulSet
#[derive(Debug, Serialize)]
pub struct StatefulSetOperationResponse {
    pub message: String,
    pub name: String,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replicas: Option<i32>,
}

// ============================================================================
// API Handlers
// ============================================================================

/// Получить список StatefulSets
pub async fn list_statefulsets(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StatefulSetListQuery>,
) -> Result<Json<Vec<StatefulSetSummary>>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let namespace = query.namespace.as_deref().unwrap_or("default");
    let api: Api<StatefulSet> = Api::namespaced(client, namespace);

    let mut lp = ListParams::default();
    if let Some(selector) = query.label_selector {
        lp.label_selector = Some(selector);
    }
    if let Some(limit) = query.limit {
        lp.limit = Some(limit);
    }

    let sfs_list = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to list statefulsets: {}", e)))?;

    let statefulsets = sfs_list.items.iter().map(statefulset_summary).collect();

    Ok(Json(statefulsets))
}

/// Получить StatefulSet по имени
pub async fn get_statefulset(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<StatefulSetDetail>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<StatefulSet> = Api::namespaced(client, &namespace);

    let sf = api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("StatefulSet {} not found: {}", name, e)))?;

    Ok(Json(statefulset_detail(&sf)))
}

/// Удалить StatefulSet
pub async fn delete_statefulset(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<StatefulSetOperationResponse>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<StatefulSet> = Api::namespaced(client, &namespace);

    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to delete statefulset: {}", e)))?;

    Ok(Json(StatefulSetOperationResponse {
        message: format!("StatefulSet {} deleted", name),
        name,
        namespace,
        replicas: None,
    }))
}

/// Scale StatefulSet
pub async fn scale_statefulset(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<ScaleStatefulSetPayload>,
) -> Result<Json<StatefulSetOperationResponse>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<StatefulSet> = Api::namespaced(client, &namespace);

    let mut sf = api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("StatefulSet {} not found: {}", name, e)))?;

    if let Some(spec) = sf.spec.as_mut() {
        spec.replicas = Some(payload.replicas);
    }

    api.replace(&name, &PostParams::default(), &sf)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to scale statefulset: {}", e)))?;

    Ok(Json(StatefulSetOperationResponse {
        message: format!("StatefulSet {} scaled to {} replicas", name, payload.replicas),
        name,
        namespace,
        replicas: Some(payload.replicas),
    }))
}

/// Получить pod'ы, управляемые StatefulSet
pub async fn list_statefulset_pods(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<Vec<serde_json::Value>>> {
    use k8s_openapi::api::core::v1::Pod;

    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let sf_api: Api<StatefulSet> = Api::namespaced(client.clone(), &namespace);
    let pod_api: Api<Pod> = Api::namespaced(client, &namespace);

    let sf = sf_api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("StatefulSet {} not found: {}", name, e)))?;

    let selector = sf
        .spec
        .and_then(|s| s.selector.match_labels)
        .unwrap_or_default();

    let label_selector = selector
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(",");

    let mut lp = ListParams::default();
    lp.label_selector = Some(label_selector);

    let pod_list = pod_api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to list pods: {}", e)))?;

    let pods = pod_list
        .items
        .iter()
        .map(|pod| {
            serde_json::json!({
                "name": pod.metadata.name.clone().unwrap_or_default(),
                "namespace": pod.metadata.namespace.clone().unwrap_or("default".to_string()),
                "status": pod.status.as_ref().and_then(|s| s.phase.clone()).unwrap_or_default(),
                "node": pod.spec.as_ref().and_then(|s| s.node_name.clone()),
            })
        })
        .collect();

    Ok(Json(pods))
}

// ============================================================================
// Helper Functions
// ============================================================================

fn statefulset_summary(sf: &StatefulSet) -> StatefulSetSummary {
    let name = sf.metadata.name.clone().unwrap_or_default();
    let namespace = sf
        .metadata
        .namespace
        .clone()
        .unwrap_or("default".to_string());

    let status = sf.status.as_ref();
    let spec = sf.spec.as_ref();

    let replicas = spec.and_then(|s| s.replicas).unwrap_or(1);
    let ready_replicas = status.and_then(|s| s.ready_replicas).unwrap_or(0);
    let current_replicas = status.and_then(|s| s.current_replicas).unwrap_or(0);
    let updated_replicas = status.and_then(|s| s.updated_replicas).unwrap_or(0);

    let age = sf
        .metadata
        .creation_timestamp
        .as_ref()
        .map(|ts| format_age(&ts.0))
        .unwrap_or_else(|| "unknown".to_string());

    StatefulSetSummary {
        name,
        namespace,
        replicas,
        ready_replicas,
        current_replicas,
        updated_replicas,
        age,
    }
}

fn statefulset_detail(sf: &StatefulSet) -> StatefulSetDetail {
    let status = sf.status.as_ref();
    let spec = sf.spec.as_ref();

    let selector = spec
        .and_then(|s| s.selector.match_labels.clone())
        .unwrap_or_default();

    let template_labels = spec
        .and_then(|s| s.template.metadata.as_ref())
        .and_then(|m| m.labels.clone())
        .unwrap_or_default();

    let containers = spec
        .and_then(|s| s.template.spec.as_ref())
        .map(|ps| {
            ps.containers
                .iter()
                .map(|c| ContainerInfo {
                    name: c.name.clone(),
                    image: c.image.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    let volume_claim_templates = spec
        .and_then(|s| s.volume_claim_templates.as_ref())
        .map(|templates| {
            templates
                .iter()
                .map(|t| {
                    let storage_class = t
                        .spec
                        .as_ref()
                        .and_then(|s| s.storage_class_name.clone());

                    let storage_size = t
                        .spec
                        .as_ref()
                        .and_then(|s| s.resources.as_ref())
                        .and_then(|r| r.requests.as_ref())
                        .and_then(|req| req.get("storage"))
                        .map(|q| q.0.clone());

                    VolumeClaimTemplate {
                        name: t.metadata.name.clone().unwrap_or_default(),
                        access_modes: t
                            .spec
                            .as_ref()
                            .and_then(|s| s.access_modes.as_ref())
                            .map(|modes| modes.iter().map(|m| m.to_string()).collect())
                            .unwrap_or_default(),
                        storage_class,
                        storage_size,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let service_name = spec
        .and_then(|s| Some(s.service_name.clone()))
        .unwrap_or_default();

    let update_strategy = spec
        .and_then(|s| s.update_strategy.as_ref())
        .and_then(|us| us.type_.as_ref())
        .cloned()
        .unwrap_or_else(|| "RollingUpdate".to_string());

    let conditions = status
        .and_then(|s| s.conditions.clone())
        .map(|conds| {
            conds.iter()
                .map(|c| StatefulSetCondition {
                    condition_type: c.type_.clone(),
                    status: c.status.clone(),
                    reason: c.reason.clone(),
                    message: c.message.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    StatefulSetDetail {
        name: sf.metadata.name.clone().unwrap_or_default(),
        namespace: sf
            .metadata
            .namespace
            .clone()
            .unwrap_or("default".to_string()),
        replicas: spec.and_then(|s| s.replicas).unwrap_or(1),
        ready_replicas: status.and_then(|s| s.ready_replicas).unwrap_or(0),
        current_replicas: status.and_then(|s| s.current_replicas).unwrap_or(0),
        updated_replicas: status.and_then(|s| s.updated_replicas).unwrap_or(0),
        selector,
        template_labels,
        containers,
        volume_claim_templates,
        service_name,
        update_strategy,
        conditions,
        created_at: sf.metadata.creation_timestamp.as_ref().map(|t| t.0),
    }
}

fn format_age(time: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*time);

    if duration.num_days() > 365 {
        format!("{}y", duration.num_days() / 365)
    } else if duration.num_days() > 30 {
        format!("{}d", duration.num_days() / 30)
    } else if duration.num_days() > 0 {
        format!("{}d", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{}h", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{}m", duration.num_minutes())
    } else {
        format!("{}s", duration.num_seconds())
    }
}
