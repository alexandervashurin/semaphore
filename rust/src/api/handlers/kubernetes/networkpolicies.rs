//! Kubernetes NetworkPolicy API handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use k8s_openapi::api::networking::v1::NetworkPolicy;
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct ListNetworkPoliciesQuery {
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct NetworkPolicySummary {
    pub name: String,
    pub namespace: String,
    pub policy_types: Vec<String>,
    pub ingress_rules: usize,
    pub egress_rules: usize,
}

#[derive(Debug, Serialize)]
pub struct NetworkPolicyView {
    pub name: String,
    pub namespace: String,
    pub policy_types: Vec<String>,
    pub ingress_rules: usize,
    pub egress_rules: usize,
    pub note: String,
}

fn to_summary(np: &NetworkPolicy) -> NetworkPolicySummary {
    let spec = np.spec.as_ref();
    NetworkPolicySummary {
        name: np
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        namespace: np
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        policy_types: spec
            .and_then(|s| s.policy_types.clone())
            .unwrap_or_default(),
        ingress_rules: spec
            .and_then(|s| s.ingress.as_ref().map(|r| r.len()))
            .unwrap_or(0),
        egress_rules: spec
            .and_then(|s| s.egress.as_ref().map(|r| r.len()))
            .unwrap_or(0),
    }
}

fn to_view(np: &NetworkPolicy) -> NetworkPolicyView {
    let summary = to_summary(np);
    NetworkPolicyView {
        name: summary.name,
        namespace: summary.namespace,
        policy_types: summary.policy_types,
        ingress_rules: summary.ingress_rules,
        egress_rules: summary.egress_rules,
        note: "NetworkPolicy effect depends on CNI implementation and cluster networking setup."
            .to_string(),
    }
}

pub async fn list_networkpolicies(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListNetworkPoliciesQuery>,
) -> Result<Json<Vec<NetworkPolicySummary>>> {
    let client = state.kubernetes_client()?;
    let api: Api<NetworkPolicy> = if let Some(namespace) = query.namespace.as_deref() {
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

    Ok(Json(items.items.iter().map(to_summary).collect()))
}

pub async fn get_networkpolicy(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<NetworkPolicyView>> {
    let client = state.kubernetes_client()?;
    let api: Api<NetworkPolicy> = client.api(Some(&namespace));

    let item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(to_view(&item)))
}

pub async fn create_networkpolicy(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(payload): Json<NetworkPolicy>,
) -> Result<Json<NetworkPolicySummary>> {
    let client = state.kubernetes_client()?;
    let api: Api<NetworkPolicy> = client.api(Some(&namespace));

    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(to_summary(&created)))
}

pub async fn update_networkpolicy(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(mut payload): Json<NetworkPolicy>,
) -> Result<Json<NetworkPolicySummary>> {
    let client = state.kubernetes_client()?;
    let api: Api<NetworkPolicy> = client.api(Some(&namespace));

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

    Ok(Json(to_summary(&updated)))
}

pub async fn delete_networkpolicy(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<NetworkPolicy> = client.api(Some(&namespace));

    let _ = api
        .delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "status": "ok",
        "message": format!("NetworkPolicy {namespace}/{name} deleted")
    })))
}
