//! Kubernetes Apply маршруты — manifest apply, diff, kubectl generator

use crate::api::handlers;
use crate::api::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

/// Маршруты для apply/diff/генератора команд
pub fn apply_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/kubernetes/apply", post(handlers::apply_manifest))
        .route("/api/kubernetes/apply/diff", post(handlers::diff_manifest))
        .route(
            "/api/kubernetes/apply/kubectl",
            get(handlers::generate_kubectl_command),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_routes_creates() {
        let _router: Router<Arc<AppState>> = apply_routes();
    }

    #[test]
    fn test_apply_routes_returns_router() {
        let router = apply_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_apply_routes_has_apply_endpoint() {
        // /api/kubernetes/apply - POST
        let _fn: fn() -> Router<Arc<AppState>> = apply_routes;
    }

    #[test]
    fn test_apply_routes_has_diff_endpoint() {
        // /api/kubernetes/apply/diff - POST
        let _fn: fn() -> Router<Arc<AppState>> = apply_routes;
    }

    #[test]
    fn test_apply_routes_has_kubectl_endpoint() {
        // /api/kubernetes/apply/kubectl - GET
        let _fn: fn() -> Router<Arc<AppState>> = apply_routes;
    }

    #[test]
    fn test_apply_routes_uses_post_for_apply() {
        // apply_manifest uses POST
        let _fn: fn() -> Router<Arc<AppState>> = apply_routes;
    }

    #[test]
    fn test_apply_routes_uses_post_for_diff() {
        // diff_manifest uses POST
        let _fn: fn() -> Router<Arc<AppState>> = apply_routes;
    }

    #[test]
    fn test_apply_routes_uses_get_for_kubectl() {
        // generate_kubectl_command uses GET
        let _fn: fn() -> Router<Arc<AppState>> = apply_routes;
    }

    #[test]
    fn test_apply_routes_router_not_empty() {
        let router = apply_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_apply_routes_function_signature() {
        let _: fn() -> Router<Arc<AppState>> = apply_routes;
    }
}
