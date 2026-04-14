//! HPA, ResourceQuota, LimitRange, CRD, динамические Custom Resources, VPA (feature-detect).

use axum::{
    Json,
    extract::{Path, Query, State},
};
use k8s_openapi::api::autoscaling::v2::HorizontalPodAutoscaler;
use k8s_openapi::api::core::v1::{LimitRange, ResourceQuota};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::{Api, DeleteParams, DynamicObject, ListParams, PostParams};
use kube::core::{ApiResource, GroupVersionKind};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct NsQuery {
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HpaSummary {
    pub name: String,
    pub namespace: String,
    pub min_replicas: Option<i32>,
    pub max_replicas: i32,
    pub target_ref: String,
    pub current_replicas: Option<i32>,
    pub desired_replicas: Option<i32>,
    pub metrics_hint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResourceQuotaSummary {
    pub name: String,
    pub namespace: String,
    pub hard: Option<serde_json::Value>,
    pub used: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct LimitRangeSummary {
    pub name: String,
    pub namespace: String,
    pub limits_count: usize,
}

#[derive(Debug, Serialize)]
pub struct CrdSummary {
    pub name: String,
    pub group: String,
    pub kind: String,
    pub plural: String,
    pub scope: String,
    pub versions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct VpaApiStatus {
    pub installed: bool,
}

#[derive(Debug, Deserialize)]
pub struct CustomObjectQuery {
    pub group: String,
    pub version: String,
    pub plural: String,
    pub kind: String,
    pub namespace: Option<String>,
}

fn gvk(group: &str, version: &str, kind: &str) -> GroupVersionKind {
    GroupVersionKind::gvk(group, version, kind)
}

fn ar(group: &str, version: &str, kind: &str, plural: &str) -> ApiResource {
    ApiResource::from_gvk_with_plural(&gvk(group, version, kind), plural)
}

fn hpa_summary(h: &HorizontalPodAutoscaler) -> HpaSummary {
    let spec = h.spec.as_ref();
    let st = h.status.as_ref();
    let target = spec
        .map(|s| &s.scale_target_ref)
        .map(|r| format!("{}/{}", r.kind, r.name))
        .unwrap_or_else(|| "—".to_string());
    let mut metrics_hint = None;
    if let Some(conditions) = st.and_then(|s| s.conditions.as_ref()) {
        let mut msgs = Vec::new();
        for c in conditions {
            if c.status == "False" {
                if let Some(m) = &c.message {
                    if m.contains("metrics") || m.contains("Metric") || m.contains("failed") {
                        msgs.push(m.clone());
                    }
                }
            }
        }
        if msgs.is_empty() {
            for c in conditions {
                if c.status == "False" {
                    if let Some(m) = &c.message {
                        msgs.push(m.clone());
                    }
                }
            }
        }
        if !msgs.is_empty() {
            metrics_hint = Some(msgs.join("; "));
        }
    }
    HpaSummary {
        name: h.metadata.name.clone().unwrap_or_default(),
        namespace: h
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        min_replicas: spec.and_then(|s| s.min_replicas),
        max_replicas: spec.map(|s| s.max_replicas).unwrap_or(0),
        target_ref: target,
        current_replicas: st.and_then(|s| s.current_replicas),
        desired_replicas: st.map(|s| s.desired_replicas),
        metrics_hint,
    }
}

pub async fn list_hpas(
    State(state): State<Arc<AppState>>,
    Query(query): Query<NsQuery>,
) -> Result<Json<Vec<HpaSummary>>> {
    let client = state.kubernetes_client()?;
    let ns = query.namespace.unwrap_or_else(|| "default".to_string());
    let api: Api<HorizontalPodAutoscaler> = client.api(Some(&ns));
    let list = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(list.items.iter().map(hpa_summary).collect()))
}

pub async fn get_hpa(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<HorizontalPodAutoscaler>> {
    let client = state.kubernetes_client()?;
    let api: Api<HorizontalPodAutoscaler> = client.api(Some(&namespace));
    let item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(item))
}

pub async fn create_hpa(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(payload): Json<HorizontalPodAutoscaler>,
) -> Result<Json<HorizontalPodAutoscaler>> {
    let client = state.kubernetes_client()?;
    let api: Api<HorizontalPodAutoscaler> = client.api(Some(&namespace));
    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(created))
}

pub async fn update_hpa(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<HorizontalPodAutoscaler>,
) -> Result<Json<HorizontalPodAutoscaler>> {
    let client = state.kubernetes_client()?;
    let api: Api<HorizontalPodAutoscaler> = client.api(Some(&namespace));
    let updated = api
        .replace(&name, &PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(updated))
}

pub async fn delete_hpa(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<HorizontalPodAutoscaler> = client.api(Some(&namespace));
    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        serde_json::json!({"status":"ok","message":format!("HPA {namespace}/{name} deleted")}),
    ))
}

