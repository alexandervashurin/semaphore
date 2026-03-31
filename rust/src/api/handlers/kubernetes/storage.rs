//! Kubernetes Storage API handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use k8s_openapi::api::core::v1::{PersistentVolume, PersistentVolumeClaim};
use k8s_openapi::api::storage::v1::StorageClass;
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct StorageListQuery {
    pub namespace: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct PersistentVolumeSummary {
    pub name: String,
    pub status: String,
    pub capacity: Option<String>,
    pub access_modes: Vec<String>,
    pub reclaim_policy: Option<String>,
    pub storage_class_name: Option<String>,
    pub claim_ref: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PersistentVolumeClaimSummary {
    pub name: String,
    pub namespace: String,
    pub phase: String,
    pub capacity: Option<String>,
    pub access_modes: Vec<String>,
    pub volume_name: Option<String>,
    pub storage_class_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StorageClassSummary {
    pub name: String,
    pub provisioner: String,
    pub reclaim_policy: Option<String>,
    pub volume_binding_mode: Option<String>,
    pub allow_volume_expansion: Option<bool>,
    pub is_default: bool,
}

fn pv_summary(pv: &PersistentVolume) -> PersistentVolumeSummary {
    let spec = pv.spec.as_ref();
    let status = pv
        .status
        .as_ref()
        .and_then(|s| s.phase.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    let claim_ref = spec.and_then(|s| s.claim_ref.as_ref()).map(|c| {
        format!(
            "{}/{}",
            c.namespace.clone().unwrap_or_default(),
            c.name.clone().unwrap_or_default()
        )
    });
    PersistentVolumeSummary {
        name: pv
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        status,
        capacity: spec
            .and_then(|s| s.capacity.as_ref())
            .and_then(|m| m.get("storage"))
            .map(|q| q.0.clone()),
        access_modes: spec
            .and_then(|s| s.access_modes.clone())
            .unwrap_or_default(),
        reclaim_policy: spec.and_then(|s| s.persistent_volume_reclaim_policy.clone()),
        storage_class_name: spec.and_then(|s| s.storage_class_name.clone()),
        claim_ref,
    }
}

fn pvc_summary(pvc: &PersistentVolumeClaim) -> PersistentVolumeClaimSummary {
    let spec = pvc.spec.as_ref();
    let status = pvc.status.as_ref();
    PersistentVolumeClaimSummary {
        name: pvc
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        namespace: pvc
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        phase: status
            .and_then(|s| s.phase.clone())
            .unwrap_or_else(|| "Unknown".to_string()),
        capacity: status
            .and_then(|s| s.capacity.as_ref())
            .and_then(|m| m.get("storage"))
            .map(|q| q.0.clone()),
        access_modes: spec
            .and_then(|s| s.access_modes.clone())
            .unwrap_or_default(),
        volume_name: spec.and_then(|s| s.volume_name.clone()),
        storage_class_name: spec.and_then(|s| s.storage_class_name.clone()),
    }
}

fn sc_summary(sc: &StorageClass) -> StorageClassSummary {
    let is_default = sc
        .metadata
        .annotations
        .as_ref()
        .and_then(|a| a.get("storageclass.kubernetes.io/is-default-class"))
        .map(|v| v == "true")
        .unwrap_or(false);
    StorageClassSummary {
        name: sc
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        provisioner: sc.provisioner.clone(),
        reclaim_policy: sc.reclaim_policy.clone(),
        volume_binding_mode: sc.volume_binding_mode.clone(),
        allow_volume_expansion: sc.allow_volume_expansion,
        is_default,
    }
}

pub async fn list_persistent_volumes(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StorageListQuery>,
) -> Result<Json<Vec<PersistentVolumeSummary>>> {
    let client = state.kubernetes_client()?;
    let api: Api<PersistentVolume> = client.api_all();
    let mut lp = ListParams::default();
    if let Some(limit) = query.limit {
        lp = lp.limit(limit);
    }
    let list = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(list.items.iter().map(pv_summary).collect()))
}

pub async fn get_persistent_volume(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<PersistentVolume>> {
    let client = state.kubernetes_client()?;
    let api: Api<PersistentVolume> = client.api_all();
    let item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(item))
}

pub async fn create_persistent_volume(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PersistentVolume>,
) -> Result<Json<PersistentVolumeSummary>> {
    let client = state.kubernetes_client()?;
    let api: Api<PersistentVolume> = client.api_all();
    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(pv_summary(&created)))
}

pub async fn delete_persistent_volume(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<PersistentVolume> = client.api_all();
    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        serde_json::json!({"status":"ok","message":format!("PersistentVolume {} deleted", name)}),
    ))
}

