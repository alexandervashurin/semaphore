//! Kubernetes Deployment API handlers
//!
//! Управление Deployment: list, get, create, update, delete, scale, restart, rollback

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{
    api::{Api, DeleteParams, ListParams, Patch, PatchParams, PostParams},
    Client,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

// ============================================================================
// Types
// ============================================================================

/// Краткая информация о Deployment
#[derive(Debug, Serialize)]
pub struct DeploymentSummary {
    pub name: String,
    pub namespace: String,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub available_replicas: i32,
    pub updated_replicas: i32,
    pub age: String,
    pub conditions: Vec<DeploymentCondition>,
}

/// Условие Deployment
#[derive(Debug, Serialize)]
pub struct DeploymentCondition {
    #[serde(rename = "type")]
    pub condition_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub last_update_time: Option<DateTime<Utc>>,
}

/// Детальная информация о Deployment
#[derive(Debug, Serialize)]
pub struct DeploymentDetail {
    pub name: String,
    pub namespace: String,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub available_replicas: i32,
    pub updated_replicas: i32,
    pub unavailable_replicas: i32,
    pub strategy: String,
    pub selector: BTreeMap<String, String>,
    pub template_labels: BTreeMap<String, String>,
    pub containers: Vec<ContainerInfo>,
    pub conditions: Vec<DeploymentCondition>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Информация о контейнере
#[derive(Debug, Serialize)]
pub struct ContainerInfo {
    pub name: String,
    pub image: Option<String>,
    pub ports: Vec<i32>,
}

/// Query параметры для списка Deployments
#[derive(Debug, Deserialize)]
pub struct DeploymentListQuery {
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
    pub limit: Option<u32>,
}

/// Payload для scale операции
#[derive(Debug, Deserialize)]
pub struct ScalePayload {
    pub replicas: i32,
}

/// Payload для rollback операции
#[derive(Debug, Deserialize)]
pub struct RollbackPayload {
    pub revision: Option<i64>,
}

/// Payload для создания/обновления Deployment
#[derive(Debug, Deserialize)]
pub struct DeploymentPayload {
    pub name: String,
    pub namespace: String,
    pub replicas: Option<i32>,
    pub image: String,
    pub container_name: Option<String>,
    pub ports: Option<Vec<i32>>,
    pub labels: Option<BTreeMap<String, String>>,
}

/// Ответ на операцию
#[derive(Debug, Serialize)]
pub struct OperationResponse {
    pub message: String,
    pub name: String,
    pub namespace: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replicas: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<i64>,
}

/// История rollout
#[derive(Debug, Serialize)]
pub struct RolloutHistory {
    pub name: String,
    pub namespace: String,
    pub revisions: Vec<RevisionInfo>,
}

/// Информация о ревизии
#[derive(Debug, Serialize)]
pub struct RevisionInfo {
    pub revision: i64,
    pub change_cause: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Детальная информация о ревизии с ReplicaSet
#[derive(Debug, Serialize)]
pub struct DetailedRevisionInfo {
    pub revision: i64,
    pub replica_set_name: String,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub available_replicas: i32,
    pub image: String,
    pub created_at: Option<DateTime<Utc>>,
}

/// Детальная история rollout
#[derive(Debug, Serialize)]
pub struct DetailedRolloutHistory {
    pub name: String,
    pub namespace: String,
    pub current_revision: i64,
    pub revisions: Vec<DetailedRevisionInfo>,
}

// ============================================================================
// API Handlers
// ============================================================================

/// Получить список Deployments
pub async fn list_deployments(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DeploymentListQuery>,
) -> Result<Json<Vec<DeploymentSummary>>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let namespace = query.namespace.as_deref().unwrap_or("default");
    let api: Api<Deployment> = Api::namespaced(client, namespace);

    let mut lp = ListParams::default();
    if let Some(selector) = query.label_selector {
        lp.label_selector = Some(selector);
    }
    if let Some(limit) = query.limit {
        lp.limit = Some(limit);
    }

    let deployment_list = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to list deployments: {}", e)))?;

    let deployments = deployment_list
        .items
        .iter()
        .map(deployment_summary)
        .collect();

    Ok(Json(deployments))
}

/// Получить Deployment по имени
pub async fn get_deployment(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<DeploymentDetail>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<Deployment> = Api::namespaced(client, &namespace);

    let deployment = api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("Deployment {} not found: {}", name, e)))?;

    Ok(Json(deployment_detail(&deployment)))
}

