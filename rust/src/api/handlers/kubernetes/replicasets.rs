//! Kubernetes ReplicaSet API handlers
//!
//! Управление ReplicaSet: list, get, delete

use axum::{
    Json,
    extract::{Path, Query, State},
};
use k8s_openapi::api::apps::v1::ReplicaSet;
use k8s_openapi::jiff::Timestamp;
use kube::api::{Api, DeleteParams, ListParams};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

// ============================================================================
// Types
// ============================================================================

/// Краткая информация о ReplicaSet
#[derive(Debug, Serialize)]
pub struct ReplicaSetSummary {
    pub name: String,
    pub namespace: String,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub available_replicas: i32,
    pub age: String,
    pub owner: Option<String>,
}

/// Детальная информация о ReplicaSet
#[derive(Debug, Serialize)]
pub struct ReplicaSetDetail {
    pub name: String,
    pub namespace: String,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub available_replicas: i32,
    pub selector: BTreeMap<String, String>,
    pub template_labels: BTreeMap<String, String>,
    pub containers: Vec<ContainerInfo>,
    pub owner_references: Vec<OwnerReference>,
    pub conditions: Vec<ReplicaSetCondition>,
    pub created_at: Option<String>,
}

/// Информация о контейнере
#[derive(Debug, Serialize)]
pub struct ContainerInfo {
    pub name: String,
    pub image: Option<String>,
}

/// Владелец ReplicaSet
#[derive(Debug, Serialize)]
pub struct OwnerReference {
    pub api_version: String,
    pub kind: String,
    pub name: String,
    pub uid: String,
}

/// Условие ReplicaSet
#[derive(Debug, Serialize)]
pub struct ReplicaSetCondition {
    #[serde(rename = "type")]
    pub condition_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
}

/// Query параметры для списка ReplicaSets
#[derive(Debug, Deserialize)]
pub struct ReplicaSetListQuery {
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
    pub limit: Option<u32>,
}

/// Ответ на операцию ReplicaSet
#[derive(Debug, Serialize)]
pub struct ReplicaSetOperationResponse {
    pub message: String,
    pub name: String,
    pub namespace: String,
}

// ============================================================================
// API Handlers
// ============================================================================

/// Получить список ReplicaSets
pub async fn list_replicasets(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ReplicaSetListQuery>,
) -> Result<Json<Vec<ReplicaSetSummary>>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let namespace = query.namespace.as_deref().unwrap_or("default");
    let api: Api<ReplicaSet> = Api::namespaced(client, namespace);

    let mut lp = ListParams::default();
    if let Some(selector) = query.label_selector {
        lp.label_selector = Some(selector);
    }
    if let Some(limit) = query.limit {
        lp.limit = Some(limit);
    }

    let rs_list = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to list replicasets: {}", e)))?;

    let replicasets = rs_list.items.iter().map(replicaset_summary).collect();

    Ok(Json(replicasets))
}

/// Получить ReplicaSet по имени
pub async fn get_replicaset(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<ReplicaSetDetail>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<ReplicaSet> = Api::namespaced(client, &namespace);

    let rs = api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("ReplicaSet {} not found: {}", name, e)))?;

    Ok(Json(replicaset_detail(&rs)))
}

/// Удалить ReplicaSet
pub async fn delete_replicaset(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<ReplicaSetOperationResponse>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<ReplicaSet> = Api::namespaced(client, &namespace);

    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to delete replicaset: {}", e)))?;

    Ok(Json(ReplicaSetOperationResponse {
        message: format!("ReplicaSet {} deleted", name),
        name,
        namespace,
    }))
}

/// Получить pod'ы, управляемые ReplicaSet
pub async fn list_replicaset_pods(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<Vec<serde_json::Value>>> {
    use k8s_openapi::api::core::v1::Pod;

    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let rs_api: Api<ReplicaSet> = Api::namespaced(client.clone(), &namespace);
    let pod_api: Api<Pod> = Api::namespaced(client, &namespace);

    let rs = rs_api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("ReplicaSet {} not found: {}", name, e)))?;

    let selector = rs
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
                "ip": pod.status.as_ref().and_then(|s| s.pod_ip.clone()),
            })
        })
        .collect();

    Ok(Json(pods))
}

// ============================================================================
// Helper Functions
// ============================================================================

