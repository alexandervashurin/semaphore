//! Kubernetes RBAC API objects: ServiceAccount, Role/Binding, ClusterRole/Binding,
//! SelfSubjectRulesReview, Pod Security Admission labels.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use k8s_openapi::api::authorization::v1::{SelfSubjectRulesReview, SelfSubjectRulesReviewSpec};
use k8s_openapi::api::core::v1::{Namespace, Secret, ServiceAccount};
use k8s_openapi::api::rbac::v1::{ClusterRole, ClusterRoleBinding, PolicyRule, Role, RoleBinding};
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct NamespaceQuery {
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ServiceAccountSummary {
    pub name: String,
    pub namespace: String,
}

#[derive(Debug, Serialize)]
pub struct SecretRefSummary {
    pub name: String,
    pub secret_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RoleLikeSummary {
    pub name: String,
    pub namespace: Option<String>,
    pub rules_count: usize,
    pub wide_rules: bool,
    pub warning: Option<String>,
    pub is_system: bool,
}

#[derive(Debug, Serialize)]
pub struct BindingSummary {
    pub name: String,
    pub namespace: Option<String>,
    pub role_kind: Option<String>,
    pub role_name: Option<String>,
    pub subjects_count: usize,
}

const PSA_ENFORCE: &str = "pod-security.kubernetes.io/enforce";
const PSA_AUDIT: &str = "pod-security.kubernetes.io/audit";
const PSA_WARN: &str = "pod-security.kubernetes.io/warn";

#[derive(Debug, Serialize)]
pub struct PodSecurityAdmissionView {
    pub enforce: Option<String>,
    pub audit: Option<String>,
    pub warn: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PodSecurityAdmissionPatch {
    /// None = не менять; Some("") = удалить метку; иначе значение уровня.
    pub enforce: Option<String>,
    pub audit: Option<String>,
    pub warn: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RulesReviewRequest {
    pub namespace: Option<String>,
}

fn policy_rule_is_wide(rule: &PolicyRule) -> bool {
    let star_slice = |v: &[String]| v.iter().any(|s| s == "*");
    let star_opt = |v: &Option<Vec<String>>| {
        v.as_ref()
            .map(|a| star_slice(a.as_slice()))
            .unwrap_or(false)
    };
    star_slice(&rule.verbs) || star_opt(&rule.resources) || star_opt(&rule.api_groups)
}

fn role_wide_rules(rules: &[PolicyRule]) -> (bool, Option<String>) {
    let wide = rules.iter().any(policy_rule_is_wide);
    let warn = if wide {
        Some("В правилах есть wildcard (*) — слишком широкий доступ.".to_string())
    } else {
        None
    };
    (wide, warn)
}

fn is_system_cluster_name(name: &str) -> bool {
    name.starts_with("system:")
        || name.starts_with("eks:")
        || name.starts_with("gcp:")
        || name == "cluster-admin"
        || name == "edit"
        || name == "view"
        || name == "admin"
}

pub async fn list_service_accounts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<NamespaceQuery>,
) -> Result<Json<Vec<ServiceAccountSummary>>> {
    let client = state.kubernetes_client()?;
    let ns = query.namespace.unwrap_or_else(|| "default".to_string());
    let api: Api<ServiceAccount> = client.api(Some(&ns));
    let list = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        list.items
            .iter()
            .map(|sa| ServiceAccountSummary {
                name: sa.metadata.name.clone().unwrap_or_default(),
                namespace: sa.metadata.namespace.clone().unwrap_or_else(|| ns.clone()),
            })
            .collect(),
    ))
}

pub async fn get_service_account(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<ServiceAccount>> {
    let client = state.kubernetes_client()?;
    let api: Api<ServiceAccount> = client.api(Some(&namespace));
    let item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(item))
}

pub async fn create_service_account(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(payload): Json<ServiceAccount>,
) -> Result<Json<ServiceAccountSummary>> {
    let client = state.kubernetes_client()?;
    let api: Api<ServiceAccount> = client.api(Some(&namespace));
    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(ServiceAccountSummary {
        name: created.metadata.name.clone().unwrap_or_default(),
        namespace: created.metadata.namespace.clone().unwrap_or(namespace),
    }))
}

pub async fn delete_service_account(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<ServiceAccount> = client.api(Some(&namespace));
    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        serde_json::json!({"status":"ok","message":format!("ServiceAccount {namespace}/{name} deleted")}),
    ))
}

pub async fn list_service_account_secrets(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<Vec<SecretRefSummary>>> {
    let client = state.kubernetes_client()?;
    let api: Api<Secret> = client.api(Some(&namespace));
    let lp = ListParams::default().labels(&format!("kubernetes.io/service-account.name={name}"));
    let list = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        list.items
            .iter()
            .map(|s| SecretRefSummary {
                name: s.metadata.name.clone().unwrap_or_default(),
                secret_type: s.type_.clone(),
            })
            .collect(),
    ))
}