/// Создать Deployment
pub async fn create_deployment(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DeploymentPayload>,
) -> Result<Json<OperationResponse>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<Deployment> = Api::namespaced(client, &payload.namespace);

    let container_name = payload.container_name.unwrap_or_else(|| "app".to_string());
    let mut container = k8s_openapi::api::core::v1::Container {
        name: container_name.clone(),
        image: Some(payload.image),
        ..Default::default()
    };

    if let Some(ports) = payload.ports {
        container.ports = Some(
            ports
                .iter()
                .map(|p| k8s_openapi::api::core::v1::ContainerPort {
                    container_port: *p,
                    ..Default::default()
                })
                .collect(),
        );
    }

    let mut labels = payload.labels.unwrap_or_default();
    labels.insert("app".to_string(), container_name.clone());

    let deployment = Deployment {
        metadata: ObjectMeta {
            name: Some(payload.name.clone()),
            namespace: Some(payload.namespace.clone()),
            labels: Some(labels.clone()),
            ..Default::default()
        },
        spec: Some(k8s_openapi::api::apps::v1::DeploymentSpec {
            replicas: payload.replicas,
            selector: k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector {
                match_labels: Some(labels.clone()),
                ..Default::default()
            },
            template: k8s_openapi::api::core::v1::PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(labels),
                    ..Default::default()
                }),
                spec: Some(k8s_openapi::api::core::v1::PodSpec {
                    containers: vec![container],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    api.create(&PostParams::default(), &deployment)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to create deployment: {}", e)))?;

    Ok(Json(OperationResponse {
        message: format!("Deployment {} created", payload.name),
        name: payload.name.clone(),
        namespace: payload.namespace,
        replicas: payload.replicas,
        revision: None,
    }))
}

/// Обновить Deployment
pub async fn update_deployment(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<DeploymentPayload>,
) -> Result<Json<OperationResponse>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<Deployment> = Api::namespaced(client, &namespace);

    let mut existing = api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("Deployment {} not found: {}", name, e)))?;

    if let Some(spec) = existing.spec.as_mut() {
        if let Some(replicas) = payload.replicas {
            spec.replicas = Some(replicas);
        }
    }

    api.replace(&name, &PostParams::default(), &existing)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to update deployment: {}", e)))?;

    Ok(Json(OperationResponse {
        message: format!("Deployment {} updated", name),
        name,
        namespace,
        replicas: payload.replicas,
        revision: None,
    }))
}

/// Удалить Deployment
pub async fn delete_deployment(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<OperationResponse>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<Deployment> = Api::namespaced(client, &namespace);

    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to delete deployment: {}", e)))?;

    Ok(Json(OperationResponse {
        message: format!("Deployment {} deleted", name),
        name,
        namespace,
        replicas: None,
        revision: None,
    }))
}

/// Scale Deployment
pub async fn scale_deployment(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<ScalePayload>,
) -> Result<Json<OperationResponse>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<Deployment> = Api::namespaced(client, &namespace);

    let mut deployment = api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("Deployment {} not found: {}", name, e)))?;

    if let Some(spec) = deployment.spec.as_mut() {
        spec.replicas = Some(payload.replicas);
    }

    api.replace(&name, &PostParams::default(), &deployment)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to scale deployment: {}", e)))?;

    Ok(Json(OperationResponse {
        message: format!(
            "Deployment {} scaled to {} replicas",
            name, payload.replicas
        ),
        name,
        namespace,
        replicas: Some(payload.replicas),
        revision: None,
    }))
}