fn rq_summary(q: &ResourceQuota) -> ResourceQuotaSummary {
    let st = q.status.as_ref();
    ResourceQuotaSummary {
        name: q.metadata.name.clone().unwrap_or_default(),
        namespace: q
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        hard: q
            .spec
            .as_ref()
            .and_then(|s| s.hard.as_ref())
            .map(|h| serde_json::to_value(h).unwrap_or_default()),
        used: st
            .and_then(|s| s.used.as_ref())
            .map(|u| serde_json::to_value(u).unwrap_or_default()),
    }
}

pub async fn list_resource_quotas(
    State(state): State<Arc<AppState>>,
    Query(query): Query<NsQuery>,
) -> Result<Json<Vec<ResourceQuotaSummary>>> {
    let client = state.kubernetes_client()?;
    let ns = query.namespace.unwrap_or_else(|| "default".to_string());
    let api: Api<ResourceQuota> = client.api(Some(&ns));
    let list = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(list.items.iter().map(rq_summary).collect()))
}

pub async fn get_resource_quota(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<ResourceQuota>> {
    let client = state.kubernetes_client()?;
    let api: Api<ResourceQuota> = client.api(Some(&namespace));
    let item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(item))
}

pub async fn create_resource_quota(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(payload): Json<ResourceQuota>,
) -> Result<Json<ResourceQuota>> {
    let client = state.kubernetes_client()?;
    let api: Api<ResourceQuota> = client.api(Some(&namespace));
    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(created))
}

pub async fn update_resource_quota(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<ResourceQuota>,
) -> Result<Json<ResourceQuota>> {
    let client = state.kubernetes_client()?;
    let api: Api<ResourceQuota> = client.api(Some(&namespace));
    let updated = api
        .replace(&name, &PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(updated))
}

pub async fn delete_resource_quota(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<ResourceQuota> = client.api(Some(&namespace));
    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        serde_json::json!({"status":"ok","message":format!("ResourceQuota {namespace}/{name} deleted")}),
    ))
}

fn lr_summary(l: &LimitRange) -> LimitRangeSummary {
    LimitRangeSummary {
        name: l.metadata.name.clone().unwrap_or_default(),
        namespace: l
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        limits_count: l.spec.as_ref().map(|s| s.limits.len()).unwrap_or(0),
    }
}

pub async fn list_limit_ranges(
    State(state): State<Arc<AppState>>,
    Query(query): Query<NsQuery>,
) -> Result<Json<Vec<LimitRangeSummary>>> {
    let client = state.kubernetes_client()?;
    let ns = query.namespace.unwrap_or_else(|| "default".to_string());
    let api: Api<LimitRange> = client.api(Some(&ns));
    let list = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(list.items.iter().map(lr_summary).collect()))
}

pub async fn get_limit_range(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<LimitRange>> {
    let client = state.kubernetes_client()?;
    let api: Api<LimitRange> = client.api(Some(&namespace));
    let item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(item))
}

pub async fn create_limit_range(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(payload): Json<LimitRange>,
) -> Result<Json<LimitRange>> {
    let client = state.kubernetes_client()?;
    let api: Api<LimitRange> = client.api(Some(&namespace));
    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(created))
}

pub async fn update_limit_range(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<LimitRange>,
) -> Result<Json<LimitRange>> {
    let client = state.kubernetes_client()?;
    let api: Api<LimitRange> = client.api(Some(&namespace));
    let updated = api
        .replace(&name, &PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(updated))
}

pub async fn delete_limit_range(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<LimitRange> = client.api(Some(&namespace));
    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        serde_json::json!({"status":"ok","message":format!("LimitRange {namespace}/{name} deleted")}),
    ))
}

