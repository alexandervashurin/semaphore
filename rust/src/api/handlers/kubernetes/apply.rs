//! Kubernetes Apply handlers — Phase 10
//!
//! YAML apply с dry-run, diff и генератором kubectl-команды

use axum::{
    extract::{Query, State},
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::error::{Error, Result};

// ── Types ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ApplyPayload {
    /// YAML или JSON манифест
    pub manifest: String,
    /// Dry-run: только проверить, не применять
    pub dry_run: Option<bool>,
    /// Field manager для SSA
    pub field_manager: Option<String>,
    /// Force conflicts при SSA
    pub force: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ApplyResult {
    pub success: bool,
    pub dry_run: bool,
    pub output: String,
    pub kubectl_command: String,
    pub resources: Vec<AppliedResource>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AppliedResource {
    pub kind: String,
    pub name: String,
    pub namespace: Option<String>,
    pub action: String, // created / configured / unchanged
}

#[derive(Debug, Deserialize)]
pub struct DiffQuery {
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DiffResult {
    pub has_diff: bool,
    pub diff: String,
    pub kubectl_command: String,
}

/// Генерирует kubectl-команду для действия
#[derive(Debug, Deserialize)]
pub struct KubectlGenQuery {
    pub action: String,       // apply, delete, scale, rollout-restart
    pub kind: Option<String>,
    pub name: Option<String>,
    pub namespace: Option<String>,
    pub replicas: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct KubectlCommand {
    pub command: String,
    pub description: String,
}

// ── Kubeconfig helper ─────────────────────────────────────────────
fn kube_env(state: &AppState) -> Option<String> {
    state.config.kubernetes.as_ref().and_then(|k| k.kubeconfig_path.clone())
}

fn kubectl_cmd(kubeconfig: &Option<String>) -> std::process::Command {
    let mut cmd = std::process::Command::new("kubectl");
    if let Some(kc) = kubeconfig {
        cmd.env("KUBECONFIG", kc);
    }
    cmd
}

// ── Handlers ──────────────────────────────────────────────────────

/// POST /api/kubernetes/apply
/// Apply YAML manifest (optionally dry-run)
pub async fn apply_manifest(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ApplyPayload>,
) -> Result<Json<ApplyResult>> {
    let kc = kube_env(&state);
    let manifest = payload.manifest.clone();
    let dry_run = payload.dry_run.unwrap_or(false);
    let field_manager = payload.field_manager.clone().unwrap_or_else(|| "velum".to_string());
    let force = payload.force.unwrap_or(false);

    // Build kubectl command string for display
    let kubectl_display = build_apply_command(&field_manager, dry_run, force);

    let result = tokio::task::spawn_blocking(move || -> Result<(String, Vec<u8>)> {
        // Write manifest to temp file
        let tmp = std::env::temp_dir().join(format!("velum-apply-{}.yaml", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()));
        std::fs::write(&tmp, manifest.as_bytes())
            .map_err(|e| Error::Other(format!("write tmp: {e}")))?;

        let mut cmd = kubectl_cmd(&kc);
        cmd.arg("apply").arg("-f").arg(&tmp)
            .arg("--field-manager").arg(&field_manager)
            .arg("-o").arg("json");
        if dry_run {
            cmd.arg("--dry-run=server");
        }
        if force {
            cmd.arg("--force-conflicts");
        }
        let out = cmd.output().map_err(|e| Error::Other(e.to_string()))?;
        let _ = std::fs::remove_file(&tmp);
        Ok((String::from_utf8_lossy(&out.stderr).to_string(), out.stdout))
    }).await.map_err(|e| Error::Other(e.to_string()))?;

    match result {
        Ok((stderr, stdout)) => {
            let warnings: Vec<String> = stderr.lines()
                .filter(|l| l.starts_with("Warning:"))
                .map(String::from).collect();

            let resources = parse_kubectl_apply_output(&stdout);
            let output_text = String::from_utf8_lossy(&stdout).to_string();

            Ok(Json(ApplyResult {
                success: true,
                dry_run,
                output: if output_text.len() > 4000 { output_text[..4000].to_string() + "…" } else { output_text },
                kubectl_command: kubectl_display,
                resources,
                warnings,
            }))
        }
        Err(e) => Ok(Json(ApplyResult {
            success: false,
            dry_run,
            output: e.to_string(),
            kubectl_command: kubectl_display,
            resources: vec![],
            warnings: vec![],
        })),
    }
}

/// POST /api/kubernetes/apply/diff
/// Compute diff between current state and manifest
pub async fn diff_manifest(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ApplyPayload>,
) -> Result<Json<DiffResult>> {
    let kc = kube_env(&state);
    let manifest = payload.manifest.clone();

    let kubectl_display = "kubectl diff -f -".to_string();

    let result = tokio::task::spawn_blocking(move || -> Result<String> {
        let tmp = std::env::temp_dir().join(format!("velum-diff-{}.yaml", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()));
        std::fs::write(&tmp, manifest.as_bytes())
            .map_err(|e| Error::Other(format!("write tmp: {e}")))?;

        let mut cmd = kubectl_cmd(&kc);
        cmd.arg("diff").arg("-f").arg(&tmp);
        let out = cmd.output().map_err(|e| Error::Other(e.to_string()))?;
        let _ = std::fs::remove_file(&tmp);

        // kubectl diff exits 1 if there are diffs, 0 if no diff, 2 on error
        let code = out.status.code().unwrap_or(2);
        if code == 2 {
            return Err(Error::Other(String::from_utf8_lossy(&out.stderr).to_string()));
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }).await.map_err(|e| Error::Other(e.to_string()))?;

    match result {
        Ok(diff) => Ok(Json(DiffResult {
            has_diff: !diff.is_empty(),
            diff,
            kubectl_command: kubectl_display,
        })),
        Err(e) => Err(e),
    }
}

/// GET /api/kubernetes/apply/kubectl
/// Генерирует kubectl-команду для заданного действия
pub async fn generate_kubectl_command(
    State(_state): State<Arc<AppState>>,
    Query(q): Query<KubectlGenQuery>,
) -> Result<Json<KubectlCommand>> {
    let ns_flag = q.namespace.as_deref().map(|n| format!(" -n {n}")).unwrap_or_default();
    let kind_name = match (&q.kind, &q.name) {
        (Some(k), Some(n)) => format!("{}/{}", k.to_lowercase(), n),
        (Some(k), None) => k.to_lowercase(),
        _ => "RESOURCE/NAME".to_string(),
    };

    let (command, description) = match q.action.as_str() {
        "apply" => (
            format!("kubectl apply -f manifest.yaml{ns_flag} --field-manager=velum"),
            "Применить манифест к кластеру".to_string(),
        ),
        "delete" => (
            format!("kubectl delete {kind_name}{ns_flag}"),
            format!("Удалить {kind_name}"),
        ),
        "scale" => {
            let replicas = q.replicas.unwrap_or(1);
            (
                format!("kubectl scale {kind_name}{ns_flag} --replicas={replicas}"),
                format!("Масштабировать до {replicas} реплик"),
            )
        }
        "rollout-restart" => (
            format!("kubectl rollout restart {kind_name}{ns_flag}"),
            "Перезапустить все поды".to_string(),
        ),
        "rollout-undo" => (
            format!("kubectl rollout undo {kind_name}{ns_flag}"),
            "Откатить к предыдущей ревизии".to_string(),
        ),
        "get-yaml" => (
            format!("kubectl get {kind_name}{ns_flag} -o yaml"),
            "Получить манифест в формате YAML".to_string(),
        ),
        "logs" => {
            let pod = q.name.as_deref().unwrap_or("POD_NAME");
            (
                format!("kubectl logs {pod}{ns_flag} --tail=100 -f"),
                "Следить за логами пода".to_string(),
            )
        }
        "exec" => {
            let pod = q.name.as_deref().unwrap_or("POD_NAME");
            (
                format!("kubectl exec -it {pod}{ns_flag} -- /bin/sh"),
                "Открыть shell в поде".to_string(),
            )
        }
        "port-forward" => {
            let pod = q.name.as_deref().unwrap_or("POD_NAME");
            (
                format!("kubectl port-forward {pod}{ns_flag} 8080:8080"),
                "Прокинуть порт из пода".to_string(),
            )
        }
        "describe" => (
            format!("kubectl describe {kind_name}{ns_flag}"),
            "Описание ресурса (события, условия)".to_string(),
        ),
        _ => (
            format!("kubectl get {kind_name}{ns_flag}"),
            "Получить ресурс".to_string(),
        ),
    };

    Ok(Json(KubectlCommand { command, description }))
}

/// GET /api/kubernetes/clusters
/// Список сконфигурированных кластеров
pub async fn list_configured_clusters(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>> {
    let kc = kube_env(&state);

    // Try to get contexts from kubeconfig
    let contexts = tokio::task::spawn_blocking(move || {
        let mut cmd = std::process::Command::new("kubectl");
        if let Some(k) = &kc { cmd.env("KUBECONFIG", k); }
        let out = cmd.arg("config").arg("get-contexts").arg("-o").arg("name")
            .output();
        match out {
            Ok(o) if o.status.success() => {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(String::from)
                    .collect::<Vec<_>>()
            }
            _ => vec!["default".to_string()],
        }
    }).await.map_err(|e| Error::Other(e.to_string()))?;

    // Get current context
    let current = tokio::task::spawn_blocking(|| {
        let out = std::process::Command::new("kubectl")
            .arg("config").arg("current-context")
            .output();
        match out {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
            _ => "default".to_string(),
        }
    }).await.unwrap_or_else(|_| "default".to_string());

    let clusters: Vec<serde_json::Value> = contexts.iter().map(|name| {
        serde_json::json!({
            "name": name,
            "current": name == &current,
            "id": name,
        })
    }).collect();

    Ok(Json(serde_json::json!({ "clusters": clusters, "current": current })))
}

/// POST /api/kubernetes/clusters/switch
/// Переключить текущий kubeconfig context
pub async fn switch_cluster_context(
    State(state): State<Arc<AppState>>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let context = body["context"].as_str().unwrap_or("").to_string();
    if context.is_empty() {
        return Err(Error::Other("context is required".to_string()));
    }
    let kc = kube_env(&state);
    let ctx = context.clone();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let mut cmd = std::process::Command::new("kubectl");
        if let Some(k) = &kc { cmd.env("KUBECONFIG", k); }
        let out = cmd.arg("config").arg("use-context").arg(&ctx)
            .output().map_err(|e| Error::Other(e.to_string()))?;
        if !out.status.success() {
            return Err(Error::Other(String::from_utf8_lossy(&out.stderr).to_string()));
        }
        Ok(())
    }).await.map_err(|e| Error::Other(e.to_string()))??;

    Ok(Json(serde_json::json!({ "switched": true, "context": context })))
}

// ── Helpers ───────────────────────────────────────────────────────

fn build_apply_command(field_manager: &str, dry_run: bool, force: bool) -> String {
    let mut parts = vec![
        "kubectl apply -f manifest.yaml".to_string(),
        format!("--field-manager={field_manager}"),
    ];
    if dry_run { parts.push("--dry-run=server".to_string()); }
    if force { parts.push("--force-conflicts".to_string()); }
    parts.join(" ")
}

fn parse_kubectl_apply_output(stdout: &[u8]) -> Vec<AppliedResource> {
    // Try to parse JSON output from kubectl apply -o json
    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(stdout) {
        let items = if v["kind"] == "List" {
            v["items"].as_array().cloned().unwrap_or_default()
        } else if v["kind"].is_string() {
            vec![v]
        } else {
            vec![]
        };

        return items.iter().map(|item| {
            AppliedResource {
                kind: item["kind"].as_str().unwrap_or("Unknown").to_string(),
                name: item["metadata"]["name"].as_str().unwrap_or("").to_string(),
                namespace: item["metadata"]["namespace"].as_str().map(String::from),
                action: item["metadata"]["annotations"]["kubectl.kubernetes.io/last-applied-configuration"]
                    .as_str().map(|_| "configured").unwrap_or("created").to_string(),
            }
        }).collect();
    }
    vec![]
}