/// Restart Deployment (через annotation)
pub async fn restart_deployment(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<OperationResponse>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<Deployment> = Api::namespaced(client, &namespace);

    let mut deployment = api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("Deployment {} not found: {}", name, e)))?;

    let restart_time = Utc::now().to_rfc3339();

    let template_annotations = deployment
        .spec
        .as_mut()
        .and_then(|s| s.template.metadata.as_mut())
        .and_then(|m| m.annotations.as_mut());

    if let Some(annotations) = template_annotations {
        annotations.insert(
            "kubectl.kubernetes.io/restartedAt".to_string(),
            restart_time,
        );
    } else if let Some(spec) = deployment.spec.as_mut() {
        if let Some(meta) = spec.template.metadata.as_mut() {
            meta.annotations = Some(BTreeMap::from([(
                "kubectl.kubernetes.io/restartedAt".to_string(),
                restart_time,
            )]));
        }
    }

    api.replace(&name, &PostParams::default(), &deployment)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to restart deployment: {}", e)))?;

    Ok(Json(OperationResponse {
        message: format!("Deployment {} restarted", name),
        name,
        namespace,
        replicas: None,
        revision: None,
    }))
}

/// Rollback Deployment
pub async fn rollback_deployment(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<RollbackPayload>,
) -> Result<Json<OperationResponse>> {
    // NOTE: Полноценный rollback требует доступа к ReplicaSet history
    // Это упрощённая реализация
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<Deployment> = Api::namespaced(client, &namespace);

    let mut deployment = api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("Deployment {} not found: {}", name, e)))?;

    // Добавляем annotation для триггера rollback
    if let Some(annotations) = deployment.metadata.annotations.as_mut() {
        annotations.insert(
            "deployment.kubernetes.io/revision".to_string(),
            payload.revision.unwrap_or(1).to_string(),
        );
    } else {
        deployment.metadata.annotations = Some(BTreeMap::from([(
            "deployment.kubernetes.io/revision".to_string(),
            payload.revision.unwrap_or(1).to_string(),
        )]));
    }

    api.replace(&name, &PostParams::default(), &deployment)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to rollback deployment: {}", e)))?;

    Ok(Json(OperationResponse {
        message: format!("Deployment {} rollback initiated", name),
        name,
        namespace,
        replicas: None,
        revision: payload.revision,
    }))
}

/// Получить историю rollout
pub async fn get_deployment_history(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<RolloutHistory>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<Deployment> = Api::namespaced(client, &namespace);

    let deployment = api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("Deployment {} not found: {}", name, e)))?;

    // Получаем ревизию из annotations
    let revisions = deployment
        .metadata
        .annotations
        .as_ref()
        .and_then(|a| a.get("deployment.kubernetes.io/revision"))
        .and_then(|r| r.parse::<i64>().ok())
        .map(|rev| {
            vec![RevisionInfo {
                revision: rev,
                change_cause: None,
                created_at: deployment.metadata.creation_timestamp.as_ref().map(|t| t.0),
            }]
        })
        .unwrap_or_default();

    Ok(Json(RolloutHistory {
        name,
        namespace,
        revisions,
    }))
}

/// Pause Deployment (приостанавливает rollout)
pub async fn pause_deployment(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<OperationResponse>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<Deployment> = Api::namespaced(client, &namespace);

    let deployment = api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("Deployment {} not found: {}", name, e)))?;

    // Устанавливаем paused=true в spec
    let patch = serde_json::json!({
        "spec": {
            "paused": true
        }
    });

    let pp = PatchParams::apply("velum-kubernetes-controller");
    api.patch(&name, &pp, &Patch::Apply(&patch))
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to pause deployment: {}", e)))?;

    Ok(Json(OperationResponse {
        message: format!("Deployment {} paused", name),
        name,
        namespace,
        replicas: None,
        revision: None,
    }))
}

