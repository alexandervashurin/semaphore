//! Kubernetes Config маршруты — ConfigMaps, Secrets

use crate::api::handlers;
use axum::{routing::{get, post, put, delete}, Router};
use std::sync::Arc;
use crate::api::state::AppState;

/// Маршруты для управления config-ресурсами
pub fn config_routes() -> Router<Arc<AppState>> {
    Router::new()
        // ConfigMaps
        .route("/api/kubernetes/configmaps", get(handlers::list_configmaps))
        .route("/api/kubernetes/namespaces/{namespace}/configmaps", post(handlers::create_configmap))
        .route("/api/kubernetes/namespaces/{namespace}/configmaps/{name}", get(handlers::get_configmap))
        .route("/api/kubernetes/namespaces/{namespace}/configmaps/{name}", put(handlers::update_configmap))
        .route("/api/kubernetes/namespaces/{namespace}/configmaps/{name}", delete(handlers::delete_configmap))
        .route("/api/kubernetes/namespaces/{namespace}/configmaps/{name}/yaml", get(handlers::get_configmap_yaml))
        .route("/api/kubernetes/configmaps/validate", post(handlers::validate_configmap))
        .route("/api/kubernetes/namespaces/{namespace}/configmaps/{name}/references", get(handlers::get_configmap_references))
        // Secrets
        .route("/api/kubernetes/secrets", get(handlers::list_secrets))
        .route("/api/kubernetes/namespaces/{namespace}/secrets", post(handlers::create_secret))
        .route("/api/kubernetes/namespaces/{namespace}/secrets/{name}", get(handlers::get_secret))
        .route("/api/kubernetes/namespaces/{namespace}/secrets/{name}", put(handlers::update_secret))
        .route("/api/kubernetes/namespaces/{namespace}/secrets/{name}", delete(handlers::delete_secret))
        .route("/api/kubernetes/namespaces/{namespace}/secrets/{name}/reveal", get(handlers::reveal_secret))
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
}
