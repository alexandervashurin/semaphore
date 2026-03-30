//! Kubernetes Observability handlers — Phase 8
//!
//! Metrics API (node/pod), cluster-wide events stream, topology

use axum::{
    extract::{Path, Query, State},
    Json,
    http,
};
use std::sync::Arc;
use std::collections::BTreeMap;
use crate::api::state::AppState;
use crate::error::{Error, Result};
use crate::api::handlers::kubernetes::client::KubeClient;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Pod, Service};
use kube::api::{Api, ListParams};
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────

/// Статус metrics-server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsStatus {
    pub available: bool,
    pub message: String,
}

/// Метрики узла
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub name: String,
    pub cpu_usage: String,
    pub memory_usage: String,
    pub cpu_usage_cores: f64,
    pub memory_usage_bytes: i64,
}

/// Метрики Pod'а
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodMetrics {
    pub name: String,
    pub namespace: String,
    pub containers: Vec<ContainerMetrics>,
    pub cpu_total: f64,
    pub memory_total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerMetrics {
    pub name: String,
    pub cpu_usage: String,
    pub memory_usage: String,
}

/// Узел топологии
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyNode {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub labels: BTreeMap<String, String>,
}

/// Ребро топологии
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyEdge {
    pub source: String,
    pub target: String,
    pub label: String,
}

/// Граф топологии
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyGraph {
    pub nodes: Vec<TopologyNode>,
    pub edges: Vec<TopologyEdge>,
}

#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    pub namespace: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct TopologyQuery {
    pub namespace: Option<String>,
}

// ─────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────

/// Parse Kubernetes quantity string to millicores (for CPU) or bytes (for memory)
fn parse_cpu_to_cores(s: &str) -> f64 {
    if let Some(m) = s.strip_suffix('m') {
        m.parse::<f64>().unwrap_or(0.0) / 1000.0
    } else {
        s.parse::<f64>().unwrap_or(0.0)
    }
}

fn parse_memory_to_bytes(s: &str) -> i64 {
    if let Some(v) = s.strip_suffix("Ki") {
        v.parse::<i64>().unwrap_or(0) * 1024
    } else if let Some(v) = s.strip_suffix("Mi") {
        v.parse::<i64>().unwrap_or(0) * 1024 * 1024
    } else if let Some(v) = s.strip_suffix("Gi") {
        v.parse::<i64>().unwrap_or(0) * 1024 * 1024 * 1024
    } else if let Some(v) = s.strip_suffix('k') {
        v.parse::<i64>().unwrap_or(0) * 1000
    } else if let Some(v) = s.strip_suffix('M') {
        v.parse::<i64>().unwrap_or(0) * 1_000_000
    } else if let Some(v) = s.strip_suffix('G') {
        v.parse::<i64>().unwrap_or(0) * 1_000_000_000
    } else {
        s.parse::<i64>().unwrap_or(0)
    }
}

