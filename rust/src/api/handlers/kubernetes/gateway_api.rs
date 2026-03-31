//! Kubernetes Gateway API handlers (optional, read-only)

use axum::{
    extract::{Query, State},
    Json,
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
