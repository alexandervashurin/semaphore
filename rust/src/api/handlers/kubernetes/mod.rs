//! Kubernetes API Handlers
//!
//! Маршруты: /api/kubernetes/...
//!
//! Фаза 1: clusters list, cluster info, namespaces
//! Фаза 2: pods list/get/delete/logs

pub mod cluster;
pub mod pods;
pub mod deployments;

pub use cluster::{list_clusters, cluster_info, list_namespaces};
pub use pods::{list_pods, get_pod, delete_pod, pod_logs};
pub use deployments::{list_deployments, get_deployment, scale_deployment, restart_deployment};
