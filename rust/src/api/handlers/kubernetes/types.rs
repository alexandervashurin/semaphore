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
