//! Kubernetes StatefulSet API handlers
//!
//! Управление StatefulSet: list, get, delete, scale

use axum::{
    extract::{Path, Query, State},
    Json,
};
use k8s_openapi::api::apps::v1::StatefulSet;
use k8s_openapi::jiff::Timestamp;
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

// ============================================================================
// Types
// ============================================================================

/// Краткая информация о StatefulSet
#[derive(Debug, Serialize)]
pub struct StatefulSetSummary {
    pub name: String,
    pub namespace: String,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub current_replicas: i32,
    pub updated_replicas: i32,
    pub age: String,
}

/// Детальная информация о StatefulSet
#[derive(Debug, Serialize)]
pub struct StatefulSetDetail {
    pub name: String,
    pub namespace: String,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub current_replicas: i32,
    pub updated_replicas: i32,
    pub selector: BTreeMap<String, String>,
    pub template_labels: BTreeMap<String, String>,
    pub containers: Vec<ContainerInfo>,
    pub volume_claim_templates: Vec<VolumeClaimTemplate>,
    pub service_name: String,
    pub update_strategy: String,
    pub conditions: Vec<StatefulSetCondition>,
    pub created_at: Option<String>,
}

/// Информация о контейнере
#[derive(Debug, Serialize)]
pub struct ContainerInfo {
    pub name: String,
    pub image: Option<String>,
}

/// Шаблон PVC
#[derive(Debug, Serialize)]
pub struct VolumeClaimTemplate {
    pub name: String,
    pub access_modes: Vec<String>,
    pub storage_class: Option<String>,
    pub storage_size: Option<String>,
}

/// Условие StatefulSet
#[derive(Debug, Serialize)]
pub struct StatefulSetCondition {
    #[serde(rename = "type")]
    pub condition_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
}

/// Query параметры для списка StatefulSets
#[derive(Debug, Deserialize)]
pub struct StatefulSetListQuery {
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
    pub limit: Option<u32>,
}

/// Payload для scale операции StatefulSet
#[derive(Debug, Deserialize)]
pub struct ScaleStatefulSetPayload {
    pub replicas: i32,
}

/// Ответ на операцию StatefulSet
#[derive(Debug, Serialize)]
pub struct StatefulSetOperationResponse {
    pub message: String,
    pub name: String,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replicas: Option<i32>,
}

// ============================================================================
// API Handlers
// ============================================================================

/// Получить список StatefulSets
pub async fn list_statefulsets(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StatefulSetListQuery>,
) -> Result<Json<Vec<StatefulSetSummary>>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let namespace = query.namespace.as_deref().unwrap_or("default");
    let api: Api<StatefulSet> = Api::namespaced(client, namespace);

    let mut lp = ListParams::default();
    if let Some(selector) = query.label_selector {
        lp.label_selector = Some(selector);
    }
    if let Some(limit) = query.limit {
        lp.limit = Some(limit);
    }

    let sfs_list = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to list statefulsets: {}", e)))?;

    let statefulsets = sfs_list.items.iter().map(statefulset_summary).collect();

    Ok(Json(statefulsets))
}

/// Получить StatefulSet по имени
pub async fn get_statefulset(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<StatefulSetDetail>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<StatefulSet> = Api::namespaced(client, &namespace);

    let sf = api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("StatefulSet {} not found: {}", name, e)))?;

    Ok(Json(statefulset_detail(&sf)))
}

/// Удалить StatefulSet
pub async fn delete_statefulset(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<StatefulSetOperationResponse>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<StatefulSet> = Api::namespaced(client, &namespace);

    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to delete statefulset: {}", e)))?;

    Ok(Json(StatefulSetOperationResponse {
        message: format!("StatefulSet {} deleted", name),
        name,
        namespace,
        replicas: None,
    }))
}

