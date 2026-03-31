//! Kubernetes Helm handlers — Phase 9
//!
//! HTTP handlers поверх `kubernetes::helm::HelmClient` (subprocess-based).
//! Все блокирующие вызовы выполняются через `spawn_blocking`.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::error::{Error, Result};

// ── Kubeconfig helper ─────────────────────────────────────────────
fn get_kubeconfig(state: &AppState) -> Option<String> {
    state.config.kubernetes.as_ref().and_then(|k| k.kubeconfig_path.clone())
}

fn helm_cmd(kubeconfig: &Option<String>) -> std::process::Command {
    let mut cmd = std::process::Command::new("helm");
    if let Some(kc) = kubeconfig {
        cmd.env("KUBECONFIG", kc);
    }
    cmd
}

// ── Types ─────────────────────────────────────────────────────────

/// Статус helm
#[derive(Debug, Serialize)]
pub struct HelmStatus {
    pub available: bool,
    pub version: Option<String>,
    pub message: String,
}

/// Helm репозиторий
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmRepoInfo {
    pub name: String,
    pub url: String,
}

/// Helm release (из `helm list -o json`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmReleaseInfo {
    pub name: String,
    pub namespace: String,
    pub revision: String,
    pub updated: String,
    pub status: String,
    pub chart: String,
    pub app_version: String,
}

/// История release (из `helm history -o json`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmHistoryEntry {
    pub revision: i64,
    pub updated: String,
    pub status: String,
    pub chart: String,
    pub app_version: String,
    pub description: String,
}

/// Результат поиска чарта
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmChartResult {
    pub name: String,
    pub version: String,
    pub app_version: String,
    pub description: String,
}

