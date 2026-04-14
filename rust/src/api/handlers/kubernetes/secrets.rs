//! Kubernetes Secret API handlers

use axum::{
    Json,
    extract::{Path, Query, State},
};
use base64::Engine;
use k8s_openapi::api::core::v1::Secret;
use kube::api::{Api, DeleteParams, ListParams, PostParams};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct ListSecretsQuery {
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SecretSummary {
    pub name: String,
    pub namespace: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub keys_count: usize,
}

#[derive(Debug, Serialize)]
pub struct SecretMaskedView {
    pub name: String,
    pub namespace: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub data: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct SecretRevealView {
    pub name: String,
    pub namespace: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub data: BTreeMap<String, String>,
    pub warning: String,
}

fn secret_type(secret: &Secret) -> String {
    secret.type_.clone().unwrap_or_else(|| "Opaque".to_string())
}

fn masked_data(secret: &Secret) -> BTreeMap<String, String> {
    secret
        .data
        .as_ref()
        .map(|m| {
            m.keys()
                .map(|k| (k.clone(), "***".to_string()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default()
}

pub async fn list_secrets(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListSecretsQuery>,
) -> Result<Json<Vec<SecretSummary>>> {
    let client = state.kubernetes_client()?;
    let api: Api<Secret> = if let Some(namespace) = query.namespace.as_deref() {
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

    let result = items
        .items
        .iter()
        .map(|secret| SecretSummary {
            name: secret
                .metadata
                .name
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            namespace: secret
                .metadata
                .namespace
                .clone()
                .unwrap_or_else(|| "default".to_string()),
            type_: secret_type(secret),
            keys_count: secret.data.as_ref().map(|m| m.len()).unwrap_or(0),
        })
        .collect();

    Ok(Json(result))
}

pub async fn get_secret(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<SecretMaskedView>> {
    let client = state.kubernetes_client()?;
    let api: Api<Secret> = client.api(Some(&namespace));

    let secret = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(SecretMaskedView {
        name,
        namespace,
        type_: secret_type(&secret),
        data: masked_data(&secret),
    }))
}

pub async fn reveal_secret(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<SecretRevealView>> {
    let client = state.kubernetes_client()?;
    let api: Api<Secret> = client.api(Some(&namespace));

    let secret = api
        .get(&name)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    let data = secret
        .data
        .as_ref()
        .map(|m| {
            m.iter()
                .map(|(k, v)| {
                    let decoded = String::from_utf8(v.0.clone())
                        .unwrap_or_else(|_| base64::engine::general_purpose::STANDARD.encode(&v.0));
                    (k.clone(), decoded)
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    Ok(Json(SecretRevealView {
        name,
        namespace,
        type_: secret_type(&secret),
        data,
        warning: "Sensitive data disclosed by explicit action. Do not store values in client state longer than session.".to_string(),
    }))
}

pub async fn create_secret(
    State(state): State<Arc<AppState>>,
    Path(namespace): Path<String>,
    Json(payload): Json<Secret>,
) -> Result<Json<SecretSummary>> {
    let client = state.kubernetes_client()?;
    let api: Api<Secret> = client.api(Some(&namespace));

    let created = api
        .create(&PostParams::default(), &payload)
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    Ok(Json(SecretSummary {
        name: created
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        namespace: created
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        type_: secret_type(&created),
        keys_count: created.data.as_ref().map(|m| m.len()).unwrap_or(0),
    }))
}

pub async fn update_secret(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
    Json(mut payload): Json<Secret>,
) -> Result<Json<SecretSummary>> {
    let client = state.kubernetes_client()?;
    let api: Api<Secret> = client.api(Some(&namespace));

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

    Ok(Json(SecretSummary {
        name: updated
            .metadata
            .name
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        namespace: updated
            .metadata
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
        type_: secret_type(&updated),
        keys_count: updated.data.as_ref().map(|m| m.len()).unwrap_or(0),
    }))
}

pub async fn delete_secret(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<BTreeMap<String, String>>> {
    let client = state.kubernetes_client()?;
    let api: Api<Secret> = client.api(Some(&namespace));

    let _ = api
        .delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| Error::Kubernetes(e.to_string()))?;

    let mut response = BTreeMap::new();
    response.insert("status".to_string(), "ok".to_string());
    response.insert(
        "message".to_string(),
        format!("Secret {namespace}/{name} deleted"),
    );
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::ByteString;

    fn make_secret(
        name: &str,
        namespace: &str,
        type_: &str,
        data: BTreeMap<String, ByteString>,
    ) -> Secret {
        Secret {
            metadata: kube::api::ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some(namespace.to_string()),
                ..Default::default()
            },
            type_: Some(type_.to_string()),
            data: if data.is_empty() { None } else { Some(data) },
            ..Default::default()
        }
    }

    #[test]
    fn test_secret_type_returns_opaque_when_none() {
        let secret = Secret {
            metadata: kube::api::ObjectMeta {
                name: Some("test".to_string()),
                ..Default::default()
            },
            type_: None,
            data: None,
            ..Default::default()
        };
        assert_eq!(secret_type(&secret), "Opaque");
    }

    #[test]
    fn test_secret_type_returns_type_when_set() {
        let secret = make_secret("test", "default", "kubernetes.io/tls", BTreeMap::new());
        assert_eq!(secret_type(&secret), "kubernetes.io/tls");
    }

    #[test]
    fn test_masked_data_returns_empty_when_no_data() {
        let secret = Secret {
            metadata: kube::api::ObjectMeta::default(),
            type_: None,
            data: None,
            ..Default::default()
        };
        let masked = masked_data(&secret);
        assert!(masked.is_empty());
    }

    #[test]
    fn test_masked_data_replaces_values_with_stars() {
        let mut data = BTreeMap::new();
        data.insert("key1".to_string(), ByteString(vec![1, 2, 3]));
        data.insert("key2".to_string(), ByteString(vec![4, 5, 6]));
        let secret = make_secret("test", "default", "Opaque", data);
        let masked = masked_data(&secret);
        assert_eq!(masked.len(), 2);
        assert_eq!(masked.get("key1").unwrap(), "***");
        assert_eq!(masked.get("key2").unwrap(), "***");
    }

    #[test]
    fn test_secret_summary_serialization() {
        let summary = SecretSummary {
            name: "my-secret".to_string(),
            namespace: "production".to_string(),
            type_: "Opaque".to_string(),
            keys_count: 3,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"name\":\"my-secret\""));
        assert!(json.contains("\"namespace\":\"production\""));
        assert!(json.contains("\"type\":\"Opaque\""));
        assert!(json.contains("\"keys_count\":3"));
    }

    #[test]
    fn test_secret_masked_view_serialization() {
        let mut data = BTreeMap::new();
        data.insert("password".to_string(), "***".to_string());
        let view = SecretMaskedView {
            name: "db-credentials".to_string(),
            namespace: "default".to_string(),
            type_: "Opaque".to_string(),
            data,
        };
        let json = serde_json::to_value(&view).unwrap();
        assert_eq!(json["name"], "db-credentials");
        assert_eq!(json["namespace"], "default");
        assert_eq!(json["type"], "Opaque");
        assert_eq!(json["data"]["password"], "***");
    }

    #[test]
    fn test_secret_reveal_view_serialization() {
        let mut data = BTreeMap::new();
        data.insert("token".to_string(), "secret-value".to_string());
        let view = SecretRevealView {
            name: "api-token".to_string(),
            namespace: "kube-system".to_string(),
            type_: "Opaque".to_string(),
            data,
            warning: "Sensitive data disclosed".to_string(),
        };
        let json = serde_json::to_value(&view).unwrap();
        assert_eq!(json["name"], "api-token");
        assert_eq!(json["warning"], "Sensitive data disclosed");
        assert_eq!(json["data"]["token"], "secret-value");
    }

    #[test]
    fn test_list_secrets_query_deserialization() {
        let json = r#"{"namespace":"default","label_selector":"app=myapp","limit":10}"#;
        let query: ListSecretsQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.namespace, Some("default".to_string()));
        assert_eq!(query.label_selector, Some("app=myapp".to_string()));
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_list_secrets_query_all_optional() {
        let json = r#"{}"#;
        let query: ListSecretsQuery = serde_json::from_str(json).unwrap();
        assert!(query.namespace.is_none());
        assert!(query.label_selector.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_secret_summary_type_field_rename() {
        let summary = SecretSummary {
            name: "test".to_string(),
            namespace: "ns".to_string(),
            type_: "kubernetes.io/dockerconfigjson".to_string(),
            keys_count: 1,
        };
        let value = serde_json::to_value(&summary).unwrap();
        assert!(value.get("type").is_some());
        assert!(value.get("type_").is_none());
        assert_eq!(value["type"], "kubernetes.io/dockerconfigjson");
    }

    #[test]
    fn test_masked_view_type_field_rename() {
        let view = SecretMaskedView {
            name: "s".to_string(),
            namespace: "n".to_string(),
            type_: "Opaque".to_string(),
            data: BTreeMap::new(),
        };
        let value = serde_json::to_value(&view).unwrap();
        assert!(value.get("type").is_some());
        assert!(value.get("type_").is_none());
    }

    #[test]
    fn test_reveal_view_type_field_rename() {
        let view = SecretRevealView {
            name: "s".to_string(),
            namespace: "n".to_string(),
            type_: "kubernetes.io/tls".to_string(),
            data: BTreeMap::new(),
            warning: "warn".to_string(),
        };
        let value = serde_json::to_value(&view).unwrap();
        assert!(value.get("type").is_some());
        assert!(value.get("type_").is_none());
    }

    #[test]
    fn test_create_secret_payload_deserialization() {
        let json = serde_json::json!({
            "metadata": {"name": "my-secret", "namespace": "default"},
            "type": "Opaque",
            "data": {"key": "dmFsdWU="}
        });
        let payload: Secret = serde_json::from_value(json).unwrap();
        assert_eq!(payload.metadata.name, Some("my-secret".to_string()));
        assert_eq!(payload.type_, Some("Opaque".to_string()));
    }
}
