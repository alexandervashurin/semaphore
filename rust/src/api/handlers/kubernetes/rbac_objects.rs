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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_policy_rule_is_wide_verbs_star() {
        let rule = PolicyRule {
            verbs: vec!["*".to_string()],
            resources: Some(vec!["pods".to_string()]),
            api_groups: Some(vec!["".to_string()]),
            non_resource_urls: None,
            resource_names: None,
        };
        assert!(policy_rule_is_wide(&rule));
    }

    #[test]
    fn test_policy_rule_is_wide_resources_star() {
        let rule = PolicyRule {
            verbs: vec!["get".to_string()],
            resources: Some(vec!["*".to_string()]),
            api_groups: Some(vec!["".to_string()]),
            non_resource_urls: None,
            resource_names: None,
        };
        assert!(policy_rule_is_wide(&rule));
    }

    #[test]
    fn test_policy_rule_is_wide_not_wide() {
        let rule = PolicyRule {
            verbs: vec!["get".to_string(), "list".to_string()],
            resources: Some(vec!["pods".to_string()]),
            api_groups: Some(vec!["".to_string()]),
            non_resource_urls: None,
            resource_names: None,
        };
        assert!(!policy_rule_is_wide(&rule));
    }

    #[test]
    fn test_role_wide_rules_has_wildcard() {
        let rules = vec![
            PolicyRule {
                verbs: vec!["get".to_string()],
                resources: Some(vec!["pods".to_string()]),
                api_groups: Some(vec!["".to_string()]),
                non_resource_urls: None,
                resource_names: None,
            },
            PolicyRule {
                verbs: vec!["*".to_string()],
                resources: Some(vec!["*".to_string()]),
                api_groups: Some(vec!["*".to_string()]),
                non_resource_urls: None,
                resource_names: None,
            },
        ];
        let (is_wide, warn) = role_wide_rules(&rules);
        assert!(is_wide);
        assert!(warn.is_some());
    }

    #[test]
    fn test_role_wide_rules_not_wide() {
        let rules = vec![
            PolicyRule {
                verbs: vec!["get".to_string()],
                resources: Some(vec!["pods".to_string()]),
                api_groups: Some(vec!["".to_string()]),
                non_resource_urls: None,
                resource_names: None,
            },
        ];
        let (is_wide, warn) = role_wide_rules(&rules);
        assert!(!is_wide);
        assert!(warn.is_none());
    }

    #[test]
    fn test_is_system_cluster_name_system_prefix() {
        assert!(is_system_cluster_name("system:kube-scheduler"));
        assert!(is_system_cluster_name("system:coredns"));
    }

    #[test]
    fn test_is_system_cluster_name_cloud_prefixes() {
        assert!(is_system_cluster_name("eks:cluster-role"));
        assert!(is_system_cluster_name("gcp:node-role"));
    }

    #[test]
    fn test_is_system_cluster_name_builtin_names() {
        assert!(is_system_cluster_name("cluster-admin"));
        assert!(is_system_cluster_name("admin"));
        assert!(is_system_cluster_name("edit"));
        assert!(is_system_cluster_name("view"));
    }

    #[test]
    fn test_is_system_cluster_name_not_system() {
        assert!(!is_system_cluster_name("my-role"));
        assert!(!is_system_cluster_name("app-reader"));
        assert!(!is_system_cluster_name(""));
    }

    #[test]
    fn test_labels_psa_view() {
        let mut labels = BTreeMap::new();
        labels.insert("pod-security.kubernetes.io/enforce".to_string(), "restricted".to_string());
        labels.insert("pod-security.kubernetes.io/audit".to_string(), "baseline".to_string());
        labels.insert("pod-security.kubernetes.io/warn".to_string(), "restricted".to_string());

        let view = labels_psa_view(&labels);
        assert_eq!(view.enforce, Some("restricted".to_string()));
        assert_eq!(view.audit, Some("baseline".to_string()));
        assert_eq!(view.warn, Some("restricted".to_string()));
    }

    #[test]
    fn test_labels_psa_view_missing_labels() {
        let labels = BTreeMap::new();
        let view = labels_psa_view(&labels);
        assert_eq!(view.enforce, None);
        assert_eq!(view.audit, None);
        assert_eq!(view.warn, None);
    }

    #[test]
    fn test_service_account_summary() {
        let summary = ServiceAccountSummary {
            name: "my-sa".to_string(),
            namespace: "kube-system".to_string(),
        };
        assert_eq!(summary.name, "my-sa");
        assert_eq!(summary.namespace, "kube-system");
    }

    #[test]
    fn test_secret_ref_summary_with_type() {
        let secret = SecretRefSummary {
            name: "my-secret".to_string(),
            secret_type: Some("kubernetes.io/tls".to_string()),
        };
        assert_eq!(secret.name, "my-secret");
        assert!(secret.secret_type.is_some());
    }

    #[test]
    fn test_secret_ref_summary_no_type() {
        let secret = SecretRefSummary {
            name: "generic-secret".to_string(),
            secret_type: None,
        };
        assert_eq!(secret.name, "generic-secret");
        assert!(secret.secret_type.is_none());
    }

    #[test]
    fn test_role_like_summary_wide_rules() {
        let summary = RoleLikeSummary {
            name: "admin-role".to_string(),
            namespace: Some("default".to_string()),
            rules_count: 5,
            wide_rules: true,
            warning: Some("Wildcard detected".to_string()),
            is_system: false,
        };
        assert!(summary.wide_rules);
        assert!(summary.warning.is_some());
        assert!(!summary.is_system);
    }

    #[test]
    fn test_role_like_summary_namespace_none() {
        let summary = RoleLikeSummary {
            name: "cluster-reader".to_string(),
            namespace: None,
            rules_count: 2,
            wide_rules: false,
            warning: None,
            is_system: false,
        };
        assert!(summary.namespace.is_none());
        assert_eq!(summary.rules_count, 2);
    }

    #[test]
    fn test_binding_summary_with_subjects() {
        let summary = BindingSummary {
            name: "my-binding".to_string(),
            namespace: Some("production".to_string()),
            role_kind: Some("ClusterRole".to_string()),
            role_name: Some("cluster-admin".to_string()),
            subjects_count: 3,
        };
        assert_eq!(summary.subjects_count, 3);
        assert_eq!(summary.role_name, Some("cluster-admin".to_string()));
    }

    #[test]
    fn test_binding_summary_no_subjects() {
        let summary = BindingSummary {
            name: "empty-binding".to_string(),
            namespace: None,
            role_kind: None,
            role_name: None,
            subjects_count: 0,
        };
        assert_eq!(summary.subjects_count, 0);
        assert!(summary.role_kind.is_none());
    }

    #[test]
    fn test_pod_security_admission_view_partial() {
        let mut labels = BTreeMap::new();
        labels.insert("pod-security.kubernetes.io/enforce".to_string(), "baseline".to_string());

        let view = labels_psa_view(&labels);
        assert_eq!(view.enforce, Some("baseline".to_string()));
        assert_eq!(view.audit, None);
        assert_eq!(view.warn, None);
    }

    #[test]
    fn test_pod_security_admission_patch_all_none() {
        let patch = PodSecurityAdmissionPatch {
            enforce: None,
            audit: None,
            warn: None,
        };
        assert!(patch.enforce.is_none());
        assert!(patch.audit.is_none());
        assert!(patch.warn.is_none());
    }

    #[test]
    fn test_pod_security_admission_patch_empty_string_removes() {
        let patch = PodSecurityAdmissionPatch {
            enforce: Some("".to_string()),
            audit: Some("baseline".to_string()),
            warn: None,
        };
        assert_eq!(patch.enforce, Some("".to_string()));
        assert_eq!(patch.audit, Some("baseline".to_string()));
    }

    #[test]
    fn test_rules_review_request_with_namespace() {
        let req = RulesReviewRequest {
            namespace: Some("kube-system".to_string()),
        };
        assert_eq!(req.namespace, Some("kube-system".to_string()));
    }

    #[test]
    fn test_rules_review_request_no_namespace() {
        let req = RulesReviewRequest {
            namespace: None,
        };
        assert!(req.namespace.is_none());
    }

    #[test]
    fn test_policy_rule_is_wide_api_groups_star() {
        let rule = PolicyRule {
            verbs: vec!["get".to_string()],
            resources: Some(vec!["pods".to_string()]),
            api_groups: Some(vec!["*".to_string()]),
            non_resource_urls: None,
            resource_names: None,
        };
        assert!(policy_rule_is_wide(&rule));
    }

    #[test]
    fn test_policy_rule_is_wide_multiple_verbs_no_star() {
        let rule = PolicyRule {
            verbs: vec!["get".to_string(), "list".to_string(), "watch".to_string()],
            resources: Some(vec!["pods".to_string(), "services".to_string()]),
            api_groups: Some(vec!["".to_string()]),
            non_resource_urls: None,
            resource_names: None,
        };
        assert!(!policy_rule_is_wide(&rule));
    }

    #[test]
    fn test_role_wide_rules_empty() {
        let rules: Vec<PolicyRule> = vec![];
        let (is_wide, warn) = role_wide_rules(&rules);
        assert!(!is_wide);
        assert!(warn.is_none());
    }

    #[test]
    fn test_role_wide_rules_single_non_wide() {
        let rules = vec![
            PolicyRule {
                verbs: vec!["get".to_string()],
                resources: Some(vec!["configmaps".to_string()]),
                api_groups: Some(vec!["".to_string()]),
                non_resource_urls: None,
                resource_names: None,
            },
        ];
        let (is_wide, warn) = role_wide_rules(&rules);
        assert!(!is_wide);
        assert!(warn.is_none());
    }

    #[test]
    fn test_is_system_cluster_name_empty_string() {
        assert!(!is_system_cluster_name(""));
    }

    #[test]
    fn test_is_system_cluster_name_case_sensitive() {
        assert!(!is_system_cluster_name("System:admin"));
        assert!(!is_system_cluster_name("ADMIN"));
    }

    #[test]
    fn test_summarize_role_basic() {
        let rules = vec![
            PolicyRule {
                verbs: vec!["get".to_string()],
                resources: Some(vec!["pods".to_string()]),
                api_groups: Some(vec!["".to_string()]),
                non_resource_urls: None,
                resource_names: None,
            },
        ];
        let summary = summarize_role("test-role", "default", &rules);
        assert_eq!(summary.name, "test-role");
        assert_eq!(summary.namespace, Some("default".to_string()));
        assert_eq!(summary.rules_count, 1);
        assert!(!summary.wide_rules);
        assert!(!summary.is_system);
    }

    #[test]
    fn test_summarize_cluster_role_system_name() {
        let rules = vec![
            PolicyRule {
                verbs: vec!["get".to_string()],
                resources: Some(vec!["nodes".to_string()]),
                api_groups: Some(vec!["".to_string()]),
                non_resource_urls: None,
                resource_names: None,
            },
        ];
        let summary = summarize_cluster_role("system:kubelet", &rules);
        assert!(summary.is_system);
        assert!(summary.warning.is_some());
        assert!(summary.warning.as_ref().unwrap().contains("Системная"));
    }

    #[test]
    fn test_summarize_cluster_role_non_system() {
        let rules = vec![
            PolicyRule {
                verbs: vec!["*".to_string()],
                resources: Some(vec!["*".to_string()]),
                api_groups: Some(vec!["*".to_string()]),
                non_resource_urls: None,
                resource_names: None,
            },
        ];
        let summary = summarize_cluster_role("my-custom-role", &rules);
        assert!(!summary.is_system);
        assert!(summary.wide_rules);
        assert_eq!(summary.rules_count, 1);
    }

    #[test]
    fn test_namespace_query_default_none() {
        let query = NamespaceQuery {
            namespace: None,
        };
        assert!(query.namespace.is_none());
    }

    #[test]
    fn test_namespace_query_with_value() {
        let query = NamespaceQuery {
            namespace: Some("monitoring".to_string()),
        };
        assert_eq!(query.namespace, Some("monitoring".to_string()));
    }

    #[test]
    fn test_psa_constants_match_labels() {
        assert_eq!(PSA_ENFORCE, "pod-security.kubernetes.io/enforce");
        assert_eq!(PSA_AUDIT, "pod-security.kubernetes.io/audit");
        assert_eq!(PSA_WARN, "pod-security.kubernetes.io/warn");
    }
}