fn summarize_role(name: &str, namespace: &str, rules: &[PolicyRule]) -> RoleLikeSummary {
    let (wide, warning) = role_wide_rules(rules);
    RoleLikeSummary {
        name: name.to_string(),
        namespace: Some(namespace.to_string()),
        rules_count: rules.len(),
        wide_rules: wide,
        warning,
        is_system: false,
    }
}

pub async fn list_roles(
    State(state): State<Arc<AppState>>,
    Query(query): Query<NamespaceQuery>,
) -> Result<Json<Vec<RoleLikeSummary>>> {
    let client = state.kubernetes_client()?;
    let ns = query.namespace.unwrap_or_else(|| "default".to_string());
    let api: Api<Role> = client.api(Some(&ns));
    let list = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        list.items
            .iter()
            .filter_map(|r| {
                let name = r.metadata.name.clone()?;
                let rules = r.rules.as_deref().unwrap_or(&[]);
                Some(summarize_role(&name, &ns, rules))
            })
            .collect(),
    ))
}

pub async fn get_role(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<Role>> {
    let client = state.kubernetes_client()?;
    let api: Api<Role> = client.api(Some(&namespace));
    let item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(item))
}

pub async fn create_role(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(payload): Json<Role>,
) -> Result<Json<Role>> {
    let client = state.kubernetes_client()?;
    let api: Api<Role> = client.api(Some(&namespace));
    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(created))
}

pub async fn update_role(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<Role>,
) -> Result<Json<Role>> {
    let client = state.kubernetes_client()?;
    let api: Api<Role> = client.api(Some(&namespace));
    let updated = api
        .replace(&name, &PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(updated))
}

pub async fn delete_role(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<Role> = client.api(Some(&namespace));
    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        serde_json::json!({"status":"ok","message":format!("Role {namespace}/{name} deleted")}),
    ))
}

fn summarize_binding(name: &str, namespace: Option<&str>, rb: &RoleBinding) -> BindingSummary {
    let r = &rb.role_ref;
    BindingSummary {
        name: name.to_string(),
        namespace: namespace.map(str::to_string),
        role_kind: Some(r.kind.clone()),
        role_name: Some(r.name.clone()),
        subjects_count: rb.subjects.as_ref().map(|s| s.len()).unwrap_or(0),
    }
}

fn summarize_cluster_binding(name: &str, rb: &ClusterRoleBinding) -> BindingSummary {
    let r = &rb.role_ref;
    BindingSummary {
        name: name.to_string(),
        namespace: None,
        role_kind: Some(r.kind.clone()),
        role_name: Some(r.name.clone()),
        subjects_count: rb.subjects.as_ref().map(|s| s.len()).unwrap_or(0),
    }
}

pub async fn list_role_bindings(
    State(state): State<Arc<AppState>>,
    Query(query): Query<NamespaceQuery>,
) -> Result<Json<Vec<BindingSummary>>> {
    let client = state.kubernetes_client()?;
    let ns = query.namespace.unwrap_or_else(|| "default".to_string());
    let api: Api<RoleBinding> = client.api(Some(&ns));
    let list = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        list.items
            .iter()
            .filter_map(|rb| {
                let name = rb.metadata.name.clone()?;
                Some(summarize_binding(&name, Some(&ns), rb))
            })
            .collect(),
    ))
}

pub async fn get_role_binding(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<RoleBinding>> {
    let client = state.kubernetes_client()?;
    let api: Api<RoleBinding> = client.api(Some(&namespace));
    let item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(item))
}

pub async fn create_role_binding(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(payload): Json<RoleBinding>,
) -> Result<Json<RoleBinding>> {
    let client = state.kubernetes_client()?;
    let api: Api<RoleBinding> = client.api(Some(&namespace));
    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(created))
}

pub async fn update_role_binding(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(payload): Json<RoleBinding>,
) -> Result<Json<RoleBinding>> {
    let client = state.kubernetes_client()?;
    let api: Api<RoleBinding> = client.api(Some(&namespace));
    let updated = api
        .replace(&name, &PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(updated))
}

pub async fn delete_role_binding(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<RoleBinding> = client.api(Some(&namespace));
    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        serde_json::json!({"status":"ok","message":format!("RoleBinding {namespace}/{name} deleted")}),
    ))
}

fn summarize_cluster_role(name: &str, rules: &[PolicyRule]) -> RoleLikeSummary {
    let (wide, warning) = role_wide_rules(rules);
    let is_system = is_system_cluster_name(name);
    let mut w = warning;
    if is_system {
        w = Some(
            "Системная или встроенная роль кластера — правка может сломать кластер.".to_string(),
        );
    }
    RoleLikeSummary {
        name: name.to_string(),
        namespace: None,
        rules_count: rules.len(),
        wide_rules: wide,
        warning: w,
        is_system,
    }
}

