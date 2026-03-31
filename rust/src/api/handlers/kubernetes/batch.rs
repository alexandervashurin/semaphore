//! Kubernetes batch/scheduling API handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use k8s_openapi::api::batch::v1::{CronJob, Job};
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::api::policy::v1::PodDisruptionBudget;
use k8s_openapi::api::scheduling::v1::PriorityClass;
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct BatchListQuery {
    pub namespace: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct JobSummary {
    pub name: String,
    pub namespace: String,
    pub active: i32,
    pub succeeded: i32,
    pub failed: i32,
    pub completions: Option<i32>,
    pub parallelism: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct CronJobSummary {
    pub name: String,
    pub namespace: String,
    pub schedule: String,
    pub suspend: bool,
    pub active: i32,
    pub last_schedule_time: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PriorityClassSummary {
    pub name: String,
    pub value: i32,
    pub global_default: bool,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PdbSummary {
    pub name: String,
    pub namespace: String,
    pub min_available: Option<String>,
    pub max_unavailable: Option<String>,
}

fn job_summary(job: &Job) -> JobSummary {
    let spec = job.spec.as_ref();
    let st = job.status.as_ref();
    JobSummary {
        name: job
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        namespace: job
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        active: st.and_then(|s| s.active).unwrap_or(0),
        succeeded: st.and_then(|s| s.succeeded).unwrap_or(0),
        failed: st.and_then(|s| s.failed).unwrap_or(0),
        completions: spec.and_then(|s| s.completions),
        parallelism: spec.and_then(|s| s.parallelism),
    }
}

fn cron_summary(cj: &CronJob) -> CronJobSummary {
    let spec = cj.spec.as_ref();
    let st = cj.status.as_ref();
    CronJobSummary {
        name: cj
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        namespace: cj
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        schedule: spec.map(|s| s.schedule.clone()).unwrap_or_default(),
        suspend: spec.and_then(|s| s.suspend).unwrap_or(false),
        active: st
            .and_then(|s| s.active.as_ref().map(|a| a.len() as i32))
            .unwrap_or(0),
        last_schedule_time: st
            .and_then(|s| s.last_schedule_time.as_ref())
            .map(|t| t.0.to_rfc3339()),
    }
}

fn pc_summary(pc: &PriorityClass) -> PriorityClassSummary {
    PriorityClassSummary {
        name: pc
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        value: pc.value,
        global_default: pc.global_default.unwrap_or(false),
        description: pc.description.clone(),
    }
}

fn pdb_summary(pdb: &PodDisruptionBudget) -> PdbSummary {
    let spec = pdb.spec.as_ref();
    PdbSummary {
        name: pdb
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        namespace: pdb
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        min_available: spec
            .and_then(|s| s.min_available.as_ref())
            .map(|v| format!("{v:?}")),
        max_unavailable: spec
            .and_then(|s| s.max_unavailable.as_ref())
            .map(|v| format!("{v:?}")),
    }
}

pub async fn list_jobs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<BatchListQuery>,
) -> Result<Json<Vec<JobSummary>>> {
    let client = state.kubernetes_client()?;
    let ns = query.namespace.unwrap_or_else(|| "default".to_string());
    let api: Api<Job> = client.api(Some(&ns));
    let mut lp = ListParams::default();
    if let Some(limit) = query.limit {
        lp = lp.limit(limit);
    }
    let list = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(list.items.iter().map(job_summary).collect()))
}

pub async fn get_job(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<Job>> {
    let client = state.kubernetes_client()?;
    let api: Api<Job> = client.api(Some(&namespace));
    let item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(item))
}

pub async fn create_job(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(payload): Json<Job>,
) -> Result<Json<JobSummary>> {
    let client = state.kubernetes_client()?;
    let api: Api<Job> = client.api(Some(&namespace));
    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(job_summary(&created)))
}

pub async fn delete_job(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Job> = client.api(Some(&namespace));
    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        serde_json::json!({"status":"ok","message":format!("Job {}/{} deleted", namespace, name)}),
    ))
}

