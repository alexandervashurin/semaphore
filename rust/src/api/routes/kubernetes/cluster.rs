//! Kubernetes Cluster & Health маршруты

use crate::api::handlers;
use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use crate::api::state::AppState;

/// Маршруты для cluster info и health checks
pub fn cluster_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Cluster info
        .route("/api/kubernetes/cluster/info", get(handlers::get_cluster_info))
        .route("/api/kubernetes/cluster/nodes", get(handlers::get_cluster_nodes))
        .route("/api/kubernetes/cluster/summary", get(handlers::get_k8s_cluster_summary))
        // Health
        .route("/api/kubernetes/health", get(handlers::kubernetes_health))
        .route("/api/kubernetes/health/detailed", get(handlers::kubernetes_health_detailed))
        .route("/api/kubernetes/cluster/health", get(handlers::get_cluster_health))
        .route("/api/kubernetes/cluster/aggregate", get(handlers::get_aggregate_view))
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
}
