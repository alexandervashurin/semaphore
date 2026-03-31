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
