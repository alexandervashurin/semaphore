//! Kubernetes API Handlers
//!
//! Маршруты: /api/kubernetes/...
//!
//! Фаза 1: clusters list, cluster info, namespaces
//! Фаза 2: pods list/get/delete/logs

pub mod cluster;
pub mod pods;

pub use cluster::{list_clusters, cluster_info, list_namespaces};
pub use pods::{list_pods, get_pod, delete_pod, pod_logs};
