//! GitOps draft integration (read-only): detect ArgoCD/Flux and list core objects.

use axum::{
    Json,
    extract::{Query, State},
};
use kube::{
    api::{Api, DynamicObject, ListParams},
    core::{ApiResource, GroupVersionKind},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct GitOpsQuery {
    pub namespace: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct GitOpsStatus {
    pub installed: bool,
    pub argo_installed: bool,
    pub flux_installed: bool,
    pub details: Vec<String>,
}

fn ar(group: &str, version: &str, kind: &str, plural: &str) -> ApiResource {
    ApiResource::from_gvk_with_plural(&GroupVersionKind::gvk(group, version, kind), plural)
}

fn dyn_api(
    raw: kube::Client,
    namespace: Option<&str>,
    api_res: &ApiResource,
) -> Api<DynamicObject> {
    if let Some(ns) = namespace {
        Api::namespaced_with(raw, ns, api_res)
    } else {
        Api::all_with(raw, api_res)
    }
}

async fn is_api_available(raw: kube::Client, api_res: &ApiResource) -> bool {
    let api: Api<DynamicObject> = Api::all_with(raw, api_res);
    api.list(&ListParams::default().limit(1)).await.is_ok()
}

pub async fn get_gitops_status(State(state): State<Arc<AppState>>) -> Result<Json<GitOpsStatus>> {
    let client = state.kubernetes_client()?;
    let raw = client.raw().clone();

    let argo_app = ar("argoproj.io", "v1alpha1", "Application", "applications");
    let argo_proj = ar("argoproj.io", "v1alpha1", "AppProject", "appprojects");
    let flux_ks = ar(
        "kustomize.toolkit.fluxcd.io",
        "v1",
        "Kustomization",
        "kustomizations",
    );
    let flux_hr = ar(
        "helm.toolkit.fluxcd.io",
        "v2",
        "HelmRelease",
        "helmreleases",
    );

    let argo_app_ok = is_api_available(raw.clone(), &argo_app).await;
    let argo_proj_ok = is_api_available(raw.clone(), &argo_proj).await;
    let flux_ks_ok = is_api_available(raw.clone(), &flux_ks).await;
    let flux_hr_ok = is_api_available(raw, &flux_hr).await;

    let argo_installed = argo_app_ok || argo_proj_ok;
    let flux_installed = flux_ks_ok || flux_hr_ok;
    let mut details = Vec::new();
    if argo_app_ok {
        details.push("argocd Applications API detected".to_string());
    }
    if argo_proj_ok {
        details.push("argocd AppProjects API detected".to_string());
    }
    if flux_ks_ok {
        details.push("flux Kustomizations API detected".to_string());
    }
    if flux_hr_ok {
        details.push("flux HelmReleases API detected".to_string());
    }

    Ok(Json(GitOpsStatus {
        installed: argo_installed || flux_installed,
        argo_installed,
        flux_installed,
        details,
    }))
}

pub async fn list_argocd_applications(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GitOpsQuery>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let client = state.kubernetes_client()?;
    let api_res = ar("argoproj.io", "v1alpha1", "Application", "applications");
    let api = dyn_api(client.raw().clone(), query.namespace.as_deref(), &api_res);
    let mut lp = ListParams::default();
    if let Some(limit) = query.limit {
        lp = lp.limit(limit);
    }
    let items = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("ArgoCD Application API not available: {e}")))?;
    Ok(Json(
        items.items.iter().map(|x| serde_json::json!(x)).collect(),
    ))
}

pub async fn list_flux_kustomizations(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GitOpsQuery>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let client = state.kubernetes_client()?;
    let api_res = ar(
        "kustomize.toolkit.fluxcd.io",
        "v1",
        "Kustomization",
        "kustomizations",
    );
    let api = dyn_api(client.raw().clone(), query.namespace.as_deref(), &api_res);
    let mut lp = ListParams::default();
    if let Some(limit) = query.limit {
        lp = lp.limit(limit);
    }
    let items = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("Flux Kustomization API not available: {e}")))?;
    Ok(Json(
        items.items.iter().map(|x| serde_json::json!(x)).collect(),
    ))
}

