//! Kubernetes ConfigMap API handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use k8s_openapi::api::apps::v1::{DaemonSet, Deployment, StatefulSet};
use k8s_openapi::api::batch::v1::{CronJob, Job};
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::api::core::v1::{Container, Pod, PodSpec};
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct ListConfigMapsQuery {
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ConfigMapSummary {
    pub name: String,
    pub namespace: String,
    pub data_keys: usize,
    pub binary_data_keys: usize,
    pub binary_total_bytes: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigMapEditorMode {
    KeyValues,
    RawYaml,
    RawJson,
}

#[derive(Debug, Deserialize)]
pub struct ValidateConfigMapRequest {
    pub mode: ConfigMapEditorMode,
    pub name: Option<String>,
    pub namespace: Option<String>,
    pub labels: Option<BTreeMap<String, String>>,
    pub annotations: Option<BTreeMap<String, String>>,
    pub data: Option<BTreeMap<String, String>>,
    pub raw: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ValidateConfigMapResponse {
    pub valid: bool,
    pub errors: Vec<String>,
    pub summary: Option<ConfigMapSummary>,
    pub normalized: Option<ConfigMap>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigMapReference {
    pub kind: String,
    pub name: String,
    pub namespace: String,
    pub field: String,
}

fn to_summary(cm: &ConfigMap) -> ConfigMapSummary {
    let data_keys = cm.data.as_ref().map(|m| m.len()).unwrap_or(0);
    let binary_data = cm.binary_data.as_ref();
    let binary_data_keys = binary_data.map(|m| m.len()).unwrap_or(0);
    let binary_total_bytes = binary_data
        .map(|m| m.values().map(|v| v.0.len()).sum())
        .unwrap_or(0);

    ConfigMapSummary {
        name: cm
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        namespace: cm
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        data_keys,
        binary_data_keys,
        binary_total_bytes,
    }
}

fn container_references_configmap(container: &Container, target_name: &str) -> bool {
    let env_ref = container
        .env
        .as_ref()
        .map(|vars| {
            vars.iter().any(|ev| {
                ev.value_from
                    .as_ref()
                    .and_then(|vf| vf.config_map_key_ref.as_ref())
                    .map(|cm| cm.name == target_name)
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    let env_from_ref = container
        .env_from
        .as_ref()
        .map(|vars| {
            vars.iter().any(|from| {
                from.config_map_ref
                    .as_ref()
                    .map(|r| r.name == target_name)
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    env_ref || env_from_ref
}

fn pod_spec_references_configmap(spec: &PodSpec, target_name: &str) -> Vec<String> {
    let mut fields = Vec::new();

    if spec
        .volumes
        .as_ref()
        .map(|vols| {
            vols.iter().any(|v| {
                v.config_map
                    .as_ref()
                    .map(|cm| cm.name == target_name)
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
    {
        fields.push("spec.volumes[*].configMap".to_string());
    }

    if spec
        .containers
        .iter()
        .any(|c| container_references_configmap(c, target_name))
    {
        fields.push("spec.containers[*].env|envFrom".to_string());
    }

    if spec
        .init_containers
        .as_ref()
        .map(|c| {
            c.iter()
                .any(|i| container_references_configmap(i, target_name))
        })
        .unwrap_or(false)
    {
        fields.push("spec.initContainers[*].env|envFrom".to_string());
    }

    fields
}

pub async fn list_configmaps(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListConfigMapsQuery>,
) -> Result<Json<Vec<ConfigMapSummary>>> {
    let client = state.kubernetes_client()?;
    let api: Api<ConfigMap> = if let Some(namespace) = query.namespace.as_deref() {
        client.api(Some(namespace))
    } else {
        client.api_all()
    };

    let mut params = ListParams::default();
    if let Some(selector) = query.label_selector {
        params = params.labels(&selector);
    }
    if let Some(limit) = query.limit {
        params = params.limit(limit);
    }

    let items = api
        .list(&params)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(items.items.iter().map(to_summary).collect()))
}

pub async fn get_configmap(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<ConfigMap>> {
    let client = state.kubernetes_client()?;
    let api: Api<ConfigMap> = client.api(Some(&namespace));

    let item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(item))
}

pub async fn get_configmap_yaml(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<String> {
    let client = state.kubernetes_client()?;
    let api: Api<ConfigMap> = client.api(Some(&namespace));

    let item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    serde_yaml::to_string(&item).map_err(|e| Error::Other(format!("YAML serialize failed: {e}")))
}

pub async fn create_configmap(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(payload): Json<ConfigMap>,
) -> Result<Json<ConfigMap>> {
    let client = state.kubernetes_client()?;
    let api: Api<ConfigMap> = client.api(Some(&namespace));

    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(created))
}

pub async fn update_configmap(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(mut payload): Json<ConfigMap>,
) -> Result<Json<ConfigMap>> {
    let client = state.kubernetes_client()?;
    let api: Api<ConfigMap> = client.api(Some(&namespace));

    if payload.metadata.name.is_none() {
        payload.metadata.name = Some(name.clone());
    }
    if payload.metadata.namespace.is_none() {
        payload.metadata.namespace = Some(namespace);
    }

    let updated = api
        .replace(&name, &PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(updated))
}

pub async fn delete_configmap(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<BTreeMap<String, String>>> {
    let client = state.kubernetes_client()?;
    let api: Api<ConfigMap> = client.api(Some(&namespace));

    let _ = api
        .delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    let mut response = BTreeMap::new();
    response.insert("status".to_string(), "ok".to_string());
    response.insert(
        "message".to_string(),
        format!("ConfigMap {namespace}/{name} deleted"),
    );
    Ok(Json(response))
}

pub async fn validate_configmap(
    Json(payload): Json<ValidateConfigMapRequest>,
) -> Result<Json<ValidateConfigMapResponse>> {
    let parsed = match payload.mode {
        ConfigMapEditorMode::KeyValues => ConfigMap {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: payload.name.clone(),
                namespace: payload.namespace.clone(),
                labels: payload.labels.clone(),
                annotations: payload.annotations.clone(),
                ..Default::default()
            },
            data: payload.data.clone(),
            ..Default::default()
        },
        ConfigMapEditorMode::RawYaml => {
            let raw = payload.raw.as_deref().ok_or_else(|| {
                Error::Validation("Field 'raw' is required for raw_yaml".to_string())
            })?;
            serde_yaml::from_str::<ConfigMap>(raw)
                .map_err(|e| Error::Validation(format!("YAML parse error: {e}")))?
        }
        ConfigMapEditorMode::RawJson => {
            let raw = payload.raw.as_deref().ok_or_else(|| {
                Error::Validation("Field 'raw' is required for raw_json".to_string())
            })?;
            serde_json::from_str::<ConfigMap>(raw)
                .map_err(|e| Error::Validation(format!("JSON parse error: {e}")))?
        }
    };

    let mut errors = Vec::new();
    if parsed.metadata.name.as_deref().unwrap_or("").is_empty() {
        errors.push("metadata.name is required".to_string());
    }

    let summary = if errors.is_empty() {
        Some(to_summary(&parsed))
    } else {
        None
    };

    Ok(Json(ValidateConfigMapResponse {
        valid: errors.is_empty(),
        errors,
        summary,
        normalized: Some(parsed),
    }))
}

pub async fn get_configmap_references(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<Vec<ConfigMapReference>>> {
    let client = state.kubernetes_client()?;
    let mut refs = Vec::new();

    let pods_api: Api<Pod> = client.api(Some(&namespace));
    let pods = pods_api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    for pod in pods.items {
        let pod_name = pod
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        if let Some(spec) = pod.spec.as_ref() {
            for field in pod_spec_references_configmap(spec, &name) {
                refs.push(ConfigMapReference {
                    kind: "Pod".to_string(),
                    name: pod_name.clone(),
                    namespace: namespace.clone(),
                    field,
                });
            }
        }
    }

    let deploy_api: Api<Deployment> = client.api(Some(&namespace));
    let deployments = deploy_api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    for d in deployments.items {
        let d_name = d
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        if let Some(spec) = d.spec.as_ref().and_then(|s| s.template.spec.as_ref()) {
            for field in pod_spec_references_configmap(spec, &name) {
                refs.push(ConfigMapReference {
                    kind: "Deployment".to_string(),
                    name: d_name.clone(),
                    namespace: namespace.clone(),
                    field,
                });
            }
        }
    }

    let sts_api: Api<StatefulSet> = client.api(Some(&namespace));
    let sets = sts_api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    for s in sets.items {
        let s_name = s
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        if let Some(spec) = s.spec.as_ref().and_then(|x| x.template.spec.as_ref()) {
            for field in pod_spec_references_configmap(spec, &name) {
                refs.push(ConfigMapReference {
                    kind: "StatefulSet".to_string(),
                    name: s_name.clone(),
                    namespace: namespace.clone(),
                    field,
                });
            }
        }
    }

    let ds_api: Api<DaemonSet> = client.api(Some(&namespace));
    let daemonsets = ds_api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    for ds in daemonsets.items {
        let ds_name = ds
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        if let Some(spec) = ds.spec.as_ref().and_then(|x| x.template.spec.as_ref()) {
            for field in pod_spec_references_configmap(spec, &name) {
                refs.push(ConfigMapReference {
                    kind: "DaemonSet".to_string(),
                    name: ds_name.clone(),
                    namespace: namespace.clone(),
                    field,
                });
            }
        }
    }

    let jobs_api: Api<Job> = client.api(Some(&namespace));
    let jobs = jobs_api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    for job in jobs.items {
        let job_name = job
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        if let Some(spec) = job.spec.as_ref().and_then(|x| x.template.spec.as_ref()) {
            for field in pod_spec_references_configmap(spec, &name) {
                refs.push(ConfigMapReference {
                    kind: "Job".to_string(),
                    name: job_name.clone(),
                    namespace: namespace.clone(),
                    field,
                });
            }
        }
    }

    let cjobs_api: Api<CronJob> = client.api(Some(&namespace));
    let cjobs = cjobs_api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    for cj in cjobs.items {
        let cj_name = cj
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        if let Some(spec) = cj
            .spec
            .as_ref()
            .and_then(|x| x.job_template.spec.as_ref())
            .and_then(|x| x.template.spec.as_ref())
        {
            for field in pod_spec_references_configmap(spec, &name) {
                refs.push(ConfigMapReference {
                    kind: "CronJob".to_string(),
                    name: cj_name.clone(),
                    namespace: namespace.clone(),
                    field,
                });
            }
        }
    }

    Ok(Json(refs))
}
