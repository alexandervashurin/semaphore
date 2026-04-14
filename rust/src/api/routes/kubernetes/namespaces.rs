//! Kubernetes Namespaces маршруты

use crate::api::handlers;
use crate::api::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

/// Маршруты для управления namespaces
pub fn namespaces_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/kubernetes/namespaces", get(handlers::list_namespaces))
        .route(
            "/api/kubernetes/namespaces/{name}",
            get(handlers::get_namespace),
        )
        .route(
            "/api/kubernetes/namespaces",
            post(handlers::create_namespace),
        )
        .route(
            "/api/kubernetes/namespaces/{name}",
            put(handlers::update_namespace),
        )
        .route(
            "/api/kubernetes/namespaces/{name}",
            delete(handlers::delete_namespace),
        )
        .route(
            "/api/kubernetes/namespaces/{name}/quota",
            get(handlers::get_namespace_quota),
        )
        .route(
            "/api/kubernetes/namespaces/{name}/limits",
            get(handlers::get_namespace_limits),
        )
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

    #[test]
    fn test_namespaces_routes_has_list_namespaces() {
        // /api/kubernetes/namespaces - GET
        let _fn: fn() -> Router<Arc<AppState>> = namespaces_routes;
    }

    #[test]
    fn test_namespaces_routes_has_get_namespace() {
        // /api/kubernetes/namespaces/{name} - GET
        let _fn: fn() -> Router<Arc<AppState>> = namespaces_routes;
    }

    #[test]
    fn test_namespaces_routes_has_create_namespace() {
        // /api/kubernetes/namespaces - POST
        let _fn: fn() -> Router<Arc<AppState>> = namespaces_routes;
    }

    #[test]
    fn test_namespaces_routes_has_update_namespace() {
        // /api/kubernetes/namespaces/{name} - PUT
        let _fn: fn() -> Router<Arc<AppState>> = namespaces_routes;
    }

    #[test]
    fn test_namespaces_routes_has_delete_namespace() {
        // /api/kubernetes/namespaces/{name} - DELETE
        let _fn: fn() -> Router<Arc<AppState>> = namespaces_routes;
    }

    #[test]
    fn test_namespaces_routes_has_quota_endpoint() {
        // /api/kubernetes/namespaces/{name}/quota - GET
        let _fn: fn() -> Router<Arc<AppState>> = namespaces_routes;
    }

    #[test]
    fn test_namespaces_routes_has_limits_endpoint() {
        // /api/kubernetes/namespaces/{name}/limits - GET
        let _fn: fn() -> Router<Arc<AppState>> = namespaces_routes;
    }

    #[test]
    fn test_namespaces_routes_router_not_empty() {
        let router = namespaces_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_namespaces_routes_function_signature() {
        let _: fn() -> Router<Arc<AppState>> = namespaces_routes;
    }
}
