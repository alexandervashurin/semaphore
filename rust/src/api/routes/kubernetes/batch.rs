//! Kubernetes Batch маршруты — Jobs, CronJobs, PriorityClass, PDB

use crate::api::handlers;
use axum::{routing::{get, post, put, delete}, Router};
use std::sync::Arc;
use crate::api::state::AppState;

/// Маршруты для управления batch-ресурсами
pub fn batch_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Jobs
        .route("/api/kubernetes/jobs", get(handlers::list_jobs))
        .route("/api/kubernetes/namespaces/{namespace}/jobs", post(handlers::create_job))
        .route("/api/kubernetes/namespaces/{namespace}/jobs/{name}", get(handlers::get_job))
        .route("/api/kubernetes/namespaces/{namespace}/jobs/{name}", delete(handlers::delete_job))
        .route("/api/kubernetes/namespaces/{namespace}/jobs/{name}/pods", get(handlers::list_job_pods))
        // CronJobs
        .route("/api/kubernetes/cronjobs", get(handlers::list_cronjobs))
        .route("/api/kubernetes/namespaces/{namespace}/cronjobs", post(handlers::create_cronjob))
        .route("/api/kubernetes/namespaces/{namespace}/cronjobs/{name}", get(handlers::get_cronjob))
        .route("/api/kubernetes/namespaces/{namespace}/cronjobs/{name}", delete(handlers::delete_cronjob))
        .route("/api/kubernetes/namespaces/{namespace}/cronjobs/{name}/suspend/{suspend}", put(handlers::update_cronjob_suspend))
        .route("/api/kubernetes/namespaces/{namespace}/cronjobs/{name}/history", get(handlers::list_cronjob_history))
        // PriorityClass
        .route("/api/kubernetes/priorityclasses", get(handlers::list_priority_classes))
        .route("/api/kubernetes/priorityclasses", post(handlers::create_priority_class))
        .route("/api/kubernetes/priorityclasses/{name}", delete(handlers::delete_priority_class))
        // PodDisruptionBudget
        .route("/api/kubernetes/poddisruptionbudgets", get(handlers::list_pdbs))
        .route("/api/kubernetes/namespaces/{namespace}/poddisruptionbudgets", post(handlers::create_pdb))
        .route("/api/kubernetes/namespaces/{namespace}/poddisruptionbudgets/{name}", delete(handlers::delete_pdb))
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
}
