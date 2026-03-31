//! Cluster API handlers
//!
//! Handlers для управления кластером Kubernetes

use crate::api::handlers::kubernetes::client::KubernetesClusterService;
use crate::api::state::AppState;
use crate::error::{Error, Result};
use axum::{extract::State, Json};
use k8s_openapi::api::core::v1::Node;
use kube::api::{Api, ListParams};
use std::sync::Arc;

use super::types::{ClusterInfo, ClusterSummary, NodeSummary};

/// Получить информацию о кластере
/// GET /api/kubernetes/cluster/info
pub async fn get_cluster_info(State(state): State<Arc<AppState>>) -> Result<Json<ClusterInfo>> {
    let client = state.kubernetes_client()?;
    let service = KubernetesClusterService::new(client);

    let info = service.get_cluster_info().await?;

    let version = info
        .get("kubernetes_version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let platform = info
        .get("platform")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    Ok(Json(ClusterInfo {
        kubernetes_version: version.to_string(),
        platform: platform.to_string(),
        git_version: info
            .get("git_version")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        git_commit: info
            .get("git_commit")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        build_date: info
            .get("build_date")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        go_version: info
            .get("go_version")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        compiler: info
            .get("compiler")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        platform_os: platform.split('/').next().unwrap_or("unknown").to_string(),
        architecture: platform.split('/').nth(1).unwrap_or("unknown").to_string(),
    }))
}

/// Получить список узлов
/// GET /api/kubernetes/cluster/nodes
pub async fn get_cluster_nodes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<NodeSummary>>> {
    let client = state.kubernetes_client()?;
    let service = KubernetesClusterService::new(client);

    let nodes = service.list_nodes().await?;

    let summaries = nodes
        .iter()
        .map(|node| NodeSummary {
            name: node
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            status: node
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            roles: node
                .get("roles")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            version: node
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            internal_ip: node
                .get("internal_ip")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            external_ip: node
                .get("external_ip")
                .and_then(|v| v.as_str())
                .map(String::from),
            os_image: node
                .get("os_image")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            kernel_version: node
                .get("kernel_version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            container_runtime: node
                .get("container_runtime")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            cpu_capacity: "0".to_string(),
            memory_capacity: "0".to_string(),
            pods_capacity: 0,
            cpu_allocatable: "0".to_string(),
            memory_allocatable: "0".to_string(),
            pods_allocatable: 0,
        })
        .collect();

    Ok(Json(summaries))
}

/// Получить сводку по кластеру
/// GET /api/kubernetes/cluster/summary
pub async fn get_cluster_summary(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ClusterSummary>> {
    let client = state.kubernetes_client()?;

    // Считаем количество узлов
    let nodes_api: Api<Node> = client.api_all();
    let nodes = nodes_api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    let nodes_count = nodes.items.len() as i32;
    let nodes_ready = nodes
        .items
        .iter()
        .filter(|n| {
            n.status
                .as_ref()
                .and_then(|s| s.conditions.as_ref())
                .map(|conds| {
                    conds
                        .iter()
                        .any(|c| c.type_ == "Ready" && c.status == "True")
                })
                .unwrap_or(false)
        })
        .count() as i32;

    // Считаем namespaces
    use k8s_openapi::api::core::v1::Namespace;
    let ns_api: Api<Namespace> = client.api_all();
    let namespaces = ns_api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    let namespaces_count = namespaces.items.len() as i32;

    // Считаем pod'ы
    use k8s_openapi::api::core::v1::Pod;
    let pods_api: Api<Pod> = client.api_all();
    let pods = pods_api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    let pods_total = pods.items.len() as i32;
    let pods_running = pods
        .items
        .iter()
        .filter(|p| {
            p.status
                .as_ref()
                .map(|s| s.phase.as_deref() == Some("Running"))
                .unwrap_or(false)
        })
        .count() as i32;
    let pods_pending = pods
        .items
        .iter()
        .filter(|p| {
            p.status
                .as_ref()
                .map(|s| s.phase.as_deref() == Some("Pending"))
                .unwrap_or(false)
        })
        .count() as i32;
    let pods_failed = pods
        .items
        .iter()
        .filter(|p| {
            p.status
                .as_ref()
                .map(|s| s.phase.as_deref() == Some("Failed"))
                .unwrap_or(false)
        })
        .count() as i32;

    Ok(Json(ClusterSummary {
        kubernetes_version: "v1.30.0".to_string(), // TODO: получить из API
        nodes_count,
        nodes_ready,
        namespaces_count,
        pods_total,
        pods_running,
        pods_pending,
        pods_failed,
        cpu_capacity: "0".to_string(),
        memory_capacity: "0".to_string(),
        cpu_allocatable: "0".to_string(),
        memory_allocatable: "0".to_string(),
    }))
}