pub async fn list_job_pods(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let client = state.kubernetes_client()?;
    let api: Api<Pod> = client.api(Some(&namespace));
    let lp = ListParams::default().labels(&format!("job-name={name}"));
    let list = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        list.items.iter().map(|p| serde_json::json!(p)).collect(),
    ))
}

pub async fn list_cronjobs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<BatchListQuery>,
) -> Result<Json<Vec<CronJobSummary>>> {
    let client = state.kubernetes_client()?;
    let ns = query.namespace.unwrap_or_else(|| "default".to_string());
    let api: Api<CronJob> = client.api(Some(&ns));
    let mut lp = ListParams::default();
    if let Some(limit) = query.limit {
        lp = lp.limit(limit);
    }
    let list = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(list.items.iter().map(cron_summary).collect()))
}

pub async fn get_cronjob(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<CronJob>> {
    let client = state.kubernetes_client()?;
    let api: Api<CronJob> = client.api(Some(&namespace));
    let item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(item))
}

pub async fn create_cronjob(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(payload): Json<CronJob>,
) -> Result<Json<CronJobSummary>> {
    let client = state.kubernetes_client()?;
    let api: Api<CronJob> = client.api(Some(&namespace));
    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(cron_summary(&created)))
}

pub async fn update_cronjob_suspend(
    State(state): State<Arc<AppState>>,
    Path((namespace, name, suspend)): Path<(String, String, bool)>,
) -> Result<Json<CronJobSummary>> {
    let client = state.kubernetes_client()?;
    let api: Api<CronJob> = client.api(Some(&namespace));
    let mut item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    if let Some(spec) = item.spec.as_mut() {
        spec.suspend = Some(suspend);
    }
    let updated = api
        .replace(&name, &PostParams::default(), &item)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(cron_summary(&updated)))
}

pub async fn delete_cronjob(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<CronJob> = client.api(Some(&namespace));
    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        serde_json::json!({"status":"ok","message":format!("CronJob {}/{} deleted", namespace, name)}),
    ))
}

pub async fn list_cronjob_history(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<Vec<JobSummary>>> {
    let client = state.kubernetes_client()?;
    let api: Api<Job> = client.api(Some(&namespace));
    let lp = ListParams::default().labels(&format!("cronjob.kubernetes.io/instance={name}"));
    let list = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(list.items.iter().map(job_summary).collect()))
}

pub async fn list_priority_classes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PriorityClassSummary>>> {
    let client = state.kubernetes_client()?;
    let api: Api<PriorityClass> = client.api_all();
    let list = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(list.items.iter().map(pc_summary).collect()))
}

pub async fn create_priority_class(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PriorityClass>,
) -> Result<Json<PriorityClassSummary>> {
    let client = state.kubernetes_client()?;
    let api: Api<PriorityClass> = client.api_all();
    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(pc_summary(&created)))
}

pub async fn delete_priority_class(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<PriorityClass> = client.api_all();
    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        serde_json::json!({"status":"ok","message":format!("PriorityClass {} deleted", name)}),
    ))
}

pub async fn list_pdbs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<BatchListQuery>,
) -> Result<Json<Vec<PdbSummary>>> {
    let client = state.kubernetes_client()?;
    let ns = query.namespace.unwrap_or_else(|| "default".to_string());
    let api: Api<PodDisruptionBudget> = client.api(Some(&ns));
    let mut lp = ListParams::default();
    if let Some(limit) = query.limit {
        lp = lp.limit(limit);
    }
    let list = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(list.items.iter().map(pdb_summary).collect()))
}

pub async fn create_pdb(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(payload): Json<PodDisruptionBudget>,
) -> Result<Json<PdbSummary>> {
    let client = state.kubernetes_client()?;
    let api: Api<PodDisruptionBudget> = client.api(Some(&namespace));
    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(pdb_summary(&created)))
}

pub async fn delete_pdb(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<PodDisruptionBudget> = client.api(Some(&namespace));
    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        serde_json::json!({"status":"ok","message":format!("PodDisruptionBudget {}/{} deleted", namespace, name)}),
    ))
}
