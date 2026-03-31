//! Kubernetes Ingress API handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use k8s_openapi::api::networking::v1::{Ingress, IngressClass};
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct ListIngressQuery {
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ListIngressClassQuery {
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IngressPathBackend {
    pub path: String,
    pub path_type: Option<String>,
    pub service_name: Option<String>,
    pub service_port: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IngressRuleView {
    pub host: String,
    pub backends: Vec<IngressPathBackend>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IngressTlsView {
    pub hosts: Vec<String>,
    pub secret_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IngressView {
    pub name: String,
    pub namespace: String,
    pub ingress_class_name: Option<String>,
    pub annotations: BTreeMap<String, String>,
    pub rules: Vec<IngressRuleView>,
    pub tls: Vec<IngressTlsView>,
}

fn parse_service_port(
    port: &k8s_openapi::api::networking::v1::ServiceBackendPort,
) -> Option<String> {
    if let Some(name) = port.name.as_deref() {
        return Some(name.to_string());
    }
    port.number.map(|n| n.to_string())
}

fn to_ingress_view(ingress: &Ingress) -> IngressView {
    let spec = ingress.spec.as_ref();
    let rules = spec
        .and_then(|s| s.rules.as_ref())
        .map(|rules| {
            rules
                .iter()
                .map(|rule| {
                    let host = rule.host.clone().unwrap_or_else(|| "*".to_string());
                    let backends = rule
                        .http
                        .as_ref()
                        .map(|http| &http.paths)
                        .map(|paths| {
                            paths
                                .iter()
                                .map(|p| IngressPathBackend {
                                    path: p.path.clone().unwrap_or_else(|| "/".to_string()),
                                    path_type: Some(p.path_type.clone()),
                                    service_name: p
                                        .backend
                                        .service
                                        .as_ref()
                                        .map(|svc| svc.name.clone()),
                                    service_port: p
                                        .backend
                                        .service
                                        .as_ref()
                                        .and_then(|svc| svc.port.as_ref())
                                        .and_then(parse_service_port),
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    IngressRuleView { host, backends }
                })
                .collect()
        })
        .unwrap_or_default();

    let tls = spec
        .and_then(|s| s.tls.as_ref())
        .map(|items| {
            items
                .iter()
                .map(|t| IngressTlsView {
                    hosts: t.hosts.clone().unwrap_or_default(),
                    secret_name: t.secret_name.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    IngressView {
        name: ingress
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        namespace: ingress
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        ingress_class_name: spec.and_then(|s| s.ingress_class_name.clone()),
        annotations: ingress.metadata.annotations.clone().unwrap_or_default(),
        rules,
        tls,
    }
}

pub async fn list_ingresses(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListIngressQuery>,
) -> Result<Json<Vec<IngressView>>> {
    let client = state.kubernetes_client()?;
    let api: Api<Ingress> = if let Some(namespace) = query.namespace.as_deref() {
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

    let ingresses = api
        .list(&params)
        .await
        .map_err(|e| Error::Other(format!("Kubernetes ingress list failed: {e}")))?;

    Ok(Json(ingresses.items.iter().map(to_ingress_view).collect()))
}

pub async fn get_ingress(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<IngressView>> {
    let client = state.kubernetes_client()?;
    let api: Api<Ingress> = client.api(Some(&namespace));

    let ingress = api
        .get(&name)
        .await
        .map_err(|e| Error::Other(format!("Kubernetes ingress get failed: {e}")))?;

    Ok(Json(to_ingress_view(&ingress)))
}

pub async fn create_ingress(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(payload): Json<Ingress>,
) -> Result<Json<Ingress>> {
    let client = state.kubernetes_client()?;
    let api: Api<Ingress> = client.api(Some(&namespace));

    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Other(format!("Kubernetes ingress create failed: {e}")))?;

    Ok(Json(created))
}

pub async fn update_ingress(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<Ingress>,
) -> Result<Json<Ingress>> {
    let client = state.kubernetes_client()?;
    let api: Api<Ingress> = client.api(Some(&namespace));

    let updated = api
        .replace(&name, &PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Other(format!("Kubernetes ingress update failed: {e}")))?;

    Ok(Json(updated))
}

pub async fn delete_ingress(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Ingress> = client.api(Some(&namespace));

    let _ = api
        .delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Other(format!("Kubernetes ingress delete failed: {e}")))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Ingress {namespace}/{name} deleted")
    })))
}

pub async fn list_ingress_classes(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListIngressClassQuery>,
) -> Result<Json<Vec<IngressClass>>> {
    let client = state.kubernetes_client()?;
    let api: Api<IngressClass> = client.api_all();

    let mut params = ListParams::default();
    if let Some(limit) = query.limit {
        params = params.limit(limit);
    }

    let classes = api
        .list(&params)
        .await
        .map_err(|e| Error::Other(format!("Kubernetes ingressclass list failed: {e}")))?;

    Ok(Json(classes.items))
}

pub async fn get_ingress_class(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<IngressClass>> {
    let client = state.kubernetes_client()?;
    let api: Api<IngressClass> = client.api_all();

    let class = api
        .get(&name)
        .await
        .map_err(|e| Error::Other(format!("Kubernetes ingressclass get failed: {e}")))?;

    Ok(Json(class))
}
