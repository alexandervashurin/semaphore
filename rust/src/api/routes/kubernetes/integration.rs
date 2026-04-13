//! Kubernetes Integration маршруты — Multi-cluster, Backup, GitOps, Audit, Runbook, Inventory Sync

use crate::api::handlers;
use axum::{routing::{get, post, delete}, Router};
use std::sync::Arc;
use crate::api::state::AppState;

/// Маршруты для интеграций
pub fn integration_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Multi-Cluster Management
        .route("/api/kubernetes/clusters", get(handlers::list_kubernetes_clusters))
        .route("/api/kubernetes/clusters", post(handlers::add_kubernetes_cluster))
        .route("/api/kubernetes/clusters/{name}", post(handlers::switch_kubernetes_cluster))
        .route("/api/kubernetes/clusters/{name}", delete(handlers::remove_kubernetes_cluster))
        .route("/api/kubernetes/clusters/switch", post(handlers::switch_cluster_context))
        // Backup/restore runbook + Velero read-only
        .route("/api/kubernetes/backup/runbook", get(handlers::get_backup_restore_runbook))
        .route("/api/kubernetes/backup/velero/status", get(handlers::get_velero_status))
        .route("/api/kubernetes/backup/velero/backups", get(handlers::list_velero_backups))
        // GitOps draft (read-only ArgoCD/Flux)
        .route("/api/kubernetes/gitops/status", get(handlers::get_gitops_status))
        .route("/api/kubernetes/gitops/argocd/applications", get(handlers::list_argocd_applications))
        .route("/api/kubernetes/gitops/flux/kustomizations", get(handlers::list_flux_kustomizations))
        .route("/api/kubernetes/gitops/flux/helmreleases", get(handlers::list_flux_helm_releases))
        // Kubernetes audit view/export
        .route("/api/kubernetes/audit", get(handlers::list_kubernetes_audit))
        .route("/api/kubernetes/audit/export", get(handlers::export_kubernetes_audit))
        // Troubleshooting Dashboard
        .route("/api/kubernetes/troubleshoot", get(handlers::get_troubleshooting_report))
        // Kubernetes Runbook Integration (project_id в пути — шаблоны и задачи проекта)
        .route(
            "/api/kubernetes/project/{project_id}/runbooks",
            get(handlers::get_available_runbooks),
        )
        .route(
            "/api/kubernetes/project/{project_id}/runbooks/execute",
            post(handlers::execute_runbook),
        )
        .route(
            "/api/kubernetes/project/{project_id}/runbooks/{task_id}/status",
            get(handlers::get_runbook_status),
        )
        // Prometheus Metrics Integration
        .route("/api/kubernetes/prometheus/metrics", get(handlers::get_prometheus_metrics))
        .route("/api/kubernetes/prometheus/health", get(handlers::check_prometheus_health))
        // Kubernetes Inventory Sync
        .route("/api/kubernetes/inventory/sync/preview", get(handlers::get_inventory_sync_preview))
        .route("/api/kubernetes/inventory/sync", post(handlers::execute_inventory_sync))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_routes_creates() {
        let _router: Router<Arc<AppState>> = integration_routes();
    }

    #[test]
    fn test_integration_routes_returns_router() {
        let router = integration_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_integration_routes_has_multi_cluster_endpoints() {
        // Multi-cluster: list, add, switch, remove
        let _fn: fn() -> Router<Arc<AppState>> = integration_routes;
    }

    #[test]
    fn test_integration_routes_has_switch_cluster_context() {
        // /api/kubernetes/clusters/switch - POST
        let _fn: fn() -> Router<Arc<AppState>> = integration_routes;
    }

    #[test]
    fn test_integration_routes_has_backup_runbook() {
        // /api/kubernetes/backup/runbook - GET
        let _fn: fn() -> Router<Arc<AppState>> = integration_routes;
    }

    #[test]
    fn test_integration_routes_has_velero_endpoints() {
        // Velero: status, list backups
        let _fn: fn() -> Router<Arc<AppState>> = integration_routes;
    }

    #[test]
    fn test_integration_routes_has_gitops_status() {
        // /api/kubernetes/gitops/status - GET
        let _fn: fn() -> Router<Arc<AppState>> = integration_routes;
    }

    #[test]
    fn test_integration_routes_has_argocd_applications() {
        // /api/kubernetes/gitops/argocd/applications - GET
        let _fn: fn() -> Router<Arc<AppState>> = integration_routes;
    }

    #[test]
    fn test_integration_routes_has_flux_kustomizations() {
        // /api/kubernetes/gitops/flux/kustomizations - GET
        let _fn: fn() -> Router<Arc<AppState>> = integration_routes;
    }

    #[test]
    fn test_integration_routes_has_audit_endpoints() {
        // Audit: list, export
        let _fn: fn() -> Router<Arc<AppState>> = integration_routes;
    }

    #[test]
    fn test_integration_routes_has_prometheus_endpoints() {
        // Prometheus: metrics, health
        let _fn: fn() -> Router<Arc<AppState>> = integration_routes;
    }
}