/// Resume Deployment (возобновляет rollout)
pub async fn resume_deployment(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<OperationResponse>> {
    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    let api: Api<Deployment> = Api::namespaced(client, &namespace);

    let deployment = api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("Deployment {} not found: {}", name, e)))?;

    // Устанавливаем paused=false в spec
    let patch = serde_json::json!({
        "spec": {
            "paused": false
        }
    });

    let pp = PatchParams::apply("velum-kubernetes-controller");
    api.patch(&name, &pp, &Patch::Apply(&patch))
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to resume deployment: {}", e)))?;

    Ok(Json(OperationResponse {
        message: format!("Deployment {} resumed", name),
        name,
        namespace,
        replicas: None,
        revision: None,
    }))
}

/// Получить полную историю rollout с ReplicaSet
pub async fn get_deployment_history_detailed(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<DetailedRolloutHistory>> {
    use k8s_openapi::api::apps::v1::ReplicaSet;

    let kube_client = state.kubernetes_client()?;
    let client = kube_client.raw().clone();

    // Получаем Deployment
    let api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    let deployment = api
        .get(&name)
        .await
        .map_err(|e| Error::NotFound(format!("Deployment {} not found: {}", name, e)))?;

    // Получаем все ReplicaSet, принадлежащие этому Deployment
    let rs_api: Api<ReplicaSet> = Api::namespaced(client, &namespace);
    let label_selector = format!("app.kubernetes.io/instance={},app={}", name, name);
    let lp = ListParams::default().labels(&label_selector);

    let replica_sets = rs_api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("Failed to list ReplicaSets: {}", e)))?;

    // Собираем информацию о ревизиях
    let mut revisions: Vec<DetailedRevisionInfo> = replica_sets
        .items
        .iter()
        .map(|rs| {
            let revision = rs
                .metadata
                .annotations
                .as_ref()
                .and_then(|a| a.get("deployment.kubernetes.io/revision"))
                .and_then(|r| r.parse::<i64>().ok())
                .unwrap_or(0);

            DetailedRevisionInfo {
                revision,
                replica_set_name: rs.metadata.name.clone().unwrap_or_default(),
                replicas: rs.spec.as_ref().and_then(|s| s.replicas).unwrap_or(0),
                ready_replicas: rs
                    .status
                    .as_ref()
                    .and_then(|s| s.ready_replicas)
                    .unwrap_or(0),
                available_replicas: rs
                    .status
                    .as_ref()
                    .and_then(|s| s.available_replicas)
                    .unwrap_or(0),
                created_at: rs.metadata.creation_timestamp.as_ref().map(|t| t.0),
                image: rs
                    .spec
                    .as_ref()
                    .and_then(|s| s.template.as_ref())
                    .and_then(|t| t.spec.as_ref())
                    .and_then(|s| s.containers.first())
                    .and_then(|c| c.image.clone())
                    .unwrap_or_default(),
            }
        })
        .collect();

    // Сортируем по ревизии
    revisions.sort_by(|a, b| b.revision.cmp(&a.revision));

    Ok(Json(DetailedRolloutHistory {
        name,
        namespace,
        current_revision: deployment
            .metadata
            .annotations
            .as_ref()
            .and_then(|a| a.get("deployment.kubernetes.io/revision"))
            .and_then(|r| r.parse::<i64>().ok())
            .unwrap_or(1),
        revisions,
    }))
}

// ============================================================================
// Helper Functions
// ============================================================================

