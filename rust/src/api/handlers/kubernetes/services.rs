//! Kubernetes Services API handlers
//!
//! Handlers для управления Kubernetes Services

use crate::api::handlers::kubernetes::client::KubeClient;
use crate::api::state::AppState;
use crate::error::{Error, Result};
use axum::{
    Json,
    extract::{Path, Query, State},
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

#[derive(Debug, Serialize)]
pub struct ServiceBackendsResponse {
    pub source: String,
    pub fallback_used: bool,
    pub items: Vec<serde_json::Value>,
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
                    .map(|t| t.0.to_string())
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

/// Основной endpoint для backend'ов сервиса:
/// EndpointSlice + fallback на legacy Endpoints.
/// GET /api/kubernetes/namespaces/{namespace}/services/{name}/endpoint-slices
pub async fn get_service_endpoint_slices(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<ServiceBackendsResponse>> {
    use k8s_openapi::api::core::v1::Endpoints;
    use k8s_openapi::api::discovery::v1::EndpointSlice;

    let client = state.kubernetes_client()?;
    let slices_api: Api<EndpointSlice> = client.api(Some(&namespace));

    let list_params = ListParams::default().labels(&format!("kubernetes.io/service-name={}", name));

    let slices = slices_api
        .list(&list_params)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    let items = slices
        .items
        .iter()
        .map(|slice| serde_json::to_value(slice).map_err(|e| Error::Kubernetes(e.to_string())))
        .collect::<Result<Vec<_>>>()?;

    if !items.is_empty() {
        return Ok(Json(ServiceBackendsResponse {
            source: "endpointslices".to_string(),
            fallback_used: false,
            items,
        }));
    }

    let endpoints_api: Api<Endpoints> = client.api(Some(&namespace));
    let legacy = endpoints_api
        .get_opt(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    let mut fallback_items = Vec::new();
    if let Some(ep) = legacy {
        fallback_items
            .push(serde_json::to_value(ep).map_err(|e| Error::Kubernetes(e.to_string()))?);
    }
    Ok(Json(ServiceBackendsResponse {
        source: "endpoints".to_string(),
        fallback_used: true,
        items: fallback_items,
    }))
}

/// Legacy endpoints endpoint для отладки/обратной совместимости.
/// GET /api/kubernetes/namespaces/{namespace}/services/{name}/endpoints
pub async fn get_service_endpoints(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<Vec<serde_json::Value>>> {
    use k8s_openapi::api::core::v1::Endpoints;
    let client = state.kubernetes_client()?;
    let api: Api<Endpoints> = client.api(Some(&namespace));
    let mut out = Vec::new();
    if let Some(ep) = api
        .get_opt(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?
    {
        out.push(serde_json::to_value(ep).map_err(|e| Error::Kubernetes(e.to_string()))?);
    }
    Ok(Json(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_port_payload() {
        let port = ServicePortPayload {
            name: Some("http".to_string()),
            port: 80,
            target_port: Some("8080".to_string()),
            protocol: Some("TCP".to_string()),
            node_port: Some(30080),
        };
        assert_eq!(port.name, Some("http".to_string()));
        assert_eq!(port.port, 80);
        assert_eq!(port.node_port, Some(30080));
    }

    #[test]
    fn test_service_summary() {
        let summary = ServiceSummary {
            name: "my-service".to_string(),
            namespace: "default".to_string(),
            uid: "uid-123".to_string(),
            type_: "ClusterIP".to_string(),
            cluster_ip: "10.0.0.1".to_string(),
            external_ips: vec![],
            ports: vec!["80/TCP".to_string()],
            selector: BTreeMap::from([("app".to_string(), "web".to_string())]),
            age: "5d".to_string(),
            load_balancer_ip: None,
        };
        assert_eq!(summary.name, "my-service");
        assert_eq!(summary.type_, "ClusterIP");
    }

    #[test]
    fn test_service_backends_response() {
        let resp = ServiceBackendsResponse {
            source: "endpoints".to_string(),
            fallback_used: false,
            items: vec![],
        };
        assert_eq!(resp.source, "endpoints");
        assert!(!resp.fallback_used);
    }

    #[test]
    fn test_list_services_query() {
        let query = ListServicesQuery {
            namespace: Some("default".to_string()),
            label_selector: Some("app=web".to_string()),
            limit: Some(10),
        };
        assert_eq!(query.namespace, Some("default".to_string()));
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_service_port_payload_minimal() {
        let port = ServicePortPayload {
            name: None,
            port: 443,
            target_port: None,
            protocol: None,
            node_port: None,
        };
        assert_eq!(port.port, 443);
        assert!(port.name.is_none());
        assert!(port.protocol.is_none());
    }

    #[test]
    fn test_service_port_payload_udp() {
        let port = ServicePortPayload {
            name: Some("dns".to_string()),
            port: 53,
            target_port: Some("53".to_string()),
            protocol: Some("UDP".to_string()),
            node_port: None,
        };
        assert_eq!(port.protocol, Some("UDP".to_string()));
        assert_eq!(port.port, 53);
    }

    #[test]
    fn test_service_spec_payload() {
        let spec = ServiceSpecPayload {
            type_: Some("LoadBalancer".to_string()),
            ports: vec![ServicePortPayload {
                name: Some("http".to_string()),
                port: 80,
                target_port: Some("8080".to_string()),
                protocol: Some("TCP".to_string()),
                node_port: None,
            }],
            selector: Some(BTreeMap::from([("app".to_string(), "web".to_string())])),
            cluster_ip: None,
            external_ips: None,
            load_balancer_ip: None,
        };
        assert_eq!(spec.type_, Some("LoadBalancer".to_string()));
        assert_eq!(spec.ports.len(), 1);
        assert!(spec.selector.is_some());
    }

    #[test]
    fn test_service_payload_full() {
        let payload = ServicePayload {
            name: "my-svc".to_string(),
            namespace: "production".to_string(),
            labels: Some(BTreeMap::from([("env".to_string(), "prod".to_string())])),
            annotations: Some(BTreeMap::new()),
            spec: ServiceSpecPayload {
                type_: Some("NodePort".to_string()),
                ports: vec![],
                selector: None,
                cluster_ip: Some("10.0.0.5".to_string()),
                external_ips: Some(vec!["192.168.1.1".to_string()]),
                load_balancer_ip: None,
            },
        };
        assert_eq!(payload.name, "my-svc");
        assert_eq!(payload.namespace, "production");
        assert_eq!(payload.spec.type_, Some("NodePort".to_string()));
    }

    #[test]
    fn test_service_summary_with_load_balancer() {
        let summary = ServiceSummary {
            name: "lb-service".to_string(),
            namespace: "default".to_string(),
            uid: "uid-lb".to_string(),
            type_: "LoadBalancer".to_string(),
            cluster_ip: "10.0.0.10".to_string(),
            external_ips: vec!["203.0.113.1".to_string()],
            ports: vec!["80:8080/TCP".to_string()],
            selector: BTreeMap::new(),
            age: "2d".to_string(),
            load_balancer_ip: Some("203.0.113.1".to_string()),
        };
        assert_eq!(summary.type_, "LoadBalancer");
        assert_eq!(summary.load_balancer_ip, Some("203.0.113.1".to_string()));
    }

    #[test]
    fn test_service_backends_response_with_fallback() {
        let resp = ServiceBackendsResponse {
            source: "endpoints".to_string(),
            fallback_used: true,
            items: vec![serde_json::json!({"kind": "Endpoints"})],
        };
        assert!(resp.fallback_used);
        assert_eq!(resp.items.len(), 1);
    }

    #[test]
    fn test_list_services_query_all_none() {
        let query = ListServicesQuery {
            namespace: None,
            label_selector: None,
            limit: None,
        };
        assert!(query.namespace.is_none());
        assert!(query.label_selector.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_service_summary_empty_selector() {
        let summary = ServiceSummary {
            name: "headless".to_string(),
            namespace: "default".to_string(),
            uid: "uid-headless".to_string(),
            type_: "ClusterIP".to_string(),
            cluster_ip: "None".to_string(),
            external_ips: vec![],
            ports: vec![],
            selector: BTreeMap::new(),
            age: "1h".to_string(),
            load_balancer_ip: None,
        };
        assert_eq!(summary.cluster_ip, "None");
        assert!(summary.selector.is_empty());
    }
}
