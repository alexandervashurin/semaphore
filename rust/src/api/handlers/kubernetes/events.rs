//! Kubernetes Events API handlers
//!
//! Events, Metrics, Topology visualization

use axum::{
    extract::{Path, Query, State},
    Json,
};
use k8s_openapi::api::core::v1::Event;
use k8s_openapi::api::apps::v1::{Deployment, ReplicaSet};
use k8s_openapi::api::core::v1::{Pod, Service};
use kube::{api::{Api, ListParams}, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

// ============================================================================
// Events
// ============================================================================

#[derive(Debug, Serialize)]
pub struct KubernetesEvent {
    pub name: String,
    pub namespace: String,
    pub type_: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
    pub involved_object: InvolvedObjectSummary,
}

#[derive(Debug, Serialize)]
pub struct InvolvedObjectSummary {
    pub kind: String,
    pub name: String,
    pub api_version: Option<String>,
    pub uid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub namespace: Option<String>,
    pub limit: Option<i32>,
    pub field_selector: Option<String>,
}

pub async fn list_events(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EventsQuery>,
) -> Result<Json<Vec<KubernetesEvent>>> {
    let client = state.kubernetes_client()?;
    let ns = query.namespace.unwrap_or_else(|| "default".to_string());
    
    let mut lp = ListParams {
        limit: Some(query.limit.unwrap_or(100) as u32),
        ..Default::default()
    };
    
    if let Some(selector) = query.field_selector {
        lp.field_selector = Some(selector);
    }
    
    let api: Api<Event> = Api::namespaced(client.raw().clone(), &ns);
    let event_list = api.list(&lp).await
        .map_err(|e| Error::Kubernetes(format!("Failed to list events: {}", e)))?;
    
    let events = event_list.items.iter().map(|e| {
        let involved = &e.involved_object;
        KubernetesEvent {
            name: e.metadata.name.clone().unwrap_or_default(),
            namespace: e.metadata.namespace.clone().unwrap_or_default(),
            type_: e.type_.clone().unwrap_or_default(),
            reason: e.reason.clone().unwrap_or_default(),
            message: e.message.clone().unwrap_or_default(),
            count: e.count.unwrap_or(1),
            first_seen: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
            last_seen: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
            involved_object: InvolvedObjectSummary {
                kind: involved.kind.clone().unwrap_or_default(),
                name: involved.name.clone().unwrap_or_default(),
                api_version: involved.api_version.clone(),
                uid: involved.uid.clone(),
            },
        }
    }).collect();
    
    Ok(Json(events))
}

pub async fn get_event(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<Event>> {
    let client = state.kubernetes_client()?;
    let api: Api<Event> = Api::namespaced(client.raw().clone(), &namespace);
    
    let event = api.get(&name).await
        .map_err(|e| Error::NotFound(format!("Event {} not found: {}", name, e)))?;
    
    Ok(Json(event))
}

// ============================================================================
// Metrics
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct PodMetrics {
    pub name: String,
    pub namespace: String,
    pub containers: Vec<ContainerMetrics>,
    pub timestamp: String,
    pub window: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerMetrics {
    pub name: String,
    pub usage: ResourceUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu: String,
    pub memory: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub name: String,
    pub usage: ResourceUsage,
    pub timestamp: String,
    pub window: String,
}

#[derive(Debug, Serialize)]
pub struct TopPods {
    pub pods: Vec<TopPodEntry>,
}

#[derive(Debug, Serialize)]
pub struct TopPodEntry {
    pub name: String,
    pub namespace: String,
    pub cpu_usage: String,
    pub memory_usage: String,
}

pub async fn get_pod_metrics(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<PodMetrics>> {
    let client = state.kubernetes_client()?;
    
    // Metrics API is separate from main Kubernetes API
    // Try to access metrics.k8s.io API group using dynamic API
    let gvk = kube::core::GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "PodMetrics");
    let api_resource = kube::core::ApiResource::from_gvk(&gvk);
    let api: kube::Api<kube::core::DynamicObject> = kube::Api::namespaced_with(client.raw().clone(), &namespace, &api_resource);
    
    let obj = api.get(&name).await
        .map_err(|e| Error::NotFound(format!("Pod metrics not available: {}. Ensure metrics-server is installed.", e)))?;
    
    // Convert to PodMetrics
    let metrics = PodMetrics {
        name: obj.metadata.name.unwrap_or_default(),
        namespace: obj.metadata.namespace.unwrap_or_default(),
        containers: vec![],
        timestamp: String::new(),
        window: String::new(),
    };
    
    Ok(Json(metrics))
}

pub async fn list_pod_metrics(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EventsQuery>,
) -> Result<Json<Vec<PodMetrics>>> {
    let client = state.kubernetes_client()?;
    let ns = query.namespace.unwrap_or_else(|| "default".to_string());
    
    let gvk = kube::core::GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "PodMetrics");
    let api_resource = kube::core::ApiResource::from_gvk(&gvk);
    let api: kube::Api<kube::core::DynamicObject> = kube::Api::namespaced_with(client.raw().clone(), &ns, &api_resource);
    
    let list = api.list(&Default::default()).await
        .map_err(|e| Error::NotFound(format!("Pod metrics not available: {}", e)))?;
    
    let items: Vec<PodMetrics> = list.items.iter().map(|obj| PodMetrics {
        name: obj.metadata.name.clone().unwrap_or_default(),
        namespace: obj.metadata.namespace.clone().unwrap_or_default(),
        containers: vec![],
        timestamp: String::new(),
        window: String::new(),
    }).collect();
    
    Ok(Json(items))
}

pub async fn get_node_metrics(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<NodeMetrics>> {
    let client = state.kubernetes_client()?;
    
    let gvk = kube::core::GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "NodeMetrics");
    let api_resource = kube::core::ApiResource::from_gvk(&gvk);
    let api: kube::Api<kube::core::DynamicObject> = kube::Api::all_with(client.raw().clone(), &api_resource);
    
    let obj = api.get(&name).await
        .map_err(|e| Error::NotFound(format!("Node metrics not available: {}", e)))?;
    
    let metrics = NodeMetrics {
        name: obj.metadata.name.unwrap_or_default(),
        usage: ResourceUsage { cpu: String::new(), memory: String::new() },
        timestamp: String::new(),
        window: String::new(),
    };
    
    Ok(Json(metrics))
}

pub async fn list_node_metrics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<NodeMetrics>>> {
    let client = state.kubernetes_client()?;
    
    let gvk = kube::core::GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "NodeMetrics");
    let api_resource = kube::core::ApiResource::from_gvk(&gvk);
    let api: kube::Api<kube::core::DynamicObject> = kube::Api::all_with(client.raw().clone(), &api_resource);
    
    let list = api.list(&Default::default()).await
        .map_err(|e| Error::NotFound(format!("Node metrics not available: {}", e)))?;
    
    let items: Vec<NodeMetrics> = list.items.iter().map(|obj| NodeMetrics {
        name: obj.metadata.name.clone().unwrap_or_default(),
        usage: ResourceUsage { cpu: String::new(), memory: String::new() },
        timestamp: String::new(),
        window: String::new(),
    }).collect();
    
    Ok(Json(items))
}

pub async fn get_top_pods(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EventsQuery>,
) -> Result<Json<TopPods>> {
    // For now, return empty list - real implementation requires metrics-server
    Ok(Json(TopPods { pods: vec![] }))
}

pub async fn get_top_nodes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<NodeMetrics>>> {
    // For now, return empty list - real implementation requires metrics-server
    Ok(Json(vec![]))
}

// ============================================================================
// Topology Visualization
// ============================================================================

#[derive(Debug, Serialize)]
pub struct TopologyData {
    pub namespace: String,
    pub nodes: Vec<TopologyNode>,
    pub edges: Vec<TopologyEdge>,
}

#[derive(Debug, Serialize)]
pub struct TopologyNode {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub replicas: Option<TopologyReplicas>,
    pub labels: Option<std::collections::BTreeMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct TopologyReplicas {
    pub desired: i32,
    pub ready: i32,
}

#[derive(Debug, Serialize)]
pub struct TopologyEdge {
    pub source: String,
    pub target: String,
    pub kind: String,
}

#[derive(Debug, Deserialize)]
pub struct TopologyQuery {
    pub namespace: Option<String>,
}

pub async fn get_topology(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TopologyQuery>,
) -> Result<Json<TopologyData>> {
    let client = state.kubernetes_client()?;
    let ns = query.namespace.unwrap_or_else(|| "default".to_string());
    
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    
    // Load Deployments
    let deployments_api: Api<Deployment> = Api::namespaced(client.raw().clone(), &ns);
    let deployments = deployments_api.list(&Default::default()).await.ok();
    
    if let Some(dep_list) = deployments {
        for dep in dep_list.items {
            let node_id = format!("deployment/{}", dep.metadata.name.as_ref().unwrap_or(&String::new()));
            
            let spec = dep.spec.as_ref();
            let status = dep.status.as_ref();
            let desired = spec.and_then(|s| s.replicas).unwrap_or(1);
            let ready = status.and_then(|s| s.ready_replicas).unwrap_or(0);
            
            let replicas = TopologyReplicas {
                desired,
                ready,
            };
            
            nodes.push(TopologyNode {
                id: node_id.clone(),
                kind: "Deployment".to_string(),
                name: dep.metadata.name.clone().unwrap_or_default(),
                namespace: dep.metadata.namespace.clone().unwrap_or_default(),
                status: get_deployment_status(&dep),
                replicas: Some(replicas),
                labels: dep.metadata.labels.clone(),
            });
        }
    }
    
    // Load ReplicaSets
    let rs_api: Api<ReplicaSet> = Api::namespaced(client.raw().clone(), &ns);
    let replica_sets = rs_api.list(&Default::default()).await.ok();
    
    if let Some(rs_list) = replica_sets {
        for rs in rs_list.items {
            let node_id = format!("replicaset/{}", rs.metadata.name.as_ref().unwrap_or(&String::new()));
            
            let spec = rs.spec.as_ref();
            let status = rs.status.as_ref();
            let spec_replicas = spec.and_then(|s| s.replicas).unwrap_or(1);
            let ready_replicas = status.and_then(|s| s.ready_replicas).unwrap_or(0);
            
            nodes.push(TopologyNode {
                id: node_id.clone(),
                kind: "ReplicaSet".to_string(),
                name: rs.metadata.name.clone().unwrap_or_default(),
                namespace: rs.metadata.namespace.clone().unwrap_or_default(),
                status: get_replicaset_status(&rs),
                replicas: Some(TopologyReplicas {
                    desired: spec_replicas,
                    ready: ready_replicas,
                }),
                labels: rs.metadata.labels.clone(),
            });
            
            // Edge: ReplicaSet → Pods (by owner reference)
            if let Some(owner_refs) = rs.metadata.owner_references.as_ref() {
                for owner in owner_refs {
                    if owner.kind == "Deployment" {
                        edges.push(TopologyEdge {
                            source: format!("deployment/{}", owner.name),
                            target: node_id.clone(),
                            kind: "manages".to_string(),
                        });
                    }
                }
            }
        }
    }
    
    // Load Pods
    let pods_api: Api<Pod> = Api::namespaced(client.raw().clone(), &ns);
    let pods = pods_api.list(&Default::default()).await.ok();
    
    if let Some(pod_list) = pods {
        for pod in pod_list.items {
            let node_id = format!("pod/{}", pod.metadata.name.as_ref().unwrap_or(&String::new()));
            
            let status = pod.status.as_ref().and_then(|s| s.phase.clone()).unwrap_or_default();
            
            nodes.push(TopologyNode {
                id: node_id.clone(),
                kind: "Pod".to_string(),
                name: pod.metadata.name.clone().unwrap_or_default(),
                namespace: pod.metadata.namespace.clone().unwrap_or_default(),
                status,
                replicas: None,
                labels: pod.metadata.labels.clone(),
            });
            
            // Edge: Pod → ReplicaSet (by owner reference)
            if let Some(owner_refs) = pod.metadata.owner_references.as_ref() {
                for owner in owner_refs {
                    if owner.kind == "ReplicaSet" {
                        edges.push(TopologyEdge {
                            source: format!("replicaset/{}", owner.name),
                            target: node_id.clone(),
                            kind: "manages".to_string(),
                        });
                    }
                }
            }
        }
    }
    
    // Load Services
    let services_api: Api<Service> = Api::namespaced(client.raw().clone(), &ns);
    let services = services_api.list(&Default::default()).await.ok();
    
    if let Some(svc_list) = services {
        for svc in svc_list.items {
            let node_id = format!("service/{}", svc.metadata.name.as_ref().unwrap_or(&String::new()));
            
            nodes.push(TopologyNode {
                id: node_id.clone(),
                kind: "Service".to_string(),
                name: svc.metadata.name.clone().unwrap_or_default(),
                namespace: svc.metadata.namespace.clone().unwrap_or_default(),
                status: "active".to_string(),
                replicas: None,
                labels: svc.metadata.labels.clone(),
            });
            
            // Edge: Service → Deployment/Pod (by selector)
            let selector = svc.spec.as_ref().and_then(|s| s.selector.as_ref());
            if let Some(selector) = selector {
                for node in &nodes {
                    if (node.kind == "Pod" || node.kind == "Deployment")
                        && matches_selector(&node.labels, selector)
                    {
                        edges.push(TopologyEdge {
                            source: node_id.clone(),
                            target: node.id.clone(),
                            kind: "routes".to_string(),
                        });
                    }
                }
            }
        }
    }
    
    Ok(Json(TopologyData {
        namespace: ns,
        nodes,
        edges,
    }))
}

fn get_deployment_status(dep: &Deployment) -> String {
    let spec = dep.spec.as_ref();
    let status = dep.status.as_ref();
    let desired = spec.and_then(|s| s.replicas).unwrap_or(1);
    let ready = status.and_then(|s| s.ready_replicas).unwrap_or(0);
    let available = status.and_then(|s| s.available_replicas).unwrap_or(0);
    
    if ready >= desired && available >= desired {
        "ready".to_string()
    } else if ready > 0 {
        "progressing".to_string()
    } else {
        "pending".to_string()
    }
}

fn get_replicaset_status(rs: &ReplicaSet) -> String {
    let spec = rs.spec.as_ref();
    let status = rs.status.as_ref();
    let desired = spec.and_then(|s| s.replicas).unwrap_or(1);
    let ready = status.and_then(|s| s.ready_replicas).unwrap_or(0);
    
    if ready >= desired {
        "ready".to_string()
    } else if ready > 0 {
        "progressing".to_string()
    } else {
        "pending".to_string()
    }
}

fn matches_selector(
    labels: &Option<std::collections::BTreeMap<String, String>>,
    selector: &std::collections::BTreeMap<String, String>,
) -> bool {
    if let Some(res_labels) = labels {
        for (key, value) in selector {
            if res_labels.get(key) != Some(value) {
                return false;
            }
        }
        true
    } else {
        false
    }
}

// Helper functions for parsing resource values
fn parse_cpu(cpu: &str) -> Option<u64> {
    if let Some(stripped) = cpu.strip_suffix('m') {
        stripped.parse::<u64>().ok()
    } else {
        cpu.parse::<u64>().ok().map(|v| v * 1000)
    }
}

fn parse_memory(mem: &str) -> Option<u64> {
    if let Some(stripped) = mem.strip_suffix("Ki") {
        stripped.parse::<u64>().ok().map(|v| v * 1024)
    } else if let Some(stripped) = mem.strip_suffix("Mi") {
        stripped.parse::<u64>().ok().map(|v| v * 1024 * 1024)
    } else if let Some(stripped) = mem.strip_suffix("Gi") {
        stripped.parse::<u64>().ok().map(|v| v * 1024 * 1024 * 1024)
    } else {
        mem.parse::<u64>().ok()
    }
}

fn format_cpu(cpu: u64) -> String {
    if cpu >= 1000 {
        format!("{}m", cpu)
    } else {
        format!("{}", cpu)
    }
}

fn format_memory(mem: u64) -> String {
    if mem >= 1024 * 1024 * 1024 {
        format!("{:.1}Gi", mem as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if mem >= 1024 * 1024 {
        format!("{:.0}Mi", mem as f64 / (1024.0 * 1024.0))
    } else if mem >= 1024 {
        format!("{:.0}Ki", mem as f64 / 1024.0)
    } else {
        format!("{}", mem)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_parse_cpu_milli() {
        assert_eq!(parse_cpu("100m"), Some(100));
        assert_eq!(parse_cpu("500m"), Some(500));
        assert_eq!(parse_cpu("1000m"), Some(1000));
    }

    #[test]
    fn test_parse_cpu_whole() {
        assert_eq!(parse_cpu("1"), Some(1000));
        assert_eq!(parse_cpu("2"), Some(2000));
        assert_eq!(parse_cpu("0"), Some(0));
    }

    #[test]
    fn test_parse_cpu_invalid() {
        assert_eq!(parse_cpu("abc"), None);
        assert_eq!(parse_cpu(""), None);
        assert_eq!(parse_cpu("10x"), None);
    }

    #[test]
    fn test_parse_memory_bytes() {
        assert_eq!(parse_memory("100"), Some(100));
        assert_eq!(parse_memory("0"), Some(0));
    }

    #[test]
    fn test_parse_memory_kibi() {
        assert_eq!(parse_memory("1024Ki"), Some(1024 * 1024));
        assert_eq!(parse_memory("512Ki"), Some(512 * 1024));
    }

    #[test]
    fn test_parse_memory_mebi() {
        assert_eq!(parse_memory("1Gi"), Some(1024 * 1024 * 1024));
        assert_eq!(parse_memory("512Mi"), Some(512 * 1024 * 1024));
    }

    #[test]
    fn test_parse_memory_invalid() {
        assert_eq!(parse_memory("abc"), None);
        assert_eq!(parse_memory(""), None);
    }

    #[test]
    fn test_format_cpu_milli() {
        assert_eq!(format_cpu(1000), "1000m");
        assert_eq!(format_cpu(5000), "5000m");
    }

    #[test]
    fn test_format_cpu_whole() {
        assert_eq!(format_cpu(100), "100");
        assert_eq!(format_cpu(500), "500");
    }

    #[test]
    fn test_format_memory_bytes() {
        assert_eq!(format_memory(100), "100");
        assert_eq!(format_memory(0), "0");
    }

    #[test]
    fn test_format_memory_kibi() {
        assert_eq!(format_memory(1024), "1Ki");
        assert_eq!(format_memory(2048), "2Ki");
    }

    #[test]
    fn test_format_memory_mebi() {
        assert_eq!(format_memory(1024 * 1024), "1Mi");
        assert_eq!(format_memory(512 * 1024 * 1024), "512Mi");
    }

    #[test]
    fn test_format_memory_gibi() {
        assert_eq!(format_memory(1024 * 1024 * 1024), "1.0Gi");
        assert_eq!(format_memory(2 * 1024 * 1024 * 1024), "2.0Gi");
    }

    #[test]
    fn test_matches_selector_empty_selector() {
        let labels = Some(BTreeMap::from([("app".to_string(), "web".to_string())]));
        let selector = BTreeMap::new();
        assert!(matches_selector(&labels, &selector));
    }

    #[test]
    fn test_matches_selector_single_match() {
        let labels = Some(BTreeMap::from([("app".to_string(), "web".to_string())]));
        let selector = BTreeMap::from([("app".to_string(), "web".to_string())]);
        assert!(matches_selector(&labels, &selector));
    }

    #[test]
    fn test_matches_selector_no_match() {
        let labels = Some(BTreeMap::from([("app".to_string(), "web".to_string())]));
        let selector = BTreeMap::from([("app".to_string(), "api".to_string())]);
        assert!(!matches_selector(&labels, &selector));
    }

    #[test]
    fn test_matches_selector_missing_label() {
        let labels = Some(BTreeMap::from([("app".to_string(), "web".to_string())]));
        let selector = BTreeMap::from([("tier".to_string(), "frontend".to_string())]);
        assert!(!matches_selector(&labels, &selector));
    }

    #[test]
    fn test_matches_selector_none_labels() {
        let selector = BTreeMap::from([("app".to_string(), "web".to_string())]);
        assert!(!matches_selector(&None, &selector));
    }

    #[test]
    fn test_matches_selector_multiple_keys() {
        let labels = Some(BTreeMap::from([
            ("app".to_string(), "web".to_string()),
            ("env".to_string(), "prod".to_string()),
        ]));
        let selector = BTreeMap::from([("app".to_string(), "web".to_string())]);
        assert!(matches_selector(&labels, &selector));
    }

    // ─────────────────────────────────────────────
    // DTO struct tests - KubernetesEvent
    // ─────────────────────────────────────────────

    #[test]
    fn test_kubernetes_event() {
        let event = KubernetesEvent {
            name: "event-1".to_string(),
            namespace: "default".to_string(),
            type_: "Warning".to_string(),
            reason: "FailedScheduling".to_string(),
            message: "0/3 nodes are available".to_string(),
            count: 5,
            first_seen: Some("2024-01-01T00:00:00Z".to_string()),
            last_seen: Some("2024-01-01T00:05:00Z".to_string()),
            involved_object: InvolvedObjectSummary {
                kind: "Pod".to_string(),
                name: "test-pod".to_string(),
                api_version: Some("v1".to_string()),
                uid: Some("uid-123".to_string()),
            },
        };
        assert_eq!(event.reason, "FailedScheduling");
        assert_eq!(event.count, 5);
        assert_eq!(event.involved_object.kind, "Pod");
    }

    #[test]
    fn test_kubernetes_event_empty_fields() {
        let event = KubernetesEvent {
            name: String::new(),
            namespace: String::new(),
            type_: String::new(),
            reason: String::new(),
            message: String::new(),
            count: 0,
            first_seen: None,
            last_seen: None,
            involved_object: InvolvedObjectSummary {
                kind: String::new(),
                name: String::new(),
                api_version: None,
                uid: None,
            },
        };
        assert!(event.name.is_empty());
        assert!(event.first_seen.is_none());
        assert!(event.involved_object.api_version.is_none());
    }

    #[test]
    fn test_involved_object_summary() {
        let obj = InvolvedObjectSummary {
            kind: "Deployment".to_string(),
            name: "web-app".to_string(),
            api_version: Some("apps/v1".to_string()),
            uid: Some("uid-456".to_string()),
        };
        assert_eq!(obj.kind, "Deployment");
        assert_eq!(obj.api_version, Some("apps/v1".to_string()));
    }

    // ─────────────────────────────────────────────
    // DTO struct tests - Metrics
    // ─────────────────────────────────────────────

    #[test]
    fn test_pod_metrics() {
        let metrics = PodMetrics {
            name: "web-pod".to_string(),
            namespace: "default".to_string(),
            containers: vec![
                ContainerMetrics {
                    name: "app".to_string(),
                    usage: ResourceUsage {
                        cpu: "100m".to_string(),
                        memory: "128Mi".to_string(),
                    },
                },
            ],
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            window: "1m".to_string(),
        };
        assert_eq!(metrics.name, "web-pod");
        assert_eq!(metrics.containers.len(), 1);
        assert_eq!(metrics.containers[0].usage.cpu, "100m");
    }

    #[test]
    fn test_pod_metrics_empty_containers() {
        let metrics = PodMetrics {
            name: "empty-pod".to_string(),
            namespace: "default".to_string(),
            containers: vec![],
            timestamp: String::new(),
            window: String::new(),
        };
        assert!(metrics.containers.is_empty());
        assert!(metrics.timestamp.is_empty());
    }

    #[test]
    fn test_container_metrics() {
        let container = ContainerMetrics {
            name: "sidecar".to_string(),
            usage: ResourceUsage {
                cpu: "50m".to_string(),
                memory: "64Mi".to_string(),
            },
        };
        assert_eq!(container.name, "sidecar");
        assert_eq!(container.usage.memory, "64Mi");
    }

    #[test]
    fn test_resource_usage() {
        let usage = ResourceUsage {
            cpu: "2000m".to_string(),
            memory: "2Gi".to_string(),
        };
        assert_eq!(usage.cpu, "2000m");
        assert_eq!(usage.memory, "2Gi");
    }

    #[test]
    fn test_node_metrics() {
        let metrics = NodeMetrics {
            name: "node-1".to_string(),
            usage: ResourceUsage {
                cpu: "500m".to_string(),
                memory: "4Gi".to_string(),
            },
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            window: "1m".to_string(),
        };
        assert_eq!(metrics.name, "node-1");
        assert_eq!(metrics.usage.cpu, "500m");
    }

    // ─────────────────────────────────────────────
    // DTO struct tests - Topology
    // ─────────────────────────────────────────────

    #[test]
    fn test_topology_data() {
        let data = TopologyData {
            namespace: "default".to_string(),
            nodes: vec![TopologyNode {
                id: "deployment/web".to_string(),
                kind: "Deployment".to_string(),
                name: "web".to_string(),
                namespace: "default".to_string(),
                status: "ready".to_string(),
                replicas: Some(TopologyReplicas {
                    desired: 3,
                    ready: 3,
                }),
                labels: Some(BTreeMap::from([("app".to_string(), "web".to_string())])),
            }],
            edges: vec![],
        };
        assert_eq!(data.namespace, "default");
        assert_eq!(data.nodes.len(), 1);
        assert_eq!(data.nodes[0].kind, "Deployment");
    }

    #[test]
    fn test_topology_node() {
        let node = TopologyNode {
            id: "pod/web-abc123".to_string(),
            kind: "Pod".to_string(),
            name: "web-abc123".to_string(),
            namespace: "prod".to_string(),
            status: "Running".to_string(),
            replicas: None,
            labels: None,
        };
        assert_eq!(node.id, "pod/web-abc123");
        assert!(node.replicas.is_none());
        assert!(node.labels.is_none());
    }

    #[test]
    fn test_topology_replicas() {
        let replicas = TopologyReplicas {
            desired: 5,
            ready: 3,
        };
        assert_eq!(replicas.desired, 5);
        assert_eq!(replicas.ready, 3);
    }

    #[test]
    fn test_topology_edge() {
        let edge = TopologyEdge {
            source: "deployment/web".to_string(),
            target: "replicaset/web-abc123".to_string(),
            kind: "manages".to_string(),
        };
        assert_eq!(edge.source, "deployment/web");
        assert_eq!(edge.kind, "manages");
    }

    // ─────────────────────────────────────────────
    // Query params tests
    // ─────────────────────────────────────────────

    #[test]
    fn test_events_query_all_fields() {
        let query = EventsQuery {
            namespace: Some("kube-system".to_string()),
            limit: Some(50),
            field_selector: Some("involvedObject.kind=Pod".to_string()),
        };
        assert_eq!(query.namespace, Some("kube-system".to_string()));
        assert_eq!(query.limit, Some(50));
        assert_eq!(query.field_selector, Some("involvedObject.kind=Pod".to_string()));
    }

    #[test]
    fn test_events_query_all_none() {
        let query = EventsQuery {
            namespace: None,
            limit: None,
            field_selector: None,
        };
        assert!(query.namespace.is_none());
        assert!(query.limit.is_none());
        assert!(query.field_selector.is_none());
    }

    #[test]
    fn test_topology_query() {
        let query = TopologyQuery {
            namespace: Some("production".to_string()),
        };
        assert_eq!(query.namespace, Some("production".to_string()));
    }

    #[test]
    fn test_topology_query_none() {
        let query = TopologyQuery {
            namespace: None,
        };
        assert!(query.namespace.is_none());
    }

    // ─────────────────────────────────────────────
    // Edge cases for helper functions
    // ─────────────────────────────────────────────

    #[test]
    fn test_matches_selector_multiple_labels_all_match() {
        let labels = Some(BTreeMap::from([
            ("app".to_string(), "web".to_string()),
            ("env".to_string(), "prod".to_string()),
            ("tier".to_string(), "frontend".to_string()),
        ]));
        let selector = BTreeMap::from([
            ("app".to_string(), "web".to_string()),
            ("env".to_string(), "prod".to_string()),
        ]);
        assert!(matches_selector(&labels, &selector));
    }

    #[test]
    fn test_matches_selector_multiple_labels_partial_match() {
        let labels = Some(BTreeMap::from([
            ("app".to_string(), "web".to_string()),
            ("env".to_string(), "staging".to_string()),
        ]));
        let selector = BTreeMap::from([
            ("app".to_string(), "web".to_string()),
            ("env".to_string(), "prod".to_string()),
        ]);
        assert!(!matches_selector(&labels, &selector));
    }

    #[test]
    fn test_parse_cpu_zero_milli() {
        assert_eq!(parse_cpu("0m"), Some(0));
    }

    #[test]
    fn test_parse_memory_zero_ki() {
        assert_eq!(parse_memory("0Ki"), Some(0));
    }

    #[test]
    fn test_format_cpu_zero() {
        assert_eq!(format_cpu(0), "0");
    }

    #[test]
    fn test_format_memory_zero() {
        assert_eq!(format_memory(0), "0");
    }

    #[test]
    fn test_format_memory_exact_gib() {
        assert_eq!(format_memory(1024 * 1024 * 1024), "1.0Gi");
    }

    #[test]
    fn test_format_memory_half_gib() {
        assert_eq!(format_memory(512 * 1024 * 1024), "512Mi");
    }

    #[test]
    fn test_format_memory_fractional_gib() {
        // 1.5 GiB = 1.5 * 1024 * 1024 * 1024
        let mem = (1.5 * 1024.0 * 1024.0 * 1024.0) as u64;
        assert_eq!(format_memory(mem), "1.5Gi");
    }
}
