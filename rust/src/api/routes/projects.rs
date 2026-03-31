//! Маршруты проектов и организаций
//!
//! Проекты, статистика, организации, брендинг, deployment environments

use crate::api::handlers;
use crate::api::handlers::projects::project;
use crate::api::state::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

/// Создаёт маршруты проектов
pub fn project_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Проекты
        .route("/api/projects", get(handlers::get_projects))
        .route("/api/projects", post(handlers::add_project))
        .route("/api/projects/restore", post(handlers::restore_project))
        .route("/api/projects/{id}", get(handlers::get_project))
        .route("/api/projects/{id}", put(handlers::update_project))
        .route("/api/projects/{id}", delete(handlers::delete_project))
        // Алиасы для Vue upstream (singular /api/project/ вместо /api/projects/)
        .route("/api/project/{id}", get(handlers::get_project))
        .route("/api/project/{id}", put(handlers::update_project))
        .route("/api/project/{id}", delete(handlers::delete_project))
        // Leave project + Project stats
        .route(
            "/api/project/{project_id}/me",
            delete(project::leave_project),
        )
        .route(
            "/api/projects/{project_id}/me",
            delete(project::leave_project),
        )
        .route(
            "/api/project/{project_id}/stats",
            get(project::get_project_stats),
        )
        .route(
            "/api/projects/{project_id}/stats",
            get(project::get_project_stats),
        )
        // Organizations (Multi-Tenancy, v4.0)
        .route(
            "/api/organizations",
            get(handlers::organization::get_organizations),
        )
        .route(
            "/api/organizations",
            post(handlers::organization::create_organization),
        )
        .route(
            "/api/organizations/{id}",
            get(handlers::organization::get_organization),
        )
        .route(
            "/api/organizations/{id}",
            put(handlers::organization::update_organization),
        )
        .route(
            "/api/organizations/{id}",
            delete(handlers::organization::delete_organization),
        )
        .route(
            "/api/organizations/{id}/users",
            get(handlers::organization::get_organization_users),
        )
        .route(
            "/api/organizations/{id}/users",
            post(handlers::organization::add_organization_user),
        )
        .route(
            "/api/organizations/{id}/users/{user_id}",
            delete(handlers::organization::remove_organization_user),
        )
        .route(
            "/api/organizations/{id}/users/{user_id}/role",
            put(handlers::organization::update_organization_user_role),
        )
        .route(
            "/api/organizations/{id}/quota/{quota_type}",
            get(handlers::organization::check_organization_quota),
        )
        .route(
            "/api/organizations/{id}/branding",
            get(handlers::organization::get_organization_branding),
        )
        .route(
            "/api/organizations/{id}/branding",
            put(handlers::organization::update_organization_branding),
        )
        .route(
            "/api/user/organizations",
            get(handlers::organization::get_my_organizations),
        )
        // Deployment Environments (FI-GL-1 — GitLab Environments)
        .route(
            "/api/project/{project_id}/deploy-environments",
            get(handlers::deployment_environment::list_deploy_environments)
                .post(handlers::deployment_environment::create_deploy_environment),
        )
        .route(
            "/api/project/{project_id}/deploy-environments/{id}",
            get(handlers::deployment_environment::get_deploy_environment)
                .put(handlers::deployment_environment::update_deploy_environment)
                .delete(handlers::deployment_environment::delete_deploy_environment),
        )
        .route(
            "/api/project/{project_id}/deploy-environments/{id}/history",
            get(handlers::deployment_environment::get_deploy_history),
        )
}
