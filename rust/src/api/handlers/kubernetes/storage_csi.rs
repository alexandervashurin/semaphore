//! Kubernetes CSI resources (optional, read-only)

use axum::{extract::State, Json};
use k8s_openapi::api::storage::v1::{CSIDriver, CSINode, VolumeAttachment};
use kube::api::{Api, ListParams};
use serde::Serialize;
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

#[derive(Debug, Serialize)]
pub struct CsiApiStatus {
    pub csi_driver: bool,
    pub csi_node: bool,
    pub volume_attachment: bool,
}

pub async fn get_csi_api_status(State(state): State<Arc<AppState>>) -> Result<Json<CsiApiStatus>> {
    let client = state.kubernetes_client()?;
    let lp = ListParams::default().limit(1);
    let csi_driver = client.api_all::<CSIDriver>().list(&lp).await.is_ok();
    let csi_node = client.api_all::<CSINode>().list(&lp).await.is_ok();
    let volume_attachment = client.api_all::<VolumeAttachment>().list(&lp).await.is_ok();
    Ok(Json(CsiApiStatus {
        csi_driver,
        csi_node,
        volume_attachment,
    }))
}

pub async fn list_csi_drivers(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let client = state.kubernetes_client()?;
    let api: Api<CSIDriver> = client.api_all();
    let items = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        items.items.iter().map(|x| serde_json::json!(x)).collect(),
    ))
}

pub async fn list_csi_nodes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let client = state.kubernetes_client()?;
    let api: Api<CSINode> = client.api_all();
    let items = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        items.items.iter().map(|x| serde_json::json!(x)).collect(),
    ))
}

pub async fn list_volume_attachments(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let client = state.kubernetes_client()?;
    let api: Api<VolumeAttachment> = client.api_all();
    let items = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        items.items.iter().map(|x| serde_json::json!(x)).collect(),
    ))
}