fn crd_summary(c: &CustomResourceDefinition) -> CrdSummary {
    let spec = &c.spec;
    let versions: Vec<String> = spec.versions.iter().map(|x| x.name.clone()).collect();
    CrdSummary {
        name: c.metadata.name.clone().unwrap_or_default(),
        group: spec.group.clone(),
        kind: spec.names.kind.clone(),
        plural: spec.names.plural.clone(),
        scope: format!("{:?}", spec.scope),
        versions,
    }
}

pub async fn list_crds(State(state): State<Arc<AppState>>) -> Result<Json<Vec<CrdSummary>>> {
    let client = state.kubernetes_client()?;
    let api: Api<CustomResourceDefinition> = client.api_all();
    let list = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(list.items.iter().map(crd_summary).collect()))
}

pub async fn get_crd(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<CustomResourceDefinition>> {
    let client = state.kubernetes_client()?;
    let api: Api<CustomResourceDefinition> = client.api_all();
    let item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(item))
}

fn dynamic_api(
    client: &crate::api::handlers::kubernetes::client::KubeClient,
    q: &CustomObjectQuery,
) -> Result<Api<DynamicObject>> {
    let api_res = ar(&q.group, &q.version, &q.kind, &q.plural);
    let raw = client.raw().clone();
    if let Some(ns) = q.namespace.as_deref() {
        Ok(Api::namespaced_with(raw, ns, &api_res))
    } else {
        Ok(Api::all_with(raw, &api_res))
    }
}

pub async fn list_custom_objects(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CustomObjectQuery>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let client = state.kubernetes_client()?;
    let api = dynamic_api(&client, &query)?;
    let list = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        list.items.iter().map(|x| serde_json::json!(x)).collect(),
    ))
}

pub async fn get_custom_object(
    State(state): State<Arc<AppState>>,
    Path((namespace, plural, name)): Path<(String, String, String)>,
    Query(query): Query<CustomObjectPayload>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let q = CustomObjectQuery {
        group: query.group,
        version: query.version,
        plural,
        kind: query.kind,
        namespace: Some(namespace),
    };
    let api = dynamic_api(&client, &q)?;
    let obj = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(serde_json::json!(obj)))
}

#[derive(Debug, Deserialize)]
pub struct CustomObjectPayload {
    pub group: String,
    pub version: String,
    pub kind: String,
}

pub async fn get_custom_object_cluster(
    State(state): State<Arc<AppState>>,
    Path((plural, name)): Path<(String, String)>,
    Query(query): Query<CustomObjectPayload>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let q = CustomObjectQuery {
        group: query.group,
        version: query.version,
        plural,
        kind: query.kind,
        namespace: None,
    };
    let api = dynamic_api(&client, &q)?;
    let obj = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(serde_json::json!(obj)))
}

pub async fn create_custom_object_query(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CustomObjectQuery>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api = dynamic_api(&client, &query)?;
    let obj: DynamicObject = serde_json::from_value(payload)
        .map_err(|e| Error::Other(format!("Invalid DynamicObject JSON: {e}")))?;
    let created = api
        .create(&PostParams::default(), &obj)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(serde_json::json!(created)))
}

pub async fn create_custom_object_cluster(
    State(state): State<Arc<AppState>>,
    Path(plural): Path<String>,
    Query(query): Query<CustomObjectPayload>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let q = CustomObjectQuery {
        group: query.group.clone(),
        version: query.version.clone(),
        plural,
        kind: query.kind.clone(),
        namespace: None,
    };
    let api = dynamic_api(&client, &q)?;
    let obj: DynamicObject = serde_json::from_value(payload)
        .map_err(|e| Error::Other(format!("Invalid DynamicObject JSON: {e}")))?;
    let created = api
        .create(&PostParams::default(), &obj)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(serde_json::json!(created)))
}

pub async fn replace_custom_object(
    State(state): State<Arc<AppState>>,
    Path((namespace, plural, name)): Path<(String, String, String)>,
    Query(query): Query<CustomObjectPayload>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let q = CustomObjectQuery {
        group: query.group.clone(),
        version: query.version.clone(),
        plural,
        kind: query.kind.clone(),
        namespace: Some(namespace),
    };
    let api = dynamic_api(&client, &q)?;
    let obj: DynamicObject = serde_json::from_value(payload)
        .map_err(|e| Error::Other(format!("Invalid DynamicObject JSON: {e}")))?;
    let updated = api
        .replace(&name, &PostParams::default(), &obj)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(serde_json::json!(updated)))
}