pub async fn list_persistent_volume_claims(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StorageListQuery>,
) -> Result<Json<Vec<PersistentVolumeClaimSummary>>> {
    let client = state.kubernetes_client()?;
    let ns = query.namespace.unwrap_or_else(|| "default".to_string());
    let api: Api<PersistentVolumeClaim> = client.api(Some(&ns));
    let mut lp = ListParams::default();
    if let Some(limit) = query.limit {
        lp = lp.limit(limit);
    }
    let list = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(list.items.iter().map(pvc_summary).collect()))
}

pub async fn get_persistent_volume_claim(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<PersistentVolumeClaim>> {
    let client = state.kubernetes_client()?;
    let api: Api<PersistentVolumeClaim> = client.api(Some(&namespace));
    let item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(item))
}

pub async fn create_persistent_volume_claim(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(payload): Json<PersistentVolumeClaim>,
) -> Result<Json<PersistentVolumeClaimSummary>> {
    let client = state.kubernetes_client()?;
    let api: Api<PersistentVolumeClaim> = client.api(Some(&namespace));
    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(pvc_summary(&created)))
}

pub async fn update_persistent_volume_claim(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(mut payload): Json<PersistentVolumeClaim>,
) -> Result<Json<PersistentVolumeClaimSummary>> {
    let client = state.kubernetes_client()?;
    let api: Api<PersistentVolumeClaim> = client.api(Some(&namespace));
    if payload.metadata.name.is_none() {
        payload.metadata.name = Some(name.clone());
    }
    if payload.metadata.namespace.is_none() {
        payload.metadata.namespace = Some(namespace);
    }
    let updated = api
        .replace(&name, &PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(pvc_summary(&updated)))
}

pub async fn delete_persistent_volume_claim(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<PersistentVolumeClaim> = client.api(Some(&namespace));
    let current = api.get(&name).await.ok();
    let bound = current
        .as_ref()
        .and_then(|p| p.status.as_ref())
        .and_then(|s| s.phase.as_deref())
        == Some("Bound");
    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(serde_json::json!({
        "status":"ok",
        "message":format!("PersistentVolumeClaim {}/{} deleted", namespace, name),
        "warning": if bound { Some("PVC was Bound. Actual data deletion depends on PV reclaimPolicy/CSI.") } else { None }
    })))
}

pub async fn list_storage_classes(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StorageListQuery>,
) -> Result<Json<Vec<StorageClassSummary>>> {
    let client = state.kubernetes_client()?;
    let api: Api<StorageClass> = client.api_all();
    let mut lp = ListParams::default();
    if let Some(limit) = query.limit {
        lp = lp.limit(limit);
    }
    let list = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(list.items.iter().map(sc_summary).collect()))
}

pub async fn get_storage_class(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<StorageClass>> {
    let client = state.kubernetes_client()?;
    let api: Api<StorageClass> = client.api_all();
    let item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(item))
}

pub async fn create_storage_class(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<StorageClass>,
) -> Result<Json<StorageClassSummary>> {
    let client = state.kubernetes_client()?;
    let api: Api<StorageClass> = client.api_all();
    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(sc_summary(&created)))
}

pub async fn delete_storage_class(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<StorageClass> = client.api_all();
    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        serde_json::json!({"status":"ok","message":format!("StorageClass {} deleted", name)}),
    ))
}
