//! Kubernetes Helm API handlers
//!
//! Helm charts, releases, repositories management

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use k8s_openapi::api::core::v1::{ConfigMap, Secret};
use kube::{api::{Api, ListParams, DeleteParams, PostParams}, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

// ============================================================================
// Helm Repositories
// ============================================================================

#[derive(Debug, Serialize)]
pub struct HelmRepository {
    pub name: String,
    pub url: String,
    pub username: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HelmRepositoryList {
    pub repositories: Vec<HelmRepository>,
}

#[derive(Debug, Deserialize)]
pub struct CreateHelmRepositoryRequest {
    pub name: String,
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub async fn list_helm_repos(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HelmRepositoryList>> {
    let client = state.kubernetes_client()?;
    
    // Helm repos are typically stored in kubeconfig or as ConfigMaps
    // For now, return a static list of common repos
    let repos = vec![
        HelmRepository {
            name: "stable".to_string(),
            url: "https://charts.helm.sh/stable".to_string(),
            username: None,
        },
        HelmRepository {
            name: "bitnami".to_string(),
            url: "https://charts.bitnami.com/bitnami".to_string(),
            username: None,
        },
        HelmRepository {
            name: "ingress-nginx".to_string(),
            url: "https://kubernetes.github.io/ingress-nginx".to_string(),
            username: None,
        },
        HelmRepository {
            name: "jetstack".to_string(),
            url: "https://charts.jetstack.io".to_string(),
            username: None,
        },
    ];
    
    Ok(Json(HelmRepositoryList { repositories: repos }))
}

pub async fn add_helm_repo(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateHelmRepositoryRequest>,
) -> Result<Json<HelmRepository>> {
    // In a real implementation, this would add the repo to helm config
    // For now, just return the repo info
    Ok(Json(HelmRepository {
        name: payload.name,
        url: payload.url,
        username: payload.username,
    }))
}

// ============================================================================
// Helm Charts
// ============================================================================

#[derive(Debug, Serialize)]
pub struct HelmChart {
    pub name: String,
    pub version: String,
    pub app_version: Option<String>,
    pub description: Option<String>,
    pub home: Option<String>,
    pub sources: Vec<String>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct HelmChartList {
    pub charts: Vec<HelmChart>,
}

#[derive(Debug, Deserialize)]
pub struct SearchChartsQuery {
    pub repo: Option<String>,
    pub query: Option<String>,
    pub limit: Option<i32>,
}

pub async fn search_helm_charts(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<SearchChartsQuery>,
) -> Result<Json<HelmChartList>> {
    // Mock chart search - in real implementation would query helm repos
    let charts = vec![
        HelmChart {
            name: "nginx".to_string(),
            version: "15.0.0".to_string(),
            app_version: Some("1.24.0".to_string()),
            description: Some("NGINX Ingress controller for Kubernetes using Helm".to_string()),
            home: Some("https://github.com/kubernetes/ingress-nginx".to_string()),
            sources: vec!["https://github.com/kubernetes/ingress-nginx".to_string()],
            keywords: vec!["ingress".to_string(), "nginx".to_string()],
        },
        HelmChart {
            name: "cert-manager".to_string(),
            version: "1.12.0".to_string(),
            app_version: Some("v1.12.0".to_string()),
            description: Some("A certificate controller for Kubernetes".to_string()),
            home: Some("https://cert-manager.io".to_string()),
            sources: vec!["https://github.com/cert-manager/cert-manager".to_string()],
            keywords: vec!["certificates".to_string(), "tls".to_string()],
        },
        HelmChart {
            name: "postgresql".to_string(),
            version: "12.0.0".to_string(),
            app_version: Some("15.0".to_string()),
            description: Some("PostgreSQL database for Kubernetes".to_string()),
            home: Some("https://www.postgresql.org".to_string()),
            sources: vec![],
            keywords: vec!["database".to_string(), "postgresql".to_string()],
        },
    ];
    
    let filtered: Vec<HelmChart> = if let Some(q) = &query.query {
        charts.into_iter().filter(|c| {
            c.name.to_lowercase().contains(&q.to_lowercase()) ||
            c.description.as_ref().map(|d| d.to_lowercase().contains(&q.to_lowercase())).unwrap_or(false)
        }).collect()
    } else {
        charts
    };
    
    let limit = query.limit.unwrap_or(50) as usize;
    let result: Vec<HelmChart> = filtered.into_iter().take(limit).collect();
    
    Ok(Json(HelmChartList { charts: result }))
}

pub async fn get_helm_chart(
    State(_state): State<Arc<AppState>>,
    Path((repo, chart)): Path<(String, String)>,
) -> Result<Json<HelmChart>> {
    // Mock - in real implementation would fetch chart metadata from repo
    Ok(Json(HelmChart {
        name: chart.clone(),
        version: "1.0.0".to_string(),
        app_version: Some("1.0.0".to_string()),
        description: Some(format!("{} chart from {} repo", chart, repo)),
        home: None,
        sources: vec![],
        keywords: vec![],
    }))
}

// ============================================================================
// Helm Releases
// ============================================================================

#[derive(Debug, Serialize)]
pub struct HelmRelease {
    pub name: String,
    pub namespace: String,
    pub chart: String,
    pub chart_version: String,
    pub app_version: Option<String>,
    pub status: String,
    pub revision: i32,
    pub deployed_at: Option<String>,
    pub values: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct HelmReleaseList {
    pub releases: Vec<HelmRelease>,
}

#[derive(Debug, Deserialize)]
pub struct ListReleasesQuery {
    pub namespace: Option<String>,
    pub all_namespaces: Option<bool>,
}

pub async fn list_helm_releases(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListReleasesQuery>,
) -> Result<Json<HelmReleaseList>> {
    let client = state.kubernetes_client()?;
    
    // Helm releases are stored as Secrets or ConfigMaps in the namespace
    let ns = query.namespace.unwrap_or_else(|| "default".to_string());
    
    // Try to get releases from Secrets (Helm v3 default)
    let secrets_api: Api<Secret> = Api::namespaced(client.raw().clone(), &ns);
    let lp = ListParams::default().labels("owner=helm");
    
    let secrets = secrets_api.list(&lp).await.ok();
    
    let mut releases = Vec::new();
    
    if let Some(secret_list) = secrets {
        for secret in secret_list.items {
            if let Some(labels) = &secret.metadata.labels {
                if labels.get("name").is_some() {
                    let release = HelmRelease {
                        name: labels.get("name").map(|v| v.as_str()).unwrap_or("unknown").to_string(),
                        namespace: ns.clone(),
                        chart: "unknown".to_string(),
                        chart_version: "unknown".to_string(),
                        app_version: None,
                        status: "deployed".to_string(),
                        revision: 1,
                        deployed_at: secret.metadata.creation_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
                        values: None,
                    };
                    releases.push(release);
                }
            }
        }
    }
    
    Ok(Json(HelmReleaseList { releases }))
}

#[derive(Debug, Deserialize)]
pub struct InstallHelmRequest {
    pub name: String,
    pub namespace: String,
    pub chart: String,
    pub version: Option<String>,
    pub repo: Option<String>,
    pub values: Option<serde_json::Value>,
    pub create_namespace: Option<bool>,
}

pub async fn install_helm_chart(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<InstallHelmRequest>,
) -> Result<Json<HelmRelease>> {
    // In real implementation, this would use helm CLI or library
    // For now, return a mock response
    
    Ok(Json(HelmRelease {
        name: payload.name,
        namespace: payload.namespace,
        chart: payload.chart,
        chart_version: payload.version.unwrap_or_else(|| "latest".to_string()),
        app_version: None,
        status: "deployed".to_string(),
        revision: 1,
        deployed_at: Some(chrono::Utc::now().to_rfc3339()),
        values: payload.values,
    }))
}

#[derive(Debug, Deserialize)]
pub struct UpgradeHelmRequest {
    pub chart: String,
    pub version: Option<String>,
    pub repo: Option<String>,
    pub values: Option<serde_json::Value>,
}

pub async fn upgrade_helm_release(
    State(_state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<UpgradeHelmRequest>,
) -> Result<Json<HelmRelease>> {
    Ok(Json(HelmRelease {
        name: name,
        namespace: namespace,
        chart: payload.chart,
        chart_version: payload.version.unwrap_or_else(|| "latest".to_string()),
        app_version: None,
        status: "upgraded".to_string(),
        revision: 2,
        deployed_at: Some(chrono::Utc::now().to_rfc3339()),
        values: payload.values,
    }))
}

pub async fn rollback_helm_release(
    State(_state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Query(query): Query<RollbackQuery>,
) -> Result<Json<HelmRelease>> {
    let revision = query.revision.unwrap_or(1);
    
    Ok(Json(HelmRelease {
        name: name,
        namespace: namespace,
        chart: "rolled-back".to_string(),
        chart_version: format!("{}.{}", revision, 0),
        app_version: None,
        status: "rolled-back".to_string(),
        revision: revision + 1,
        deployed_at: Some(chrono::Utc::now().to_rfc3339()),
        values: None,
    }))
}

#[derive(Debug, Deserialize)]
pub struct RollbackQuery {
    pub revision: Option<i32>,
}

pub async fn uninstall_helm_release(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<StatusCode> {
    let client = state.kubernetes_client()?;
    
    // Delete Helm release Secret
    let secrets_api: Api<Secret> = Api::namespaced(client.raw().clone(), &namespace);
    
    // Try to delete the release secret
    let lp = ListParams::default().labels(format!("name={}", name).as_str());
    let secrets = secrets_api.list(&lp).await.ok();
    
    if let Some(secret_list) = secrets {
        for secret in secret_list.items {
            if let Some(name) = &secret.metadata.name {
                let _ = secrets_api.delete(name, &DeleteParams::default()).await;
            }
        }
    }
    
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_helm_release_history(
    State(_state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<Vec<HelmRelease>>> {
    // Mock history
    Ok(Json(vec![
        HelmRelease {
            name: name.clone(),
            namespace: namespace.clone(),
            chart: "mychart".to_string(),
            chart_version: "1.0.0".to_string(),
            app_version: Some("1.0.0".to_string()),
            status: "superseded".to_string(),
            revision: 1,
            deployed_at: Some(chrono::Utc::now().to_rfc3339()),
            values: None,
        },
        HelmRelease {
            name: name,
            namespace: namespace,
            chart: "mychart".to_string(),
            chart_version: "1.1.0".to_string(),
            app_version: Some("1.1.0".to_string()),
            status: "deployed".to_string(),
            revision: 2,
            deployed_at: Some(chrono::Utc::now().to_rfc3339()),
            values: None,
        },
    ]))
}

pub async fn get_helm_release_values(
    State(_state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    // Mock values
    Ok(Json(serde_json::json!({
        "replicaCount": 1,
        "image": {
            "repository": "nginx",
            "tag": "latest",
            "pullPolicy": "IfNotPresent"
        }
    })))
}

pub async fn update_helm_release_values(
    State(_state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(values): Json<serde_json::Value>,
) -> Result<Json<HelmRelease>> {
    Ok(Json(HelmRelease {
        name: name,
        namespace: namespace,
        chart: "updated".to_string(),
        chart_version: "1.0.0".to_string(),
        app_version: None,
        status: "updated".to_string(),
        revision: 3,
        deployed_at: Some(chrono::Utc::now().to_rfc3339()),
        values: Some(values),
    }))
}
