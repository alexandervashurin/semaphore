//! Kubernetes ReplicaSet API handlers
//!
//! Управление ReplicaSet: list, get, delete

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::DateTime;
use chrono::Utc;
use k8s_openapi::api::apps::v1::ReplicaSet;
use kube::api::{Api, DeleteParams, ListParams};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

// ============================================================================
// Types
// ============================================================================

/// Краткая информация о ReplicaSet
#[derive(Debug, Serialize)]
pub struct ReplicaSetSummary {
    pub name: String,
    pub namespace: String,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub available_replicas: i32,
    pub age: String,
    pub owner: Option<String>,
}

/// Детальная информация о ReplicaSet
#[derive(Debug, Serialize)]
pub struct ReplicaSetDetail {
    pub name: String,
    pub namespace: String,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub available_replicas: i32,
    pub selector: BTreeMap<String, String>,
    pub template_labels: BTreeMap<String, String>,
    pub containers: Vec<ContainerInfo>,
    pub owner_references: Vec<OwnerReference>,
    pub conditions: Vec<ReplicaSetCondition>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Информация о контейнере
#[derive(Debug, Serialize)]
pub struct ContainerInfo {
    pub name: String,
    pub image: Option<String>,
}

/// Владелец ReplicaSet
#[derive(Debug, Serialize)]
pub struct OwnerReference {
    pub api_version: String,
    pub kind: String,
    pub name: String,
    pub uid: String,
}

/// Условие ReplicaSet
#[derive(Debug, Serialize)]
pub struct ReplicaSetCondition {
    #[serde(rename = "type")]
    pub condition_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
}

/// Query параметры для списка ReplicaSets
#[derive(Debug, Deserialize)]
pub struct ReplicaSetListQuery {
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
    pub limit: Option<u32>,
}

/// Ответ на операцию ReplicaSet
#[derive(Debug, Serialize)]
pub struct ReplicaSetOperationResponse {
    pub message: String,
    pub name: String,
    pub namespace: String,
}

// ============================================================================
// API Handlers
// ============================================================================

/// Получить список ReplicaSets
pub async fn list_replicasets(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ReplicaSetListQuery>,
) -> Result<Json<Vec<ReplicaSetSummary>>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let namespace = query.namespace.as_deref().unwrap_or("default");
    let api: Api<ReplicaSet> = Api::namespaced(client, namespace);

    let mut lp = ListParams::default();
    if let Some(selector) = query.label_selector {
        lp.label_selector = Some(selector);
    }
    if let Some(limit) = query.limit {
        lp.limit = Some(limit);
    }

    let rs_list = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to list replicasets: {}", e)))?;

    let replicasets = rs_list.items.iter().map(replicaset_summary).collect();

    Ok(Json(replicasets))
}

/// Получить ReplicaSet по имени
pub async fn get_replicaset(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<ReplicaSetDetail>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<ReplicaSet> = Api::namespaced(client, &namespace);

    let rs = api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("ReplicaSet {} not found: {}", name, e)))?;

    Ok(Json(replicaset_detail(&rs)))
}

/// Удалить ReplicaSet
pub async fn delete_replicaset(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<ReplicaSetOperationResponse>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<ReplicaSet> = Api::namespaced(client, &namespace);

    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to delete replicaset: {}", e)))?;

    Ok(Json(ReplicaSetOperationResponse {
        message: format!("ReplicaSet {} deleted", name),
        name,
        namespace,
    }))
}

/// Получить pod'ы, управляемые ReplicaSet
pub async fn list_replicaset_pods(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<Vec<serde_json::Value>>> {
    use k8s_openapi::api::core::v1::Pod;

    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let rs_api: Api<ReplicaSet> = Api::namespaced(client.clone(), &namespace);
    let pod_api: Api<Pod> = Api::namespaced(client, &namespace);

    let rs = rs_api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("ReplicaSet {} not found: {}", name, e)))?;

    let selector = rs
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
                "ip": pod.status.as_ref().and_then(|s| s.pod_ip.clone()),
            })
        })
        .collect();

    Ok(Json(pods))
}

// ============================================================================
// Helper Functions
// ============================================================================

fn replicaset_summary(rs: &ReplicaSet) -> ReplicaSetSummary {
    let name = rs.metadata.name.clone().unwrap_or_default();
    let namespace = rs
        .metadata
        .namespace
        .clone()
        .unwrap_or("default".to_string());

    let status = rs.status.as_ref();
    let spec = rs.spec.as_ref();

    let replicas = spec.and_then(|s| s.replicas).unwrap_or(1);
    let ready_replicas = status.and_then(|s| s.ready_replicas).unwrap_or(0);
    let available_replicas = status.and_then(|s| s.available_replicas).unwrap_or(0);

    let owner = rs
        .metadata
        .owner_references
        .as_ref()
        .and_then(|refs| refs.first())
        .map(|r| r.name.clone());

    let age = rs
        .metadata
        .creation_timestamp
        .as_ref()
        .map(|ts| format_age(&ts.0))
        .unwrap_or_else(|| "unknown".to_string());

    ReplicaSetSummary {
        name,
        namespace,
        replicas,
        ready_replicas,
        available_replicas,
        age,
        owner,
    }
}

fn replicaset_detail(rs: &ReplicaSet) -> ReplicaSetDetail {
    let status = rs.status.as_ref();
    let spec = rs.spec.as_ref();

    let selector = spec
        .and_then(|s| s.selector.match_labels.clone())
        .unwrap_or_default();

    let template_labels = spec
        .and_then(|s| s.template.as_ref())
        .and_then(|t| t.metadata.as_ref())
        .and_then(|m| m.labels.clone())
        .unwrap_or_default();

    let containers = spec
        .and_then(|s| s.template.as_ref())
        .and_then(|t| t.spec.as_ref())
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

    let owner_references = rs
        .metadata
        .owner_references
        .as_ref()
        .map(|refs| {
            refs.iter()
                .map(|r| OwnerReference {
                    api_version: r.api_version.clone(),
                    kind: r.kind.clone(),
                    name: r.name.clone(),
                    uid: r.uid.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    let conditions = status
        .and_then(|s| s.conditions.clone())
        .map(|conds| {
            conds.iter()
                .map(|c| ReplicaSetCondition {
                    condition_type: c.type_.clone(),
                    status: c.status.clone(),
                    reason: c.reason.clone(),
                    message: c.message.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    ReplicaSetDetail {
        name: rs.metadata.name.clone().unwrap_or_default(),
        namespace: rs
            .metadata
            .namespace
            .clone()
            .unwrap_or("default".to_string()),
        replicas: spec.and_then(|s| s.replicas).unwrap_or(1),
        ready_replicas: status.and_then(|s| s.ready_replicas).unwrap_or(0),
        available_replicas: status.and_then(|s| s.available_replicas).unwrap_or(0),
        selector,
        template_labels,
        containers,
        owner_references,
        conditions,
        created_at: rs.metadata.creation_timestamp.as_ref().map(|t| t.0),
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
