//! Kubernetes API Handlers
//!
//! Маршруты: /api/kubernetes/...
//!
//! Фаза 1: clusters list, cluster info, namespaces

pub mod cluster;

pub use cluster::{list_clusters, cluster_info, list_namespaces};