fn replicaset_summary(rs: &ReplicaSet) -> ReplicaSetSummary {
    let name = rs.metadata.name.clone().unwrap_or_default();
    let namespace = rs
        .metadata
        .namespace
        .clone()
        .unwrap_or("default".to_string());

    let status = rs.status.as_ref();
    let spec = rs.spec.as_ref();

    let replicas = spec.and_then(|s| s.replicas).unwrap_or(1);
    let ready_replicas = status.and_then(|s| s.ready_replicas).unwrap_or(0);
    let available_replicas = status.and_then(|s| s.available_replicas).unwrap_or(0);

    let owner = rs
        .metadata
        .owner_references
        .as_ref()
        .and_then(|refs| refs.first())
        .map(|r| r.name.clone());

    let age = rs
        .metadata
        .creation_timestamp
        .as_ref()
        .map(|ts| format_age(&ts.0))
        .unwrap_or_else(|| "unknown".to_string());

    ReplicaSetSummary {
        name,
        namespace,
        replicas,
        ready_replicas,
        available_replicas,
        age,
        owner,
    }
}

fn replicaset_detail(rs: &ReplicaSet) -> ReplicaSetDetail {
    let status = rs.status.as_ref();
    let spec = rs.spec.as_ref();

    let selector = spec
        .and_then(|s| s.selector.match_labels.clone())
        .unwrap_or_default();

    let template_labels = spec
        .and_then(|s| s.template.as_ref())
        .and_then(|t| t.metadata.as_ref())
        .and_then(|m| m.labels.clone())
        .unwrap_or_default();

    let containers = spec
        .and_then(|s| s.template.as_ref())
        .and_then(|t| t.spec.as_ref())
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

    let owner_references = rs
        .metadata
        .owner_references
        .as_ref()
        .map(|refs| {
            refs.iter()
                .map(|r| OwnerReference {
                    api_version: r.api_version.clone(),
                    kind: r.kind.clone(),
                    name: r.name.clone(),
                    uid: r.uid.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    let conditions = status
        .and_then(|s| s.conditions.clone())
        .map(|conds| {
            conds
                .iter()
                .map(|c| ReplicaSetCondition {
                    condition_type: c.type_.clone(),
                    status: c.status.clone(),
                    reason: c.reason.clone(),
                    message: c.message.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    ReplicaSetDetail {
        name: rs.metadata.name.clone().unwrap_or_default(),
        namespace: rs
            .metadata
            .namespace
            .clone()
            .unwrap_or("default".to_string()),
        replicas: spec.and_then(|s| s.replicas).unwrap_or(1),
        ready_replicas: status.and_then(|s| s.ready_replicas).unwrap_or(0),
        available_replicas: status.and_then(|s| s.available_replicas).unwrap_or(0),
        selector,
        template_labels,
        containers,
        owner_references,
        conditions,
        created_at: rs
            .metadata
            .creation_timestamp
            .as_ref()
            .map(|t| t.0.to_string()),
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
        assert_eq!(format_age(&make_ts(15)), "15s");
    }

    #[test]
    fn test_format_age_minutes() {
        assert_eq!(format_age(&make_ts(300)), "5m");
    }

    #[test]
    fn test_format_age_hours() {
        assert_eq!(format_age(&make_ts(14400)), "4h");
    }

    #[test]
    fn test_format_age_days() {
        assert_eq!(format_age(&make_ts(432000)), "5d");
    }

    #[test]
    fn test_format_age_months() {
        let age = format_age(&make_ts(2592000)); // 30 days
        assert!(age.ends_with('d'));
    }

    #[test]
    fn test_format_age_years() {
        let age = format_age(&make_ts(63072000)); // 2 years
        assert!(age.ends_with('y'));
    }

    #[test]
    fn test_replicaset_summary_struct_fields() {
        let summary = ReplicaSetSummary {
            name: "my-rs".to_string(),
            namespace: "production".to_string(),
            replicas: 3,
            ready_replicas: 2,
            available_replicas: 2,
            age: "5d".to_string(),
            owner: Some("my-deployment".to_string()),
        };
        assert_eq!(summary.name, "my-rs");
        assert_eq!(summary.namespace, "production");
        assert_eq!(summary.replicas, 3);
        assert_eq!(summary.ready_replicas, 2);
        assert_eq!(summary.available_replicas, 2);
        assert_eq!(summary.age, "5d");
        assert_eq!(summary.owner, Some("my-deployment".to_string()));
    }

    #[test]
    fn test_replicaset_summary_owner_none() {
        let summary = ReplicaSetSummary {
            name: "standalone-rs".to_string(),
            namespace: "default".to_string(),
            replicas: 1,
            ready_replicas: 1,
            available_replicas: 1,
            age: "1h".to_string(),
            owner: None,
        };
        assert!(summary.owner.is_none());
    }

    #[test]
    fn test_replicaset_detail_struct() {
        let mut selector = BTreeMap::new();
        selector.insert("app".to_string(), "web".to_string());
        let detail = ReplicaSetDetail {
            name: "web-rs".to_string(),
            namespace: "default".to_string(),
            replicas: 3,
            ready_replicas: 3,
            available_replicas: 3,
            selector: selector.clone(),
            template_labels: selector,
            containers: vec![ContainerInfo {
                name: "nginx".to_string(),
                image: Some("nginx:1.21".to_string()),
            }],
            owner_references: vec![OwnerReference {
                api_version: "apps/v1".to_string(),
                kind: "Deployment".to_string(),
                name: "web".to_string(),
                uid: "uid-123".to_string(),
            }],
            conditions: vec![ReplicaSetCondition {
                condition_type: "ReplicaFailure".to_string(),
                status: "False".to_string(),
                reason: None,
                message: None,
            }],
            created_at: Some(Timestamp::now().to_string()),
        };
        assert_eq!(detail.name, "web-rs");
        assert_eq!(detail.containers.len(), 1);
        assert_eq!(detail.containers[0].image, Some("nginx:1.21".to_string()));
        assert_eq!(detail.owner_references.len(), 1);
        assert_eq!(detail.conditions.len(), 1);
    }

    #[test]
    fn test_container_info_without_image() {
        let info = ContainerInfo {
            name: "sidecar".to_string(),
            image: None,
        };
        assert_eq!(info.name, "sidecar");
        assert!(info.image.is_none());
    }

    #[test]
    fn test_owner_reference_struct() {
        let owner = OwnerReference {
            api_version: "apps/v1".to_string(),
            kind: "Deployment".to_string(),
            name: "my-deploy".to_string(),
            uid: "uid-456".to_string(),
        };
        assert_eq!(owner.kind, "Deployment");
        assert_eq!(owner.api_version, "apps/v1");
    }

    #[test]
    fn test_replicaset_condition_with_reason() {
        let cond = ReplicaSetCondition {
            condition_type: "ReplicaFailure".to_string(),
            status: "True".to_string(),
            reason: Some("FailedCreate".to_string()),
            message: Some("Error creating pod".to_string()),
        };
        assert_eq!(cond.condition_type, "ReplicaFailure");
        assert_eq!(cond.status, "True");
        assert!(cond.reason.is_some());
        assert!(cond.message.is_some());
    }

    #[test]
    fn test_replicaset_list_query_all_none() {
        let query = ReplicaSetListQuery {
            namespace: None,
            label_selector: None,
            limit: None,
        };
        assert!(query.namespace.is_none());
        assert!(query.label_selector.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_replicaset_list_query_with_params() {
        let query = ReplicaSetListQuery {
            namespace: Some("kube-system".to_string()),
            label_selector: Some("app=coredns".to_string()),
            limit: Some(50),
        };
        assert_eq!(query.namespace, Some("kube-system".to_string()));
        assert_eq!(query.label_selector, Some("app=coredns".to_string()));
        assert_eq!(query.limit, Some(50));
    }

    #[test]
    fn test_replicaset_operation_response() {
        let resp = ReplicaSetOperationResponse {
            message: "ReplicaSet test-rs deleted".to_string(),
            name: "test-rs".to_string(),
            namespace: "default".to_string(),
        };
        assert!(resp.message.contains("deleted"));
        assert_eq!(resp.name, "test-rs");
    }

    #[test]
    fn test_format_age_zero_seconds() {
        let age = format_age(&make_ts(0));
        assert!(age.ends_with("s"));
    }

    #[test]
    fn test_format_age_exactly_one_minute() {
        let age = format_age(&make_ts(60));
        assert_eq!(age, "1m");
    }

    #[test]
    fn test_format_age_exactly_one_hour() {
        let age = format_age(&make_ts(3600));
        assert_eq!(age, "1h");
    }
}