/// Payload для добавления репо
#[derive(Debug, Deserialize)]
pub struct AddRepoPayload {
    pub name: String,
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// Payload для install/upgrade
#[derive(Debug, Deserialize)]
pub struct InstallPayload {
    pub release_name: String,
    pub chart: String,
    pub version: Option<String>,
    pub namespace: String,
    pub values: Option<HashMap<String, String>>,
    pub dry_run: Option<bool>,
}

/// Payload для rollback
#[derive(Debug, Deserialize)]
pub struct RollbackPayload {
    pub revision: i32,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub repo: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseListQuery {
    pub namespace: Option<String>,
    pub all_namespaces: Option<bool>,
}

// ── Handlers ──────────────────────────────────────────────────────

/// GET /api/kubernetes/helm/status
pub async fn helm_status(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<HelmStatus>> {
    let result = tokio::task::spawn_blocking(move || -> Result<String> {
        // Run `helm version --short`
        let output = std::process::Command::new("helm")
            .arg("version")
            .arg("--short")
            .output()
            .map_err(|e| Error::Other(format!("helm not found: {e}")))?;
        if !output.status.success() {
            return Err(Error::Other("helm not working".to_string()));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }).await.map_err(|e| Error::Other(e.to_string()))?;

    match result {
        Ok(version) => Ok(Json(HelmStatus {
            available: true,
            version: Some(version.clone()),
            message: format!("Helm доступен: {version}"),
        })),
        Err(e) => Ok(Json(HelmStatus {
            available: false,
            version: None,
            message: e.to_string(),
        })),
    }
}

// ── Repositories ──────────────────────────────────────────────────

/// GET /api/kubernetes/helm/repos
pub async fn list_helm_repos(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<HelmRepoInfo>>> {
    let repos = tokio::task::spawn_blocking(move || -> Result<Vec<HelmRepoInfo>> {
        let output = std::process::Command::new("helm")
            .arg("repo").arg("list").arg("-o").arg("json")
            .output()
            .map_err(|e| Error::Other(format!("helm error: {e}")))?;

        if !output.status.success() {
            // Empty repo list returns exit 1 with "no repositories"
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("no repositories") || output.stdout.is_empty() {
                return Ok(vec![]);
            }
            return Err(Error::Other(stderr.to_string()));
        }
        let repos: Vec<HelmRepoInfo> = serde_json::from_slice(&output.stdout)
            .map_err(|e| Error::Other(format!("parse error: {e}")))?;
        Ok(repos)
    }).await.map_err(|e| Error::Other(e.to_string()))??;

    Ok(Json(repos))
}

/// POST /api/kubernetes/helm/repos
pub async fn add_helm_repo(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddRepoPayload>,
) -> Result<Json<serde_json::Value>> {
    let kc = get_kubeconfig(&state);
    let name = payload.name.clone();
    let url = payload.url.clone();
    let username = payload.username.clone();
    let password = payload.password.clone();

    tokio::task::spawn_blocking(move || -> Result<()> {
        let mut cmd = helm_cmd(&kc);
        cmd.arg("repo").arg("add").arg(&name).arg(&url);
        if let Some(u) = &username { cmd.arg("--username").arg(u); }
        if let Some(p) = &password { cmd.arg("--password").arg(p); }
        let out = cmd.output().map_err(|e| Error::Other(e.to_string()))?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            return Err(Error::Other(stderr));
        }
        let _ = std::process::Command::new("helm").arg("repo").arg("update").output();
        Ok(())
    }).await.map_err(|e| Error::Other(e.to_string()))??;

    Ok(Json(serde_json::json!({"added": true, "name": payload.name})))
}

/// DELETE /api/kubernetes/helm/repos/{name}
pub async fn remove_helm_repo(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let repo_name = name.clone();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let out = std::process::Command::new("helm")
            .arg("repo").arg("remove").arg(&repo_name)
            .output()
            .map_err(|e| Error::Other(e.to_string()))?;
        if !out.status.success() {
            return Err(Error::Other(String::from_utf8_lossy(&out.stderr).to_string()));
        }
        Ok(())
    }).await.map_err(|e| Error::Other(e.to_string()))??;

    Ok(Json(serde_json::json!({"removed": true, "name": name})))
}

/// POST /api/kubernetes/helm/repos/update
pub async fn update_helm_repos(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>> {
    let out = tokio::task::spawn_blocking(|| {
        std::process::Command::new("helm").arg("repo").arg("update")
            .output()
            .map_err(|e| Error::Other(e.to_string()))
    }).await.map_err(|e| Error::Other(e.to_string()))??;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        return Err(Error::Other(stderr));
    }
    let msg = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(Json(serde_json::json!({"updated": true, "message": msg})))
}

// ── Releases ──────────────────────────────────────────────────────

/// GET /api/kubernetes/helm/releases
pub async fn list_helm_releases(
    State(_state): State<Arc<AppState>>,
    Query(q): Query<ReleaseListQuery>,
) -> Result<Json<Vec<HelmReleaseInfo>>> {
    let ns = q.namespace.clone();
    let all_ns = q.all_namespaces.unwrap_or(true);

    let releases = tokio::task::spawn_blocking(move || -> Result<Vec<HelmReleaseInfo>> {
        let mut cmd = std::process::Command::new("helm");
        cmd.arg("list").arg("-o").arg("json");
        if all_ns || ns.is_none() {
            cmd.arg("--all-namespaces");
        } else if let Some(n) = &ns {
            cmd.arg("-n").arg(n);
        }
        let out = cmd.output().map_err(|e| Error::Other(e.to_string()))?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            if stderr.contains("no releases") || out.stdout == b"[]" || out.stdout.is_empty() {
                return Ok(vec![]);
            }
            return Err(Error::Other(stderr.to_string()));
        }
        if out.stdout.is_empty() || out.stdout == b"null" {
            return Ok(vec![]);
        }
        let releases: Vec<HelmReleaseInfo> = serde_json::from_slice(&out.stdout)
            .map_err(|e| Error::Other(format!("parse error: {e}; raw: {}", String::from_utf8_lossy(&out.stdout))))?;
        Ok(releases)
    }).await.map_err(|e| Error::Other(e.to_string()))??;

