//! Kubernetes Workload handlers — Pods, Deployments, DaemonSets, StatefulSets, ReplicaSets, Events
//!
//! Фаза 2: полный CRUD/list для основных workload-ресурсов

use axum::{
    extract::{Path, Query, State},
    Json,
};
use std::sync::Arc;
use std::collections::BTreeMap;
use crate::api::state::AppState;
use crate::error::{Error, Result};
use crate::api::handlers::kubernetes::client::KubeClient;
use k8s_openapi::api::apps::v1::{Deployment, DaemonSet, StatefulSet, ReplicaSet};
use k8s_openapi::api::core::v1::{Pod, Event};
use kube::api::{Api, ListParams, DeleteParams};
use serde::{Deserialize, Serialize};
use chrono::Utc;

// ─────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────

fn age_from_ts(ts: Option<&k8s_openapi::apimachinery::pkg::apis::meta::v1::Time>) -> String {
    match ts {
        Some(t) => {
            let secs = (Utc::now() - t.0).num_seconds().max(0);
            if secs < 60 { format!("{secs}s") }
            else if secs < 3600 { format!("{}m", secs / 60) }
            else if secs < 86400 { format!("{}h", secs / 3600) }
            else { format!("{}d", secs / 86400) }
        }
        None => "unknown".to_string(),
    }
}

#[derive(Debug, Deserialize)]
pub struct WorkloadListQuery {
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
    pub limit: Option<i32>,
}

// ─────────────────────────────────────────────
// Pods
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodSummary {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub phase: String,
    pub node_name: Option<String>,
    pub pod_ip: Option<String>,
    pub containers: Vec<ContainerSummary>,
    pub labels: BTreeMap<String, String>,
    pub age: String,
    pub ready: String,
    pub restarts: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSummary {
    pub name: String,
    pub image: String,
    pub ready: bool,
    pub state: String,
    pub restarts: i32,
}

/// GET /api/kubernetes/pods
pub async fn list_pods(
    State(state): State<Arc<AppState>>,
    Query(query): Query<WorkloadListQuery>,
) -> Result<Json<Vec<PodSummary>>> {
    let client = state.kubernetes_client()?;
    let mut lp = ListParams::default();
    if let Some(sel) = &query.label_selector { lp = lp.labels(sel); }
    if let Some(l) = query.limit { lp = lp.limit(l as u32); }

    let api: Api<Pod> = match &query.namespace {
        Some(ns) => client.api(Some(ns.as_str())),
        None => client.api_all(),
    };

    let list = api.list(&lp).await.map_err(|e| Error::Kubernetes(e.to_string()))?;

    let summaries = list.items.iter().map(|p| {
        let meta = &p.metadata;
        let status = p.status.as_ref();
        let phase = status.and_then(|s| s.phase.as_deref()).unwrap_or("Unknown").to_string();
        let node_name = p.spec.as_ref().and_then(|s| s.node_name.clone());
        let pod_ip = status.and_then(|s| s.pod_ip.clone());

        let containers: Vec<ContainerSummary> = p.spec.as_ref()
            .map(|sp| sp.containers.iter().enumerate().map(|(i, c)| {
                let cs = status.and_then(|s| s.container_statuses.as_ref())
                    .and_then(|css| css.get(i));
                let ready = cs.map(|cs| cs.ready).unwrap_or(false);
                let restarts = cs.map(|cs| cs.restart_count).unwrap_or(0);
                let state = cs.and_then(|cs| cs.state.as_ref()).map(|s| {
                    if s.running.is_some() { "Running" }
                    else if s.waiting.is_some() { "Waiting" }
                    else if s.terminated.is_some() { "Terminated" }
                    else { "Unknown" }
                }).unwrap_or("Unknown").to_string();
                ContainerSummary {
                    name: c.name.clone(),
                    image: c.image.clone().unwrap_or_default(),
                    ready,
                    state,
                    restarts,
                }
            }).collect()).unwrap_or_default();

        let total = containers.len();
        let ready_count = containers.iter().filter(|c| c.ready).count();
        let restarts: i32 = containers.iter().map(|c| c.restarts).sum();

        PodSummary {
            name: meta.name.as_deref().unwrap_or_default().to_string(),
            namespace: meta.namespace.as_deref().unwrap_or_default().to_string(),
            uid: meta.uid.as_deref().unwrap_or_default().to_string(),
            phase,
            node_name,
            pod_ip,
            containers,
            labels: meta.labels.clone().unwrap_or_default(),
            age: age_from_ts(meta.creation_timestamp.as_ref()),
            ready: format!("{ready_count}/{total}"),
            restarts,
        }
    }).collect();

    Ok(Json(summaries))
}

/// GET /api/kubernetes/namespaces/{namespace}/pods/{name}
pub async fn get_pod(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Pod> = client.api(Some(&namespace));
    let pod = api.get(&name).await.map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(serde_json::to_value(pod).unwrap_or_default()))
}

