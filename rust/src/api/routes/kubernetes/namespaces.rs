//! Kubernetes Namespaces маршруты

use crate::api::handlers;
use axum::{routing::{get, post, put, delete}, Router};
use std::sync::Arc;
use crate::api::state::AppState;

/// Маршруты для управления namespaces
pub fn namespaces_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/kubernetes/namespaces", get(handlers::list_namespaces))
        .route("/api/kubernetes/namespaces/{name}", get(handlers::get_namespace))
        .route("/api/kubernetes/namespaces", post(handlers::create_namespace))
        .route("/api/kubernetes/namespaces/{name}", put(handlers::update_namespace))
        .route("/api/kubernetes/namespaces/{name}", delete(handlers::delete_namespace))
        .route("/api/kubernetes/namespaces/{name}/quota", get(handlers::get_namespace_quota))
        .route("/api/kubernetes/namespaces/{name}/limits", get(handlers::get_namespace_limits))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespaces_routes_creates() {
        let _router: Router<Arc<AppState>> = namespaces_routes();
    }

    #[test]
    fn test_namespaces_routes_returns_router() {
        let router = namespaces_routes();
        let _: Router<Arc<AppState>> = router;
    }
}
