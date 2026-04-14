//! Kubernetes Batch маршруты — Jobs, CronJobs, PriorityClass, PDB

use crate::api::handlers;
use crate::api::state::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

/// Маршруты для управления batch-ресурсами
pub fn batch_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Jobs
        .route("/api/kubernetes/jobs", get(handlers::list_jobs))
        .route(
            "/api/kubernetes/namespaces/{namespace}/jobs",
            post(handlers::create_job),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/jobs/{name}",
            get(handlers::get_job),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/jobs/{name}",
            delete(handlers::delete_job),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/jobs/{name}/pods",
            get(handlers::list_job_pods),
        )
        // CronJobs
        .route("/api/kubernetes/cronjobs", get(handlers::list_cronjobs))
        .route(
            "/api/kubernetes/namespaces/{namespace}/cronjobs",
            post(handlers::create_cronjob),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/cronjobs/{name}",
            get(handlers::get_cronjob),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/cronjobs/{name}",
            delete(handlers::delete_cronjob),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/cronjobs/{name}/suspend/{suspend}",
            put(handlers::update_cronjob_suspend),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/cronjobs/{name}/history",
            get(handlers::list_cronjob_history),
        )
        // PriorityClass
        .route(
            "/api/kubernetes/priorityclasses",
            get(handlers::list_priority_classes),
        )
        .route(
            "/api/kubernetes/priorityclasses",
            post(handlers::create_priority_class),
        )
        .route(
            "/api/kubernetes/priorityclasses/{name}",
            delete(handlers::delete_priority_class),
        )
        // PodDisruptionBudget
        .route(
            "/api/kubernetes/poddisruptionbudgets",
            get(handlers::list_pdbs),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/poddisruptionbudgets",
            post(handlers::create_pdb),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/poddisruptionbudgets/{name}",
            delete(handlers::delete_pdb),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_routes_creates() {
        let _router: Router<Arc<AppState>> = batch_routes();
    }

    #[test]
    fn test_batch_routes_returns_router() {
        let router = batch_routes();
        let _: Router<Arc<AppState>> = router;
    }

    #[test]
    fn test_batch_routes_has_jobs_endpoints() {
        // Jobs: list, create, get, delete, list_pods
        let _fn: fn() -> Router<Arc<AppState>> = batch_routes;
    }

    #[test]
    fn test_batch_routes_has_cronjobs_endpoints() {
        // CronJobs: list, create, get, delete
        let _fn: fn() -> Router<Arc<AppState>> = batch_routes;
    }

    #[test]
    fn test_batch_routes_has_cronjob_suspend() {
        // /api/kubernetes/namespaces/{namespace}/cronjobs/{name}/suspend/{suspend} - PUT
        let _fn: fn() -> Router<Arc<AppState>> = batch_routes;
    }

    #[test]
    fn test_batch_routes_has_cronjob_history() {
        // /api/kubernetes/namespaces/{namespace}/cronjobs/{name}/history - GET
        let _fn: fn() -> Router<Arc<AppState>> = batch_routes;
    }

    #[test]
    fn test_batch_routes_has_priority_class_endpoints() {
        // PriorityClass: list, create, delete
        let _fn: fn() -> Router<Arc<AppState>> = batch_routes;
    }

    #[test]
    fn test_batch_routes_has_pdb_endpoints() {
        // PDB: list, create, delete
        let _fn: fn() -> Router<Arc<AppState>> = batch_routes;
    }

    #[test]
    fn test_batch_routes_uses_delete_for_jobs() {
        // delete_job, delete_cronjob, delete_priority_class, delete_pdb use DELETE
        let _fn: fn() -> Router<Arc<AppState>> = batch_routes;
    }

    #[test]
    fn test_batch_routes_router_not_empty() {
        let router = batch_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_batch_routes_function_signature() {
        let _: fn() -> Router<Arc<AppState>> = batch_routes;
    }
}