/// DELETE /api/kubernetes/namespaces/{namespace}/pods/{name}
pub async fn delete_pod(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Pod> = client.api(Some(&namespace));
    api.delete(&name, &DeleteParams::default()).await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(serde_json::json!({"deleted": true, "name": name})))
}

/// GET /api/kubernetes/namespaces/{namespace}/pods/{name}/logs
pub async fn pod_logs(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Query(q): Query<PodLogsQuery>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Pod> = client.api(Some(&namespace));
    let mut lp = kube::api::LogParams::default();
    if let Some(c) = &q.container { lp.container = Some(c.clone()); }
    if let Some(n) = q.tail_lines { lp.tail_lines = Some(n as i64); }
    lp.previous = q.previous.unwrap_or(false);
    let logs = api.logs(&name, &lp).await.map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(serde_json::json!({"logs": logs, "container": q.container})))
}

/// POST /api/kubernetes/namespaces/{namespace}/pods/{name}/evict
/// 
/// Evict pod using Kubernetes Eviction API (Policy/V1).
/// Handles 429 Too Many Requests when PDB blocks the eviction.
pub async fn evict_pod(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    use k8s_openapi::api::policy::v1::Eviction;
    use kube::api::{DeleteParams, EvictParams};
    use axum::http::StatusCode;
    
    let client = state.kubernetes_client()?;
    let api: Api<Pod> = client.api(Some(&namespace));
    
    let evict_params = EvictParams {
        delete_options: Some(DeleteParams {
            grace_period_seconds: Some(30),
            ..DeleteParams::default()
        }),
        ..EvictParams::default()
    };
    
    match api.evict(&name, &evict_params).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "evicted": true,
            "name": name,
            "namespace": namespace,
            "message": "Pod evicted successfully"
        }))),
        Err(kube::Error::Api(api_err)) if api_err.code == 429 => {
            // PDB is blocking the eviction
            Err(Error::Http {
                status: StatusCode::TOO_MANY_REQUESTS,
                message: format!(
                    "Eviction blocked by PodDisruptionBudget: {}. \
                     The PDB requires more pods to be available before this one can be evicted.",
                    name
                ),
            })
        }
        Err(e) => Err(Error::Kubernetes(e.to_string())),
    }
}

#[derive(Debug, Deserialize)]
pub struct PodLogsQuery {
    pub container: Option<String>,
    pub tail_lines: Option<u32>,
    pub previous: Option<bool>,
}