pub async fn delete_custom_object(
    State(state): State<Arc<AppState>>,
    Path((namespace, plural, name)): Path<(String, String, String)>,
    Query(query): Query<CustomObjectPayload>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let q = CustomObjectQuery {
        group: query.group,
        version: query.version,
        plural,
        kind: query.kind,
        namespace: Some(namespace),
    };
    let api = dynamic_api(&client, &q)?;
    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(serde_json::json!({"status":"ok"})))
}

pub async fn replace_custom_object_cluster(
    State(state): State<Arc<AppState>>,
    Path((plural, name)): Path<(String, String)>,
    Query(query): Query<CustomObjectPayload>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let q = CustomObjectQuery {
        group: query.group.clone(),
        version: query.version.clone(),
        plural,
        kind: query.kind.clone(),
        namespace: None,
    };
    let api = dynamic_api(&client, &q)?;
    let obj: DynamicObject = serde_json::from_value(payload)
        .map_err(|e| Error::Other(format!("Invalid DynamicObject JSON: {e}")))?;
    let updated = api
        .replace(&name, &PostParams::default(), &obj)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(serde_json::json!(updated)))
}

pub async fn delete_custom_object_cluster(
    State(state): State<Arc<AppState>>,
    Path((plural, name)): Path<(String, String)>,
    Query(query): Query<CustomObjectPayload>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let q = CustomObjectQuery {
        group: query.group,
        version: query.version,
        plural,
        kind: query.kind,
        namespace: None,
    };
    let api = dynamic_api(&client, &q)?;
    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(serde_json::json!({"status":"ok"})))
}

pub async fn get_vpa_status(State(state): State<Arc<AppState>>) -> Result<Json<VpaApiStatus>> {
    let client = state.kubernetes_client()?;
    let lp = ListParams::default().limit(1);
    let api_res = ar(
        "autoscaling.k8s.io",
        "v1",
        "VerticalPodAutoscaler",
        "verticalpodautoscalers",
    );
    let api: Api<DynamicObject> = Api::all_with(client.raw().clone(), &api_res);
    let installed = api.list(&lp).await.is_ok();
    Ok(Json(VpaApiStatus { installed }))
}

pub async fn list_vertical_pod_autoscalers(
    State(state): State<Arc<AppState>>,
    Query(query): Query<NsQuery>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let client = state.kubernetes_client()?;
    let ns = query.namespace.unwrap_or_else(|| "default".to_string());
    let api_res = ar(
        "autoscaling.k8s.io",
        "v1",
        "VerticalPodAutoscaler",
        "verticalpodautoscalers",
    );
    let api: Api<DynamicObject> = Api::namespaced_with(client.raw().clone(), &ns, &api_res);
    let list = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(format!("VPA API: {e}")))?;
    Ok(Json(
        list.items.iter().map(|x| serde_json::json!(x)).collect(),
    ))
}

