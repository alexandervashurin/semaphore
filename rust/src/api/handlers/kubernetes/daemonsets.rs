//! Kubernetes DaemonSet API handlers
//!
//! Управление DaemonSet: list, get, delete

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::DateTime;
use chrono::Utc;
use k8s_openapi::api::apps::v1::DaemonSet;
use kube::api::{Api, DeleteParams, ListParams};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

// ============================================================================
// Types
// ============================================================================

/// Краткая информация о DaemonSet
#[derive(Debug, Serialize)]
pub struct DaemonSetSummary {
    pub name: String,
    pub namespace: String,
    pub desired_number_scheduled: i32,
    pub current_number_scheduled: i32,
    pub number_ready: i32,
    pub number_available: i32,
    pub number_misscheduled: i32,
    pub age: String,
}

/// Детальная информация о DaemonSet
#[derive(Debug, Serialize)]
pub struct DaemonSetDetail {
    pub name: String,
    pub namespace: String,
    pub desired_number_scheduled: i32,
    pub current_number_scheduled: i32,
    pub number_ready: i32,
    pub number_available: i32,
    pub number_misscheduled: i32,
    pub selector: BTreeMap<String, String>,
    pub template_labels: BTreeMap<String, String>,
    pub containers: Vec<ContainerInfo>,
    pub conditions: Vec<DaemonSetCondition>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Информация о контейнере
#[derive(Debug, Serialize)]
pub struct ContainerInfo {
    pub name: String,
    pub image: Option<String>,
}

/// Условие DaemonSet
#[derive(Debug, Serialize)]
pub struct DaemonSetCondition {
    #[serde(rename = "type")]
    pub condition_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
}

/// Query параметры для списка DaemonSets
#[derive(Debug, Deserialize)]
pub struct DaemonSetListQuery {
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
    pub limit: Option<u32>,
}

/// Ответ на операцию DaemonSet
#[derive(Debug, Serialize)]
pub struct DaemonSetOperationResponse {
    pub message: String,
    pub name: String,
    pub namespace: String,
}

// ============================================================================
// API Handlers
// ============================================================================

/// Получить список DaemonSets
pub async fn list_daemonsets(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DaemonSetListQuery>,
) -> Result<Json<Vec<DaemonSetSummary>>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let namespace = query.namespace.as_deref().unwrap_or("default");
    let api: Api<DaemonSet> = Api::namespaced(client, namespace);

    let mut lp = ListParams::default();
    if let Some(selector) = query.label_selector {
        lp.label_selector = Some(selector);
    }
    if let Some(limit) = query.limit {
        lp.limit = Some(limit);
    }

    let ds_list = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to list daemonsets: {}", e)))?;

    let daemonsets = ds_list.items.iter().map(daemonset_summary).collect();

    Ok(Json(daemonsets))
}

/// Получить DaemonSet по имени
pub async fn get_daemonset(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<DaemonSetDetail>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<DaemonSet> = Api::namespaced(client, &namespace);

    let ds = api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("DaemonSet {} not found: {}", name, e)))?;

    Ok(Json(daemonset_detail(&ds)))
}

/// Удалить DaemonSet
pub async fn delete_daemonset(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<DaemonSetOperationResponse>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<DaemonSet> = Api::namespaced(client, &namespace);

    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to delete daemonset: {}", e)))?;

    Ok(Json(DaemonSetOperationResponse {
        message: format!("DaemonSet {} deleted", name),
        name,
        namespace,
    }))
}

/// Получить pod'ы, управляемые DaemonSet
pub async fn list_daemonset_pods(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<Vec<serde_json::Value>>> {
    use k8s_openapi::api::core::v1::Pod;

    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let ds_api: Api<DaemonSet> = Api::namespaced(client.clone(), &namespace);
    let pod_api: Api<Pod> = Api::namespaced(client, &namespace);

    let ds = ds_api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("DaemonSet {} not found: {}", name, e)))?;

    let selector = ds
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

fn daemonset_summary(ds: &DaemonSet) -> DaemonSetSummary {
    let name = ds.metadata.name.clone().unwrap_or_default();
    let namespace = ds
        .metadata
        .namespace
        .clone()
        .unwrap_or("default".to_string());

    let status = ds.status.as_ref();

    let desired_number_scheduled = status.map(|s| s.desired_number_scheduled).unwrap_or(0);
    let current_number_scheduled = status.map(|s| s.current_number_scheduled).unwrap_or(0);
    let number_ready = status.map(|s| s.number_ready).unwrap_or(0);
    let number_available = status.and_then(|s| s.number_available).unwrap_or(0);
    let number_misscheduled = status.map(|s| s.number_misscheduled).unwrap_or(0);

    let age = ds
        .metadata
        .creation_timestamp
        .as_ref()
        .map(|ts| format_age(&ts.0))
        .unwrap_or_else(|| "unknown".to_string());

    DaemonSetSummary {
        name,
        namespace,
        desired_number_scheduled,
        current_number_scheduled,
        number_ready,
        number_available,
        number_misscheduled,
        age,
    }
}

fn daemonset_detail(ds: &DaemonSet) -> DaemonSetDetail {
    let status = ds.status.as_ref();
    let spec = ds.spec.as_ref();

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

    let conditions = status
        .and_then(|s| s.conditions.clone())
        .map(|conds| {
            conds.iter()
                .map(|c| DaemonSetCondition {
                    condition_type: c.type_.clone(),
                    status: c.status.clone(),
                    reason: c.reason.clone(),
                    message: c.message.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    DaemonSetDetail {
        name: ds.metadata.name.clone().unwrap_or_default(),
        namespace: ds
            .metadata
            .namespace
            .clone()
            .unwrap_or("default".to_string()),
        desired_number_scheduled: status.map(|s| s.desired_number_scheduled).unwrap_or(0),
        current_number_scheduled: status.map(|s| s.current_number_scheduled).unwrap_or(0),
        number_ready: status.map(|s| s.number_ready).unwrap_or(0),
        number_available: status.and_then(|s| s.number_available).unwrap_or(0),
        number_misscheduled: status.map(|s| s.number_misscheduled).unwrap_or(0),
        selector,
        template_labels,
        containers,
        conditions,
        created_at: ds.metadata.creation_timestamp.as_ref().map(|t| t.0),
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