// ─────────────────────────────────────────────
// Deployments
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSummary {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub available_replicas: i32,
    pub updated_replicas: i32,
    pub labels: BTreeMap<String, String>,
    pub age: String,
    pub conditions: Vec<DeploymentCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentCondition {
    pub type_: String,
    pub status: String,
    pub message: Option<String>,
}

/// GET /api/kubernetes/deployments
pub async fn list_deployments(
    State(state): State<Arc<AppState>>,
    Query(query): Query<WorkloadListQuery>,
) -> Result<Json<Vec<DeploymentSummary>>> {
    let client = state.kubernetes_client()?;
    let mut lp = ListParams::default();
    if let Some(sel) = &query.label_selector { lp = lp.labels(sel); }
    if let Some(l) = query.limit { lp = lp.limit(l as u32); }

    let api: Api<Deployment> = match &query.namespace {
        Some(ns) => client.api(Some(ns.as_str())),
        None => client.api_all(),
    };

    let list = api.list(&lp).await.map_err(|e| Error::Kubernetes(e.to_string()))?;

    let summaries = list.items.iter().map(|d| {
        let meta = &d.metadata;
        let status = d.status.as_ref();
        let spec_replicas = d.spec.as_ref().and_then(|s| s.replicas).unwrap_or(0);
        let ready = status.and_then(|s| s.ready_replicas).unwrap_or(0);
        let available = status.and_then(|s| s.available_replicas).unwrap_or(0);
        let updated = status.and_then(|s| s.updated_replicas).unwrap_or(0);

        let conditions = status.and_then(|s| s.conditions.as_ref())
            .map(|conds| conds.iter().map(|c| DeploymentCondition {
                type_: c.type_.clone(),
                status: c.status.clone(),
                message: c.message.clone(),
            }).collect())
            .unwrap_or_default();

        DeploymentSummary {
            name: meta.name.as_deref().unwrap_or_default().to_string(),
            namespace: meta.namespace.as_deref().unwrap_or_default().to_string(),
            uid: meta.uid.as_deref().unwrap_or_default().to_string(),
            replicas: spec_replicas,
            ready_replicas: ready,
            available_replicas: available,
            updated_replicas: updated,
            labels: meta.labels.clone().unwrap_or_default(),
            age: age_from_ts(meta.creation_timestamp.as_ref()),
            conditions,
        }
    }).collect();

    Ok(Json(summaries))
}

/// GET /api/kubernetes/namespaces/{namespace}/deployments/{name}
pub async fn get_deployment(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Deployment> = client.api(Some(&namespace));
    let d = api.get(&name).await.map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(serde_json::to_value(d).unwrap_or_default()))
}

/// PUT /api/kubernetes/namespaces/{namespace}/deployments/{name}/scale
pub async fn scale_deployment(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(body): Json<ScaleBody>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Deployment> = client.api(Some(&namespace));
    let patch = serde_json::json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "spec": { "replicas": body.replicas }
    });
    api.patch(&name,
        &kube::api::PatchParams::apply("velum").force(),
        &kube::api::Patch::Apply(patch),
    ).await.map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(serde_json::json!({"scaled": true, "replicas": body.replicas})))
}

/// POST /api/kubernetes/namespaces/{namespace}/deployments/{name}/restart
pub async fn restart_deployment(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Deployment> = client.api(Some(&namespace));
    let now = Utc::now().to_rfc3339();
    let patch = serde_json::json!({
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "spec": {
            "template": {
                "metadata": {
                    "annotations": { "kubectl.kubernetes.io/restartedAt": now }
                }
            }
        }
    });
    api.patch(&name,
        &kube::api::PatchParams::apply("velum").force(),
        &kube::api::Patch::Apply(patch),
    ).await.map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(serde_json::json!({"restarted": true})))
}

#[derive(Debug, Deserialize)]
pub struct ScaleBody { pub replicas: i32 }

// ─────────────────────────────────────────────
// DaemonSets
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSetSummary {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub desired: i32,
    pub current: i32,
    pub ready: i32,
    pub updated: i32,
    pub available: i32,
    pub labels: BTreeMap<String, String>,
    pub age: String,
}

/// GET /api/kubernetes/daemonsets
pub async fn list_daemonsets(
    State(state): State<Arc<AppState>>,
    Query(query): Query<WorkloadListQuery>,
) -> Result<Json<Vec<DaemonSetSummary>>> {
    let client = state.kubernetes_client()?;
    let mut lp = ListParams::default();
    if let Some(sel) = &query.label_selector { lp = lp.labels(sel); }
    if let Some(l) = query.limit { lp = lp.limit(l as u32); }

    let api: Api<DaemonSet> = match &query.namespace {
        Some(ns) => client.api(Some(ns.as_str())),
        None => client.api_all(),
    };

    let list = api.list(&lp).await.map_err(|e| Error::Kubernetes(e.to_string()))?;

    let summaries = list.items.iter().map(|ds| {
        let meta = ds.metadata.clone();
        let status = ds.status.as_ref();
        DaemonSetSummary {
            name: meta.name.clone().unwrap_or_default(),
            namespace: meta.namespace.clone().unwrap_or_default(),
            uid: meta.uid.clone().unwrap_or_default(),
            desired: status.map(|s| s.desired_number_scheduled).unwrap_or(0),
            current: status.map(|s| s.current_number_scheduled).unwrap_or(0),
            ready: status.map(|s| s.number_ready).unwrap_or(0),
            updated: status.and_then(|s| s.updated_number_scheduled).unwrap_or(0),
            available: status.and_then(|s| s.number_available).unwrap_or(0),
            labels: meta.labels.clone().unwrap_or_default(),
            age: age_from_ts(meta.creation_timestamp.as_ref()),
        }
    }).collect();

    Ok(Json(summaries))
}

