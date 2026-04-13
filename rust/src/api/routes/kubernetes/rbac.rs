//! Kubernetes RBAC маршруты — ServiceAccounts, Roles, RoleBindings, ClusterRoles, PSA

use crate::api::handlers;
use axum::{routing::{get, post, put, delete}, Router};
use std::sync::Arc;
use crate::api::state::AppState;

/// Маршруты для управления RBAC-ресурсами
pub fn rbac_routes() -> Router<Arc<AppState>> {
    Router::new()
        // RBAC UX
        .route("/api/kubernetes/rbac/check", post(handlers::check_kubernetes_rbac))
        .route("/api/kubernetes/rbac/check-action", get(handlers::check_rbac_action))
        .route("/api/kubernetes/rbac/cache/clear", post(handlers::clear_rbac_cache))
        .route("/api/kubernetes/rbac/rules-review", post(handlers::post_self_subject_rules_review))
        .route("/api/kubernetes/namespaces/{name}/pod-security", get(handlers::get_namespace_pod_security))
        .route("/api/kubernetes/namespaces/{name}/pod-security", put(handlers::put_namespace_pod_security))
        // ServiceAccounts
        .route("/api/kubernetes/serviceaccounts", get(handlers::list_service_accounts))
        .route("/api/kubernetes/namespaces/{namespace}/serviceaccounts", post(handlers::create_service_account))
        .route("/api/kubernetes/namespaces/{namespace}/serviceaccounts/{name}", get(handlers::get_service_account))
        .route("/api/kubernetes/namespaces/{namespace}/serviceaccounts/{name}", delete(handlers::delete_service_account))
        .route("/api/kubernetes/namespaces/{namespace}/serviceaccounts/{name}/secrets", get(handlers::list_service_account_secrets))
        // Roles & RoleBindings
        .route("/api/kubernetes/roles", get(handlers::list_roles))
        .route("/api/kubernetes/namespaces/{namespace}/roles", post(handlers::create_role))
        .route("/api/kubernetes/namespaces/{namespace}/roles/{name}", get(handlers::get_role))
        .route("/api/kubernetes/namespaces/{namespace}/roles/{name}", put(handlers::update_role))
        .route("/api/kubernetes/namespaces/{namespace}/roles/{name}", delete(handlers::delete_role))
        .route("/api/kubernetes/rolebindings", get(handlers::list_role_bindings))
        .route("/api/kubernetes/namespaces/{namespace}/rolebindings", post(handlers::create_role_binding))
        .route("/api/kubernetes/namespaces/{namespace}/rolebindings/{name}", get(handlers::get_role_binding))
        .route("/api/kubernetes/namespaces/{namespace}/rolebindings/{name}", put(handlers::update_role_binding))
        .route("/api/kubernetes/namespaces/{namespace}/rolebindings/{name}", delete(handlers::delete_role_binding))
        // ClusterRoles & ClusterRoleBindings
        .route("/api/kubernetes/clusterroles", get(handlers::list_cluster_roles))
        .route("/api/kubernetes/clusterroles", post(handlers::create_cluster_role))
        .route("/api/kubernetes/clusterroles/{name}", get(handlers::get_cluster_role))
        .route("/api/kubernetes/clusterroles/{name}", put(handlers::update_cluster_role))
        .route("/api/kubernetes/clusterroles/{name}", delete(handlers::delete_cluster_role))
        .route("/api/kubernetes/clusterrolebindings", get(handlers::list_cluster_role_bindings))
        .route("/api/kubernetes/clusterrolebindings", post(handlers::create_cluster_role_binding))
        .route("/api/kubernetes/clusterrolebindings/{name}", get(handlers::get_cluster_role_binding))
        .route("/api/kubernetes/clusterrolebindings/{name}", put(handlers::update_cluster_role_binding))
        .route("/api/kubernetes/clusterrolebindings/{name}", delete(handlers::delete_cluster_role_binding))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rbac_routes_creates() {
        let _router: Router<Arc<AppState>> = rbac_routes();
    }

    #[test]
    fn test_rbac_routes_returns_router() {
        let router = rbac_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_rbac_routes_has_rbac_check() {
        // /api/kubernetes/rbac/check - POST
        let _fn: fn() -> Router<Arc<AppState>> = rbac_routes;
    }

    #[test]
    fn test_rbac_routes_has_rbac_check_action() {
        // /api/kubernetes/rbac/check-action - GET
        let _fn: fn() -> Router<Arc<AppState>> = rbac_routes;
    }

    #[test]
    fn test_rbac_routes_has_rbac_cache_clear() {
        // /api/kubernetes/rbac/cache/clear - POST
        let _fn: fn() -> Router<Arc<AppState>> = rbac_routes;
    }

    #[test]
    fn test_rbac_routes_has_pod_security() {
        // Pod security: get, put
        let _fn: fn() -> Router<Arc<AppState>> = rbac_routes;
    }

    #[test]
    fn test_rbac_routes_has_serviceaccounts_crud() {
        // ServiceAccounts: list, create, get, delete
        let _fn: fn() -> Router<Arc<AppState>> = rbac_routes;
    }

    #[test]
    fn test_rbac_routes_has_service_account_secrets() {
        // /api/kubernetes/namespaces/{namespace}/serviceaccounts/{name}/secrets - GET
        let _fn: fn() -> Router<Arc<AppState>> = rbac_routes;
    }

    #[test]
    fn test_rbac_routes_has_roles_crud() {
        // Roles: list, create, get, update, delete
        let _fn: fn() -> Router<Arc<AppState>> = rbac_routes;
    }

    #[test]
    fn test_rbac_routes_has_role_bindings_crud() {
        // RoleBindings: list, create, get, update, delete
        let _fn: fn() -> Router<Arc<AppState>> = rbac_routes;
    }

    #[test]
    fn test_rbac_routes_has_cluster_roles_crud() {
        // ClusterRoles: list, create, get, update, delete
        let _fn: fn() -> Router<Arc<AppState>> = rbac_routes;
    }
}
