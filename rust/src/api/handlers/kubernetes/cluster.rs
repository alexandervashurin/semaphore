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

    // Получить реальную версию Kubernetes из API-сервера
    let kubernetes_version = client
        .raw()
        .apiserver_version()
        .await
        .map(|v| v.git_version.clone())
        .unwrap_or_else(|_| "unknown".to_string());

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
        kubernetes_version,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_info_serialization() {
        let info = ClusterInfo {
            kubernetes_version: "v1.28.0".to_string(),
            platform: "linux/amd64".to_string(),
            git_version: "v1.28.0+k3s1".to_string(),
            git_commit: "abc123".to_string(),
            build_date: "2024-01-01T00:00:00Z".to_string(),
            go_version: "go1.21.0".to_string(),
            compiler: "gc".to_string(),
            platform_os: "linux".to_string(),
            architecture: "amd64".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["kubernetes_version"], "v1.28.0");
        assert_eq!(parsed["platform"], "linux/amd64");
        assert_eq!(parsed["go_version"], "go1.21.0");
    }

    #[test]
    fn test_node_summary_serialization() {
        let node = NodeSummary {
            name: "node-1".to_string(),
            status: "Ready".to_string(),
            roles: vec!["control-plane".to_string(), "master".to_string()],
            version: "v1.28.0".to_string(),
            internal_ip: "10.0.0.1".to_string(),
            external_ip: Some("203.0.113.1".to_string()),
            os_image: "Ubuntu 22.04".to_string(),
            kernel_version: "5.15.0".to_string(),
            container_runtime: "containerd://1.7.0".to_string(),
            cpu_capacity: "8".to_string(),
            memory_capacity: "16Gi".to_string(),
            pods_capacity: 110,
            cpu_allocatable: "7.5".to_string(),
            memory_allocatable: "15Gi".to_string(),
            pods_allocatable: 100,
        };
        let json = serde_json::to_string(&node).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["name"], "node-1");
        assert_eq!(parsed["status"], "Ready");
        assert_eq!(parsed["roles"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["cpu_capacity"], "8");
        assert_eq!(parsed["pods_capacity"], 110);
    }

    #[test]
    fn test_node_summary_no_external_ip() {
        let node = NodeSummary {
            name: "worker-1".to_string(),
            status: "Ready".to_string(),
            roles: vec!["worker".to_string()],
            version: "v1.27.0".to_string(),
            internal_ip: "10.0.0.2".to_string(),
            external_ip: None,
            os_image: "Debian 12".to_string(),
            kernel_version: "6.1.0".to_string(),
            container_runtime: "containerd://1.6.0".to_string(),
            cpu_capacity: "4".to_string(),
            memory_capacity: "8Gi".to_string(),
            pods_capacity: 110,
            cpu_allocatable: "3.5".to_string(),
            memory_allocatable: "7Gi".to_string(),
            pods_allocatable: 100,
        };
        let json = serde_json::to_string(&node).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["external_ip"].is_null());
    }

    #[test]
    fn test_cluster_summary_serialization() {
        let summary = ClusterSummary {
            kubernetes_version: "v1.29.0".to_string(),
            nodes_count: 5,
            nodes_ready: 5,
            namespaces_count: 12,
            pods_total: 80,
            pods_running: 70,
            pods_pending: 5,
            pods_failed: 5,
            cpu_capacity: "40".to_string(),
            memory_capacity: "128Gi".to_string(),
            cpu_allocatable: "38".to_string(),
            memory_allocatable: "120Gi".to_string(),
        };
        let json = serde_json::to_string(&summary).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["nodes_count"], 5);
        assert_eq!(parsed["nodes_ready"], 5);
        assert_eq!(parsed["pods_total"], 80);
        assert_eq!(parsed["pods_failed"], 5);
    }

    #[test]
    fn test_cluster_info_display_fields_correct() {
        let info = ClusterInfo {
            kubernetes_version: "v1.28.0".to_string(),
            platform: "linux/arm64".to_string(),
            git_version: "v1.28.0".to_string(),
            git_commit: "def456".to_string(),
            build_date: "2024-02-01".to_string(),
            go_version: "go1.21.1".to_string(),
            compiler: "gc".to_string(),
            platform_os: "linux".to_string(),
            architecture: "arm64".to_string(),
        };
        assert_eq!(info.platform_os, "linux");
        assert_eq!(info.architecture, "arm64");
    }

    #[test]
    fn test_cluster_info_empty_roles_node() {
        let node = NodeSummary {
            name: "worker-2".to_string(),
            status: "NotReady".to_string(),
            roles: vec![],
            version: "v1.26.0".to_string(),
            internal_ip: "10.0.0.3".to_string(),
            external_ip: None,
            os_image: "Alpine".to_string(),
            kernel_version: "5.10.0".to_string(),
            container_runtime: "cri-o://1.25".to_string(),
            cpu_capacity: "2".to_string(),
            memory_capacity: "4Gi".to_string(),
            pods_capacity: 50,
            cpu_allocatable: "1.8".to_string(),
            memory_allocatable: "3.5Gi".to_string(),
            pods_allocatable: 45,
        };
        let json = serde_json::to_string(&node).unwrap();
        let parsed: NodeSummary = serde_json::from_str(&json).unwrap();
        assert!(parsed.roles.is_empty());
        assert_eq!(parsed.status, "NotReady");
    }

    #[test]
    fn test_cluster_summary_zero_nodes() {
        let summary = ClusterSummary {
            kubernetes_version: "v1.28.0".to_string(),
            nodes_count: 0,
            nodes_ready: 0,
            namespaces_count: 1,
            pods_total: 0,
            pods_running: 0,
            pods_pending: 0,
            pods_failed: 0,
            cpu_capacity: "0".to_string(),
            memory_capacity: "0".to_string(),
            cpu_allocatable: "0".to_string(),
            memory_allocatable: "0".to_string(),
        };
        assert_eq!(summary.nodes_count, 0);
        assert_eq!(summary.pods_total, 0);
    }

    #[test]
    fn test_node_summary_multiple_roles() {
        let node = NodeSummary {
            name: "multi-role".to_string(),
            status: "Ready".to_string(),
            roles: vec![
                "master".to_string(),
                "worker".to_string(),
                "etcd".to_string(),
            ],
            version: "v1.28.0".to_string(),
            internal_ip: "10.0.0.10".to_string(),
            external_ip: None,
            os_image: "Ubuntu".to_string(),
            kernel_version: "5.15.0".to_string(),
            container_runtime: "containerd".to_string(),
            cpu_capacity: "16".to_string(),
            memory_capacity: "64Gi".to_string(),
            pods_capacity: 200,
            cpu_allocatable: "15".to_string(),
            memory_allocatable: "60Gi".to_string(),
            pods_allocatable: 180,
        };
        assert_eq!(node.roles.len(), 3);
        assert!(node.roles.contains(&"etcd".to_string()));
    }

    #[test]
    fn test_cluster_info_serialization_roundtrip() {
        let original = ClusterInfo {
            kubernetes_version: "v1.29.1".to_string(),
            platform: "darwin/amd64".to_string(),
            git_version: "v1.29.1".to_string(),
            git_commit: "xyz789".to_string(),
            build_date: "2024-03-01".to_string(),
            go_version: "go1.22.0".to_string(),
            compiler: "gc".to_string(),
            platform_os: "darwin".to_string(),
            architecture: "amd64".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: ClusterInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.kubernetes_version, original.kubernetes_version);
        assert_eq!(parsed.platform, original.platform);
    }

    #[test]
    fn test_cluster_health_zero_nodes_scenario() {
        let summary = ClusterSummary {
            kubernetes_version: "unknown".to_string(),
            nodes_count: 0,
            nodes_ready: 0,
            namespaces_count: 0,
            pods_total: 0,
            pods_running: 0,
            pods_pending: 0,
            pods_failed: 0,
            cpu_capacity: "0".to_string(),
            memory_capacity: "0".to_string(),
            cpu_allocatable: "0".to_string(),
            memory_allocatable: "0".to_string(),
        };
        assert_eq!(summary.kubernetes_version, "unknown");
        assert_eq!(summary.nodes_ready, 0);
    }
}