pub async fn get_vertical_pod_autoscaler(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api_res = ar(
        "autoscaling.k8s.io",
        "v1",
        "VerticalPodAutoscaler",
        "verticalpodautoscalers",
    );
    let api: Api<DynamicObject> = Api::namespaced_with(client.raw().clone(), &namespace, &api_res);
    let obj = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(serde_json::json!(obj)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hpa_summary() {
        let summary = HpaSummary {
            name: "web-hpa".to_string(),
            namespace: "default".to_string(),
            min_replicas: Some(2),
            max_replicas: 10,
            target_ref: "Deployment/web".to_string(),
            current_replicas: Some(3),
            desired_replicas: Some(4),
            metrics_hint: Some("CPU at 75%".to_string()),
        };
        assert_eq!(summary.name, "web-hpa");
        assert_eq!(summary.max_replicas, 10);
    }

    #[test]
    fn test_resource_quota_summary() {
        let summary = ResourceQuotaSummary {
            name: "default-quota".to_string(),
            namespace: "default".to_string(),
            hard: Some(serde_json::json!({"cpu": "4", "memory": "8Gi"})),
            used: Some(serde_json::json!({"cpu": "2", "memory": "4Gi"})),
        };
        assert_eq!(summary.name, "default-quota");
        assert!(summary.hard.is_some());
    }

    #[test]
    fn test_limit_range_summary() {
        let summary = LimitRangeSummary {
            name: "default-limits".to_string(),
            namespace: "default".to_string(),
            limits_count: 3,
        };
        assert_eq!(summary.name, "default-limits");
        assert_eq!(summary.limits_count, 3);
    }

    #[test]
    fn test_crd_summary() {
        let summary = CrdSummary {
            name: "ingresses.networking.k8s.io".to_string(),
            group: "networking.k8s.io".to_string(),
            kind: "Ingress".to_string(),
            plural: "ingresses".to_string(),
            scope: "Namespaced".to_string(),
            versions: vec!["v1".to_string()],
        };
        assert_eq!(summary.kind, "Ingress");
        assert_eq!(summary.scope, "Namespaced");
    }

    #[test]
    fn test_vpa_api_status() {
        let status = VpaApiStatus { installed: true };
        assert!(status.installed);
    }

    #[test]
    fn test_custom_object_query() {
        let query = CustomObjectQuery {
            group: "networking.k8s.io".to_string(),
            version: "v1".to_string(),
            plural: "ingresses".to_string(),
            kind: "Ingress".to_string(),
            namespace: Some("default".to_string()),
        };
        assert_eq!(query.group, "networking.k8s.io");
        assert_eq!(query.namespace, Some("default".to_string()));
    }

    #[test]
    fn test_ns_query() {
        let query = NsQuery {
            namespace: Some("kube-system".to_string()),
        };
        assert_eq!(query.namespace, Some("kube-system".to_string()));
    }

    // ─────────────────────────────────────────────
    // DTO struct edge cases
    // ─────────────────────────────────────────────

    #[test]
    fn test_hpa_summary_minimal() {
        let summary = HpaSummary {
            name: String::new(),
            namespace: String::new(),
            min_replicas: None,
            max_replicas: 0,
            target_ref: String::new(),
            current_replicas: None,
            desired_replicas: None,
            metrics_hint: None,
        };
        assert!(summary.name.is_empty());
        assert!(summary.min_replicas.is_none());
        assert_eq!(summary.max_replicas, 0);
    }

    #[test]
    fn test_hpa_summary_full() {
        let summary = HpaSummary {
            name: "cpu-hpa".to_string(),
            namespace: "production".to_string(),
            min_replicas: Some(3),
            max_replicas: 20,
            target_ref: "Deployment/api".to_string(),
            current_replicas: Some(5),
            desired_replicas: Some(7),
            metrics_hint: Some("CPU utilization 85%".to_string()),
        };
        assert_eq!(summary.name, "cpu-hpa");
        assert_eq!(summary.min_replicas, Some(3));
        assert_eq!(
            summary.metrics_hint,
            Some("CPU utilization 85%".to_string())
        );
    }

    #[test]
    fn test_resource_quota_summary_empty() {
        let summary = ResourceQuotaSummary {
            name: String::new(),
            namespace: String::new(),
            hard: None,
            used: None,
        };
        assert!(summary.hard.is_none());
        assert!(summary.used.is_none());
    }

    #[test]
    fn test_resource_quota_summary_with_values() {
        let summary = ResourceQuotaSummary {
            name: "compute-quota".to_string(),
            namespace: "dev".to_string(),
            hard: Some(serde_json::json!({
                "requests.cpu": "10",
                "requests.memory": "16Gi",
                "limits.cpu": "20"
            })),
            used: Some(serde_json::json!({
                "requests.cpu": "5",
                "requests.memory": "8Gi",
                "limits.cpu": "12"
            })),
        };
        assert_eq!(summary.name, "compute-quota");
        assert!(summary.hard.is_some());
        assert!(summary.used.is_some());
    }

    #[test]
    fn test_limit_range_summary_zero() {
        let summary = LimitRangeSummary {
            name: "empty-limits".to_string(),
            namespace: "test".to_string(),
            limits_count: 0,
        };
        assert_eq!(summary.limits_count, 0);
    }

    #[test]
    fn test_crd_summary_empty_strings() {
        let summary = CrdSummary {
            name: String::new(),
            group: String::new(),
            kind: String::new(),
            plural: String::new(),
            scope: String::new(),
            versions: vec![],
        };
        assert!(summary.name.is_empty());
        assert!(summary.versions.is_empty());
    }

    #[test]
    fn test_crd_summary_multiple_versions() {
        let summary = CrdSummary {
            name: "certificates.cert-manager.io".to_string(),
            group: "cert-manager.io".to_string(),
            kind: "Certificate".to_string(),
            plural: "certificates".to_string(),
            scope: "Namespaced".to_string(),
            versions: vec![
                "v1".to_string(),
                "v1alpha2".to_string(),
                "v1alpha3".to_string(),
            ],
        };
        assert_eq!(summary.versions.len(), 3);
        assert_eq!(summary.versions[0], "v1");
    }

    #[test]
    fn test_vpa_api_status_not_installed() {
        let status = VpaApiStatus { installed: false };
        assert!(!status.installed);
    }

    // ─────────────────────────────────────────────
    // Query params tests
    // ─────────────────────────────────────────────

    #[test]
    fn test_ns_query_none() {
        let query = NsQuery { namespace: None };
        assert!(query.namespace.is_none());
    }

    #[test]
    fn test_custom_object_query_all_fields() {
        let query = CustomObjectQuery {
            group: "monitoring.coreos.com".to_string(),
            version: "v1".to_string(),
            plural: "servicemonitors".to_string(),
            kind: "ServiceMonitor".to_string(),
            namespace: Some("monitoring".to_string()),
        };
        assert_eq!(query.group, "monitoring.coreos.com");
        assert_eq!(query.version, "v1");
        assert_eq!(query.plural, "servicemonitors");
        assert_eq!(query.kind, "ServiceMonitor");
        assert_eq!(query.namespace, Some("monitoring".to_string()));
    }

    #[test]
    fn test_custom_object_query_no_namespace() {
        let query = CustomObjectQuery {
            group: "apiextensions.k8s.io".to_string(),
            version: "v1".to_string(),
            plural: "customresourcedefinitions".to_string(),
            kind: "CustomResourceDefinition".to_string(),
            namespace: None,
        };
        assert!(query.namespace.is_none());
    }

    // ─────────────────────────────────────────────
    // Helper function tests
    // ─────────────────────────────────────────────

    #[test]
    fn test_gvk_function() {
        let gvk_result = gvk("apps", "v1", "Deployment");
        assert_eq!(gvk_result.group, "apps");
        assert_eq!(gvk_result.version, "v1");
        assert_eq!(gvk_result.kind, "Deployment");
    }

    #[test]
    fn test_gvk_function_empty_group() {
        let gvk_result = gvk("", "v1", "Pod");
        assert_eq!(gvk_result.group, "");
        assert_eq!(gvk_result.version, "v1");
        assert_eq!(gvk_result.kind, "Pod");
    }

    #[test]
    fn test_ar_function() {
        let ar_result = ar("apps", "v1", "Deployment", "deployments");
        assert_eq!(ar_result.group, "apps");
        assert_eq!(ar_result.api_version, "apps/v1");
    }

    #[test]
    fn test_ar_function_core_api() {
        let ar_result = ar("", "v1", "Pod", "pods");
        assert_eq!(ar_result.group, "");
        assert_eq!(ar_result.api_version, "v1");
        assert_eq!(ar_result.kind, "Pod");
        assert_eq!(ar_result.plural, "pods");
    }

    // ─────────────────────────────────────────────
    // CustomObjectPayload tests
    // ─────────────────────────────────────────────

    #[test]
    fn test_custom_object_payload() {
        let payload = CustomObjectPayload {
            group: "stable.example.com".to_string(),
            version: "v1".to_string(),
            kind: "Crontab".to_string(),
        };
        assert_eq!(payload.group, "stable.example.com");
        assert_eq!(payload.kind, "Crontab");
    }

    // ─────────────────────────────────────────────
    // Edge cases and boundary tests
    // ─────────────────────────────────────────────

    #[test]
    fn test_hpa_summary_max_replicas_zero() {
        let summary = HpaSummary {
            name: "broken-hpa".to_string(),
            namespace: "default".to_string(),
            min_replicas: None,
            max_replicas: 0,
            target_ref: "—".to_string(),
            current_replicas: None,
            desired_replicas: None,
            metrics_hint: None,
        };
        assert_eq!(summary.max_replicas, 0);
    }

    #[test]
    fn test_crd_summary_cluster_scope() {
        let summary = CrdSummary {
            name: "clusterissuers.cert-manager.io".to_string(),
            group: "cert-manager.io".to_string(),
            kind: "ClusterIssuer".to_string(),
            plural: "clusterissuers".to_string(),
            scope: "Cluster".to_string(),
            versions: vec!["v1".to_string()],
        };
        assert_eq!(summary.scope, "Cluster");
    }
}