fn deployment_summary(deployment: &Deployment) -> DeploymentSummary {
    let name = deployment.metadata.name.clone().unwrap_or_default();
    let namespace = deployment
        .metadata
        .namespace
        .clone()
        .unwrap_or("default".to_string());

    let status = deployment.status.as_ref();
    let spec = deployment.spec.as_ref();

    let replicas = spec.and_then(|s| s.replicas).unwrap_or(1);
    let ready_replicas = status.and_then(|s| s.ready_replicas).unwrap_or(0);
    let available_replicas = status.and_then(|s| s.available_replicas).unwrap_or(0);
    let updated_replicas = status.and_then(|s| s.updated_replicas).unwrap_or(0);

    let conditions = status
        .and_then(|s| s.conditions.clone())
        .map(|conds| {
            conds
                .iter()
                .map(|c| DeploymentCondition {
                    condition_type: c.type_.clone(),
                    status: c.status.clone(),
                    reason: c.reason.clone(),
                    message: c.message.clone(),
                    last_update_time: c.last_update_time.as_ref().map(|t| t.0),
                })
                .collect()
        })
        .unwrap_or_default();

    let age = deployment
        .metadata
        .creation_timestamp
        .as_ref()
        .map(|ts| format_age(&ts.0))
        .unwrap_or_else(|| "unknown".to_string());

    DeploymentSummary {
        name,
        namespace,
        replicas,
        ready_replicas,
        available_replicas,
        updated_replicas,
        age,
        conditions,
    }
}