/// POST /api/kubernetes/namespaces/{namespace}/daemonsets/{name}/restart
pub async fn restart_daemonset(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<DaemonSet> = client.api(Some(&namespace));
    let now = Utc::now().to_rfc3339();
    let patch = serde_json::json!({
        "apiVersion": "apps/v1", "kind": "DaemonSet",
        "spec": { "template": { "metadata": {
            "annotations": { "kubectl.kubernetes.io/restartedAt": now }
        }}}
    });
    api.patch(&name, &kube::api::PatchParams::apply("velum").force(), &kube::api::Patch::Apply(patch))
        .await.map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(serde_json::json!({"restarted": true})))
}

// ─────────────────────────────────────────────
// StatefulSets
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatefulSetSummary {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub current_replicas: i32,
    pub service_name: String,
    pub labels: BTreeMap<String, String>,
    pub age: String,
}

/// GET /api/kubernetes/statefulsets
pub async fn list_statefulsets(
    State(state): State<Arc<AppState>>,
    Query(query): Query<WorkloadListQuery>,
) -> Result<Json<Vec<StatefulSetSummary>>> {
    let client = state.kubernetes_client()?;
    let mut lp = ListParams::default();
    if let Some(sel) = &query.label_selector { lp = lp.labels(sel); }
    if let Some(l) = query.limit { lp = lp.limit(l as u32); }

    let api: Api<StatefulSet> = match &query.namespace {
        Some(ns) => client.api(Some(ns.as_str())),
        None => client.api_all(),
    };

    let list = api.list(&lp).await.map_err(|e| Error::Kubernetes(e.to_string()))?;

    let summaries = list.items.iter().map(|ss| {
        let meta = ss.metadata.clone();
        let status = ss.status.as_ref();
        StatefulSetSummary {
            name: meta.name.clone().unwrap_or_default(),
            namespace: meta.namespace.clone().unwrap_or_default(),
            uid: meta.uid.clone().unwrap_or_default(),
            replicas: ss.spec.as_ref().and_then(|s| s.replicas).unwrap_or(0),
            ready_replicas: status.and_then(|s| s.ready_replicas).unwrap_or(0),
            current_replicas: status.and_then(|s| s.current_replicas).unwrap_or(0),
            service_name: ss.spec.as_ref().map(|s| s.service_name.clone()).unwrap_or_default(),
            labels: meta.labels.clone().unwrap_or_default(),
            age: age_from_ts(meta.creation_timestamp.as_ref()),
        }
    }).collect();

    Ok(Json(summaries))
}

/// PUT /api/kubernetes/namespaces/{namespace}/statefulsets/{name}/scale
pub async fn scale_statefulset(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(body): Json<ScaleBody>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<StatefulSet> = client.api(Some(&namespace));
    let patch = serde_json::json!({
        "apiVersion": "apps/v1", "kind": "StatefulSet",
        "spec": { "replicas": body.replicas }
    });
    api.patch(&name, &kube::api::PatchParams::apply("velum").force(), &kube::api::Patch::Apply(patch))
        .await.map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(serde_json::json!({"scaled": true, "replicas": body.replicas})))
}

// ─────────────────────────────────────────────
// ReplicaSets
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaSetSummary {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub desired: i32,
    pub ready: i32,
    pub available: i32,
    pub owner: Option<String>,
    pub labels: BTreeMap<String, String>,
    pub age: String,
}

