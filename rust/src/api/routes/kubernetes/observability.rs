//! Kubernetes Observability маршруты — Events, Metrics, Topology

use crate::api::handlers;
use axum::{routing::get, Router};
use std::sync::Arc;
use crate::api::state::AppState;

/// Маршруты для observability-ресурсов
pub fn observability_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Events
        .route("/api/kubernetes/events", get(handlers::list_events))
        .route("/api/kubernetes/namespaces/{namespace}/events", get(handlers::list_events))
        .route("/api/kubernetes/namespaces/{namespace}/events/{name}", get(handlers::get_event))
        .route("/api/kubernetes/namespaces/{namespace}/events/stream", get(handlers::events_websocket))
        .route("/api/kubernetes/events/stream", get(handlers::cluster_events_websocket))
        // Metrics
        .route("/api/kubernetes/metrics/nodes", get(handlers::list_node_metrics))
        .route("/api/kubernetes/metrics/nodes/{name}", get(handlers::get_node_metrics))
        .route("/api/kubernetes/metrics/top/pods", get(handlers::get_top_pods))
        .route("/api/kubernetes/metrics/top/nodes", get(handlers::get_top_nodes))
        // Topology
        .route("/api/kubernetes/topology", get(handlers::get_topology))
}
