//! Kubernetes CSI snapshots API handlers (optional, read-only)

use axum::{
    extract::{Query, State},
    Json,
};
use kube::{
    api::{Api, DynamicObject, ListParams},
    core::{ApiResource, GroupVersionKind},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct SnapshotListQuery {
    pub namespace: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SnapshotApiStatus {
    pub installed: bool,
    pub volume_snapshot: bool,
    pub volume_snapshot_class: bool,
}

fn gvk(group: &str, version: &str, kind: &str) -> GroupVersionKind {
    GroupVersionKind::gvk(group, version, kind)
}

fn ar(group: &str, version: &str, kind: &str, plural: &str) -> ApiResource {
    ApiResource::from_gvk_with_plural(&gvk(group, version, kind), plural)
}

pub async fn get_snapshot_api_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SnapshotApiStatus>> {
    let client = state.kubernetes_client()?;
    let lp = ListParams::default().limit(1);
    let vs_api: Api<DynamicObject> = Api::all_with(
        client.raw().clone(),
        &ar(
            "snapshot.storage.k8s.io",
            "v1",
            "VolumeSnapshot",
            "volumesnapshots",
        ),
    );
    let vsc_api: Api<DynamicObject> = Api::all_with(
        client.raw().clone(),
        &ar(
            "snapshot.storage.k8s.io",
            "v1",
            "VolumeSnapshotClass",
            "volumesnapshotclasses",
        ),
    );
    let volume_snapshot = vs_api.list(&lp).await.is_ok();
    let volume_snapshot_class = vsc_api.list(&lp).await.is_ok();
    Ok(Json(SnapshotApiStatus {
        installed: volume_snapshot || volume_snapshot_class,
        volume_snapshot,
        volume_snapshot_class,
    }))
}

pub async fn list_volume_snapshots(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SnapshotListQuery>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let client = state.kubernetes_client()?;
    let api_res = ar(
        "snapshot.storage.k8s.io",
        "v1",
        "VolumeSnapshot",
        "volumesnapshots",
    );
    let api: Api<DynamicObject> = if let Some(ns) = query.namespace.as_deref() {
        Api::namespaced_with(client.raw().clone(), ns, &api_res)
    } else {
        Api::all_with(client.raw().clone(), &api_res)
    };
    let mut lp = ListParams::default();
    if let Some(limit) = query.limit {
        lp = lp.limit(limit);
    }
    let items = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("VolumeSnapshot API not available: {e}")))?;
    Ok(Json(
        items.items.iter().map(|x| serde_json::json!(x)).collect(),
    ))
}

pub async fn list_volume_snapshot_classes(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SnapshotListQuery>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let client = state.kubernetes_client()?;
    let api_res = ar(
        "snapshot.storage.k8s.io",
        "v1",
        "VolumeSnapshotClass",
        "volumesnapshotclasses",
    );
    let api: Api<DynamicObject> = Api::all_with(client.raw().clone(), &api_res);
    let mut lp = ListParams::default();
    if let Some(limit) = query.limit {
        lp = lp.limit(limit);
    }
    let items = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("VolumeSnapshotClass API not available: {e}")))?;
    Ok(Json(
        items.items.iter().map(|x| serde_json::json!(x)).collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_list_query_default() {
        let q: SnapshotListQuery = serde_json::from_str("{}").unwrap();
        assert!(q.namespace.is_none());
        assert!(q.limit.is_none());
    }

    #[test]
    fn test_snapshot_list_query_with_namespace() {
        let q: SnapshotListQuery = serde_json::from_str(r#"{"namespace": "kube-system"}"#).unwrap();
        assert_eq!(q.namespace, Some("kube-system".to_string()));
        assert!(q.limit.is_none());
    }

    #[test]
    fn test_snapshot_list_query_with_limit() {
        let q: SnapshotListQuery = serde_json::from_str(r#"{"limit": 50}"#).unwrap();
        assert_eq!(q.limit, Some(50));
        assert!(q.namespace.is_none());
    }

    #[test]
    fn test_snapshot_list_query_full() {
        let q: SnapshotListQuery =
            serde_json::from_str(r#"{"namespace": "default", "limit": 10}"#).unwrap();
        assert_eq!(q.namespace, Some("default".to_string()));
        assert_eq!(q.limit, Some(10));
    }

    #[test]
    fn test_snapshot_list_query_debug_format() {
        let q = SnapshotListQuery {
            namespace: Some("test".into()),
            limit: Some(5),
        };
        let debug_str = format!("{:?}", q);
        assert!(debug_str.contains("SnapshotListQuery"));
    }

    #[test]
    fn test_snapshot_api_status_default_true() {
        let status = SnapshotApiStatus {
            installed: true,
            volume_snapshot: true,
            volume_snapshot_class: true,
        };
        let serialized = serde_json::to_string(&status).unwrap();
        assert!(serialized.contains("true"));
    }

    #[test]
    fn test_snapshot_api_status_all_false() {
        let status = SnapshotApiStatus {
            installed: false,
            volume_snapshot: false,
            volume_snapshot_class: false,
        };
        let serialized = serde_json::to_string(&status).unwrap();
        assert!(serialized.contains(r#""installed":false"#));
        assert!(serialized.contains(r#""volume_snapshot":false"#));
        assert!(serialized.contains(r#""volume_snapshot_class":false"#));
    }

    #[test]
    fn test_snapshot_api_status_mixed() {
        let status = SnapshotApiStatus {
            installed: true,
            volume_snapshot: true,
            volume_snapshot_class: false,
        };
        let json_val = serde_json::to_value(&status).unwrap();
        assert_eq!(json_val["installed"], true);
        assert_eq!(json_val["volume_snapshot"], true);
        assert_eq!(json_val["volume_snapshot_class"], false);
    }

    #[test]
    fn test_gvk_creation() {
        let gvk_result = gvk("snapshot.storage.k8s.io", "v1", "VolumeSnapshot");
        assert_eq!(gvk_result.group, "snapshot.storage.k8s.io");
        assert_eq!(gvk_result.version, "v1");
        assert_eq!(gvk_result.kind, "VolumeSnapshot");
    }

    #[test]
    fn test_api_resource_creation() {
        let ar_result = ar(
            "snapshot.storage.k8s.io",
            "v1",
            "VolumeSnapshot",
            "volumesnapshots",
        );
        assert_eq!(ar_result.group, "snapshot.storage.k8s.io");
        assert_eq!(ar_result.version, "v1");
        assert_eq!(ar_result.plural, "volumesnapshots");
    }

    #[test]
    fn test_snapshot_api_status_serialize_fields() {
        let original = SnapshotApiStatus {
            installed: true,
            volume_snapshot: false,
            volume_snapshot_class: true,
        };
        let serialized = serde_json::to_string(&original).unwrap();
        assert!(serialized.contains(r#""installed":true"#));
        assert!(serialized.contains(r#""volume_snapshot":false"#));
        assert!(serialized.contains(r#""volume_snapshot_class":true"#));
    }

    #[test]
    fn test_snapshot_api_status_debug_format() {
        let status = SnapshotApiStatus {
            installed: true,
            volume_snapshot: false,
            volume_snapshot_class: true,
        };
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("SnapshotApiStatus"));
    }
}