/// Scale StatefulSet
pub async fn scale_statefulset(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<ScaleStatefulSetPayload>,
) -> Result<Json<StatefulSetOperationResponse>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<StatefulSet> = Api::namespaced(client, &namespace);

    let mut sf = api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("StatefulSet {} not found: {}", name, e)))?;

    if let Some(spec) = sf.spec.as_mut() {
        spec.replicas = Some(payload.replicas);
    }

    api.replace(&name, &PostParams::default(), &sf)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to scale statefulset: {}", e)))?;

    Ok(Json(StatefulSetOperationResponse {
        message: format!(
            "StatefulSet {} scaled to {} replicas",
            name, payload.replicas
        ),
        name,
        namespace,
        replicas: Some(payload.replicas),
    }))
}

/// Получить pod'ы, управляемые StatefulSet
pub async fn list_statefulset_pods(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<Vec<serde_json::Value>>> {
    use k8s_openapi::api::core::v1::Pod;

    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let sf_api: Api<StatefulSet> = Api::namespaced(client.clone(), &namespace);
    let pod_api: Api<Pod> = Api::namespaced(client, &namespace);

    let sf = sf_api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("StatefulSet {} not found: {}", name, e)))?;

    let selector = sf
        .spec
        .and_then(|s| s.selector.match_labels)
        .unwrap_or_default();

    let label_selector = selector
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(",");

    let lp = ListParams {
        label_selector: Some(label_selector),
        ..Default::default()
    };

    let pod_list = pod_api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to list pods: {}", e)))?;

    let pods = pod_list
        .items
        .iter()
        .map(|pod| {
            serde_json::json!({
                "name": pod.metadata.name.clone().unwrap_or_default(),
                "namespace": pod.metadata.namespace.clone().unwrap_or("default".to_string()),
                "status": pod.status.as_ref().and_then(|s| s.phase.clone()).unwrap_or_default(),
                "node": pod.spec.as_ref().and_then(|s| s.node_name.clone()),
            })
        })
        .collect();

    Ok(Json(pods))
}

// ============================================================================
// Helper Functions
// ============================================================================

fn statefulset_summary(sf: &StatefulSet) -> StatefulSetSummary {
    let name = sf.metadata.name.clone().unwrap_or_default();
    let namespace = sf
        .metadata
        .namespace
        .clone()
        .unwrap_or("default".to_string());

    let status = sf.status.as_ref();
    let spec = sf.spec.as_ref();

    let replicas = spec.and_then(|s| s.replicas).unwrap_or(1);
    let ready_replicas = status.and_then(|s| s.ready_replicas).unwrap_or(0);
    let current_replicas = status.and_then(|s| s.current_replicas).unwrap_or(0);
    let updated_replicas = status.and_then(|s| s.updated_replicas).unwrap_or(0);

    let age = sf
        .metadata
        .creation_timestamp
        .as_ref()
        .map(|ts| format_age(&ts.0))
        .unwrap_or_else(|| "unknown".to_string());

    StatefulSetSummary {
        name,
        namespace,
        replicas,
        ready_replicas,
        current_replicas,
        updated_replicas,
        age,
    }
}