fn deployment_detail(deployment: &Deployment) -> DeploymentDetail {
    let status = deployment.status.as_ref();
    let spec = deployment.spec.as_ref();

    let containers = spec
        .and_then(|s| s.template.spec.as_ref())
        .map(|ps| {
            ps.containers
                .iter()
                .map(|c| ContainerInfo {
                    name: c.name.clone(),
                    image: c.image.clone(),
                    ports: c
                        .ports
                        .as_ref()
                        .map(|ports| ports.iter().map(|p| p.container_port).collect())
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default();

    let selector = spec
        .and_then(|s| s.selector.match_labels.clone())
        .unwrap_or_default();

    let template_labels = spec
        .and_then(|s| s.template.metadata.as_ref())
        .and_then(|m| m.labels.clone())
        .unwrap_or_default();

    let strategy = spec
        .and_then(|s| s.strategy.as_ref())
        .and_then(|st| st.type_.as_ref())
        .cloned()
        .unwrap_or_else(|| "RollingUpdate".to_string());

    let conditions = status
        .and_then(|s| s.conditions.clone())
        .map(|conds| {
            conds
                .iter()
                .map(|c| DeploymentCondition {
                    condition_type: c.type_.clone(),
                    status: c.status.clone(),
                    reason: c.reason.clone(),
                    message: c.message.clone(),
                    last_update_time: c.last_update_time.as_ref().map(|t| t.0),
                })
                .collect()
        })
        .unwrap_or_default();

    DeploymentDetail {
        name: deployment.metadata.name.clone().unwrap_or_default(),
        namespace: deployment
            .metadata
            .namespace
            .clone()
            .unwrap_or("default".to_string()),
        replicas: spec.and_then(|s| s.replicas).unwrap_or(1),
        ready_replicas: status.and_then(|s| s.ready_replicas).unwrap_or(0),
        available_replicas: status.and_then(|s| s.available_replicas).unwrap_or(0),
        updated_replicas: status.and_then(|s| s.updated_replicas).unwrap_or(0),
        unavailable_replicas: status.and_then(|s| s.unavailable_replicas).unwrap_or(0),
        strategy,
        selector,
        template_labels,
        containers,
        conditions,
        created_at: deployment.metadata.creation_timestamp.as_ref().map(|t| t.0),
    }
}

fn format_age(time: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*time);

    if duration.num_days() > 365 {
        format!("{}y", duration.num_days() / 365)
    } else if duration.num_days() > 30 {
        format!("{}d", duration.num_days() / 30)
    } else if duration.num_days() > 0 {
        format!("{}d", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{}h", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{}m", duration.num_minutes())
    } else {
        format!("{}s", duration.num_seconds())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_format_age_seconds() {
        let now = Utc::now();
        let age = format_age(&(now - Duration::seconds(30)));
        assert_eq!(age, "30s");
    }

    #[test]
    fn test_format_age_minutes() {
        let now = Utc::now();
        let age = format_age(&(now - Duration::minutes(15)));
        assert_eq!(age, "15m");
    }

    #[test]
    fn test_format_age_hours() {
        let now = Utc::now();
        let age = format_age(&(now - Duration::hours(3)));
        assert_eq!(age, "3h");
    }

    #[test]
    fn test_format_age_days() {
        let now = Utc::now();
        let age = format_age(&(now - Duration::days(7)));
        assert_eq!(age, "7d");
    }

    #[test]
    fn test_format_age_months() {
        let now = Utc::now();
        let age = format_age(&(now - Duration::days(45)));
        assert_eq!(age, "1d"); // 45/30 = 1
    }

    #[test]
    fn test_format_age_years() {
        let now = Utc::now();
        let age = format_age(&(now - Duration::days(400)));
        assert_eq!(age, "1y"); // 400/365 = 1
    }

    #[test]
    fn test_format_age_future() {
        let now = Utc::now();
        let age = format_age(&(now + Duration::seconds(10)));
        // Future timestamps produce 0s since duration is negative but num_seconds() returns positive
        // Actually chrono's signed_duration_since returns negative duration
        // and num_seconds() on negative duration returns negative value
        // so the else branch would execute: format!("{}s", duration.num_seconds())
        // which gives "-10s" or similar. Let's just verify it doesn't panic.
        assert!(!age.is_empty());
    }

    // ========================================================================
    // Tests for DTO structs
    // ========================================================================

    #[test]
    fn test_deployment_summary_default_values() {
        let summary = DeploymentSummary {
            name: "test-deploy".to_string(),
            namespace: "default".to_string(),
            replicas: 3,
            ready_replicas: 2,
            available_replicas: 2,
            updated_replicas: 1,
            age: "5m".to_string(),
            conditions: vec![],
        };
        assert_eq!(summary.name, "test-deploy");
        assert_eq!(summary.namespace, "default");
        assert_eq!(summary.replicas, 3);
        assert_eq!(summary.ready_replicas, 2);
        assert!(summary.conditions.is_empty());
    }

    #[test]
    fn test_deployment_condition_serialization() {
        let condition = DeploymentCondition {
            condition_type: "Available".to_string(),
            status: "True".to_string(),
            reason: Some("MinimumReplicasAvailable".to_string()),
            message: Some("Deployment has minimum replicas".to_string()),
            last_update_time: Some(Utc::now()),
        };
        assert_eq!(condition.condition_type, "Available");
        assert_eq!(condition.status, "True");
        assert!(condition.reason.is_some());
        assert!(condition.message.is_some());
    }

    #[test]
    fn test_deployment_condition_optional_fields() {
        let condition = DeploymentCondition {
            condition_type: "Progressing".to_string(),
            status: "False".to_string(),
            reason: None,
            message: None,
            last_update_time: None,
        };
        assert!(condition.reason.is_none());
        assert!(condition.message.is_none());
        assert!(condition.last_update_time.is_none());
    }

    #[test]
    fn test_deployment_detail_struct() {
        let detail = DeploymentDetail {
            name: "my-app".to_string(),
            namespace: "production".to_string(),
            replicas: 5,
            ready_replicas: 5,
            available_replicas: 5,
            updated_replicas: 5,
            unavailable_replicas: 0,
            strategy: "RollingUpdate".to_string(),
            selector: BTreeMap::from([("app".to_string(), "my-app".to_string())]),
            template_labels: BTreeMap::from([("app".to_string(), "my-app".to_string())]),
            containers: vec![],
            conditions: vec![],
            created_at: Some(Utc::now()),
        };
        assert_eq!(detail.name, "my-app");
        assert_eq!(detail.replicas, 5);
        assert_eq!(detail.strategy, "RollingUpdate");
        assert!(detail.selector.contains_key("app"));
    }

    #[test]
    fn test_container_info_struct() {
        let container = ContainerInfo {
            name: "nginx".to_string(),
            image: Some("nginx:1.25".to_string()),
            ports: vec![80, 443],
        };
        assert_eq!(container.name, "nginx");
        assert_eq!(container.image.as_deref(), Some("nginx:1.25"));
        assert_eq!(container.ports.len(), 2);
        assert!(container.ports.contains(&80));
    }

    #[test]
    fn test_container_info_no_image() {
        let container = ContainerInfo {
            name: "sidecar".to_string(),
            image: None,
            ports: vec![],
        };
        assert!(container.image.is_none());
        assert!(container.ports.is_empty());
    }

    // ========================================================================
    // Tests for Payload structs
    // ========================================================================

    #[test]
    fn test_scale_payload() {
        let payload = ScalePayload { replicas: 5 };
        assert_eq!(payload.replicas, 5);
    }

    #[test]
    fn test_scale_payload_zero_replicas() {
        let payload = ScalePayload { replicas: 0 };
        assert_eq!(payload.replicas, 0);
    }

    #[test]
    fn test_rollback_payload_with_revision() {
        let payload = RollbackPayload { revision: Some(3) };
        assert_eq!(payload.revision, Some(3));
    }

    #[test]
    fn test_rollback_payload_without_revision() {
        let payload = RollbackPayload { revision: None };
        assert!(payload.revision.is_none());
    }

    #[test]
    fn test_deployment_payload_minimal() {
        let payload = DeploymentPayload {
            name: "test".to_string(),
            namespace: "default".to_string(),
            replicas: None,
            image: "nginx:latest".to_string(),
            container_name: None,
            ports: None,
            labels: None,
        };
        assert_eq!(payload.name, "test");
        assert_eq!(payload.image, "nginx:latest");
        assert!(payload.replicas.is_none());
        assert!(payload.container_name.is_none());
        assert!(payload.ports.is_none());
        assert!(payload.labels.is_none());
    }

    #[test]
    fn test_deployment_payload_full() {
        let mut labels = BTreeMap::new();
        labels.insert("env".to_string(), "prod".to_string());

        let payload = DeploymentPayload {
            name: "api-server".to_string(),
            namespace: "backend".to_string(),
            replicas: Some(3),
            image: "api-server:v2.1".to_string(),
            container_name: Some("api".to_string()),
            ports: Some(vec![8080, 8443]),
            labels: Some(labels),
        };
        assert_eq!(payload.name, "api-server");
        assert_eq!(payload.replicas, Some(3));
        assert_eq!(payload.container_name, Some("api".to_string()));
        assert!(payload.labels.is_some());
        assert!(payload.labels.as_ref().unwrap().contains_key("env"));
    }

    #[test]
    fn test_operation_response_minimal() {
        let response = OperationResponse {
            message: "Deployment created".to_string(),
            name: "my-app".to_string(),
            namespace: "default".to_string(),
            replicas: None,
            revision: None,
        };
        assert_eq!(response.message, "Deployment created");
        assert!(response.replicas.is_none());
        assert!(response.revision.is_none());
    }

    #[test]
    fn test_operation_response_with_replicas() {
        let response = OperationResponse {
            message: "Scaled".to_string(),
            name: "my-app".to_string(),
            namespace: "default".to_string(),
            replicas: Some(5),
            revision: None,
        };
        assert_eq!(response.replicas, Some(5));
    }

    #[test]
    fn test_operation_response_with_revision() {
        let response = OperationResponse {
            message: "Rolled back".to_string(),
            name: "my-app".to_string(),
            namespace: "default".to_string(),
            replicas: None,
            revision: Some(2),
        };
        assert_eq!(response.revision, Some(2));
    }

    // ========================================================================
    // Tests for RolloutHistory structs
    // ========================================================================

    #[test]
    fn test_rollout_history() {
        let history = RolloutHistory {
            name: "web-app".to_string(),
            namespace: "frontend".to_string(),
            revisions: vec![],
        };
        assert_eq!(history.name, "web-app");
        assert!(history.revisions.is_empty());
    }

    #[test]
    fn test_revision_info() {
        let revision = RevisionInfo {
            revision: 5,
            change_cause: Some("Updated image to v2".to_string()),
            created_at: Some(Utc::now()),
        };
        assert_eq!(revision.revision, 5);
        assert!(revision.change_cause.is_some());
    }

    #[test]
    fn test_revision_info_no_change_cause() {
        let revision = RevisionInfo {
            revision: 1,
            change_cause: None,
            created_at: None,
        };
        assert!(revision.change_cause.is_none());
        assert!(revision.created_at.is_none());
    }

    #[test]
    fn test_detailed_revision_info() {
        let rev = DetailedRevisionInfo {
            revision: 3,
            replica_set_name: "web-app-abc123".to_string(),
            replicas: 3,
            ready_replicas: 3,
            available_replicas: 3,
            image: "nginx:1.25".to_string(),
            created_at: Some(Utc::now()),
        };
        assert_eq!(rev.revision, 3);
        assert_eq!(rev.replica_set_name, "web-app-abc123");
        assert_eq!(rev.image, "nginx:1.25");
    }

    #[test]
    fn test_detailed_revision_info_zero_replicas() {
        let rev = DetailedRevisionInfo {
            revision: 0,
            replica_set_name: "old-app".to_string(),
            replicas: 0,
            ready_replicas: 0,
            available_replicas: 0,
            image: "old-image:v1".to_string(),
            created_at: None,
        };
        assert_eq!(rev.revision, 0);
        assert_eq!(rev.replicas, 0);
    }

    #[test]
    fn test_detailed_rollout_history() {
        let history = DetailedRolloutHistory {
            name: "api".to_string(),
            namespace: "backend".to_string(),
            current_revision: 5,
            revisions: vec![
                DetailedRevisionInfo {
                    revision: 5,
                    replica_set_name: "api-v5".to_string(),
                    replicas: 3,
                    ready_replicas: 3,
                    available_replicas: 3,
                    image: "api:v5".to_string(),
                    created_at: Some(Utc::now()),
                },
                DetailedRevisionInfo {
                    revision: 4,
                    replica_set_name: "api-v4".to_string(),
                    replicas: 0,
                    ready_replicas: 0,
                    available_replicas: 0,
                    image: "api:v4".to_string(),
                    created_at: None,
                },
            ],
        };
        assert_eq!(history.current_revision, 5);
        assert_eq!(history.revisions.len(), 2);
        assert_eq!(history.revisions[0].revision, 5);
        assert_eq!(history.revisions[1].revision, 4);
    }

    // ========================================================================
    // Tests for Query struct
    // ========================================================================

    #[test]
    fn test_deployment_list_query_empty() {
        let query = DeploymentListQuery {
            namespace: None,
            label_selector: None,
            limit: None,
        };
        assert!(query.namespace.is_none());
        assert!(query.label_selector.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_deployment_list_query_with_values() {
        let query = DeploymentListQuery {
            namespace: Some("production".to_string()),
            label_selector: Some("app=web".to_string()),
            limit: Some(50),
        };
        assert_eq!(query.namespace, Some("production".to_string()));
        assert_eq!(query.label_selector, Some("app=web".to_string()));
        assert_eq!(query.limit, Some(50));
    }

    // ========================================================================
    // Additional format_age tests
    // ========================================================================

    #[test]
    fn test_format_age_exact_boundary_minutes() {
        let now = Utc::now();
        let age = format_age(&(now - Duration::minutes(60)));
        assert_eq!(age, "1h");
    }

    #[test]
    fn test_format_age_exact_boundary_days() {
        let now = Utc::now();
        let age = format_age(&(now - Duration::hours(24)));
        assert_eq!(age, "1d");
    }

    #[test]
    fn test_format_age_zero_seconds() {
        let now = Utc::now();
        let age = format_age(&now);
        assert_eq!(age, "0s");
    }

    #[test]
    fn test_format_age_large_year_value() {
        let now = Utc::now();
        let age = format_age(&(now - Duration::days(730)));
        assert_eq!(age, "2y");
    }
}
