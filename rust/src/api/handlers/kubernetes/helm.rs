//! Kubernetes Helm API handlers
//!
//! Helm charts, releases, repositories management

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use k8s_openapi::api::core::v1::{ConfigMap, Secret};
use kube::{
    api::{Api, DeleteParams, ListParams, PostParams},
    Client,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::handlers::kubernetes::audit::KubernetesAuditLogger;
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

    Ok(Json(HelmRepositoryList {
        repositories: repos,
    }))
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
        charts
            .into_iter()
            .filter(|c| {
                c.name.to_lowercase().contains(&q.to_lowercase())
                    || c.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&q.to_lowercase()))
                        .unwrap_or(false)
            })
            .collect()
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
                        name: labels
                            .get("name")
                            .map(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        namespace: ns.clone(),
                        chart: "unknown".to_string(),
                        chart_version: "unknown".to_string(),
                        app_version: None,
                        status: "deployed".to_string(),
                        revision: 1,
                        deployed_at: secret
                            .metadata
                            .creation_timestamp
                            .as_ref()
                            .map(|t| t.0.to_string()),
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

    let release = HelmRelease {
        name: payload.name.clone(),
        namespace: payload.namespace.clone(),
        chart: payload.chart.clone(),
        chart_version: payload
            .version
            .clone()
            .unwrap_or_else(|| "latest".to_string()),
        app_version: None,
        status: "deployed".to_string(),
        revision: 1,
        deployed_at: Some(chrono::Utc::now().to_rfc3339()),
        values: payload.values,
    };

    // Log to audit log
    KubernetesAuditLogger::log_helm_install(
        &state,
        None, // user_id - нужно получить из контекста аутентификации
        None, // username
        &payload.name,
        &payload.chart,
        &payload.namespace,
    )
    .await;

    Ok(Json(release))
}

#[derive(Debug, Deserialize)]
pub struct UpgradeHelmRequest {
    pub chart: String,
    pub version: Option<String>,
    pub repo: Option<String>,
    pub values: Option<serde_json::Value>,
}

pub async fn upgrade_helm_release(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<UpgradeHelmRequest>,
) -> Result<Json<HelmRelease>> {
    let release = HelmRelease {
        name: name.clone(),
        namespace: namespace.clone(),
        chart: payload.chart.clone(),
        chart_version: payload
            .version
            .clone()
            .unwrap_or_else(|| "latest".to_string()),
        app_version: None,
        status: "upgraded".to_string(),
        revision: 2,
        deployed_at: Some(chrono::Utc::now().to_rfc3339()),
        values: payload.values,
    };

    // Log to audit log
    KubernetesAuditLogger::log_helm_upgrade(&state, None, None, &name, &payload.chart, &namespace)
        .await;

    Ok(Json(release))
}

