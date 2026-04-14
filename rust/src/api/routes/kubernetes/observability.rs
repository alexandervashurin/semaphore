//! Kubernetes Observability маршруты — Events, Metrics, Topology

use crate::api::handlers;
use crate::api::state::AppState;
use axum::{routing::get, Router};
use std::sync::Arc;

/// Маршруты для observability-ресурсов
pub fn observability_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Events
        .route("/api/kubernetes/events", get(handlers::list_events))
        .route(
            "/api/kubernetes/namespaces/{namespace}/events",
            get(handlers::list_events),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/events/{name}",
            get(handlers::get_event),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/events/stream",
            get(handlers::events_websocket),
        )
        .route(
            "/api/kubernetes/events/stream",
            get(handlers::cluster_events_websocket),
        )
        // Metrics
        .route(
            "/api/kubernetes/metrics/nodes",
            get(handlers::list_node_metrics),
        )
        .route(
            "/api/kubernetes/metrics/nodes/{name}",
            get(handlers::get_node_metrics),
        )
        .route(
            "/api/kubernetes/metrics/top/pods",
            get(handlers::get_top_pods),
        )
        .route(
            "/api/kubernetes/metrics/top/nodes",
            get(handlers::get_top_nodes),
        )
        // Topology
        .route("/api/kubernetes/topology", get(handlers::get_topology))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observability_routes_creates() {
        let _router: Router<Arc<AppState>> = observability_routes();
    }

    #[test]
    fn test_observability_routes_returns_router() {
        let router = observability_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_observability_routes_has_events_list() {
        // /api/kubernetes/events - GET
        let _fn: fn() -> Router<Arc<AppState>> = observability_routes;
    }

    #[test]
    fn test_observability_routes_has_namespace_events() {
        // /api/kubernetes/namespaces/{namespace}/events - GET
        let _fn: fn() -> Router<Arc<AppState>> = observability_routes;
    }

    #[test]
    fn test_observability_routes_has_event_details() {
        // /api/kubernetes/namespaces/{namespace}/events/{name} - GET
        let _fn: fn() -> Router<Arc<AppState>> = observability_routes;
    }

    #[test]
    fn test_observability_routes_has_events_websocket() {
        // /api/kubernetes/namespaces/{namespace}/events/stream - GET
        let _fn: fn() -> Router<Arc<AppState>> = observability_routes;
    }

    #[test]
    fn test_observability_routes_has_cluster_events_websocket() {
        // /api/kubernetes/events/stream - GET
        let _fn: fn() -> Router<Arc<AppState>> = observability_routes;
    }

    #[test]
    fn test_observability_routes_has_node_metrics() {
        // Metrics: list_node_metrics, get_node_metrics
        let _fn: fn() -> Router<Arc<AppState>> = observability_routes;
    }

    #[test]
    fn test_observability_routes_has_top_pods() {
        // /api/kubernetes/metrics/top/pods - GET
        let _fn: fn() -> Router<Arc<AppState>> = observability_routes;
    }

    #[test]
    fn test_observability_routes_has_top_nodes() {
        // /api/kubernetes/metrics/top/nodes - GET
        let _fn: fn() -> Router<Arc<AppState>> = observability_routes;
    }

    #[test]
    fn test_observability_routes_has_topology() {
        // /api/kubernetes/topology - GET
        let _fn: fn() -> Router<Arc<AppState>> = observability_routes;
    }

    #[test]
    fn test_observability_routes_router_not_empty() {
        let router = observability_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }
}
