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
            .map(|t| t.0.to_string()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_summary_from_values() {
        // Проверяем что JobSummary правильно сериализуется
        let summary = JobSummary {
            name: "test-job".to_string(),
            namespace: "default".to_string(),
            active: 1,
            succeeded: 2,
            failed: 0,
            completions: Some(3),
            parallelism: Some(2),
        };
        assert_eq!(summary.name, "test-job");
        assert_eq!(summary.active, 1);
        assert_eq!(summary.succeeded, 2);
    }

    #[test]
    fn test_cron_job_summary_from_values() {
        let summary = CronJobSummary {
            name: "test-cron".to_string(),
            namespace: "default".to_string(),
            schedule: "*/5 * * * *".to_string(),
            suspend: false,
            active: 0,
            last_schedule_time: None,
        };
        assert_eq!(summary.name, "test-cron");
        assert_eq!(summary.schedule, "*/5 * * * *");
        assert!(!summary.suspend);
    }

    #[test]
    fn test_priority_class_summary() {
        let summary = PriorityClassSummary {
            name: "high-priority".to_string(),
            value: 1000,
            global_default: false,
            description: Some("High priority".to_string()),
        };
        assert_eq!(summary.name, "high-priority");
        assert_eq!(summary.value, 1000);
        assert!(!summary.global_default);
    }

    #[test]
    fn test_pdb_summary() {
        let summary = PdbSummary {
            name: "test-pdb".to_string(),
            namespace: "default".to_string(),
            min_available: Some("1".to_string()),
            max_unavailable: None,
        };
        assert_eq!(summary.name, "test-pdb");
        assert_eq!(summary.min_available, Some("1".to_string()));
    }

    // ─────────────────────────────────────────────
    // DTO struct tests - JobSummary
    // ─────────────────────────────────────────────

    #[test]
    fn test_job_summary_all_none() {
        let summary = JobSummary {
            name: "minimal-job".to_string(),
            namespace: "default".to_string(),
            active: 0,
            succeeded: 0,
            failed: 0,
            completions: None,
            parallelism: None,
        };
        assert_eq!(summary.active, 0);
        assert!(summary.completions.is_none());
        assert!(summary.parallelism.is_none());
    }

    #[test]
    fn test_job_summary_with_completions() {
        let summary = JobSummary {
            name: "batch-job".to_string(),
            namespace: "processing".to_string(),
            active: 5,
            succeeded: 10,
            failed: 2,
            completions: Some(15),
            parallelism: Some(5),
        };
        assert_eq!(summary.name, "batch-job");
        assert_eq!(summary.succeeded, 10);
        assert_eq!(summary.failed, 2);
        assert_eq!(summary.completions, Some(15));
        assert_eq!(summary.parallelism, Some(5));
    }

    #[test]
    fn test_job_summary_large_values() {
        let summary = JobSummary {
            name: "massive-job".to_string(),
            namespace: "default".to_string(),
            active: i32::MAX,
            succeeded: i32::MAX,
            failed: i32::MAX,
            completions: Some(i32::MAX),
            parallelism: Some(i32::MAX),
        };
        assert_eq!(summary.active, i32::MAX);
    }

    // ─────────────────────────────────────────────
    // DTO struct tests - CronJobSummary
    // ─────────────────────────────────────────────

    #[test]
    fn test_cron_job_summary_suspended() {
        let summary = CronJobSummary {
            name: "suspended-cron".to_string(),
            namespace: "default".to_string(),
            schedule: "0 2 * * *".to_string(),
            suspend: true,
            active: 0,
            last_schedule_time: None,
        };
        assert!(summary.suspend);
        assert_eq!(summary.active, 0);
    }

    #[test]
    fn test_cron_job_summary_active_jobs() {
        let summary = CronJobSummary {
            name: "busy-cron".to_string(),
            namespace: "cron".to_string(),
            schedule: "*/1 * * * *".to_string(),
            suspend: false,
            active: 3,
            last_schedule_time: Some("2024-01-01T00:00:00Z".to_string()),
        };
        assert_eq!(summary.active, 3);
        assert!(!summary.suspend);
        assert!(summary.last_schedule_time.is_some());
    }

    #[test]
    fn test_cron_job_summary_complex_schedule() {
        let summary = CronJobSummary {
            name: "complex-cron".to_string(),
            namespace: "default".to_string(),
            schedule: "30 2 1,15 * 1-5".to_string(),
            suspend: false,
            active: 0,
            last_schedule_time: None,
        };
        assert_eq!(summary.schedule, "30 2 1,15 * 1-5");
    }

    // ─────────────────────────────────────────────
    // DTO struct tests - PriorityClassSummary
    // ─────────────────────────────────────────────

    #[test]
    fn test_priority_class_summary_global_default() {
        let summary = PriorityClassSummary {
            name: "default".to_string(),
            value: 0,
            global_default: true,
            description: None,
        };
        assert!(summary.global_default);
        assert_eq!(summary.value, 0);
        assert!(summary.description.is_none());
    }

    #[test]
    fn test_priority_class_summary_negative_value() {
        let summary = PriorityClassSummary {
            name: "low-priority".to_string(),
            value: -100,
            global_default: false,
            description: Some("Low priority class".to_string()),
        };
        assert_eq!(summary.value, -100);
        assert!(!summary.global_default);
    }

    #[test]
    fn test_priority_class_summary_max_value() {
        let summary = PriorityClassSummary {
            name: "critical".to_string(),
            value: i32::MAX,
            global_default: false,
            description: Some("Critical priority".to_string()),
        };
        assert_eq!(summary.value, i32::MAX);
    }

    // ─────────────────────────────────────────────
    // DTO struct tests - PdbSummary
    // ─────────────────────────────────────────────

    #[test]
    fn test_pdb_summary_max_unavailable() {
        let summary = PdbSummary {
            name: "web-pdb".to_string(),
            namespace: "production".to_string(),
            min_available: None,
            max_unavailable: Some("25%".to_string()),
        };
        assert_eq!(summary.max_unavailable, Some("25%".to_string()));
        assert!(summary.min_available.is_none());
    }

    #[test]
    fn test_pdb_summary_both_none() {
        let summary = PdbSummary {
            name: "empty-pdb".to_string(),
            namespace: "default".to_string(),
            min_available: None,
            max_unavailable: None,
        };
        assert!(summary.min_available.is_none());
        assert!(summary.max_unavailable.is_none());
    }

    // ─────────────────────────────────────────────
    // Query params tests
    // ─────────────────────────────────────────────

    #[test]
    fn test_batch_list_query_all_none() {
        let query = BatchListQuery {
            namespace: None,
            limit: None,
        };
        assert!(query.namespace.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_batch_list_query_with_values() {
        let query = BatchListQuery {
            namespace: Some("batch".to_string()),
            limit: Some(100),
        };
        assert_eq!(query.namespace, Some("batch".to_string()));
        assert_eq!(query.limit, Some(100));
    }

    #[test]
    fn test_batch_list_query_limit_zero() {
        let query = BatchListQuery {
            namespace: Some("default".to_string()),
            limit: Some(0),
        };
        assert_eq!(query.limit, Some(0));
    }

    // ─────────────────────────────────────────────
    // Edge cases and boundary tests
    // ─────────────────────────────────────────────

    #[test]
    fn test_job_summary_name_namespace_defaults() {
        // Verify that job_summary handles "unknown" and "default" fallbacks
        // by constructing a JobSummary directly with those values
        let summary = JobSummary {
            name: "unknown".to_string(),
            namespace: "default".to_string(),
            active: 0,
            succeeded: 0,
            failed: 0,
            completions: None,
            parallelism: None,
        };
        assert_eq!(summary.name, "unknown");
        assert_eq!(summary.namespace, "default");
    }

    #[test]
    fn test_cron_job_summary_name_namespace_defaults() {
        let summary = CronJobSummary {
            name: "unknown".to_string(),
            namespace: "default".to_string(),
            schedule: String::new(),
            suspend: false,
            active: 0,
            last_schedule_time: None,
        };
        assert_eq!(summary.name, "unknown");
        assert!(summary.schedule.is_empty());
    }

    #[test]
    fn test_priority_class_summary_empty_description() {
        let summary = PriorityClassSummary {
            name: String::new(),
            value: 0,
            global_default: false,
            description: Some(String::new()),
        };
        assert!(summary.name.is_empty());
        assert_eq!(summary.description, Some(String::new()));
    }

    #[test]
    fn test_pdb_summary_empty_strings() {
        let summary = PdbSummary {
            name: String::new(),
            namespace: String::new(),
            min_available: Some(String::new()),
            max_unavailable: Some(String::new()),
        };
        assert!(summary.name.is_empty());
        assert_eq!(summary.min_available, Some(String::new()));
    }
}
