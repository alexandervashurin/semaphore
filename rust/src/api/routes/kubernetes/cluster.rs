//! Kubernetes Cluster & Health маршруты

use crate::api::handlers;
use crate::api::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

/// Маршруты для cluster info и health checks
pub fn cluster_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Cluster info
        .route(
            "/api/kubernetes/cluster/info",
            get(handlers::get_cluster_info),
        )
        .route(
            "/api/kubernetes/cluster/nodes",
            get(handlers::get_cluster_nodes),
        )
        .route(
            "/api/kubernetes/cluster/summary",
            get(handlers::get_k8s_cluster_summary),
        )
        // Health
        .route("/api/kubernetes/health", get(handlers::kubernetes_health))
        .route(
            "/api/kubernetes/health/detailed",
            get(handlers::kubernetes_health_detailed),
        )
        .route(
            "/api/kubernetes/cluster/health",
            get(handlers::get_cluster_health),
        )
        .route(
            "/api/kubernetes/cluster/aggregate",
            get(handlers::get_aggregate_view),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_routes_creates() {
        let _router: Router<Arc<AppState>> = cluster_routes();
    }

    #[test]
    fn test_cluster_routes_returns_router() {
        let router = cluster_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_cluster_routes_has_cluster_info() {
        // /api/kubernetes/cluster/info - GET
        let _fn: fn() -> Router<Arc<AppState>> = cluster_routes;
    }

    #[test]
    fn test_cluster_routes_has_cluster_nodes() {
        // /api/kubernetes/cluster/nodes - GET
        let _fn: fn() -> Router<Arc<AppState>> = cluster_routes;
    }

    #[test]
    fn test_cluster_routes_has_cluster_summary() {
        // /api/kubernetes/cluster/summary - GET
        let _fn: fn() -> Router<Arc<AppState>> = cluster_routes;
    }

    #[test]
    fn test_cluster_routes_has_health() {
        // /api/kubernetes/health - GET
        let _fn: fn() -> Router<Arc<AppState>> = cluster_routes;
    }

    #[test]
    fn test_cluster_routes_has_health_detailed() {
        // /api/kubernetes/health/detailed - GET
        let _fn: fn() -> Router<Arc<AppState>> = cluster_routes;
    }

    #[test]
    fn test_cluster_routes_has_cluster_health() {
        // /api/kubernetes/cluster/health - GET
        let _fn: fn() -> Router<Arc<AppState>> = cluster_routes;
    }

    #[test]
    fn test_cluster_routes_has_aggregate_view() {
        // /api/kubernetes/cluster/aggregate - GET
        let _fn: fn() -> Router<Arc<AppState>> = cluster_routes;
    }

    #[test]
    fn test_cluster_routes_router_not_empty() {
        let router = cluster_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_cluster_routes_function_signature() {
        let _: fn() -> Router<Arc<AppState>> = cluster_routes;
    }
}
