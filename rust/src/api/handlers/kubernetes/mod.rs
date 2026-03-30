//! Kubernetes API Handlers
//!
//! Маршруты: /api/kubernetes/...
//!
//! Фаза 1: clusters list, cluster info, namespaces
//! Фаза 2: pods list/get/delete/logs

pub mod cluster;
pub mod pods;
pub mod deployments;
pub mod workloads;
pub mod networking;
pub mod config;

pub use cluster::{list_clusters, cluster_info, list_namespaces};
pub use pods::{list_pods, get_pod, delete_pod, pod_logs};
pub use deployments::{list_deployments, get_deployment, scale_deployment, restart_deployment};
pub use workloads::{
    list_daemonsets, get_daemonset, restart_daemonset,
    list_statefulsets, get_statefulset, scale_statefulset,
    list_replicasets,
    list_events,
};
pub use networking::{
    list_services, get_service, delete_service,
    list_ingresses, get_ingress, delete_ingress,
};
pub use config::{
    list_configmaps, get_configmap, delete_configmap, update_configmap,
    list_secrets, get_secret, delete_secret,
};