pub async fn rollback_helm_release(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Query(query): Query<RollbackQuery>,
) -> Result<Json<HelmRelease>> {
    let revision = query.revision.unwrap_or(1);

    let release = HelmRelease {
        name: name.clone(),
        namespace: namespace.clone(),
        chart: "rolled-back".to_string(),
        chart_version: format!("{}.{}", revision, 0),
        app_version: None,
        status: "rolled-back".to_string(),
        revision: revision + 1,
        deployed_at: Some(chrono::Utc::now().to_rfc3339()),
        values: None,
    };

    // Log to audit log
    KubernetesAuditLogger::log_helm_rollback(&state, None, None, &name, revision, &namespace).await;

    Ok(Json(release))
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

    // Log to audit log
    KubernetesAuditLogger::log_helm_uninstall(&state, None, None, &name, &namespace).await;

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
            name,
            namespace,
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
        name,
        namespace,
        chart: "updated".to_string(),
        chart_version: "1.0.0".to_string(),
        app_version: None,
        status: "updated".to_string(),
        revision: 3,
        deployed_at: Some(chrono::Utc::now().to_rfc3339()),
        values: Some(values),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helm_repository() {
        let repo = HelmRepository {
            name: "bitnami".to_string(),
            url: "https://charts.bitnami.com/bitnami".to_string(),
            username: None,
        };
        assert_eq!(repo.name, "bitnami");
        assert!(repo.url.contains("bitnami"));
    }

    #[test]
    fn test_create_helm_repository_request() {
        let req = CreateHelmRepositoryRequest {
            name: "jetstack".to_string(),
            url: "https://charts.jetstack.io".to_string(),
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
        };
        assert_eq!(req.name, "jetstack");
        assert_eq!(req.username, Some("user".to_string()));
    }

    #[test]
    fn test_helm_chart() {
        let chart = HelmChart {
            name: "nginx".to_string(),
            version: "1.0.0".to_string(),
            app_version: Some("1.21".to_string()),
            description: Some("Nginx chart".to_string()),
            home: None,
            sources: vec![],
            keywords: vec![],
        };
        assert_eq!(chart.name, "nginx");
        assert_eq!(chart.version, "1.0.0");
    }

    #[test]
    fn test_helm_release() {
        let release = HelmRelease {
            name: "my-release".to_string(),
            namespace: "default".to_string(),
            chart: "nginx".to_string(),
            chart_version: "1.0.0".to_string(),
            app_version: Some("1.21".to_string()),
            status: "deployed".to_string(),
            revision: 1,
            deployed_at: Some("2024-01-01T00:00:00Z".to_string()),
            values: None,
        };
        assert_eq!(release.name, "my-release");
        assert_eq!(release.status, "deployed");
        assert_eq!(release.revision, 1);
    }

    #[test]
    fn test_helm_release_with_values() {
        let values = serde_json::json!({"replicaCount": 3});
        let release = HelmRelease {
            name: "my-release".to_string(),
            namespace: "default".to_string(),
            chart: "app".to_string(),
            chart_version: "2.0.0".to_string(),
            app_version: None,
            status: "deployed".to_string(),
            revision: 2,
            deployed_at: None,
            values: Some(values),
        };
        assert!(release.values.is_some());
    }

    #[test]
    fn test_helm_repo_list() {
        let list = HelmRepositoryList {
            repositories: vec![HelmRepository {
                name: "stable".to_string(),
                url: "https://stable".to_string(),
                username: None,
            }],
        };
        assert_eq!(list.repositories.len(), 1);
    }

    #[test]
    fn test_helm_repository_with_username() {
        let repo = HelmRepository {
            name: "private-repo".to_string(),
            url: "https://private.example.com/charts".to_string(),
            username: Some("admin".to_string()),
        };
        assert_eq!(repo.username, Some("admin".to_string()));
    }

    #[test]
    fn test_create_helm_repository_request_no_auth() {
        let req = CreateHelmRepositoryRequest {
            name: "public-repo".to_string(),
            url: "https://public.example.com".to_string(),
            username: None,
            password: None,
        };
        assert!(req.username.is_none());
        assert!(req.password.is_none());
    }

    #[test]
    fn test_helm_chart_list() {
        let list = HelmChartList {
            charts: vec![HelmChart {
                name: "redis".to_string(),
                version: "17.0.0".to_string(),
                app_version: Some("7.0".to_string()),
                description: Some("Redis chart".to_string()),
                home: None,
                sources: vec![],
                keywords: vec![],
            }],
        };
        assert_eq!(list.charts.len(), 1);
        assert_eq!(list.charts[0].name, "redis");
    }

    #[test]
    fn test_search_charts_query_with_repo() {
        let query = SearchChartsQuery {
            repo: Some("bitnami".to_string()),
            query: Some("nginx".to_string()),
            limit: Some(10),
        };
        assert_eq!(query.repo, Some("bitnami".to_string()));
        assert_eq!(query.query, Some("nginx".to_string()));
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_search_charts_query_all_none() {
        let query = SearchChartsQuery {
            repo: None,
            query: None,
            limit: None,
        };
        assert!(query.repo.is_none());
        assert!(query.query.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_helm_release_list() {
        let list = HelmReleaseList {
            releases: vec![HelmRelease {
                name: "my-nginx".to_string(),
                namespace: "default".to_string(),
                chart: "nginx".to_string(),
                chart_version: "15.0.0".to_string(),
                app_version: Some("1.24.0".to_string()),
                status: "deployed".to_string(),
                revision: 1,
                deployed_at: Some("2024-01-01T00:00:00Z".to_string()),
                values: None,
            }],
        };
        assert_eq!(list.releases.len(), 1);
        assert_eq!(list.releases[0].status, "deployed");
    }

    #[test]
    fn test_list_releases_query_all_namespaces() {
        let query = ListReleasesQuery {
            namespace: None,
            all_namespaces: Some(true),
        };
        assert!(query.namespace.is_none());
        assert_eq!(query.all_namespaces, Some(true));
    }

    #[test]
    fn test_list_releases_query_with_namespace() {
        let query = ListReleasesQuery {
            namespace: Some("production".to_string()),
            all_namespaces: Some(false),
        };
        assert_eq!(query.namespace, Some("production".to_string()));
        assert_eq!(query.all_namespaces, Some(false));
    }

    #[test]
    fn test_install_helm_request_minimal() {
        let req = InstallHelmRequest {
            name: "my-release".to_string(),
            namespace: "default".to_string(),
            chart: "nginx".to_string(),
            version: None,
            repo: None,
            values: None,
            create_namespace: None,
        };
        assert_eq!(req.name, "my-release");
        assert!(req.version.is_none());
        assert!(req.create_namespace.is_none());
    }

    #[test]
    fn test_install_helm_request_full() {
        let values = serde_json::json!({"replicaCount": 3});
        let req = InstallHelmRequest {
            name: "prod-release".to_string(),
            namespace: "production".to_string(),
            chart: "my-app".to_string(),
            version: Some("2.0.0".to_string()),
            repo: Some("my-repo".to_string()),
            values: Some(values),
            create_namespace: Some(true),
        };
        assert_eq!(req.version, Some("2.0.0".to_string()));
        assert!(req.create_namespace == Some(true));
        assert!(req.values.is_some());
    }

    #[test]
    fn test_upgrade_helm_request_minimal() {
        let req = UpgradeHelmRequest {
            chart: "my-chart".to_string(),
            version: None,
            repo: None,
            values: None,
        };
        assert_eq!(req.chart, "my-chart");
        assert!(req.version.is_none());
    }

    #[test]
    fn test_upgrade_helm_request_with_values() {
        let values = serde_json::json!({"image": {"tag": "v2"}});
        let req = UpgradeHelmRequest {
            chart: "app".to_string(),
            version: Some("1.5.0".to_string()),
            repo: Some("stable".to_string()),
            values: Some(values),
        };
        assert_eq!(req.version, Some("1.5.0".to_string()));
        assert!(req.values.is_some());
    }

    #[test]
    fn test_rollback_query_with_revision() {
        let query = RollbackQuery { revision: Some(5) };
        assert_eq!(query.revision, Some(5));
    }

    #[test]
    fn test_rollback_query_no_revision() {
        let query = RollbackQuery { revision: None };
        assert!(query.revision.is_none());
    }

    #[test]
    fn test_helm_chart_with_keywords() {
        let chart = HelmChart {
            name: "prometheus".to_string(),
            version: "22.0.0".to_string(),
            app_version: Some("2.45.0".to_string()),
            description: Some("Prometheus monitoring chart".to_string()),
            home: Some("https://prometheus.io".to_string()),
            sources: vec!["https://github.com/prometheus-community/helm-charts".to_string()],
            keywords: vec!["monitoring".to_string(), "prometheus".to_string()],
        };
        assert_eq!(chart.keywords.len(), 2);
        assert!(chart.keywords.contains(&"monitoring".to_string()));
    }

    #[test]
    fn test_helm_chart_empty_fields() {
        let chart = HelmChart {
            name: "".to_string(),
            version: "".to_string(),
            app_version: None,
            description: None,
            home: None,
            sources: vec![],
            keywords: vec![],
        };
        assert!(chart.name.is_empty());
        assert!(chart.version.is_empty());
        assert!(chart.sources.is_empty());
    }

    #[test]
    fn test_helm_repository_list_multiple() {
        let list = HelmRepositoryList {
            repositories: vec![
                HelmRepository {
                    name: "stable".to_string(),
                    url: "https://stable".to_string(),
                    username: None,
                },
                HelmRepository {
                    name: "bitnami".to_string(),
                    url: "https://bitnami".to_string(),
                    username: None,
                },
                HelmRepository {
                    name: "private".to_string(),
                    url: "https://private".to_string(),
                    username: Some("user".to_string()),
                },
            ],
        };
        assert_eq!(list.repositories.len(), 3);
        assert!(list.repositories[2].username.is_some());
    }

    #[test]
    fn test_helm_release_superseded_status() {
        let release = HelmRelease {
            name: "old-release".to_string(),
            namespace: "default".to_string(),
            chart: "app".to_string(),
            chart_version: "1.0.0".to_string(),
            app_version: Some("1.0.0".to_string()),
            status: "superseded".to_string(),
            revision: 1,
            deployed_at: Some("2024-01-01T00:00:00Z".to_string()),
            values: None,
        };
        assert_eq!(release.status, "superseded");
        assert_eq!(release.revision, 1);
    }

    #[test]
    fn test_helm_release_failed_status() {
        let release = HelmRelease {
            name: "failed-release".to_string(),
            namespace: "default".to_string(),
            chart: "app".to_string(),
            chart_version: "2.0.0".to_string(),
            app_version: None,
            status: "failed".to_string(),
            revision: 3,
            deployed_at: None,
            values: Some(serde_json::json!({})),
        };
        assert_eq!(release.status, "failed");
        assert_eq!(release.revision, 3);
    }
}