pub async fn list_cluster_roles(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<RoleLikeSummary>>> {
    let client = state.kubernetes_client()?;
    let api: Api<ClusterRole> = client.api_all();
    let list = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        list.items
            .iter()
            .filter_map(|r| {
                let name = r.metadata.name.clone()?;
                let rules = r.rules.as_deref().unwrap_or(&[]);
                Some(summarize_cluster_role(&name, rules))
            })
            .collect(),
    ))
}

pub async fn get_cluster_role(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<ClusterRole>> {
    let client = state.kubernetes_client()?;
    let api: Api<ClusterRole> = client.api_all();
    let item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(item))
}

pub async fn create_cluster_role(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ClusterRole>,
) -> Result<Json<ClusterRole>> {
    let client = state.kubernetes_client()?;
    let api: Api<ClusterRole> = client.api_all();
    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(created))
}

pub async fn update_cluster_role(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<ClusterRole>,
) -> Result<Json<ClusterRole>> {
    let client = state.kubernetes_client()?;
    let api: Api<ClusterRole> = client.api_all();
    let updated = api
        .replace(&name, &PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(updated))
}

pub async fn delete_cluster_role(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<ClusterRole> = client.api_all();
    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        serde_json::json!({"status":"ok","message":format!("ClusterRole {name} deleted")}),
    ))
}

pub async fn list_cluster_role_bindings(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<BindingSummary>>> {
    let client = state.kubernetes_client()?;
    let api: Api<ClusterRoleBinding> = client.api_all();
    let list = api
        .list(&ListParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        list.items
            .iter()
            .filter_map(|rb| {
                let name = rb.metadata.name.clone()?;
                Some(summarize_cluster_binding(&name, rb))
            })
            .collect(),
    ))
}

pub async fn get_cluster_role_binding(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<ClusterRoleBinding>> {
    let client = state.kubernetes_client()?;
    let api: Api<ClusterRoleBinding> = client.api_all();
    let item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(item))
}

pub async fn create_cluster_role_binding(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ClusterRoleBinding>,
) -> Result<Json<ClusterRoleBinding>> {
    let client = state.kubernetes_client()?;
    let api: Api<ClusterRoleBinding> = client.api_all();
    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(created))
}

pub async fn update_cluster_role_binding(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<ClusterRoleBinding>,
) -> Result<Json<ClusterRoleBinding>> {
    let client = state.kubernetes_client()?;
    let api: Api<ClusterRoleBinding> = client.api_all();
    let updated = api
        .replace(&name, &PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(updated))
}

pub async fn delete_cluster_role_binding(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<ClusterRoleBinding> = client.api_all();
    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(
        serde_json::json!({"status":"ok","message":format!("ClusterRoleBinding {name} deleted")}),
    ))
}

pub async fn post_self_subject_rules_review(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RulesReviewRequest>,
) -> Result<Json<SelfSubjectRulesReview>> {
    let client = state.kubernetes_client()?;
    let api: Api<SelfSubjectRulesReview> = Api::all(client.raw().clone());
    let review = SelfSubjectRulesReview {
        metadata: Default::default(),
        spec: SelfSubjectRulesReviewSpec {
            namespace: payload.namespace,
        },
        status: None,
    };
    let created = api
        .create(&PostParams::default(), &review)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    Ok(Json(created))
}

fn labels_psa_view(
    labels: &std::collections::BTreeMap<String, String>,
) -> PodSecurityAdmissionView {
    PodSecurityAdmissionView {
        enforce: labels.get(PSA_ENFORCE).cloned(),
        audit: labels.get(PSA_AUDIT).cloned(),
        warn: labels.get(PSA_WARN).cloned(),
    }
}

pub async fn get_namespace_pod_security(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<PodSecurityAdmissionView>> {
    let client = state.kubernetes_client()?;
    let api: Api<Namespace> = client.api_all();
    let ns = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    let labels = ns.metadata.labels.as_ref().cloned().unwrap_or_default();
    Ok(Json(labels_psa_view(&labels)))
}

pub async fn put_namespace_pod_security(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<PodSecurityAdmissionPatch>,
) -> Result<Json<PodSecurityAdmissionView>> {
    let client = state.kubernetes_client()?;
    let api: Api<Namespace> = client.api_all();
    let mut ns = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    let labels = ns.metadata.labels.get_or_insert_with(Default::default);

    fn merge_psa(
        labels: &mut std::collections::BTreeMap<String, String>,
        key: &str,
        val: Option<String>,
    ) {
        match val {
            None => {}
            Some(s) if s.is_empty() => {
                labels.remove(key);
            }
            Some(s) => {
                labels.insert(key.to_string(), s);
            }
        }
    }

    merge_psa(labels, PSA_ENFORCE, payload.enforce);
    merge_psa(labels, PSA_AUDIT, payload.audit);
    merge_psa(labels, PSA_WARN, payload.warn);

    let updated = api
        .replace(&name, &PostParams::default(), &ns)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;
    let out = updated
        .metadata
        .labels
        .as_ref()
        .cloned()
        .unwrap_or_default();
    Ok(Json(labels_psa_view(&out)))
}
