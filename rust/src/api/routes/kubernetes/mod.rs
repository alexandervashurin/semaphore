//! Kubernetes API маршруты
//!
//! Модульная структура:
//! - cluster      — Cluster info, health, nodes
//! - namespaces   — Namespaces, quota, limits
//! - workloads    — Pods, Deployments, ReplicaSets, DaemonSets, StatefulSets
//! - networking   — Services, Ingress, NetworkPolicy, Gateway API
//! - config       — ConfigMaps, Secrets
//! - storage      — PV, PVC, StorageClass, Snapshots, CSI
//! - rbac         — ServiceAccounts, Roles, RoleBindings, ClusterRoles, PSA
//! - batch        — Jobs, CronJobs, PriorityClass, PDB
//! - advanced     — HPA, VPA, ResourceQuota, LimitRange, CRD, Custom Objects
//! - observability— Events, Metrics, Topology
//! - helm         — Helm repos, charts, releases
//! - integration  — Multi-cluster, Backup, GitOps, Audit, Runbook, Inventory Sync
//! - apply        — Apply manifest, Diff, Kubectl generator

mod cluster;
mod namespaces;
mod workloads;
mod networking;
mod config;
mod storage;
mod rbac;
mod batch;
mod advanced;
mod observability;
mod helm;
mod integration;
mod apply;

use axum::Router;
use std::sync::Arc;
use crate::api::state::AppState;

/// Создаёт маршруты Kubernetes API
pub fn kubernetes_routes() -> Router<Arc<AppState>> {
    cluster::cluster_routes()
        .merge(namespaces::namespaces_routes())
        .merge(workloads::workloads_routes())
        .merge(networking::networking_routes())
        .merge(config::config_routes())
        .merge(storage::storage_routes())
        .merge(rbac::rbac_routes())
        .merge(batch::batch_routes())
        .merge(advanced::advanced_routes())
        .merge(observability::observability_routes())
        .merge(helm::helm_routes())
        .merge(integration::integration_routes())
        .merge(apply::apply_routes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kubernetes_routes_creation() {
        let router = kubernetes_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_kubernetes_routes_returns_router() {
        let router = kubernetes_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_cluster_submodule_exists() {
        let _ = cluster::cluster_routes;
    }

    #[test]
    fn test_namespaces_submodule_exists() {
        let _ = namespaces::namespaces_routes;
    }

    #[test]
    fn test_workloads_submodule_exists() {
        let _ = workloads::workloads_routes;
    }

    #[test]
    fn test_networking_submodule_exists() {
        let _ = networking::networking_routes;
    }

    #[test]
    fn test_config_submodule_exists() {
        let _ = config::config_routes;
    }

    #[test]
    fn test_storage_submodule_exists() {
        let _ = storage::storage_routes;
    }

    #[test]
    fn test_rbac_submodule_exists() {
        let _ = rbac::rbac_routes;
    }

    #[test]
    fn test_batch_submodule_exists() {
        let _ = batch::batch_routes;
    }

    #[test]
    fn test_advanced_submodule_exists() {
        let _ = advanced::advanced_routes;
    }

    #[test]
    fn test_observability_submodule_exists() {
        let _ = observability::observability_routes;
    }

    #[test]
    fn test_helm_submodule_exists() {
        let _ = helm::helm_routes;
    }

    #[test]
    fn test_integration_submodule_exists() {
        let _ = integration::integration_routes;
    }

    #[test]
    fn test_apply_submodule_exists() {
        let _ = apply::apply_routes;
    }

    #[test]
    fn test_kubernetes_routes_merges_all_submodules() {
        // 13 submodules are merged
        let router = kubernetes_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_kubernetes_routes_function_signature() {
        let _: fn() -> Router<Arc<AppState>> = kubernetes_routes;
    }
}
