//! Kubernetes Apply маршруты — manifest apply, diff, kubectl generator

use crate::api::handlers;
use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use crate::api::state::AppState;

/// Маршруты для apply/diff/генератора команд
pub fn apply_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/kubernetes/apply", post(handlers::apply_manifest))
        .route("/api/kubernetes/apply/diff", post(handlers::diff_manifest))
        .route("/api/kubernetes/apply/kubectl", get(handlers::generate_kubectl_command))
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
}
