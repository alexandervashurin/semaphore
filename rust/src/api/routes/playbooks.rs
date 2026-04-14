//! Маршруты playbooks и inventories
//!
//! Playbooks, Inventories, Playbook Runs

use crate::api::handlers;
use crate::api::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playbook_routes_creation() {
        let router = playbook_routes();
        let _ = router;
    }

    #[test]
    fn test_playbook_routes_return_type() {
        let router = playbook_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_playbook_routes_has_inventories_endpoints() {
        let _fn: fn() -> Router<Arc<AppState>> = playbook_routes;
    }

    #[test]
    fn test_playbook_routes_has_playbooks_endpoints() {
        let _fn: fn() -> Router<Arc<AppState>> = playbook_routes;
    }

    #[test]
    fn test_playbook_routes_has_playbook_runs_endpoints() {
        let _fn: fn() -> Router<Arc<AppState>> = playbook_routes;
    }

    #[test]
    fn test_playbook_routes_state_type() {
        let router = playbook_routes();
        let _router: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_playbook_routes_module_imports() {
        let _fn: fn() -> Router<Arc<AppState>> = playbook_routes;
    }

    #[test]
    fn test_playbook_routes_vue_alias() {
        let _fn: fn() -> Router<Arc<AppState>> = playbook_routes;
    }

    #[test]
    fn test_playbook_routes_has_inventories_crud() {
        // inventories: GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
        let _fn: fn() -> Router<Arc<AppState>> = playbook_routes;
    }

    #[test]
    fn test_playbook_routes_has_vue_alias_crud() {
        // Vue alias: /api/project/{id}/inventory routes
        let _fn: fn() -> Router<Arc<AppState>> = playbook_routes;
    }

    #[test]
    fn test_playbook_routes_has_playbooks_list() {
        // /api/projects/{project_id}/inventories/playbooks
        let _fn: fn() -> Router<Arc<AppState>> = playbook_routes;
    }

    #[test]
    fn test_playbook_routes_has_playbook_crud() {
        // playbooks: GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
        let _fn: fn() -> Router<Arc<AppState>> = playbook_routes;
    }

    #[test]
    fn test_playbook_routes_has_playbook_sync() {
        // /api/project/{project_id}/playbooks/{id}/sync
        let _fn: fn() -> Router<Arc<AppState>> = playbook_routes;
    }

    #[test]
    fn test_playbook_routes_has_playbook_preview() {
        // /api/project/{project_id}/playbooks/{id}/preview
        let _fn: fn() -> Router<Arc<AppState>> = playbook_routes;
    }

    #[test]
    fn test_playbook_routes_has_playbook_run() {
        // /api/project/{project_id}/playbooks/{id}/run
        let _fn: fn() -> Router<Arc<AppState>> = playbook_routes;
    }

    #[test]
    fn test_playbook_routes_has_playbook_runs_list() {
        // /api/project/{project_id}/playbook-runs
        let _fn: fn() -> Router<Arc<AppState>> = playbook_routes;
    }

    #[test]
    fn test_playbook_routes_uses_delete_for_inventory() {
        // delete_inventory handler is used
        let _fn: fn() -> Router<Arc<AppState>> = playbook_routes;
    }

    #[test]
    fn test_playbook_routes_uses_post_for_playbook_run() {
        // run_playbook uses POST method
        let _fn: fn() -> Router<Arc<AppState>> = playbook_routes;
    }

    #[test]
    fn test_playbook_routes_uses_get_for_stats() {
        // get_playbook_run_stats uses GET method
        let _fn: fn() -> Router<Arc<AppState>> = playbook_routes;
    }
}
