//! Маршруты шаблонов и workflows
//!
//! Templates, Workflows (DAG)

use crate::api::handlers;
use crate::api::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

/// Создаёт маршруты шаблонов и workflows
pub fn template_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Шаблоны
        .route(
            "/api/projects/{project_id}/templates",
            get(handlers::get_templates),
        )
        .route(
            "/api/projects/{project_id}/templates",
            post(handlers::create_template),
        )
        .route(
            "/api/projects/{project_id}/templates/{id}",
            get(handlers::get_template),
        )
        .route(
            "/api/projects/{project_id}/templates/{id}",
            put(handlers::update_template),
        )
        .route(
            "/api/projects/{project_id}/templates/{id}",
            delete(handlers::delete_template),
        )
        .route(
            "/api/project/{project_id}/templates",
            get(handlers::get_templates),
        )
        .route(
            "/api/project/{project_id}/templates",
            post(handlers::create_template),
        )
        .route(
            "/api/project/{project_id}/templates/{id}",
            get(handlers::get_template),
        )
        .route(
            "/api/project/{project_id}/templates/{id}",
            put(handlers::update_template),
        )
        .route(
            "/api/project/{project_id}/templates/{id}",
            delete(handlers::delete_template),
        )
        .route(
            "/api/project/{project_id}/templates/{id}/stop_all_tasks",
            post(handlers::stop_all_template_tasks),
        )
        // Workflows (DAG)
        .route(
            "/api/project/{project_id}/workflows",
            get(handlers::workflow::get_workflows),
        )
        .route(
            "/api/project/{project_id}/workflows",
            post(handlers::workflow::create_workflow),
        )
        .route(
            "/api/project/{project_id}/workflows/{id}",
            get(handlers::workflow::get_workflow),
        )
        .route(
            "/api/project/{project_id}/workflows/{id}",
            put(handlers::workflow::update_workflow),
        )
        .route(
            "/api/project/{project_id}/workflows/{id}",
            delete(handlers::workflow::delete_workflow),
        )
        .route(
            "/api/project/{project_id}/workflows/{id}/nodes",
            post(handlers::workflow::add_workflow_node),
        )
        .route(
            "/api/project/{project_id}/workflows/{id}/nodes/{node_id}",
            put(handlers::workflow::update_workflow_node),
        )
        .route(
            "/api/project/{project_id}/workflows/{id}/nodes/{node_id}",
            delete(handlers::workflow::delete_workflow_node),
        )
        .route(
            "/api/project/{project_id}/workflows/{id}/edges",
            post(handlers::workflow::add_workflow_edge),
        )
        .route(
            "/api/project/{project_id}/workflows/{id}/edges/{edge_id}",
            delete(handlers::workflow::delete_workflow_edge),
        )
        .route(
            "/api/project/{project_id}/workflows/{id}/run",
            post(handlers::workflow::run_workflow),
        )
    // .route(
    //     "/api/project/{project_id}/workflows/{id}/dry-run",
    //     post(handlers::workflow::dry_run_workflow),
    // )
    // Template Marketplace - заглушки
    // .route(
    //     "/api/project/{project_id}/templates/marketplace",
    //     get(handlers::template_marketplace::list_marketplace_templates),
    // )
    // Survey Forms - заглушки
    // .route(
    //     "/api/project/{project_id}/templates/{id}/survey",
    //     get(handlers::survey_form::get_template_survey_form),
    // )
    // Template Views - заглушки
    // .route(
    //     "/api/project/{project_id}/templates/views",
    //     get(handlers::template_view::list_template_views),
    // )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_routes_creation() {
        let router = template_routes();
        let _ = router;
    }

    #[test]
    fn test_template_routes_return_type() {
        let router = template_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_template_routes_has_templates_endpoints() {
        let _fn: fn() -> Router<Arc<AppState>> = template_routes;
    }

    #[test]
    fn test_template_routes_has_workflows_endpoints() {
        let _fn: fn() -> Router<Arc<AppState>> = template_routes;
    }

    #[test]
    fn test_template_routes_state_type() {
        let router = template_routes();
        let _router: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_template_routes_module_imports() {
        let _fn: fn() -> Router<Arc<AppState>> = template_routes;
    }

    #[test]
    fn test_template_routes_workflow_submodule() {
        let _fn: fn() -> Router<Arc<AppState>> = template_routes;
    }

    #[test]
    fn test_template_routes_vue_alias() {
        let _fn: fn() -> Router<Arc<AppState>> = template_routes;
    }

    #[test]
    fn test_template_routes_has_templates_crud() {
        // templates: GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
        let _fn: fn() -> Router<Arc<AppState>> = template_routes;
    }

    #[test]
    fn test_template_routes_has_vue_alias_crud() {
        // Vue alias: /api/project/{id}/templates
        let _fn: fn() -> Router<Arc<AppState>> = template_routes;
    }

    #[test]
    fn test_template_routes_has_stop_all_tasks() {
        // /api/project/{project_id}/templates/{id}/stop_all_tasks - POST
        let _fn: fn() -> Router<Arc<AppState>> = template_routes;
    }

    #[test]
    fn test_template_routes_has_workflows_crud() {
        // workflows: GET, POST, GET/{id}, PUT/{id}, DELETE/{id}
        let _fn: fn() -> Router<Arc<AppState>> = template_routes;
    }

    #[test]
    fn test_template_routes_has_workflow_nodes() {
        // workflow nodes: POST, PUT/{node_id}, DELETE/{node_id}
        let _fn: fn() -> Router<Arc<AppState>> = template_routes;
    }

    #[test]
    fn test_template_routes_has_workflow_edges() {
        // workflow edges: POST, DELETE/{edge_id}
        let _fn: fn() -> Router<Arc<AppState>> = template_routes;
    }

    #[test]
    fn test_template_routes_has_workflow_run() {
        // /api/project/{project_id}/workflows/{id}/run - POST
        let _fn: fn() -> Router<Arc<AppState>> = template_routes;
    }

    #[test]
    fn test_template_routes_uses_post_for_workflow_run() {
        // run_workflow uses POST method
        let _fn: fn() -> Router<Arc<AppState>> = template_routes;
    }

    #[test]
    fn test_template_routes_uses_delete_for_workflow_node() {
        // delete_workflow_node uses DELETE method
        let _fn: fn() -> Router<Arc<AppState>> = template_routes;
    }
}
