//! Kubernetes Services API handlers
//!
//! Handlers для управления Kubernetes Services

use crate::api::handlers::kubernetes::client::KubeClient;
use crate::api::state::AppState;
use crate::error::{Error, Result};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use k8s_openapi::api::core::v1::{Service, ServicePort, ServiceSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Query параметры для list services
#[derive(Debug, Deserialize)]
pub struct ListServicesQuery {
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
    pub limit: Option<i32>,
}

/// Payload для создания/обновления Service
#[derive(Debug, Deserialize)]
pub struct ServicePayload {
    pub name: String,
    pub namespace: String,
    #[serde(default)]
    pub labels: Option<BTreeMap<String, String>>,
    #[serde(default)]
    pub annotations: Option<BTreeMap<String, String>>,
    pub spec: ServiceSpecPayload,
}

#[derive(Debug, Deserialize)]
pub struct ServiceSpecPayload {
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub ports: Vec<ServicePortPayload>,
    pub selector: Option<BTreeMap<String, String>>,
    pub cluster_ip: Option<String>,
    pub external_ips: Option<Vec<String>>,
    pub load_balancer_ip: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ServicePortPayload {
    pub name: Option<String>,
    pub port: i32,
    pub target_port: Option<String>,
    pub protocol: Option<String>,
    pub node_port: Option<i32>,
}

/// Сводка по Service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSummary {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub cluster_ip: String,
    pub external_ips: Vec<String>,
    pub ports: Vec<String>,
    pub selector: BTreeMap<String, String>,
    pub age: String,
    pub load_balancer_ip: Option<String>,
}

/// Список Services
/// GET /api/kubernetes/services
pub async fn list_services(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListServicesQuery>,
) -> Result<Json<Vec<ServiceSummary>>> {
    let client = state.kubernetes_client()?;

    let namespace = query.namespace.as_deref();
    let api: Api<Service> = if let Some(ns) = namespace {
        client.api(Some(ns))
    } else {
        client.api_all()
    };

    let mut list_params = ListParams::default();
    if let Some(selector) = query.label_selector {
        list_params = list_params.labels(&selector);
    }
    if let Some(limit) = query.limit {
        list_params = list_params.limit(limit as u32);
    }

    let services = api
        .list(&list_params)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    let summaries = services
        .items
        .iter()
        .map(|svc| {
            let spec = &svc.spec;
            let meta = &svc.metadata;

            let ports_str: Vec<String> = spec
                .as_ref()
                .and_then(|s| s.ports.as_ref())
                .map(|ports| {
                    ports
                        .iter()
                        .map(|p| {
                            let protocol = p.protocol.as_deref().unwrap_or("TCP");
                            let target = p
                                .target_port
                                .as_ref()
                                .map(|tp| match tp {
                                    IntOrString::Int(i) => i.to_string(),
                                    IntOrString::String(s) => s.clone(),
                                })
                                .unwrap_or_default();
                            format!("{}:{}/{}", p.port, target, protocol)
                        })
                        .collect()
                })
                .unwrap_or_default();

            let external_ips = spec
                .as_ref()
                .and_then(|s| s.external_ips.as_ref())
                .cloned()
                .unwrap_or_default();

            let svc_type = spec
                .as_ref()
                .and_then(|s| s.type_.as_ref())
                .map(|t| t.as_str())
                .unwrap_or("ClusterIP")
                .to_string();

            let cluster_ip = spec
                .as_ref()
                .and_then(|s| s.cluster_ip.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("None")
                .to_string();

            ServiceSummary {
                name: meta.name.clone().unwrap_or_else(|| "unknown".to_string()),
                namespace: meta.namespace.as_deref().unwrap_or("default").to_string(),
                uid: meta.uid.clone().unwrap_or_else(|| "unknown".to_string()),
                type_: svc_type,
                cluster_ip,
                external_ips,
                ports: ports_str,
                selector: spec
                    .as_ref()
                    .and_then(|s| s.selector.clone())
                    .unwrap_or_default(),
                age: meta
                    .creation_timestamp
                    .as_ref()
                    .map(|t| t.0.to_rfc3339())
                    .unwrap_or_else(|| "unknown".to_string()),
                load_balancer_ip: spec.as_ref().and_then(|s| s.load_balancer_ip.clone()),
            }
        })
        .collect();

    Ok(Json(summaries))
}

/// Детали Service
/// GET /api/kubernetes/namespaces/{namespace}/services/{name}
pub async fn get_service(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Service> = client.api(Some(&namespace));

    let svc = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(
        serde_json::to_value(svc).map_err(|e| Error::Kubernetes(e.to_string()))?,
    ))
}

/// Создать Service
/// POST /api/kubernetes/namespaces/{namespace}/services
pub async fn create_service(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(payload): Json<ServicePayload>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Service> = client.api(Some(&namespace));

    let ports: Vec<ServicePort> = payload
        .spec
        .ports
        .iter()
        .map(|p| ServicePort {
            name: p.name.clone(),
            port: p.port,
            target_port: p
                .target_port
                .as_ref()
                .map(|tp| IntOrString::String(tp.clone())),
            protocol: p.protocol.clone(),
            node_port: p.node_port,
            ..Default::default()
        })
        .collect();

    let svc_type = payload.spec.type_.map(|t| {
        match t.as_str() {
            "ClusterIP" => "ClusterIP",
            "NodePort" => "NodePort",
            "LoadBalancer" => "LoadBalancer",
            "ExternalName" => "ExternalName",
            _ => "ClusterIP",
        }
        .to_string()
    });

    let service = Service {
        metadata: ObjectMeta {
            name: Some(payload.name.clone()),
            namespace: Some(namespace),
            labels: payload.labels,
            annotations: payload.annotations,
            ..Default::default()
        },
        spec: Some(ServiceSpec {
            type_: svc_type,
            ports: Some(ports),
            selector: payload.spec.selector,
            cluster_ip: payload.spec.cluster_ip,
            external_ips: payload.spec.external_ips,
            load_balancer_ip: payload.spec.load_balancer_ip,
            ..Default::default()
        }),
        ..Default::default()
    };

    let created = api
        .create(&PostParams::default(), &service)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(
        serde_json::to_value(created).map_err(|e| Error::Kubernetes(e.to_string()))?,
    ))
}

