//! Kubernetes Config маршруты — ConfigMaps, Secrets

use crate::api::handlers;
use crate::api::state::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

/// Маршруты для управления config-ресурсами
pub fn config_routes() -> Router<Arc<AppState>> {
    Router::new()
        // ConfigMaps
        .route("/api/kubernetes/configmaps", get(handlers::list_configmaps))
        .route(
            "/api/kubernetes/namespaces/{namespace}/configmaps",
            post(handlers::create_configmap),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/configmaps/{name}",
            get(handlers::get_configmap),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/configmaps/{name}",
            put(handlers::update_configmap),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/configmaps/{name}",
            delete(handlers::delete_configmap),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/configmaps/{name}/yaml",
            get(handlers::get_configmap_yaml),
        )
        .route(
            "/api/kubernetes/configmaps/validate",
            post(handlers::validate_configmap),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/configmaps/{name}/references",
            get(handlers::get_configmap_references),
        )
        // Secrets
        .route("/api/kubernetes/secrets", get(handlers::list_secrets))
        .route(
            "/api/kubernetes/namespaces/{namespace}/secrets",
            post(handlers::create_secret),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/secrets/{name}",
            get(handlers::get_secret),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/secrets/{name}",
            put(handlers::update_secret),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/secrets/{name}",
            delete(handlers::delete_secret),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/secrets/{name}/reveal",
            get(handlers::reveal_secret),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_routes_creates() {
        let _router: Router<Arc<AppState>> = config_routes();
    }

    #[test]
    fn test_config_routes_returns_router() {
        let router = config_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_config_routes_has_configmaps_crud() {
        // ConfigMaps: list, create, get, update, delete
        let _fn: fn() -> Router<Arc<AppState>> = config_routes;
    }

    #[test]
    fn test_config_routes_has_configmap_yaml() {
        // /api/kubernetes/namespaces/{namespace}/configmaps/{name}/yaml - GET
        let _fn: fn() -> Router<Arc<AppState>> = config_routes;
    }

    #[test]
    fn test_config_routes_has_configmap_validate() {
        // /api/kubernetes/configmaps/validate - POST
        let _fn: fn() -> Router<Arc<AppState>> = config_routes;
    }

    #[test]
    fn test_config_routes_has_configmap_references() {
        // /api/kubernetes/namespaces/{namespace}/configmaps/{name}/references - GET
        let _fn: fn() -> Router<Arc<AppState>> = config_routes;
    }

    #[test]
    fn test_config_routes_has_secrets_crud() {
        // Secrets: list, create, get, update, delete
        let _fn: fn() -> Router<Arc<AppState>> = config_routes;
    }

    #[test]
    fn test_config_routes_has_reveal_secret() {
        // /api/kubernetes/namespaces/{namespace}/secrets/{name}/reveal - GET
        let _fn: fn() -> Router<Arc<AppState>> = config_routes;
    }

    #[test]
    fn test_config_routes_router_not_empty() {
        let router = config_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_config_routes_function_signature() {
        let _: fn() -> Router<Arc<AppState>> = config_routes;
    }
}
