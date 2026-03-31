//! Kubernetes Secret API handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use base64::Engine;
use k8s_openapi::api::core::v1::Secret;
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct ListSecretsQuery {
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SecretSummary {
    pub name: String,
    pub namespace: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub keys_count: usize,
}

#[derive(Debug, Serialize)]
pub struct SecretMaskedView {
    pub name: String,
    pub namespace: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub data: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct SecretRevealView {
    pub name: String,
    pub namespace: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub data: BTreeMap<String, String>,
    pub warning: String,
}

fn secret_type(secret: &Secret) -> String {
    secret.type_.clone().unwrap_or_else(|| "Opaque".to_string())
}

fn masked_data(secret: &Secret) -> BTreeMap<String, String> {
    secret
        .data
        .as_ref()
        .map(|m| {
            m.keys()
                .map(|k| (k.clone(), "***".to_string()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default()
}

pub async fn list_secrets(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListSecretsQuery>,
) -> Result<Json<Vec<SecretSummary>>> {
    let client = state.kubernetes_client()?;
    let api: Api<Secret> = if let Some(namespace) = query.namespace.as_deref() {
        client.api(Some(namespace))
    } else {
        client.api_all()
    };

    let mut params = ListParams::default();
    if let Some(selector) = query.label_selector {
        params = params.labels(&selector);
    }
    if let Some(limit) = query.limit {
        params = params.limit(limit);
    }

    let items = api
        .list(&params)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    let result = items
        .items
        .iter()
        .map(|secret| SecretSummary {
            name: secret
                .metadata
                .name
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            namespace: secret
                .metadata
                .namespace
                .clone()
                .unwrap_or_else(|| "default".to_string()),
            type_: secret_type(secret),
            keys_count: secret.data.as_ref().map(|m| m.len()).unwrap_or(0),
        })
        .collect();

    Ok(Json(result))
}

pub async fn get_secret(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<SecretMaskedView>> {
    let client = state.kubernetes_client()?;
    let api: Api<Secret> = client.api(Some(&namespace));

    let secret = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(SecretMaskedView {
        name,
        namespace,
        type_: secret_type(&secret),
        data: masked_data(&secret),
    }))
}

pub async fn reveal_secret(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<SecretRevealView>> {
    let client = state.kubernetes_client()?;
    let api: Api<Secret> = client.api(Some(&namespace));

    let secret = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    let data = secret
        .data
        .as_ref()
        .map(|m| {
            m.iter()
                .map(|(k, v)| {
                    let decoded = String::from_utf8(v.0.clone())
                        .unwrap_or_else(|_| base64::engine::general_purpose::STANDARD.encode(&v.0));
                    (k.clone(), decoded)
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    Ok(Json(SecretRevealView {
        name,
        namespace,
        type_: secret_type(&secret),
        data,
        warning: "Sensitive data disclosed by explicit action. Do not store values in client state longer than session.".to_string(),
    }))
}

pub async fn create_secret(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(payload): Json<Secret>,
) -> Result<Json<SecretSummary>> {
    let client = state.kubernetes_client()?;
    let api: Api<Secret> = client.api(Some(&namespace));

    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(SecretSummary {
        name: created
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        namespace: created
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        type_: secret_type(&created),
        keys_count: created.data.as_ref().map(|m| m.len()).unwrap_or(0),
    }))
}

pub async fn update_secret(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(mut payload): Json<Secret>,
) -> Result<Json<SecretSummary>> {
    let client = state.kubernetes_client()?;
    let api: Api<Secret> = client.api(Some(&namespace));

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

    Ok(Json(SecretSummary {
        name: updated
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        namespace: updated
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        type_: secret_type(&updated),
        keys_count: updated.data.as_ref().map(|m| m.len()).unwrap_or(0),
    }))
}

pub async fn delete_secret(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<BTreeMap<String, String>>> {
    let client = state.kubernetes_client()?;
    let api: Api<Secret> = client.api(Some(&namespace));

    let _ = api
        .delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    let mut response = BTreeMap::new();
    response.insert("status".to_string(), "ok".to_string());
    response.insert(
        "message".to_string(),
        format!("Secret {namespace}/{name} deleted"),
    );
    Ok(Json(response))
}
