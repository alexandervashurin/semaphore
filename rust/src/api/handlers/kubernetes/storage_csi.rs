//! Kubernetes CSI resources (optional, read-only)

use axum::{Json, extract::State};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csi_api_status_all_true() {
        let status = CsiApiStatus {
            csi_driver: true,
            csi_node: true,
            volume_attachment: true,
        };
        let json_val = serde_json::to_value(&status).unwrap();
        assert_eq!(json_val["csi_driver"], true);
        assert_eq!(json_val["csi_node"], true);
        assert_eq!(json_val["volume_attachment"], true);
    }

    #[test]
    fn test_csi_api_status_all_false() {
        let status = CsiApiStatus {
            csi_driver: false,
            csi_node: false,
            volume_attachment: false,
        };
        let json_val = serde_json::to_value(&status).unwrap();
        assert_eq!(json_val["csi_driver"], false);
        assert_eq!(json_val["csi_node"], false);
        assert_eq!(json_val["volume_attachment"], false);
    }

    #[test]
    fn test_csi_api_status_mixed() {
        let status = CsiApiStatus {
            csi_driver: true,
            csi_node: false,
            volume_attachment: true,
        };
        let json_val = serde_json::to_value(&status).unwrap();
        assert_eq!(json_val["csi_driver"], true);
        assert_eq!(json_val["csi_node"], false);
        assert_eq!(json_val["volume_attachment"], true);
    }

    #[test]
    fn test_csi_api_status_serialize() {
        let status = CsiApiStatus {
            csi_driver: true,
            csi_node: true,
            volume_attachment: false,
        };
        let serialized = serde_json::to_string(&status).unwrap();
        assert!(serialized.contains(r#""csi_driver":true"#));
        assert!(serialized.contains(r#""csi_node":true"#));
        assert!(serialized.contains(r#""volume_attachment":false"#));
    }

    #[test]
    fn test_csi_api_status_debug_format() {
        let status = CsiApiStatus {
            csi_driver: true,
            csi_node: false,
            volume_attachment: true,
        };
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("CsiApiStatus"));
    }

    #[test]
    fn test_csi_driver_type_exists() {
        let _type_check: fn() -> CSIDriver = CSIDriver::default;
    }

    #[test]
    fn test_csi_node_type_exists() {
        let _type_check: fn() -> CSINode = CSINode::default;
    }

    #[test]
    fn test_volume_attachment_type_exists() {
        let _type_check: fn() -> VolumeAttachment = VolumeAttachment::default;
    }

    #[test]
    fn test_csi_api_status_serialize_fields() {
        let original = CsiApiStatus {
            csi_driver: true,
            csi_node: false,
            volume_attachment: true,
        };
        let serialized = serde_json::to_string(&original).unwrap();
        assert!(serialized.contains(r#""csi_driver":true"#));
        assert!(serialized.contains(r#""csi_node":false"#));
        assert!(serialized.contains(r#""volume_attachment":true"#));
    }

    #[test]
    fn test_csi_api_status_field_ordering() {
        let status = CsiApiStatus {
            csi_driver: true,
            csi_node: false,
            volume_attachment: true,
        };
        let serialized = serde_json::to_string(&status).unwrap();
        let csi_driver_pos = serialized.find("csi_driver").unwrap();
        let csi_node_pos = serialized.find("csi_node").unwrap();
        let volume_attachment_pos = serialized.find("volume_attachment").unwrap();
        assert!(csi_driver_pos < csi_node_pos);
        assert!(csi_node_pos < volume_attachment_pos);
    }
}
