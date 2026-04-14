//! Kubernetes Gateway API handlers (optional, read-only)

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
pub struct GatewayApiNamespaceQuery {
    pub namespace: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct GatewayApiStatus {
    pub installed: bool,
    pub gateway: bool,
    pub httproute: bool,
    pub grpcroute: bool,
}

fn gvk(group: &str, version: &str, kind: &str) -> GroupVersionKind {
    GroupVersionKind::gvk(group, version, kind)
}

fn ar(group: &str, version: &str, kind: &str, plural: &str) -> ApiResource {
    ApiResource::from_gvk_with_plural(&gvk(group, version, kind), plural)
}

pub async fn get_gateway_api_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<GatewayApiStatus>> {
    let client = state.kubernetes_client()?;
    let lp = ListParams::default().limit(1);

    let gw_api: Api<DynamicObject> = Api::all_with(
        client.raw().clone(),
        &ar("gateway.networking.k8s.io", "v1", "Gateway", "gateways"),
    );
    let hr_api: Api<DynamicObject> = Api::all_with(
        client.raw().clone(),
        &ar("gateway.networking.k8s.io", "v1", "HTTPRoute", "httproutes"),
    );
    let gr_api: Api<DynamicObject> = Api::all_with(
        client.raw().clone(),
        &ar("gateway.networking.k8s.io", "v1", "GRPCRoute", "grpcroutes"),
    );

    let gateway = gw_api.list(&lp).await.is_ok();
    let httproute = hr_api.list(&lp).await.is_ok();
    let grpcroute = gr_api.list(&lp).await.is_ok();

    Ok(Json(GatewayApiStatus {
        installed: gateway || httproute || grpcroute,
        gateway,
        httproute,
        grpcroute,
    }))
}

pub async fn list_gateways(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GatewayApiNamespaceQuery>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let client = state.kubernetes_client()?;
    let api_res = ar("gateway.networking.k8s.io", "v1", "Gateway", "gateways");
    let api: Api<DynamicObject> = if let Some(ns) = query.namespace.as_deref() {
        Api::namespaced_with(client.raw().clone(), ns, &api_res)
    } else {
        Api::all_with(client.raw().clone(), &api_res)
    };

    let mut lp = ListParams::default();
    if let Some(limit) = query.limit {
        lp = lp.limit(limit);
    }

    let items = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("Gateway API not available: {e}")))?;

    Ok(Json(
        items.items.iter().map(|x| serde_json::json!(x)).collect(),
    ))
}

pub async fn list_httproutes(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GatewayApiNamespaceQuery>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let client = state.kubernetes_client()?;
    let api_res = ar("gateway.networking.k8s.io", "v1", "HTTPRoute", "httproutes");
    let api: Api<DynamicObject> = if let Some(ns) = query.namespace.as_deref() {
        Api::namespaced_with(client.raw().clone(), ns, &api_res)
    } else {
        Api::all_with(client.raw().clone(), &api_res)
    };

    let mut lp = ListParams::default();
    if let Some(limit) = query.limit {
        lp = lp.limit(limit);
    }

    let items = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("HTTPRoute API not available: {e}")))?;

    Ok(Json(
        items.items.iter().map(|x| serde_json::json!(x)).collect(),
    ))
}

