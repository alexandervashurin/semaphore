//! Kubernetes Pod API handlers
//!
//! Управление Pod: list, get, delete, logs

use axum::{
    extract::{Path, Query, State},
    Json,
};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, DeleteParams, ListParams, LogParams},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

// ============================================================================
// Pod Types
// ============================================================================

/// Краткая информация о Pod
#[derive(Debug, Serialize)]
pub struct PodSummary {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub ip: Option<String>,
    pub node: Option<String>,
    pub restart_count: i32,
    pub age: String,
}

/// Query параметры для списка Pod
#[derive(Debug, Deserialize)]
pub struct PodListQuery {
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
    pub limit: Option<u32>,
}

/// Детальная информация о Pod
#[derive(Debug, Serialize)]
pub struct PodDetail {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub ip: Option<String>,
    pub host_ip: Option<String>,
    pub phase: String,
    pub start_time: Option<String>,
    pub containers: Vec<String>,
    pub labels: std::collections::BTreeMap<String, String>,
}

/// Ответ на запрос логов
#[derive(Debug, Serialize)]
pub struct LogsResponse {
    pub logs: String,
}

/// Ответ на удаление
#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub message: String,
    pub name: String,
    pub namespace: String,
}

// ============================================================================
// API Handlers
// ============================================================================

/// Получить список Pod
pub async fn list_pods(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PodListQuery>,
) -> Result<Json<Vec<PodSummary>>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();
    
    let namespace = query.namespace.as_deref().unwrap_or("default");
    let api: Api<Pod> = Api::namespaced(client, namespace);

    let mut lp = ListParams {
        limit: query.limit,
        ..Default::default()
    };
    
    if let Some(selector) = query.label_selector {
        lp.label_selector = Some(selector);
    }
    
    let pod_list = api.list(&lp).await
        .map_err(|e| Error::Kubernetes(format!("Failed to list pods: {}", e)))?;
    
    let pods = pod_list.items.iter().map(pod_summary).collect();
    
    Ok(Json(pods))
}

/// Получить Pod по имени
pub async fn get_pod(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<PodDetail>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();
    
    let api: Api<Pod> = Api::namespaced(client, &namespace);
    
    let pod = api.get(&name).await
        .map_err(|e| Error::NotFound(format!("Pod {} not found: {}", name, e)))?;
    
    Ok(Json(pod_detail(&pod)))
}

/// Удалить Pod
pub async fn delete_pod(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<DeleteResponse>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();
    
    let api: Api<Pod> = Api::namespaced(client, &namespace);
    
    let dp = DeleteParams::default();
    api.delete(&name, &dp).await
        .map_err(|e| Error::Kubernetes(format!("Failed to delete pod: {}", e)))?;
    
    Ok(Json(DeleteResponse {
        message: format!("Pod {} deleted", name),
        name,
        namespace,
    }))
}

/// Перезапустить Pod (удалить и создать заново)
pub async fn restart_pod(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<DeleteResponse>> {
    // Для restart удаляем Pod — контроллер создаст новый
    delete_pod(State(state), Path((namespace, name))).await
}

/// Получить логи Pod
pub async fn get_pod_logs(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Query(query): Query<LogsQuery>,
) -> Result<Json<LogsResponse>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();
    
    let api: Api<Pod> = Api::namespaced(client, &namespace);

    let lp = LogParams {
        container: query.container.clone(),
        tail_lines: query.tail,
        timestamps: query.timestamps.unwrap_or(false),
        ..Default::default()
    };
    
    let logs = api.logs(&name, &lp).await
        .map_err(|e| Error::Kubernetes(format!("Failed to get logs: {}", e)))?;
    
    Ok(Json(LogsResponse { logs }))
}

// ============================================================================
// Helper Functions
// ============================================================================

fn pod_summary(pod: &Pod) -> PodSummary {
    let name = pod.metadata.name.clone().unwrap_or_default();
    let namespace = pod.metadata.namespace.clone().unwrap_or("default".to_string());
    
    let status = pod.status.as_ref();
    let phase = status
        .and_then(|s| s.phase.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    
    let ip = status.and_then(|s| s.pod_ip.clone());
    let node = pod.spec.as_ref().and_then(|s| s.node_name.clone());
    
    let restart_count = status
        .and_then(|s| s.container_statuses.as_ref())
        .map(|statuses| statuses.iter().map(|cs| cs.restart_count).sum())
        .unwrap_or(0);
    
    let age = pod.metadata
        .creation_timestamp
        .as_ref()
        .map(|ts| format_age(&ts.0))
        .unwrap_or_else(|| "unknown".to_string());
    
    PodSummary {
        name,
        namespace,
        status: phase,
        ip,
        node,
        restart_count,
        age,
    }
}

fn pod_detail(pod: &Pod) -> PodDetail {
    let meta = &pod.metadata;
    let spec = pod.spec.as_ref();
    let status = pod.status.as_ref();
    
    PodDetail {
        name: meta.name.clone().unwrap_or_default(),
        namespace: meta.namespace.clone().unwrap_or("default".to_string()),
        status: status
            .and_then(|s| s.phase.clone())
            .unwrap_or_default(),
        ip: status.and_then(|s| s.pod_ip.clone()),
        host_ip: status.and_then(|s| s.host_ip.clone()),
        phase: status
            .and_then(|s| s.phase.clone())
            .unwrap_or_default(),
        start_time: status
            .and_then(|s| s.start_time.as_ref().map(|t| t.0.to_rfc3339())),
        containers: spec
            .map(|s| s.containers.iter().map(|c| c.name.clone()).collect())
            .unwrap_or_default(),
        labels: meta.labels.clone().unwrap_or_default(),
    }
}

fn format_age(time: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
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

// ============================================================================
// Query Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    pub container: Option<String>,
    pub tail: Option<i64>,
    pub timestamps: Option<bool>,
}
