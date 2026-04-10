//! Kubernetes Storage маршруты — PV, PVC, StorageClass, Snapshots, CSI

use crate::api::handlers;
use axum::{routing::{get, post, put, delete}, Router};
use std::sync::Arc;
use crate::api::state::AppState;

/// Маршруты для управления storage-ресурсами
pub fn storage_routes() -> Router<Arc<AppState>> {
    Router::new()
        // PersistentVolumes
        .route("/api/kubernetes/persistentvolumes", get(handlers::list_persistent_volumes))
        .route("/api/kubernetes/persistentvolumes", post(handlers::create_persistent_volume))
        .route("/api/kubernetes/persistentvolumes/{name}", get(handlers::get_persistent_volume))
        .route("/api/kubernetes/persistentvolumes/{name}", delete(handlers::delete_persistent_volume))
        // PersistentVolumeClaims
        .route("/api/kubernetes/persistentvolumeclaims", get(handlers::list_persistent_volume_claims))
        .route("/api/kubernetes/namespaces/{namespace}/persistentvolumeclaims", post(handlers::create_persistent_volume_claim))
        .route("/api/kubernetes/namespaces/{namespace}/persistentvolumeclaims/{name}", get(handlers::get_persistent_volume_claim))
        .route("/api/kubernetes/namespaces/{namespace}/persistentvolumeclaims/{name}", put(handlers::update_persistent_volume_claim))
        .route("/api/kubernetes/namespaces/{namespace}/persistentvolumeclaims/{name}", delete(handlers::delete_persistent_volume_claim))
        // StorageClasses
        .route("/api/kubernetes/storageclasses", get(handlers::list_storage_classes))
        .route("/api/kubernetes/storageclasses", post(handlers::create_storage_class))
        .route("/api/kubernetes/storageclasses/{name}", get(handlers::get_storage_class))
        .route("/api/kubernetes/storageclasses/{name}", delete(handlers::delete_storage_class))
        // CSI snapshots (read-only)
        .route("/api/kubernetes/snapshots/status", get(handlers::get_snapshot_api_status))
        .route("/api/kubernetes/volumesnapshots", get(handlers::list_volume_snapshots))
        .route("/api/kubernetes/volumesnapshotclasses", get(handlers::list_volume_snapshot_classes))
        // CSI details (read-only)
        .route("/api/kubernetes/csi/status", get(handlers::get_csi_api_status))
        .route("/api/kubernetes/csidrivers", get(handlers::list_csi_drivers))
        .route("/api/kubernetes/csinodes", get(handlers::list_csi_nodes))
        .route("/api/kubernetes/volumeattachments", get(handlers::list_volume_attachments))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_routes_creates() {
        let _router: Router<Arc<AppState>> = storage_routes();
    }

    #[test]
    fn test_storage_routes_returns_router() {
        let router = storage_routes();
        let _: Router<Arc<AppState>> = router;
    }
}
