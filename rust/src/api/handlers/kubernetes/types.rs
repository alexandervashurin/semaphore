//! Общие типы для Kubernetes модуля

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Метаданные ресурса Kubernetes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubeResourceMeta {
    pub name: String,
    pub namespace: Option<String>,
    pub uid: String,
    pub resource_version: Option<String>,
    pub created_at: DateTime<Utc>,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
}

/// Статус ресурса Kubernetes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum KubeResourceStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Unknown,
    Terminating,
    Active,
    Bound,
    Released,
}

impl std::fmt::Display for KubeResourceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KubeResourceStatus::Pending => write!(f, "Pending"),
            KubeResourceStatus::Running => write!(f, "Running"),
            KubeResourceStatus::Succeeded => write!(f, "Succeeded"),
            KubeResourceStatus::Failed => write!(f, "Failed"),
            KubeResourceStatus::Unknown => write!(f, "Unknown"),
            KubeResourceStatus::Terminating => write!(f, "Terminating"),
            KubeResourceStatus::Active => write!(f, "Active"),
            KubeResourceStatus::Bound => write!(f, "Bound"),
            KubeResourceStatus::Released => write!(f, "Released"),
        }
    }
}

/// Сводка по Namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceSummary {
    pub name: String,
    pub uid: String,
    pub status: String,
    pub created_at: String,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub pods_count: Option<i32>,
    pub services_count: Option<i32>,
    pub deployments_count: Option<i32>,
}

/// Сводка по узлу кластера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSummary {
    pub name: String,
    pub status: String,
    pub roles: Vec<String>,
    pub version: String,
    pub internal_ip: String,
    pub external_ip: Option<String>,
    pub os_image: String,
    pub kernel_version: String,
    pub container_runtime: String,
    pub cpu_capacity: String,
    pub memory_capacity: String,
    pub pods_capacity: i32,
    pub cpu_allocatable: String,
    pub memory_allocatable: String,
    pub pods_allocatable: i32,
}

/// Информация о кластере
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub kubernetes_version: String,
    pub platform: String,
    pub git_version: String,
    pub git_commit: String,
    pub build_date: String,
    pub go_version: String,
    pub compiler: String,
    pub platform_os: String,
    pub architecture: String,
}

/// Сводка по кластеру
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterSummary {
    pub kubernetes_version: String,
    pub nodes_count: i32,
    pub nodes_ready: i32,
    pub namespaces_count: i32,
    pub pods_total: i32,
    pub pods_running: i32,
    pub pods_pending: i32,
    pub pods_failed: i32,
    pub cpu_capacity: String,
    pub memory_capacity: String,
    pub cpu_allocatable: String,
    pub memory_allocatable: String,
}

/// Статус подключения Kubernetes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesHealth {
    pub connected: bool,
    pub cluster_name: Option<String>,
    pub kubernetes_version: Option<String>,
    pub nodes_count: Option<i32>,
    pub error: Option<String>,
}

/// Query параметры для list операций
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
    pub field_selector: Option<String>,
    pub limit: Option<i32>,
    #[serde(default)]
    pub continue_token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kube_resource_status_display() {
        assert_eq!(KubeResourceStatus::Pending.to_string(), "Pending");
        assert_eq!(KubeResourceStatus::Running.to_string(), "Running");
        assert_eq!(KubeResourceStatus::Succeeded.to_string(), "Succeeded");
        assert_eq!(KubeResourceStatus::Failed.to_string(), "Failed");
        assert_eq!(KubeResourceStatus::Unknown.to_string(), "Unknown");
        assert_eq!(KubeResourceStatus::Terminating.to_string(), "Terminating");
        assert_eq!(KubeResourceStatus::Active.to_string(), "Active");
        assert_eq!(KubeResourceStatus::Bound.to_string(), "Bound");
        assert_eq!(KubeResourceStatus::Released.to_string(), "Released");
    }

    #[test]
    fn test_kube_resource_meta() {
        let meta = KubeResourceMeta {
            name: "test-pod".to_string(),
            namespace: Some("default".to_string()),
            uid: "uid-123".to_string(),
            resource_version: Some("12345".to_string()),
            created_at: Utc::now(),
            labels: HashMap::from([("app".to_string(), "test".to_string())]),
            annotations: HashMap::new(),
        };
        assert_eq!(meta.name, "test-pod");
        assert_eq!(meta.namespace, Some("default".to_string()));
    }

    #[test]
    fn test_namespace_summary() {
        let ns = NamespaceSummary {
            name: "default".to_string(),
            uid: "uid-1".to_string(),
            status: "Active".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            labels: HashMap::new(),
            annotations: HashMap::new(),
            pods_count: Some(10),
            services_count: Some(5),
            deployments_count: Some(3),
        };
        assert_eq!(ns.name, "default");
        assert_eq!(ns.pods_count, Some(10));
    }

    #[test]
    fn test_cluster_summary() {
        let summary = ClusterSummary {
            kubernetes_version: "v1.28.0".to_string(),
            nodes_count: 3,
            nodes_ready: 3,
            namespaces_count: 5,
            pods_total: 50,
            pods_running: 45,
            pods_pending: 3,
            pods_failed: 2,
            cpu_capacity: "12".to_string(),
            memory_capacity: "32Gi".to_string(),
            cpu_allocatable: "11".to_string(),
            memory_allocatable: "30Gi".to_string(),
        };
        assert_eq!(summary.nodes_count, 3);
        assert_eq!(summary.pods_running, 45);
    }

    #[test]
    fn test_kubernetes_health() {
        let health = KubernetesHealth {
            connected: true,
            cluster_name: Some("prod-cluster".to_string()),
            kubernetes_version: Some("v1.28.0".to_string()),
            nodes_count: Some(5),
            error: None,
        };
        assert!(health.connected);
        assert_eq!(health.nodes_count, Some(5));
    }

    #[test]
    fn test_list_query() {
        let query = ListQuery {
            namespace: Some("default".to_string()),
            label_selector: Some("app=web".to_string()),
            field_selector: None,
            limit: Some(100),
            continue_token: None,
        };
        assert_eq!(query.namespace, Some("default".to_string()));
        assert_eq!(query.limit, Some(100));
    }
}