/// Обновить Service
/// PUT /api/kubernetes/namespaces/{namespace}/services/{name}
pub async fn update_service(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<ServicePayload>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Service> = client.api(Some(&namespace));

    let mut svc = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    // Обновляем metadata и spec
    svc.metadata.labels = payload.labels;
    svc.metadata.annotations = payload.annotations;

    let ports: Vec<ServicePort> = payload
        .spec
        .ports
        .iter()
        .map(|p| ServicePort {
            name: p.name.clone(),
            port: p.port,
            target_port: p
                .target_port
                .as_ref()
                .map(|tp| IntOrString::String(tp.clone())),
            protocol: p.protocol.clone(),
            node_port: p.node_port,
            ..Default::default()
        })
        .collect();

    let svc_type = payload.spec.type_.map(|t| {
        match t.as_str() {
            "ClusterIP" => "ClusterIP",
            "NodePort" => "NodePort",
            "LoadBalancer" => "LoadBalancer",
            "ExternalName" => "ExternalName",
            _ => "ClusterIP",
        }
        .to_string()
    });

    if let Some(spec) = svc.spec.as_mut() {
        spec.type_ = svc_type;
        spec.ports = Some(ports);
        spec.selector = payload.spec.selector;
        spec.cluster_ip = payload.spec.cluster_ip;
        spec.external_ips = payload.spec.external_ips;
        spec.load_balancer_ip = payload.spec.load_balancer_ip;
    }

    let updated = api
        .replace(&name, &PostParams::default(), &svc)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(
        serde_json::to_value(updated).map_err(|e| Error::Kubernetes(e.to_string()))?,
    ))
}

/// Удалить Service
/// DELETE /api/kubernetes/namespaces/{namespace}/services/{name}
pub async fn delete_service(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Service> = client.api(Some(&namespace));

    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Service {}/{} deleted", namespace, name)
    })))
}

/// Получить EndpointSlice для Service
/// GET /api/kubernetes/namespaces/{namespace}/services/{name}/endpoints
pub async fn get_service_endpoints(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<Vec<serde_json::Value>>> {
    use k8s_openapi::api::discovery::v1::EndpointSlice;

    let client = state.kubernetes_client()?;
    let api: Api<EndpointSlice> = client.api(Some(&namespace));

    let list_params = ListParams::default().labels(&format!("kubernetes.io/service-name={}", name));

    let slices = api
        .list(&list_params)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    let endpoints = slices
        .items
        .iter()
        .map(|slice| serde_json::to_value(slice).map_err(|e| Error::Kubernetes(e.to_string())))
        .collect::<Result<Vec<_>>>()?;

    Ok(Json(endpoints))
}