fn statefulset_detail(sf: &StatefulSet) -> StatefulSetDetail {
    let status = sf.status.as_ref();
    let spec = sf.spec.as_ref();

    let selector = spec
        .and_then(|s| s.selector.match_labels.clone())
        .unwrap_or_default();

    let template_labels = spec
        .and_then(|s| s.template.metadata.as_ref())
        .and_then(|m| m.labels.clone())
        .unwrap_or_default();

    let containers = spec
        .and_then(|s| s.template.spec.as_ref())
        .map(|ps| {
            ps.containers
                .iter()
                .map(|c| ContainerInfo {
                    name: c.name.clone(),
                    image: c.image.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    let volume_claim_templates = spec
        .and_then(|s| s.volume_claim_templates.as_ref())
        .map(|templates| {
            templates
                .iter()
                .map(|t| {
                    let storage_class = t.spec.as_ref().and_then(|s| s.storage_class_name.clone());

                    let storage_size = t
                        .spec
                        .as_ref()
                        .and_then(|s| s.resources.as_ref())
                        .and_then(|r| r.requests.as_ref())
                        .and_then(|req| req.get("storage"))
                        .map(|q| q.0.clone());

                    VolumeClaimTemplate {
                        name: t.metadata.name.clone().unwrap_or_default(),
                        access_modes: t
                            .spec
                            .as_ref()
                            .and_then(|s| s.access_modes.as_ref())
                            .map(|modes| modes.iter().map(|m| m.to_string()).collect())
                            .unwrap_or_default(),
                        storage_class,
                        storage_size,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let service_name = spec.and_then(|s| s.service_name.clone()).unwrap_or_default();

    let update_strategy = spec
        .and_then(|s| s.update_strategy.as_ref())
        .and_then(|us| us.type_.as_ref())
        .cloned()
        .unwrap_or_else(|| "RollingUpdate".to_string());

    let conditions = status
        .and_then(|s| s.conditions.clone())
        .map(|conds| {
            conds
                .iter()
                .map(|c| StatefulSetCondition {
                    condition_type: c.type_.clone(),
                    status: c.status.clone(),
                    reason: c.reason.clone(),
                    message: c.message.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    StatefulSetDetail {
        name: sf.metadata.name.clone().unwrap_or_default(),
        namespace: sf
            .metadata
            .namespace
            .clone()
            .unwrap_or("default".to_string()),
        replicas: spec.and_then(|s| s.replicas).unwrap_or(1),
        ready_replicas: status.and_then(|s| s.ready_replicas).unwrap_or(0),
        current_replicas: status.and_then(|s| s.current_replicas).unwrap_or(0),
        updated_replicas: status.and_then(|s| s.updated_replicas).unwrap_or(0),
        selector,
        template_labels,
        containers,
        volume_claim_templates,
        service_name,
        update_strategy,
        conditions,
        created_at: sf.metadata.creation_timestamp.as_ref().map(|t| t.0.to_string()),
    }
}

fn format_age(time: &Timestamp) -> String {
    let now = Timestamp::now();
    let duration = now.duration_since(*time);
    let total_secs = duration.as_secs().abs();
    let days = total_secs / 86400;
    if days > 365 {
        format!("{}y", days / 365)
    } else if days > 30 {
        format!("{}d", days / 30)
    } else if days > 0 {
        format!("{}d", days)
    } else {
        let hours = total_secs / 3600;
        if hours > 0 {
            format!("{}h", hours)
        } else {
            let mins = total_secs / 60;
            if mins > 0 {
                format!("{}m", mins)
            } else {
                format!("{}s", total_secs)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ts(seconds_ago: i64) -> Timestamp {
        let now = Timestamp::now();
        let dur = k8s_openapi::jiff::SignedDuration::from_secs(seconds_ago);
        now.checked_sub(dur).unwrap()
    }

    #[test]
    fn test_format_age_seconds() {
        let age = format_age(&make_ts(30));
        assert_eq!(age, "30s");
    }

    #[test]
    fn test_format_age_minutes() {
        let age = format_age(&make_ts(120));
        assert_eq!(age, "2m");
    }

    #[test]
    fn test_format_age_hours() {
        let age = format_age(&make_ts(7200));
        assert_eq!(age, "2h");
    }

    #[test]
    fn test_format_age_days() {
        let age = format_age(&make_ts(172800));
        assert_eq!(age, "2d");
    }

    #[test]
    fn test_format_age_months() {
        let age = format_age(&make_ts(2592000)); // 30 days
        assert_eq!(age, "30d");
    }

    #[test]
    fn test_format_age_years() {
        let age = format_age(&make_ts(31536000 * 2)); // 2 years
        assert!(age.ends_with('y'));
    }

    #[test]
    fn test_statefulset_summary_struct() {
        let summary = StatefulSetSummary {
            name: "postgres".to_string(),
            namespace: "database".to_string(),
            replicas: 3,
            ready_replicas: 3,
            current_replicas: 3,
            updated_replicas: 3,
            age: "10d".to_string(),
        };
        assert_eq!(summary.name, "postgres");
        assert_eq!(summary.replicas, 3);
        assert_eq!(summary.ready_replicas, 3);
    }

    #[test]
    fn test_statefulset_detail_struct() {
        let detail = StatefulSetDetail {
            name: "redis".to_string(),
            namespace: "cache".to_string(),
            replicas: 5,
            ready_replicas: 5,
            current_replicas: 5,
            updated_replicas: 4,
            selector: BTreeMap::new(),
            template_labels: BTreeMap::new(),
            containers: vec![ContainerInfo {
                name: "redis".to_string(),
                image: Some("redis:7".to_string()),
            }],
            volume_claim_templates: vec![VolumeClaimTemplate {
                name: "data".to_string(),
                access_modes: vec!["ReadWriteOnce".to_string()],
                storage_class: Some("gp2".to_string()),
                storage_size: Some("10Gi".to_string()),
            }],
            service_name: "redis-headless".to_string(),
            update_strategy: "RollingUpdate".to_string(),
            conditions: vec![],
            created_at: Some(Timestamp::now().to_string()),
        };
        assert_eq!(detail.name, "redis");
        assert_eq!(detail.volume_claim_templates.len(), 1);
        assert_eq!(detail.service_name, "redis-headless");
        assert_eq!(detail.update_strategy, "RollingUpdate");
    }

    #[test]
    fn test_volume_claim_template() {
        let vct = VolumeClaimTemplate {
            name: "data".to_string(),
            access_modes: vec!["ReadWriteOnce".to_string(), "ReadOnlyMany".to_string()],
            storage_class: Some("standard".to_string()),
            storage_size: Some("50Gi".to_string()),
        };
        assert_eq!(vct.access_modes.len(), 2);
        assert!(vct.access_modes.contains(&"ReadWriteOnce".to_string()));
    }

    #[test]
    fn test_statefulset_condition() {
        let cond = StatefulSetCondition {
            condition_type: "Available".to_string(),
            status: "True".to_string(),
            reason: Some("MinimumReplicasAvailable".to_string()),
            message: None,
        };
        assert_eq!(cond.condition_type, "Available");
        assert_eq!(cond.status, "True");
        assert!(cond.reason.is_some());
    }

    #[test]
    fn test_scale_statefulset_payload() {
        let payload = ScaleStatefulSetPayload { replicas: 5 };
        assert_eq!(payload.replicas, 5);
    }

    #[test]
    fn test_statefulset_operation_response_with_replicas() {
        let resp = StatefulSetOperationResponse {
            message: "StatefulSet scaled".to_string(),
            name: "web".to_string(),
            namespace: "default".to_string(),
            replicas: Some(5),
        };
        assert_eq!(resp.replicas, Some(5));
        assert!(resp.message.contains("scaled"));
    }

    #[test]
    fn test_statefulset_operation_response_delete() {
        let resp = StatefulSetOperationResponse {
            message: "StatefulSet web deleted".to_string(),
            name: "web".to_string(),
            namespace: "default".to_string(),
            replicas: None,
        };
        assert!(resp.replicas.is_none());
    }

    #[test]
    fn test_statefulset_list_query_with_namespace() {
        let query = StatefulSetListQuery {
            namespace: Some("production".to_string()),
            label_selector: Some("tier=backend".to_string()),
            limit: Some(20),
        };
        assert_eq!(query.namespace, Some("production".to_string()));
        assert_eq!(query.limit, Some(20));
    }

    #[test]
    fn test_statefulset_list_query_empty() {
        let query = StatefulSetListQuery {
            namespace: None,
            label_selector: None,
            limit: None,
        };
        assert!(query.namespace.is_none());
        assert!(query.label_selector.is_none());
    }

    #[test]
    fn test_container_info_empty_image() {
        let info = ContainerInfo {
            name: "init-container".to_string(),
            image: None,
        };
        assert_eq!(info.name, "init-container");
        assert!(info.image.is_none());
    }

    #[test]
    fn test_format_age_zero() {
        let age = format_age(&make_ts(0));
        assert!(age.ends_with("s"));
    }
}