fn format_bytes(b: i64) -> String {
    if b >= 1024 * 1024 * 1024 {
        format!("{:.1}Gi", b as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if b >= 1024 * 1024 {
        format!("{:.1}Mi", b as f64 / (1024.0 * 1024.0))
    } else if b >= 1024 {
        format!("{:.1}Ki", b as f64 / 1024.0)
    } else {
        format!("{b}B")
    }
}

// ─────────────────────────────────────────────
// Metrics-server status
// ─────────────────────────────────────────────

/// GET /api/kubernetes/metrics/status
/// Проверяет, доступен ли metrics-server
pub async fn get_metrics_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<MetricsStatus>> {
    let client = state.kubernetes_client()?;
    let raw = client.raw();

    // Пробуем достучаться до /apis/metrics.k8s.io/v1beta1
    match raw.request_text(
        http::Request::builder()
            .uri("/apis/metrics.k8s.io/v1beta1")
            .body(Vec::<u8>::new())
            .map_err(|e| Error::Kubernetes(e.to_string()))?,
    ).await {
        Ok(_) => Ok(Json(MetricsStatus {
            available: true,
            message: "metrics-server is available".to_string(),
        })),
        Err(e) => Ok(Json(MetricsStatus {
            available: false,
            message: format!("metrics-server not available: {e}"),
        })),
    }
}

// ─────────────────────────────────────────────
// Node metrics
// ─────────────────────────────────────────────

/// GET /api/kubernetes/metrics/nodes
pub async fn get_node_metrics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<NodeMetrics>>> {
    let client = state.kubernetes_client()?;
    let raw = client.raw();

    let resp = raw.request_text(
        http::Request::builder()
            .uri("/apis/metrics.k8s.io/v1beta1/nodes")
            .body(Vec::<u8>::new())
            .map_err(|e| Error::Kubernetes(e.to_string()))?,
    ).await.map_err(|e| Error::Kubernetes(format!("metrics-server unavailable: {e}")))?;

    let json: serde_json::Value = serde_json::from_str(&resp)
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    let items = json["items"].as_array().cloned().unwrap_or_default();
    let metrics = items.iter().map(|item| {
        let name = item["metadata"]["name"].as_str().unwrap_or("unknown").to_string();
        let cpu_str = item["usage"]["cpu"].as_str().unwrap_or("0m").to_string();
        let mem_str = item["usage"]["memory"].as_str().unwrap_or("0Ki").to_string();
        let cpu_cores = parse_cpu_to_cores(&cpu_str);
        let mem_bytes = parse_memory_to_bytes(&mem_str);
        NodeMetrics {
            name,
            cpu_usage: if cpu_str.ends_with('m') {
                cpu_str.clone()
            } else {
                format!("{:.0}m", cpu_cores * 1000.0)
            },
            memory_usage: format_bytes(mem_bytes),
            cpu_usage_cores: cpu_cores,
            memory_usage_bytes: mem_bytes,
        }
    }).collect();

    Ok(Json(metrics))
}

// ─────────────────────────────────────────────
// Pod metrics
// ─────────────────────────────────────────────

/// GET /api/kubernetes/metrics/pods
pub async fn get_pod_metrics(
    State(state): State<Arc<AppState>>,
    Query(query): Query<MetricsQuery>,
) -> Result<Json<Vec<PodMetrics>>> {
    let client = state.kubernetes_client()?;
    let raw = client.raw();

    let uri = if let Some(ns) = &query.namespace {
        format!("/apis/metrics.k8s.io/v1beta1/namespaces/{ns}/pods")
    } else {
        "/apis/metrics.k8s.io/v1beta1/pods".to_string()
    };

    let resp = raw.request_text(
        http::Request::builder()
            .uri(uri.as_str())
            .body(Vec::<u8>::new())
            .map_err(|e| Error::Kubernetes(e.to_string()))?,
    ).await.map_err(|e| Error::Kubernetes(format!("metrics-server unavailable: {e}")))?;

    let json: serde_json::Value = serde_json::from_str(&resp)
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    let items = json["items"].as_array().cloned().unwrap_or_default();
    let limit = query.limit.unwrap_or(100) as usize;

    let metrics = items.iter().take(limit).map(|item| {
        let name = item["metadata"]["name"].as_str().unwrap_or("unknown").to_string();
        let namespace = item["metadata"]["namespace"].as_str().unwrap_or("default").to_string();
        let containers_raw = item["containers"].as_array().cloned().unwrap_or_default();

        let containers: Vec<ContainerMetrics> = containers_raw.iter().map(|c| {
            let cname = c["name"].as_str().unwrap_or("unknown").to_string();
            let cpu = c["usage"]["cpu"].as_str().unwrap_or("0m").to_string();
            let mem = c["usage"]["memory"].as_str().unwrap_or("0Ki").to_string();
            ContainerMetrics {
                name: cname,
                cpu_usage: cpu,
                memory_usage: format_bytes(parse_memory_to_bytes(&mem)),
            }
        }).collect();

        let cpu_total: f64 = containers_raw.iter()
            .map(|c| parse_cpu_to_cores(c["usage"]["cpu"].as_str().unwrap_or("0m")))
            .sum();
        let memory_total: i64 = containers_raw.iter()
            .map(|c| parse_memory_to_bytes(c["usage"]["memory"].as_str().unwrap_or("0Ki")))
            .sum();

        PodMetrics { name, namespace, containers, cpu_total, memory_total }
    }).collect();

    Ok(Json(metrics))
}

// ─────────────────────────────────────────────
// Topology
// ─────────────────────────────────────────────

/// GET /api/kubernetes/topology
/// Строит упрощённый граф: Services → Deployments → Pods
pub async fn get_topology(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TopologyQuery>,
) -> Result<Json<TopologyGraph>> {
    let client = state.kubernetes_client()?;
    let ns = query.namespace.as_deref();
    let lp = ListParams::default().limit(50);

    // Fetch Services
    let svc_api: Api<Service> = match ns {
        Some(n) => client.api(Some(n)),
        None => client.api_all(),
    };
    let services = svc_api.list(&lp).await.map_err(|e| Error::Kubernetes(e.to_string()))?;

    // Fetch Deployments
    let dep_api: Api<Deployment> = match ns {
        Some(n) => client.api(Some(n)),
        None => client.api_all(),
    };
    let deployments = dep_api.list(&lp).await.map_err(|e| Error::Kubernetes(e.to_string()))?;

    // Fetch Pods
    let pod_api: Api<Pod> = match ns {
        Some(n) => client.api(Some(n)),
        None => client.api_all(),
    };
    let pods = pod_api.list(&lp).await.map_err(|e| Error::Kubernetes(e.to_string()))?;

    let mut nodes: Vec<TopologyNode> = Vec::new();
    let mut edges: Vec<TopologyEdge> = Vec::new();

    // Add service nodes
    for svc in &services.items {
        let meta = &svc.metadata;
        let name = meta.name.clone().unwrap_or_default();
        let ns_name = meta.namespace.clone().unwrap_or_default();
        let id = format!("svc/{ns_name}/{name}");
        nodes.push(TopologyNode {
            id: id.clone(),
            kind: "Service".to_string(),
            name: name.clone(),
            namespace: ns_name.clone(),
            status: "Active".to_string(),
            labels: meta.labels.clone().unwrap_or_default(),
        });
    }

    // Add deployment nodes + edges to matching services
    for dep in &deployments.items {
        let meta = &dep.metadata;
        let name = meta.name.clone().unwrap_or_default();
        let ns_name = meta.namespace.clone().unwrap_or_default();
        let id = format!("deploy/{ns_name}/{name}");
        let dep_labels = dep.spec.as_ref()
            .and_then(|s| s.selector.match_labels.as_ref())
            .cloned()
            .unwrap_or_default();

        let ready = dep.status.as_ref().and_then(|s| s.ready_replicas).unwrap_or(0);
        let desired = dep.spec.as_ref().and_then(|s| s.replicas).unwrap_or(0);
        let status = if ready >= desired && desired > 0 { "Ready" } else { "NotReady" };

        nodes.push(TopologyNode {
            id: id.clone(),
            kind: "Deployment".to_string(),
            name: name.clone(),
            namespace: ns_name.clone(),
            status: status.to_string(),
            labels: meta.labels.clone().unwrap_or_default(),
        });

        // Connect service → deployment if selector matches dep labels
        for svc in &services.items {
            let svc_meta = &svc.metadata;
            let svc_ns = svc_meta.namespace.as_deref().unwrap_or_default();
            if svc_ns != ns_name { continue; }
            let svc_selector = svc.spec.as_ref()
                .and_then(|s| s.selector.as_ref())
                .cloned()
                .unwrap_or_default();
            // Match: all service selector labels present in dep labels
            if !svc_selector.is_empty() && svc_selector.iter().all(|(k, v)| dep_labels.get(k) == Some(v)) {
                let svc_name = svc_meta.name.clone().unwrap_or_default();
                edges.push(TopologyEdge {
                    source: format!("svc/{svc_ns}/{svc_name}"),
                    target: id.clone(),
                    label: "selects".to_string(),
                });
            }
        }
    }

    // Add pod nodes + edges from their owner (deployment via replicaset ownerRef)
    for pod in &pods.items {
        let meta = &pod.metadata;
        let name = meta.name.clone().unwrap_or_default();
        let ns_name = meta.namespace.clone().unwrap_or_default();
        let id = format!("pod/{ns_name}/{name}");

        let phase = pod.status.as_ref()
            .and_then(|s| s.phase.as_deref())
            .unwrap_or("Unknown");

        let pod_labels = meta.labels.clone().unwrap_or_default();

        nodes.push(TopologyNode {
            id: id.clone(),
            kind: "Pod".to_string(),
            name: name.clone(),
            namespace: ns_name.clone(),
            status: phase.to_string(),
            labels: pod_labels.clone(),
        });

        // Connect deployment → pod if pod labels match deployment selector
        for dep in &deployments.items {
            let dep_ns = dep.metadata.namespace.as_deref().unwrap_or_default();
            if dep_ns != ns_name { continue; }
            let dep_selector = dep.spec.as_ref()
                .and_then(|s| s.selector.match_labels.as_ref())
                .cloned()
                .unwrap_or_default();
            if !dep_selector.is_empty() && dep_selector.iter().all(|(k, v)| pod_labels.get(k) == Some(v)) {
                let dep_name = dep.metadata.name.clone().unwrap_or_default();
                edges.push(TopologyEdge {
                    source: format!("deploy/{dep_ns}/{dep_name}"),
                    target: id.clone(),
                    label: "manages".to_string(),
                });
            }
        }
    }

    Ok(Json(TopologyGraph { nodes, edges }))
}

/// GET /api/kubernetes/topology/namespaces/{namespace}
pub async fn get_namespace_topology(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
) -> Result<Json<TopologyGraph>> {
    get_topology(
        State(state),
        Query(TopologyQuery { namespace: Some(namespace) }),
    ).await
}
