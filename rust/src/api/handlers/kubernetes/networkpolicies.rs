//! Kubernetes NetworkPolicy API handlers

use axum::{
    Json,
    extract::{Path, Query, State},
};
use k8s_openapi::api::networking::v1::NetworkPolicy;
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct ListNetworkPoliciesQuery {
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct NetworkPolicySummary {
    pub name: String,
    pub namespace: String,
    pub policy_types: Vec<String>,
    pub ingress_rules: usize,
    pub egress_rules: usize,
}

#[derive(Debug, Serialize)]
pub struct NetworkPolicyView {
    pub name: String,
    pub namespace: String,
    pub policy_types: Vec<String>,
    pub ingress_rules: usize,
    pub egress_rules: usize,
    pub note: String,
}

fn to_summary(np: &NetworkPolicy) -> NetworkPolicySummary {
    let spec = np.spec.as_ref();
    NetworkPolicySummary {
        name: np
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        namespace: np
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        policy_types: spec
            .and_then(|s| s.policy_types.clone())
            .unwrap_or_default(),
        ingress_rules: spec
            .and_then(|s| s.ingress.as_ref().map(|r| r.len()))
            .unwrap_or(0),
        egress_rules: spec
            .and_then(|s| s.egress.as_ref().map(|r| r.len()))
            .unwrap_or(0),
    }
}

fn to_view(np: &NetworkPolicy) -> NetworkPolicyView {
    let summary = to_summary(np);
    NetworkPolicyView {
        name: summary.name,
        namespace: summary.namespace,
        policy_types: summary.policy_types,
        ingress_rules: summary.ingress_rules,
        egress_rules: summary.egress_rules,
        note: "NetworkPolicy effect depends on CNI implementation and cluster networking setup."
            .to_string(),
    }
}

pub async fn list_networkpolicies(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListNetworkPoliciesQuery>,
) -> Result<Json<Vec<NetworkPolicySummary>>> {
    let client = state.kubernetes_client()?;
    let api: Api<NetworkPolicy> = if let Some(namespace) = query.namespace.as_deref() {
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

pub async fn get_networkpolicy(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<NetworkPolicyView>> {
    let client = state.kubernetes_client()?;
    let api: Api<NetworkPolicy> = client.api(Some(&namespace));

    let item = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(to_view(&item)))
}

pub async fn create_networkpolicy(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(payload): Json<NetworkPolicy>,
) -> Result<Json<NetworkPolicySummary>> {
    let client = state.kubernetes_client()?;
    let api: Api<NetworkPolicy> = client.api(Some(&namespace));

    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(to_summary(&created)))
}

pub async fn update_networkpolicy(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(mut payload): Json<NetworkPolicy>,
) -> Result<Json<NetworkPolicySummary>> {
    let client = state.kubernetes_client()?;
    let api: Api<NetworkPolicy> = client.api(Some(&namespace));

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

    Ok(Json(to_summary(&updated)))
}

pub async fn delete_networkpolicy(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let client = state.kubernetes_client()?;
    let api: Api<NetworkPolicy> = client.api(Some(&namespace));

    let _ = api
        .delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "status": "ok",
        "message": format!("NetworkPolicy {namespace}/{name} deleted")
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_network_policies_query_with_namespace() {
        let json = r#"{"namespace": "kube-system", "limit": 20}"#;
        let query: ListNetworkPoliciesQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.namespace, Some("kube-system".to_string()));
        assert_eq!(query.limit, Some(20));
    }

    #[test]
    fn test_list_network_policies_query_with_label_selector() {
        let json = r#"{"label_selector": "app=my-app", "limit": 50}"#;
        let query: ListNetworkPoliciesQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.label_selector, Some("app=my-app".to_string()));
        assert_eq!(query.limit, Some(50));
    }

    #[test]
    fn test_list_network_policies_query_empty() {
        let json = r#"{}"#;
        let query: ListNetworkPoliciesQuery = serde_json::from_str(json).unwrap();
        assert!(query.namespace.is_none());
        assert!(query.label_selector.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_network_policy_summary_serialization() {
        let mut np = NetworkPolicy::default();
        np.metadata.name = Some("deny-all".to_string());
        np.metadata.namespace = Some("default".to_string());

        let summary = to_summary(&np);
        assert_eq!(summary.name, "deny-all");
        assert_eq!(summary.namespace, "default");
        assert!(summary.policy_types.is_empty());
        assert_eq!(summary.ingress_rules, 0);
        assert_eq!(summary.egress_rules, 0);
    }

    #[test]
    fn test_network_policy_view_serialization() {
        let mut np = NetworkPolicy::default();
        np.metadata.name = Some("allow-dns".to_string());
        np.metadata.namespace = Some("production".to_string());

        let view = to_view(&np);
        assert_eq!(view.name, "allow-dns");
        assert_eq!(view.namespace, "production");
        assert!(view.note.contains("CNI"));
    }

    #[test]
    fn test_network_policy_summary_with_types() {
        use k8s_openapi::api::networking::v1::NetworkPolicySpec;
        let mut np = NetworkPolicy::default();
        np.metadata.name = Some("restrict".to_string());
        np.metadata.namespace = Some("ns1".to_string());
        np.spec = Some(NetworkPolicySpec {
            policy_types: Some(vec!["Ingress".to_string(), "Egress".to_string()]),
            ..Default::default()
        });

        let summary = to_summary(&np);
        assert_eq!(summary.policy_types.len(), 2);
        assert!(summary.policy_types.contains(&"Ingress".to_string()));
        assert!(summary.policy_types.contains(&"Egress".to_string()));
    }

    #[test]
    fn test_network_policy_summary_default_name() {
        let np = NetworkPolicy::default();
        let summary = to_summary(&np);
        assert_eq!(summary.name, "unknown");
    }

    #[test]
    fn test_network_policy_summary_default_namespace() {
        let mut np = NetworkPolicy::default();
        np.metadata.name = Some("test".to_string());
        let summary = to_summary(&np);
        assert_eq!(summary.namespace, "default");
    }

    #[test]
    fn test_delete_response_json_format() {
        let namespace = "production";
        let name = "deny-all";
        let expected = serde_json::json!({
            "status": "ok",
            "message": format!("NetworkPolicy {namespace}/{name} deleted")
        });
        assert_eq!(expected["status"], "ok");
        assert!(expected["message"].as_str().unwrap().contains("deny-all"));
    }

    #[test]
    fn test_network_policy_view_note_not_empty() {
        let np = NetworkPolicy::default();
        let view = to_view(&np);
        assert!(!view.note.is_empty());
        assert!(view.note.contains("NetworkPolicy"));
    }

    #[test]
    fn test_network_policy_summary_with_ingress_rules() {
        use k8s_openapi::api::networking::v1::{
            NetworkPolicyIngressRule, NetworkPolicyPeer, NetworkPolicySpec,
        };
        let mut np = NetworkPolicy::default();
        np.metadata.name = Some("allow-ingress".to_string());
        np.metadata.namespace = Some("web".to_string());

        let ingress_rule = NetworkPolicyIngressRule {
            from: Some(vec![NetworkPolicyPeer::default()]),
            ..Default::default()
        };
        np.spec = Some(NetworkPolicySpec {
            ingress: Some(vec![ingress_rule]),
            ..Default::default()
        });

        let summary = to_summary(&np);
        assert_eq!(summary.ingress_rules, 1);
        assert_eq!(summary.egress_rules, 0);
    }

    #[test]
    fn test_list_query_label_selector_only() {
        let json = r#"{"label_selector": "env=prod"}"#;
        let query: ListNetworkPoliciesQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.label_selector, Some("env=prod".to_string()));
        assert!(query.namespace.is_none());
    }
}
