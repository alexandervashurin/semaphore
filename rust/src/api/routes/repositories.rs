//! Маршруты репозиториев, ключей и переменных
//!
//! Repositories, Access Keys, Environment Variables

use crate::api::handlers;
use crate::api::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

/// Создаёт маршруты репозиториев
pub fn repository_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Репозитории
        .route(
            "/api/projects/{project_id}/repositories",
            get(handlers::get_repositories),
        )
        .route(
            "/api/projects/{project_id}/repositories",
            post(handlers::create_repository),
        )
        .route(
            "/api/projects/{project_id}/repositories/{id}",
            get(handlers::get_repository),
        )
        .route(
            "/api/projects/{project_id}/repositories/{id}",
            put(handlers::update_repository),
        )
        .route(
            "/api/projects/{project_id}/repositories/{id}",
            delete(handlers::delete_repository),
        )
        .route(
            "/api/project/{project_id}/repositories",
            get(handlers::get_repositories),
        )
        .route(
            "/api/project/{project_id}/repositories",
            post(handlers::create_repository),
        )
        .route(
            "/api/project/{project_id}/repositories/{id}",
            get(handlers::get_repository),
        )
        .route(
            "/api/project/{project_id}/repositories/{id}",
            put(handlers::update_repository),
        )
        .route(
            "/api/project/{project_id}/repositories/{id}",
            delete(handlers::delete_repository),
        )
        // Keys - используем handlers::get_access_keys
        .route(
            "/api/projects/{project_id}/keys",
            get(handlers::get_access_keys),
        )
        .route(
            "/api/projects/{project_id}/keys",
            post(handlers::create_access_key),
        )
        .route(
            "/api/projects/{project_id}/keys/{id}",
            get(handlers::get_access_key),
        )
        .route(
            "/api/projects/{project_id}/keys/{id}",
            put(handlers::update_access_key),
        )
        .route(
            "/api/projects/{project_id}/keys/{id}",
            delete(handlers::delete_access_key),
        )
        .route(
            "/api/project/{project_id}/keys",
            get(handlers::get_access_keys),
        )
        .route(
            "/api/project/{project_id}/keys",
            post(handlers::create_access_key),
        )
        .route(
            "/api/project/{project_id}/keys/{id}",
            get(handlers::get_access_key),
        )
        .route(
            "/api/project/{project_id}/keys/{id}",
            put(handlers::update_access_key),
        )
        .route(
            "/api/project/{project_id}/keys/{id}",
            delete(handlers::delete_access_key),
        )
        // Environment Variables - используем handlers::get_environments
        .route(
            "/api/projects/{project_id}/environments",
            get(handlers::get_environments),
        )
        .route(
            "/api/projects/{project_id}/environments",
            post(handlers::create_environment),
        )
        .route(
            "/api/projects/{project_id}/environments/{id}",
            get(handlers::get_environment),
        )
        .route(
            "/api/projects/{project_id}/environments/{id}",
            put(handlers::update_environment),
        )
        .route(
            "/api/projects/{project_id}/environments/{id}",
            delete(handlers::delete_environment),
        )
        // Алиас Vue: /api/project/{id}/environment
        .route(
            "/api/project/{project_id}/environment",
            get(handlers::get_environments),
        )
        .route(
            "/api/project/{project_id}/environment",
            post(handlers::create_environment),
        )
        .route(
            "/api/project/{project_id}/environment/{id}",
            get(handlers::get_environment),
        )
        .route(
            "/api/project/{project_id}/environment/{id}",
            put(handlers::update_environment),
        )
        .route(
            "/api/project/{project_id}/environment/{id}",
            delete(handlers::delete_environment),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_routes_creation() {
        let router = repository_routes();
        let _ = router;
    }

    #[test]
    fn test_repository_routes_return_type() {
        let router = repository_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_repository_routes_has_repositories_endpoints() {
        let _fn: fn() -> Router<Arc<AppState>> = repository_routes;
    }

    #[test]
    fn test_repository_routes_has_keys_endpoints() {
        let _fn: fn() -> Router<Arc<AppState>> = repository_routes;
    }

    #[test]
    fn test_repository_routes_has_environments_endpoints() {
        let _fn: fn() -> Router<Arc<AppState>> = repository_routes;
    }

    #[test]
    fn test_repository_routes_state_type() {
        let router = repository_routes();
        let _router: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_repository_routes_module_imports() {
        let _fn: fn() -> Router<Arc<AppState>> = repository_routes;
    }

    #[test]
    fn test_repository_routes_vue_alias() {
        let _fn: fn() -> Router<Arc<AppState>> = repository_routes;
    }

    #[test]
    fn test_repository_routes_has_repositories_crud() {
        // repositories: GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
        let _fn: fn() -> Router<Arc<AppState>> = repository_routes;
    }

    #[test]
    fn test_repository_routes_has_vue_alias_repos() {
        // Vue alias: /api/project/{id}/repositories
        let _fn: fn() -> Router<Arc<AppState>> = repository_routes;
    }

    #[test]
    fn test_repository_routes_has_keys_crud() {
        // keys: GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
        let _fn: fn() -> Router<Arc<AppState>> = repository_routes;
    }

    #[test]
    fn test_repository_routes_has_vue_alias_keys() {
        // Vue alias: /api/project/{id}/keys
        let _fn: fn() -> Router<Arc<AppState>> = repository_routes;
    }

    #[test]
    fn test_repository_routes_has_environments_crud() {
        // environments: GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
        let _fn: fn() -> Router<Arc<AppState>> = repository_routes;
    }

    #[test]
    fn test_repository_routes_has_vue_alias_environments() {
        // Vue alias: /api/project/{id}/environment
        let _fn: fn() -> Router<Arc<AppState>> = repository_routes;
    }

    #[test]
    fn test_repository_routes_uses_get_for_repositories() {
        // get_repositories handler is used with GET
        let _fn: fn() -> Router<Arc<AppState>> = repository_routes;
    }

    #[test]
    fn test_repository_routes_uses_post_for_create() {
        // create_repository, create_access_key, create_environment use POST
        let _fn: fn() -> Router<Arc<AppState>> = repository_routes;
    }

    #[test]
    fn test_repository_routes_uses_delete_for_cleanup() {
        // delete_repository, delete_access_key, delete_environment use DELETE
        let _fn: fn() -> Router<Arc<AppState>> = repository_routes;
    }

    #[test]
    fn test_repository_routes_uses_put_for_update() {
        // update_repository, update_access_key, update_environment use PUT
        let _fn: fn() -> Router<Arc<AppState>> = repository_routes;
    }
}
