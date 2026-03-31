//! Маршруты репозиториев, ключей и переменных
//!
//! Repositories, Access Keys, Environment Variables

use crate::api::handlers;
use crate::api::state::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
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