/// GET /api/kubernetes/replicasets
pub async fn list_replicasets(
    State(state): State<Arc<AppState>>,
    Query(query): Query<WorkloadListQuery>,
) -> Result<Json<Vec<ReplicaSetSummary>>> {
    let client = state.kubernetes_client()?;
    let mut lp = ListParams::default();
    if let Some(sel) = &query.label_selector { lp = lp.labels(sel); }
    if let Some(l) = query.limit { lp = lp.limit(l as u32); }

    let api: Api<ReplicaSet> = match &query.namespace {
        Some(ns) => client.api(Some(ns.as_str())),
        None => client.api_all(),
    };

    let list = api.list(&lp).await.map_err(|e| Error::Kubernetes(e.to_string()))?;

    let summaries = list.items.iter().map(|rs| {
        let meta = rs.metadata.clone();
        let status = rs.status.as_ref();
        let owner = meta.owner_references.as_ref()
            .and_then(|r| r.first())
            .map(|o| format!("{}/{}", o.kind, o.name));
        ReplicaSetSummary {
            name: meta.name.clone().unwrap_or_default(),
            namespace: meta.namespace.clone().unwrap_or_default(),
            uid: meta.uid.clone().unwrap_or_default(),
            desired: rs.spec.as_ref().and_then(|s| s.replicas).unwrap_or(0),
            ready: status.and_then(|s| s.ready_replicas).unwrap_or(0),
            available: status.and_then(|s| s.available_replicas).unwrap_or(0),
            owner,
            labels: meta.labels.clone().unwrap_or_default(),
            age: age_from_ts(meta.creation_timestamp.as_ref()),
        }
    }).collect();

    Ok(Json(summaries))
}

// ─────────────────────────────────────────────
// Events
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSummary {
    pub name: String,
    pub namespace: String,
    pub reason: String,
    pub message: String,
    pub type_: String,
    pub count: i32,
    pub involved_kind: String,
    pub involved_name: String,
    pub age: String,
    pub last_timestamp: String,
    pub source: String,
}

#[derive(Debug, Deserialize)]
pub struct EventListQuery {
    pub namespace: Option<String>,
    pub kind: Option<String>,
    pub type_: Option<String>,
    pub limit: Option<i32>,
}

/// GET /api/kubernetes/events
pub async fn list_k8s_events(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EventListQuery>,
) -> Result<Json<Vec<EventSummary>>> {
    let client = state.kubernetes_client()?;
    let mut lp = ListParams::default();
    if let Some(l) = query.limit { lp = lp.limit(l as u32); }

    let api: Api<Event> = match &query.namespace {
        Some(ns) => client.api(Some(ns.as_str())),
        None => client.api_all(),
    };

    let list = api.list(&lp).await.map_err(|e| Error::Kubernetes(e.to_string()))?;

    let mut summaries: Vec<EventSummary> = list.items.iter().filter_map(|e| {
        let meta = &e.metadata;
        let involved = &e.involved_object;
        let kind = involved.kind.as_deref().unwrap_or("Unknown");
        let ev_type = e.type_.as_deref().unwrap_or("Normal");

        // Filter by kind/type if specified
        if let Some(fk) = &query.kind {
            if !kind.eq_ignore_ascii_case(fk) { return None; }
        }
        if let Some(ft) = &query.type_ {
            if !ev_type.eq_ignore_ascii_case(ft) { return None; }
        }

        let last_ts = e.last_timestamp.as_ref()
            .map(|t| t.0.to_rfc3339())
            .or_else(|| e.event_time.as_ref().map(|t| t.0.to_rfc3339()))
            .unwrap_or_default();

        let source = e.source.as_ref()
            .and_then(|s| s.component.as_deref())
            .unwrap_or("").to_string();

        Some(EventSummary {
            name: meta.name.clone().unwrap_or_default(),
            namespace: meta.namespace.clone().unwrap_or_default(),
            reason: e.reason.clone().unwrap_or_default(),
            message: e.message.clone().unwrap_or_default(),
            type_: ev_type.to_string(),
            count: e.count.unwrap_or(1),
            involved_kind: kind.to_string(),
            involved_name: involved.name.clone().unwrap_or_default(),
            age: age_from_ts(meta.creation_timestamp.as_ref()),
            last_timestamp: last_ts,
            source,
        })
    }).collect();

    // Sort: Warning first, then by count desc
    summaries.sort_by(|a, b| {
        b.type_.cmp(&a.type_).then(b.count.cmp(&a.count))
    });

    Ok(Json(summaries))
}
