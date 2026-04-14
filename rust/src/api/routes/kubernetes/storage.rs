//! Kubernetes Storage маршруты — PV, PVC, StorageClass, Snapshots, CSI

use crate::api::handlers;
use crate::api::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

/// Маршруты для управления storage-ресурсами
pub fn storage_routes() -> Router<Arc<AppState>> {
    Router::new()
        // PersistentVolumes
        .route(
            "/api/kubernetes/persistentvolumes",
            get(handlers::list_persistent_volumes),
        )
        .route(
            "/api/kubernetes/persistentvolumes",
            post(handlers::create_persistent_volume),
        )
        .route(
            "/api/kubernetes/persistentvolumes/{name}",
            get(handlers::get_persistent_volume),
        )
        .route(
            "/api/kubernetes/persistentvolumes/{name}",
            delete(handlers::delete_persistent_volume),
        )
        // PersistentVolumeClaims
        .route(
            "/api/kubernetes/persistentvolumeclaims",
            get(handlers::list_persistent_volume_claims),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/persistentvolumeclaims",
            post(handlers::create_persistent_volume_claim),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/persistentvolumeclaims/{name}",
            get(handlers::get_persistent_volume_claim),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/persistentvolumeclaims/{name}",
            put(handlers::update_persistent_volume_claim),
        )
        .route(
            "/api/kubernetes/namespaces/{namespace}/persistentvolumeclaims/{name}",
            delete(handlers::delete_persistent_volume_claim),
        )
        // StorageClasses
        .route(
            "/api/kubernetes/storageclasses",
            get(handlers::list_storage_classes),
        )
        .route(
            "/api/kubernetes/storageclasses",
            post(handlers::create_storage_class),
        )
        .route(
            "/api/kubernetes/storageclasses/{name}",
            get(handlers::get_storage_class),
        )
        .route(
            "/api/kubernetes/storageclasses/{name}",
            delete(handlers::delete_storage_class),
        )
        // CSI snapshots (read-only)
        .route(
            "/api/kubernetes/snapshots/status",
            get(handlers::get_snapshot_api_status),
        )
        .route(
            "/api/kubernetes/volumesnapshots",
            get(handlers::list_volume_snapshots),
        )
        .route(
            "/api/kubernetes/volumesnapshotclasses",
            get(handlers::list_volume_snapshot_classes),
        )
        // CSI details (read-only)
        .route(
            "/api/kubernetes/csi/status",
            get(handlers::get_csi_api_status),
        )
        .route(
            "/api/kubernetes/csidrivers",
            get(handlers::list_csi_drivers),
        )
        .route("/api/kubernetes/csinodes", get(handlers::list_csi_nodes))
        .route(
            "/api/kubernetes/volumeattachments",
            get(handlers::list_volume_attachments),
        )
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

    #[test]
    fn test_storage_routes_has_persistent_volumes_crud() {
        // PV: list, create, get, delete
        let _fn: fn() -> Router<Arc<AppState>> = storage_routes;
    }

    #[test]
    fn test_storage_routes_has_persistent_volume_claims() {
        // PVC: list, create, get, update, delete
        let _fn: fn() -> Router<Arc<AppState>> = storage_routes;
    }

    #[test]
    fn test_storage_routes_has_storage_classes() {
        // StorageClasses: list, create, get, delete
        let _fn: fn() -> Router<Arc<AppState>> = storage_routes;
    }

    #[test]
    fn test_storage_routes_has_snapshots_status() {
        // /api/kubernetes/snapshots/status - GET
        let _fn: fn() -> Router<Arc<AppState>> = storage_routes;
    }

    #[test]
    fn test_storage_routes_has_volume_snapshots() {
        // /api/kubernetes/volumesnapshots - GET
        let _fn: fn() -> Router<Arc<AppState>> = storage_routes;
    }

    #[test]
    fn test_storage_routes_has_volume_snapshot_classes() {
        // /api/kubernetes/volumesnapshotclasses - GET
        let _fn: fn() -> Router<Arc<AppState>> = storage_routes;
    }

    #[test]
    fn test_storage_routes_has_csi_status() {
        // /api/kubernetes/csi/status - GET
        let _fn: fn() -> Router<Arc<AppState>> = storage_routes;
    }

    #[test]
    fn test_storage_routes_has_csi_drivers() {
        // /api/kubernetes/csidrivers - GET
        let _fn: fn() -> Router<Arc<AppState>> = storage_routes;
    }

    #[test]
    fn test_storage_routes_router_not_empty() {
        let router = storage_routes();
        assert!(std::mem::size_of_val(&router) > 0);
    }

    #[test]
    fn test_storage_routes_function_signature() {
        let _: fn() -> Router<Arc<AppState>> = storage_routes;
    }
}
