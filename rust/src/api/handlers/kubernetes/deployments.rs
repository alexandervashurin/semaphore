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

    let deployments = deployment_list.items.iter().map(deployment_summary).collect();

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
        message: format!("Deployment {} scaled to {} replicas", name, payload.replicas),
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
    } else {
        if let Some(spec) = deployment.spec.as_mut() {
            if let Some(meta) = spec.template.metadata.as_mut() {
                meta.annotations = Some(BTreeMap::from([(
                    "kubectl.kubernetes.io/restartedAt".to_string(),
                    restart_time,
                )]));
            }
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
            conds.iter()
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
            conds.iter()
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
