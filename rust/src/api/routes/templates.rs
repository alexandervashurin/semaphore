//! Маршруты шаблонов и workflows
//!
//! Templates, Workflows (DAG)

use crate::api::handlers;
use crate::api::state::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
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