pub async fn list_grpcroutes(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GatewayApiNamespaceQuery>,
) -> Result<Json<Vec<serde_json::Value>>> {
    let client = state.kubernetes_client()?;
    let api_res = ar("gateway.networking.k8s.io", "v1", "GRPCRoute", "grpcroutes");
    let api: Api<DynamicObject> = if let Some(ns) = query.namespace.as_deref() {
        Api::namespaced_with(client.raw().clone(), ns, &api_res)
    } else {
        Api::all_with(client.raw().clone(), &api_res)
    };

    let mut lp = ListParams::default();
    if let Some(limit) = query.limit {
        lp = lp.limit(limit);
    }

    let items = api
        .list(&lp)
        .await
        .map_err(|e| Error::Kubernetes(format!("GRPCRoute API not available: {e}")))?;

    Ok(Json(
        items.items.iter().map(|x| serde_json::json!(x)).collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_api_namespace_query_with_namespace() {
        let json = r#"{"namespace": "istio-system", "limit": 10}"#;
        let query: GatewayApiNamespaceQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.namespace, Some("istio-system".to_string()));
        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_gateway_api_namespace_query_empty() {
        let json = r#"{}"#;
        let query: GatewayApiNamespaceQuery = serde_json::from_str(json).unwrap();
        assert!(query.namespace.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_gateway_api_namespace_query_limit_only() {
        let json = r#"{"limit": 50}"#;
        let query: GatewayApiNamespaceQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, Some(50));
        assert!(query.namespace.is_none());
    }

    #[test]
    fn test_gateway_api_status_not_installed() {
        let status = GatewayApiStatus {
            installed: false,
            gateway: false,
            httproute: false,
            grpcroute: false,
        };
        assert!(!status.installed);
        assert!(!status.gateway);
        assert!(!status.httproute);
        assert!(!status.grpcroute);
    }

    #[test]
    fn test_gateway_api_status_only_gateway() {
        let status = GatewayApiStatus {
            installed: true,
            gateway: true,
            httproute: false,
            grpcroute: false,
        };
        assert!(status.installed);
        assert!(status.gateway);
        assert!(!status.httproute);
        assert!(!status.grpcroute);
    }

    #[test]
    fn test_gateway_api_status_only_httproute() {
        let status = GatewayApiStatus {
            installed: true,
            gateway: false,
            httproute: true,
            grpcroute: false,
        };
        assert!(status.installed);
        assert!(!status.gateway);
        assert!(status.httproute);
        assert!(!status.grpcroute);
    }

    #[test]
    fn test_gateway_api_status_all_enabled() {
        let status = GatewayApiStatus {
            installed: true,
            gateway: true,
            httproute: true,
            grpcroute: true,
        };
        assert!(status.installed);
        assert!(status.gateway);
        assert!(status.httproute);
        assert!(status.grpcroute);
    }

    #[test]
    fn test_gateway_api_status_serialization() {
        let status = GatewayApiStatus {
            installed: true,
            gateway: true,
            httproute: false,
            grpcroute: true,
        };
        let json = serde_json::to_string(&status).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["installed"], true);
        assert_eq!(parsed["gateway"], true);
        assert_eq!(parsed["httproute"], false);
        assert_eq!(parsed["grpcroute"], true);
    }

    #[test]
    fn test_gateway_api_resource_gateway() {
        let ar = ar("gateway.networking.k8s.io", "v1", "Gateway", "gateways");
        assert_eq!(ar.group, "gateway.networking.k8s.io");
        assert_eq!(ar.version, "v1");
        assert_eq!(ar.kind, "Gateway");
        assert_eq!(ar.plural, "gateways");
    }

    #[test]
    fn test_gateway_api_resource_httproute() {
        let ar = ar("gateway.networking.k8s.io", "v1", "HTTPRoute", "httproutes");
        assert_eq!(ar.group, "gateway.networking.k8s.io");
        assert_eq!(ar.kind, "HTTPRoute");
        assert_eq!(ar.plural, "httproutes");
    }

    #[test]
    fn test_gateway_api_resource_grpcroute() {
        let ar = ar("gateway.networking.k8s.io", "v1", "GRPCRoute", "grpcroutes");
        assert_eq!(ar.group, "gateway.networking.k8s.io");
        assert_eq!(ar.kind, "GRPCRoute");
        assert_eq!(ar.plural, "grpcroutes");
    }

    #[test]
    fn test_gvk_creation() {
        let gvk = gvk("gateway.networking.k8s.io", "v1", "Gateway");
        assert_eq!(gvk.group, "gateway.networking.k8s.io");
        assert_eq!(gvk.version, "v1");
        assert_eq!(gvk.kind, "Gateway");
    }
}
