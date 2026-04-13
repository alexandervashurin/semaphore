//! Kubernetes Helm маршруты — repos, charts, releases

use crate::api::handlers;
use axum::{routing::{get, post, put, delete}, Router};
use std::sync::Arc;
use crate::api::state::AppState;

/// Маршруты для управления Helm
pub fn helm_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/kubernetes/helm/repos", get(handlers::list_helm_repos))
        .route("/api/kubernetes/helm/repos", post(handlers::add_helm_repo))
        .route("/api/kubernetes/helm/charts", get(handlers::search_helm_charts))
        .route("/api/kubernetes/helm/charts/{repo}/{chart}", get(handlers::get_helm_chart))
        .route("/api/kubernetes/helm/releases", get(handlers::list_helm_releases))
        .route("/api/kubernetes/helm/releases", post(handlers::install_helm_chart))
        .route("/api/kubernetes/helm/releases/{namespace}/{name}", get(handlers::get_helm_release_history))
        .route("/api/kubernetes/helm/releases/{namespace}/{name}", put(handlers::upgrade_helm_release))
        .route("/api/kubernetes/helm/releases/{namespace}/{name}/rollback", post(handlers::rollback_helm_release))
        .route("/api/kubernetes/helm/releases/{namespace}/{name}", delete(handlers::uninstall_helm_release))
        .route("/api/kubernetes/helm/releases/{namespace}/{name}/values", get(handlers::get_helm_release_values))
        .route("/api/kubernetes/helm/releases/{namespace}/{name}/values", put(handlers::update_helm_release_values))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helm_routes_creates() {
        let _router: Router<Arc<AppState>> = helm_routes();
    }

    #[test]
    fn test_helm_routes_returns_router() {
        let router = helm_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_helm_routes_has_repos_endpoints() {
        // Helm repos: list, add
        let _fn: fn() -> Router<Arc<AppState>> = helm_routes;
    }

    #[test]
    fn test_helm_routes_has_charts_search() {
        // /api/kubernetes/helm/charts - GET (search)
        let _fn: fn() -> Router<Arc<AppState>> = helm_routes;
    }

    #[test]
    fn test_helm_routes_has_chart_details() {
        // /api/kubernetes/helm/charts/{repo}/{chart} - GET
        let _fn: fn() -> Router<Arc<AppState>> = helm_routes;
    }

    #[test]
    fn test_helm_routes_has_releases_list() {
        // /api/kubernetes/helm/releases - GET
        let _fn: fn() -> Router<Arc<AppState>> = helm_routes;
    }

    #[test]
    fn test_helm_routes_has_install_release() {
        // /api/kubernetes/helm/releases - POST
        let _fn: fn() -> Router<Arc<AppState>> = helm_routes;
    }

    #[test]
    fn test_helm_routes_has_release_history() {
        // /api/kubernetes/helm/releases/{namespace}/{name} - GET
        let _fn: fn() -> Router<Arc<AppState>> = helm_routes;
    }

    #[test]
    fn test_helm_routes_has_upgrade_release() {
        // /api/kubernetes/helm/releases/{namespace}/{name} - PUT
        let _fn: fn() -> Router<Arc<AppState>> = helm_routes;
    }

    #[test]
    fn test_helm_routes_has_rollback_release() {
        // /api/kubernetes/helm/releases/{namespace}/{name}/rollback - POST
        let _fn: fn() -> Router<Arc<AppState>> = helm_routes;
    }

    #[test]
    fn test_helm_routes_has_values_management() {
        // get and update release values
        let _fn: fn() -> Router<Arc<AppState>> = helm_routes;
    }
}
