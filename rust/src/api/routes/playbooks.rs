//! Маршруты playbooks и inventories
//!
//! Playbooks, Inventories, Playbook Runs

use crate::api::handlers;
use crate::api::state::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

/// Создаёт маршруты playbooks
pub fn playbook_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Инвентари
        .route(
            "/api/projects/{project_id}/inventories",
            get(handlers::get_inventories),
        )
        .route(
            "/api/projects/{project_id}/inventories",
            post(handlers::create_inventory),
        )
        .route(
            "/api/projects/{project_id}/inventories/{id}",
            get(handlers::get_inventory),
        )
        .route(
            "/api/projects/{project_id}/inventories/{id}",
            put(handlers::update_inventory),
        )
        .route(
            "/api/projects/{project_id}/inventories/{id}",
            delete(handlers::delete_inventory),
        )
        // Алиас Vue: /api/project/{id}/inventory
        .route(
            "/api/project/{project_id}/inventory",
            get(handlers::get_inventories),
        )
        .route(
            "/api/project/{project_id}/inventory",
            post(handlers::create_inventory),
        )
        .route(
            "/api/project/{project_id}/inventory/{id}",
            get(handlers::get_inventory),
        )
        .route(
            "/api/project/{project_id}/inventory/{id}",
            put(handlers::update_inventory),
        )
        .route(
            "/api/project/{project_id}/inventory/{id}",
            delete(handlers::delete_inventory),
        )
        // Playbooks endpoint (из upstream)
        .route(
            "/api/projects/{project_id}/inventories/playbooks",
            get(handlers::get_playbooks),
        )
        // Playbooks - новые endpoints
        .route(
            "/api/project/{project_id}/playbooks",
            get(handlers::playbook::get_project_playbooks),
        )
        .route(
            "/api/project/{project_id}/playbooks",
            post(handlers::playbook::create_playbook),
        )
        .route(
            "/api/project/{project_id}/playbooks/{id}",
            get(handlers::playbook::get_playbook),
        )
        .route(
            "/api/project/{project_id}/playbooks/{id}",
            put(handlers::playbook::update_playbook),
        )
        .route(
            "/api/project/{project_id}/playbooks/{id}",
            delete(handlers::playbook::delete_playbook),
        )
        .route(
            "/api/project/{project_id}/playbooks/{id}/sync",
            post(handlers::playbook::sync_playbook),
        )
        .route(
            "/api/project/{project_id}/playbooks/{id}/preview",
            get(handlers::playbook::preview_playbook),
        )
        .route(
            "/api/project/{project_id}/playbooks/{id}/run",
            post(handlers::playbook::run_playbook),
        )
        // Playbook Runs - история запусков
        .route(
            "/api/project/{project_id}/playbook-runs",
            get(handlers::playbook_runs::get_playbook_runs),
        )
        .route(
            "/api/project/{project_id}/playbook-runs/{id}",
            get(handlers::playbook_runs::get_playbook_run),
        )
        .route(
            "/api/project/{project_id}/playbook-runs/{id}",
            delete(handlers::playbook_runs::delete_playbook_run),
        )
        .route(
            "/api/project/{project_id}/playbooks/{playbook_id}/runs/stats",
            get(handlers::playbook_runs::get_playbook_run_stats),
        )
}