    Ok(Json(releases))
}

/// GET /api/kubernetes/helm/namespaces/{ns}/releases/{name}
pub async fn get_helm_release(
    State(_state): State<Arc<AppState>>,
    Path((ns, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let (ns2, name2) = (ns.clone(), name.clone());
    let info = tokio::task::spawn_blocking(move || -> Result<serde_json::Value> {
        let out = std::process::Command::new("helm")
            .arg("status").arg(&name2).arg("-n").arg(&ns2).arg("-o").arg("json")
            .output()
            .map_err(|e| Error::Other(e.to_string()))?;
        if !out.status.success() {
            return Err(Error::Other(String::from_utf8_lossy(&out.stderr).to_string()));
        }
        let v: serde_json::Value = serde_json::from_slice(&out.stdout)
            .map_err(|e| Error::Other(e.to_string()))?;
        Ok(v)
    }).await.map_err(|e| Error::Other(e.to_string()))??;

    Ok(Json(info))
}

/// POST /api/kubernetes/helm/releases  (install)
pub async fn install_helm_release(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<InstallPayload>,
) -> Result<Json<serde_json::Value>> {
    let kc = get_kubeconfig(&state);
    let out = tokio::task::spawn_blocking(move || -> Result<String> {
        let mut cmd = helm_cmd(&kc);
        cmd.arg("install")
            .arg(&payload.release_name)
            .arg(&payload.chart)
            .arg("-n").arg(&payload.namespace)
            .arg("-o").arg("json");
        if let Some(v) = &payload.version { cmd.arg("--version").arg(v); }
        if payload.dry_run.unwrap_or(false) { cmd.arg("--dry-run"); }
        if let Some(vals) = &payload.values {
            for (k, v) in vals { cmd.arg("--set").arg(format!("{k}={v}")); }
        }
        let out = cmd.output().map_err(|e| Error::Other(e.to_string()))?;
        if !out.status.success() {
            return Err(Error::Other(String::from_utf8_lossy(&out.stderr).to_string()));
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }).await.map_err(|e| Error::Other(e.to_string()))??;

    let v: serde_json::Value = serde_json::from_str(&out).unwrap_or(serde_json::json!({"output": out}));
    Ok(Json(v))
}

/// PUT /api/kubernetes/helm/namespaces/{ns}/releases/{name}  (upgrade)
pub async fn upgrade_helm_release(
    State(state): State<Arc<AppState>>,
    Path((ns, name)): Path<(String, String)>,
    Json(payload): Json<InstallPayload>,
) -> Result<Json<serde_json::Value>> {
    let kc = get_kubeconfig(&state);
    let out = tokio::task::spawn_blocking(move || -> Result<String> {
        let mut cmd = helm_cmd(&kc);
        cmd.arg("upgrade")
            .arg(&name)
            .arg(&payload.chart)
            .arg("-n").arg(&ns)
            .arg("-o").arg("json");
        if let Some(v) = &payload.version { cmd.arg("--version").arg(v); }
        if payload.dry_run.unwrap_or(false) { cmd.arg("--dry-run"); }
        if let Some(vals) = &payload.values {
            for (k, v) in vals { cmd.arg("--set").arg(format!("{k}={v}")); }
        }
        let out = cmd.output().map_err(|e| Error::Other(e.to_string()))?;
        if !out.status.success() {
            return Err(Error::Other(String::from_utf8_lossy(&out.stderr).to_string()));
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }).await.map_err(|e| Error::Other(e.to_string()))??;

    let v: serde_json::Value = serde_json::from_str(&out).unwrap_or(serde_json::json!({"output": out}));
    Ok(Json(v))
}

/// DELETE /api/kubernetes/helm/namespaces/{ns}/releases/{name}
pub async fn uninstall_helm_release(
    State(state): State<Arc<AppState>>,
    Path((ns, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let kc = get_kubeconfig(&state);
    let (ns2, name2) = (ns.clone(), name.clone());
    tokio::task::spawn_blocking(move || -> Result<()> {
        let mut cmd = helm_cmd(&kc);
        cmd.arg("uninstall").arg(&name2).arg("-n").arg(&ns2);
        let out = cmd.output().map_err(|e| Error::Other(e.to_string()))?;
        if !out.status.success() {
            return Err(Error::Other(String::from_utf8_lossy(&out.stderr).to_string()));
        }
        Ok(())
    }).await.map_err(|e| Error::Other(e.to_string()))??;

    Ok(Json(serde_json::json!({"uninstalled": true, "name": name})))
}

/// POST /api/kubernetes/helm/namespaces/{ns}/releases/{name}/rollback
pub async fn rollback_helm_release(
    State(state): State<Arc<AppState>>,
    Path((ns, name)): Path<(String, String)>,
    Json(payload): Json<RollbackPayload>,
) -> Result<Json<serde_json::Value>> {
    let kc = get_kubeconfig(&state);
    let (ns2, name2) = (ns.clone(), name.clone());
    let rev = payload.revision;
    tokio::task::spawn_blocking(move || -> Result<()> {
        let mut cmd = helm_cmd(&kc);
        cmd.arg("rollback").arg(&name2).arg(rev.to_string()).arg("-n").arg(&ns2);
        let out = cmd.output().map_err(|e| Error::Other(e.to_string()))?;
        if !out.status.success() {
            return Err(Error::Other(String::from_utf8_lossy(&out.stderr).to_string()));
        }
        Ok(())
    }).await.map_err(|e| Error::Other(e.to_string()))??;

    Ok(Json(serde_json::json!({"rolled_back": true, "revision": rev})))
}

/// GET /api/kubernetes/helm/namespaces/{ns}/releases/{name}/history
pub async fn helm_release_history(
    State(_state): State<Arc<AppState>>,
    Path((ns, name)): Path<(String, String)>,
) -> Result<Json<Vec<HelmHistoryEntry>>> {
    let history = tokio::task::spawn_blocking(move || -> Result<Vec<HelmHistoryEntry>> {
        let out = std::process::Command::new("helm")
            .arg("history").arg(&name).arg("-n").arg(&ns).arg("-o").arg("json")
            .output()
            .map_err(|e| Error::Other(e.to_string()))?;
        if !out.status.success() {
            return Err(Error::Other(String::from_utf8_lossy(&out.stderr).to_string()));
        }
        let h: Vec<HelmHistoryEntry> = serde_json::from_slice(&out.stdout)
            .map_err(|e| Error::Other(format!("parse error: {e}")))?;
        Ok(h)
    }).await.map_err(|e| Error::Other(e.to_string()))??;

    Ok(Json(history))
}

/// GET /api/kubernetes/helm/releases/{name}/values  (get values)
pub async fn get_helm_release_values(
    State(_state): State<Arc<AppState>>,
    Path((ns, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let vals = tokio::task::spawn_blocking(move || -> Result<serde_json::Value> {
        let out = std::process::Command::new("helm")
            .arg("get").arg("values").arg(&name).arg("-n").arg(&ns).arg("-o").arg("json")
            .output()
            .map_err(|e| Error::Other(e.to_string()))?;
        if !out.status.success() {
            return Ok(serde_json::json!({}));
        }
        let v: serde_json::Value = serde_json::from_slice(&out.stdout)
            .unwrap_or(serde_json::json!({}));
        Ok(v)
    }).await.map_err(|e| Error::Other(e.to_string()))??;

    Ok(Json(vals))
}

/// GET /api/kubernetes/helm/namespaces/{ns}/releases/{name}/manifest
pub async fn get_helm_release_manifest(
    State(_state): State<Arc<AppState>>,
    Path((ns, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let manifest = tokio::task::spawn_blocking(move || -> Result<String> {
        let out = std::process::Command::new("helm")
            .arg("get").arg("manifest").arg(&name).arg("-n").arg(&ns)
            .output()
            .map_err(|e| Error::Other(e.to_string()))?;
        if !out.status.success() {
            return Err(Error::Other(String::from_utf8_lossy(&out.stderr).to_string()));
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }).await.map_err(|e| Error::Other(e.to_string()))??;

    Ok(Json(serde_json::json!({"manifest": manifest})))
}

// ── Search ────────────────────────────────────────────────────────

/// GET /api/kubernetes/helm/search
pub async fn search_helm_charts(
    State(_state): State<Arc<AppState>>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<HelmChartResult>>> {
    let query = q.q.unwrap_or_default();
    let charts = tokio::task::spawn_blocking(move || -> Result<Vec<HelmChartResult>> {
        let mut cmd = std::process::Command::new("helm");
        cmd.arg("search").arg("repo").arg(&query).arg("-o").arg("json");
        let out = cmd.output().map_err(|e| Error::Other(e.to_string()))?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            if stderr.contains("no repositories") {
                return Ok(vec![]);
            }
            return Err(Error::Other(stderr.to_string()));
        }
        if out.stdout.is_empty() || out.stdout == b"null" {
            return Ok(vec![]);
        }
        // helm search repo -o json returns objects with name/version/app_version/description
        #[derive(Deserialize)]
        struct Raw {
            name: String,
            version: String,
            app_version: String,
            description: String,
        }
        let raw: Vec<Raw> = serde_json::from_slice(&out.stdout)
            .map_err(|e| Error::Other(format!("parse error: {e}")))?;
        Ok(raw.into_iter().map(|r| HelmChartResult {
            name: r.name,
            version: r.version,
            app_version: r.app_version,
            description: r.description,
        }).collect())
    }).await.map_err(|e| Error::Other(e.to_string()))??;

    Ok(Json(charts))
}

/// GET /api/kubernetes/helm/charts/{chart}/values
/// Get default values for a chart (helm show values {chart})
pub async fn get_chart_default_values(
    State(_state): State<Arc<AppState>>,
    Path(chart): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let vals = tokio::task::spawn_blocking(move || -> Result<String> {
        let out = std::process::Command::new("helm")
            .arg("show").arg("values").arg(&chart)
            .output()
            .map_err(|e| Error::Other(e.to_string()))?;
        if !out.status.success() {
            return Err(Error::Other(String::from_utf8_lossy(&out.stderr).to_string()));
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }).await.map_err(|e| Error::Other(e.to_string()))??;

    Ok(Json(serde_json::json!({"values_yaml": vals})))
}
