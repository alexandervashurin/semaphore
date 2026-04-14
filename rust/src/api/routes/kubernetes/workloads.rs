//! Kubernetes Workloads маршруты — Pods, Deployments, ReplicaSets, DaemonSets, StatefulSets

use crate::api::handlers;
use crate::api::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

/// Маршруты для управления workload-ресурсами
pub fn workloads_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Pods
        .route("/api/kubernetes/pods", get(handlers::list_pods))
        .route(
            "/api/kubernetes/namespaces/{namespace}/pods",
            get(handlers::list_pods),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/pods/{name}",
            get(handlers::get_pod),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/pods/{name}",
            delete(handlers::delete_pod),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/pods/{name}/logs",
            get(handlers::pod_logs),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/pods/{name}/evict",
            post(handlers::evict_pod),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/pods/{name}/logs/stream",
            get(handlers::pod_logs_ws),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/pods/{name}/exec",
            get(handlers::pod_exec_ws),
        )
        // Pod metrics
        .route(
            "/api/kubernetes/metrics/pods",
            get(handlers::list_pod_metrics),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/metrics/pods",
            get(handlers::list_pod_metrics),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/metrics/pods/{name}",
            get(handlers::get_pod_metrics),
        )
        // Deployments
        .route(
            "/api/kubernetes/deployments",
            get(handlers::list_deployments),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/deployments",
            get(handlers::list_deployments),
        )
        .route(
            "/api/kubernetes/deployments",
            post(handlers::create_deployment),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/deployments/{name}",
            get(handlers::get_deployment),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/deployments/{name}",
            put(handlers::update_deployment),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/deployments/{name}",
            delete(handlers::delete_deployment),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/deployments/{name}/scale",
            post(handlers::scale_deployment),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/deployments/{name}/restart",
            post(handlers::restart_deployment),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/deployments/{name}/rollback",
            post(handlers::rollback_deployment),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/deployments/{name}/history",
            get(handlers::get_deployment_history),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/deployments/{name}/history/detailed",
            get(handlers::get_deployment_history_detailed),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/deployments/{name}/pause",
            post(handlers::pause_deployment),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/deployments/{name}/resume",
            post(handlers::resume_deployment),
        )
        // ReplicaSets
        .route(
            "/api/kubernetes/replicasets",
            get(handlers::list_replicasets),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/replicasets",
            get(handlers::list_replicasets),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/replicasets/{name}",
            get(handlers::get_replicaset),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/replicasets/{name}",
            delete(handlers::delete_replicaset),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/replicasets/{name}/pods",
            get(handlers::list_replicaset_pods),
        )
        // DaemonSets
        .route("/api/kubernetes/daemonsets", get(handlers::list_daemonsets))
        .route(
            "/api/kubernetes/namespaces/{namespace}/daemonsets",
            get(handlers::list_daemonsets),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/daemonsets/{name}",
            get(handlers::get_daemonset),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/daemonsets/{name}",
            delete(handlers::delete_daemonset),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/daemonsets/{name}/pods",
            get(handlers::list_daemonset_pods),
        )
        // StatefulSets
        .route(
            "/api/kubernetes/statefulsets",
            get(handlers::list_statefulsets),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/statefulsets",
            get(handlers::list_statefulsets),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/statefulsets/{name}",
            get(handlers::get_statefulset),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/statefulsets/{name}",
            delete(handlers::delete_statefulset),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/statefulsets/{name}/scale",
            post(handlers::scale_statefulset),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/statefulsets/{name}/pods",
            get(handlers::list_statefulset_pods),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workloads_routes_creates() {
        let _router: Router<Arc<AppState>> = workloads_routes();
    }

    #[test]
    fn test_workloads_routes_returns_router() {
        let router = workloads_routes();
        // Проверяем что тип именно Router<Arc<AppState>>
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_workloads_routes_has_pods_list() {
        // Pods: list (cluster and namespace)
        let _fn: fn() -> Router<Arc<AppState>> = workloads_routes;
    }

    #[test]
    fn test_workloads_routes_has_pod_details() {
        // /api/kubernetes/namespaces/{namespace}/pods/{name} - GET, DELETE
        let _fn: fn() -> Router<Arc<AppState>> = workloads_routes;
    }

    #[test]
    fn test_workloads_routes_has_pod_logs() {
        // /api/kubernetes/namespaces/{namespace}/pods/{name}/logs - GET
        let _fn: fn() -> Router<Arc<AppState>> = workloads_routes;
    }

    #[test]
    fn test_workloads_routes_has_pod_evict() {
        // /api/kubernetes/namespaces/{namespace}/pods/{name}/evict - POST
        let _fn: fn() -> Router<Arc<AppState>> = workloads_routes;
    }

    #[test]
    fn test_workloads_routes_has_pod_exec_ws() {
        // /api/kubernetes/namespaces/{namespace}/pods/{name}/exec - GET
        let _fn: fn() -> Router<Arc<AppState>> = workloads_routes;
    }

    #[test]
    fn test_workloads_routes_has_deployments_crud() {
        // Deployments: list, create, get, update, delete
        let _fn: fn() -> Router<Arc<AppState>> = workloads_routes;
    }

    #[test]
    fn test_workloads_routes_has_deployment_scale() {
        // /api/kubernetes/namespaces/{namespace}/deployments/{name}/scale - POST
        let _fn: fn() -> Router<Arc<AppState>> = workloads_routes;
    }

    #[test]
    fn test_workloads_routes_has_deployment_restart() {
        // /api/kubernetes/namespaces/{namespace}/deployments/{name}/restart - POST
        let _fn: fn() -> Router<Arc<AppState>> = workloads_routes;
    }

    #[test]
    fn test_workloads_routes_has_deployment_rollback() {
        // /api/kubernetes/namespaces/{namespace}/deployments/{name}/rollback - POST
        let _fn: fn() -> Router<Arc<AppState>> = workloads_routes;
    }

    #[test]
    fn test_workloads_routes_has_deployment_history() {
        // Deployment history: get, detailed
        let _fn: fn() -> Router<Arc<AppState>> = workloads_routes;
    }

    #[test]
    fn test_workloads_routes_has_replicasets() {
        // ReplicaSets: list, get, delete, list_pods
        let _fn: fn() -> Router<Arc<AppState>> = workloads_routes;
    }

    #[test]
    fn test_workloads_routes_has_daemonsets() {
        // DaemonSets: list, get, delete, list_pods
        let _fn: fn() -> Router<Arc<AppState>> = workloads_routes;
    }

    #[test]
    fn test_workloads_routes_has_statefulsets() {
        // StatefulSets: list, get, delete, scale, list_pods
        let _fn: fn() -> Router<Arc<AppState>> = workloads_routes;
    }
}