pub async fn list_flux_helm_releases(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GitOpsQuery>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let client = state.kubernetes_client()?;
    let api_res = ar(
        "helm.toolkit.fluxcd.io",
        "v2",
        "HelmRelease",
        "helmreleases",
    );
    let api = dyn_api(client.raw().clone(), query.namespace.as_deref(), &api_res);
    let mut lp = ListParams::default();
    if let Some(limit) = query.limit {
        lp = lp.limit(limit);
    }
    let items = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("Flux HelmRelease API not available: {e}")))?;
    Ok(Json(
        items.items.iter().map(|x| serde_json::json!(x)).collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitops_query_with_namespace() {
        let json = r#"{"namespace": "argocd", "limit": 10}"#;
        let query: GitOpsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.namespace, Some("argocd".to_string()));
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_gitops_query_empty() {
        let json = r#"{}"#;
        let query: GitOpsQuery = serde_json::from_str(json).unwrap();
        assert!(query.namespace.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_gitops_query_limit_only() {
        let json = r#"{"limit": 100}"#;
        let query: GitOpsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, Some(100));
        assert!(query.namespace.is_none());
    }

    #[test]
    fn test_gitops_status_not_installed() {
        let status = GitOpsStatus {
            installed: false,
            argo_installed: false,
            flux_installed: false,
            details: vec![],
        };
        assert!(!status.installed);
        assert!(!status.argo_installed);
        assert!(!status.flux_installed);
        assert!(status.details.is_empty());
    }

    #[test]
    fn test_gitops_status_argo_only() {
        let status = GitOpsStatus {
            installed: true,
            argo_installed: true,
            flux_installed: false,
            details: vec!["argocd Applications API detected".to_string()],
        };
        assert!(status.installed);
        assert!(status.argo_installed);
        assert!(!status.flux_installed);
        assert_eq!(status.details.len(), 1);
    }

    #[test]
    fn test_gitops_status_flux_only() {
        let status = GitOpsStatus {
            installed: true,
            argo_installed: false,
            flux_installed: true,
            details: vec!["flux Kustomizations API detected".to_string()],
        };
        assert!(status.installed);
        assert!(!status.argo_installed);
        assert!(status.flux_installed);
    }

    #[test]
    fn test_gitops_status_both_installed() {
        let status = GitOpsStatus {
            installed: true,
            argo_installed: true,
            flux_installed: true,
            details: vec![
                "argocd Applications API detected".to_string(),
                "flux Kustomizations API detected".to_string(),
            ],
        };
        assert!(status.installed);
        assert!(status.argo_installed);
        assert!(status.flux_installed);
        assert_eq!(status.details.len(), 2);
    }

    #[test]
    fn test_gitops_status_serialization() {
        let status = GitOpsStatus {
            installed: true,
            argo_installed: true,
            flux_installed: false,
            details: vec!["test".to_string()],
        };
        let json = serde_json::to_string(&status).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["installed"], true);
        assert_eq!(parsed["argo_installed"], true);
        assert_eq!(parsed["flux_installed"], false);
    }

    #[test]
    fn test_api_resource_argocd_application() {
        let ar = ar("argoproj.io", "v1alpha1", "Application", "applications");
        assert_eq!(ar.group, "argoproj.io");
        assert_eq!(ar.version, "v1alpha1");
        assert_eq!(ar.kind, "Application");
    }

    #[test]
    fn test_api_resource_flux_kustomization() {
        let ar = ar(
            "kustomize.toolkit.fluxcd.io",
            "v1",
            "Kustomization",
            "kustomizations",
        );
        assert_eq!(ar.group, "kustomize.toolkit.fluxcd.io");
        assert_eq!(ar.version, "v1");
        assert_eq!(ar.kind, "Kustomization");
    }

    #[test]
    fn test_api_resource_flux_helmrelease() {
        let ar = ar(
            "helm.toolkit.fluxcd.io",
            "v2",
            "HelmRelease",
            "helmreleases",
        );
        assert_eq!(ar.group, "helm.toolkit.fluxcd.io");
        assert_eq!(ar.version, "v2");
        assert_eq!(ar.kind, "HelmRelease");
    }

    #[test]
    fn test_api_resource_argocd_appproject() {
        let ar = ar("argoproj.io", "v1alpha1", "AppProject", "appprojects");
        assert_eq!(ar.group, "argoproj.io");
        assert_eq!(ar.kind, "AppProject");
    }

    #[test]
    fn test_gitops_status_details_multiple() {
        let status = GitOpsStatus {
            installed: true,
            argo_installed: true,
            flux_installed: true,
            details: vec![
                "argocd Applications API detected".to_string(),
                "argocd AppProjects API detected".to_string(),
                "flux Kustomizations API detected".to_string(),
                "flux HelmReleases API detected".to_string(),
            ],
        };
        assert_eq!(status.details.len(), 4);
        assert!(status.details.iter().any(|d| d.contains("argocd")));
        assert!(status.details.iter().any(|d| d.contains("flux")));
    }
}
