//! Terraform HTTP Backend handlers (Phase 1)
//!
//! Terraform state HTTP backend protocol:
//!   GET    /api/project/{pid}/terraform/state/{ws}  — get current state
//!   POST   /api/project/{pid}/terraform/state/{ws}  — upload new state
//!   DELETE /api/project/{pid}/terraform/state/{ws}  — delete workspace state
//!   LOCK   /api/project/{pid}/terraform/state/{ws}  — acquire lock (custom method)
//!   UNLOCK /api/project/{pid}/terraform/state/{ws}  — release lock (custom method)

use crate::api::state::AppState;
use crate::db::store::TerraformStateManager;
use axum::{
    body::Bytes,
    extract::{Path, Query, Request, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::Arc;

// ─── single dispatch handler (handles all HTTP methods) ────────────────────

/// Unified state endpoint — dispatches by HTTP method including LOCK/UNLOCK
pub async fn state_dispatch(
    State(state): State<Arc<AppState>>,
    Path((project_id, workspace)): Path<(i32, String)>,
    req: Request,
) -> impl IntoResponse {
    let method = req.method().to_string();
    let query  = req.uri().query().unwrap_or("").to_string();
    let body   = axum::body::to_bytes(req.into_body(), 64 * 1024 * 1024)
        .await
        .unwrap_or_default();

    let store = state.store.store();

    match method.as_str() {
        "GET"    => handle_get(store, project_id, &workspace).await,
        "POST"   => handle_post(store, project_id, &workspace, body).await,
        "DELETE" => handle_delete(store, project_id, &workspace).await,
        "LOCK"   => handle_lock(store, project_id, &workspace, body).await,
        "UNLOCK" => handle_unlock(store, project_id, &workspace, &query, body).await,
        other    => (
            StatusCode::METHOD_NOT_ALLOWED,
            Json(json!({"error": format!("method {other} not supported")})),
        ).into_response(),
    }
}

async fn handle_get(
    store: &(dyn crate::db::Store + Send + Sync),
    project_id: i32,
    workspace:  &str,
) -> axum::response::Response {
    match store.get_terraform_state(project_id, workspace).await {
        Ok(Some(s)) => (StatusCode::OK, s.state_data).into_response(),
        Ok(None)    => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ).into_response(),
    }
}

async fn handle_post(
    store:      &(dyn crate::db::Store + Send + Sync),
    project_id: i32,
    workspace:  &str,
    body:       Bytes,
) -> axum::response::Response {
    if body.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "empty state body"}))).into_response();
    }

    // Parse serial + lineage from the state JSON
    let (serial, lineage) = extract_serial_lineage(&body);

    // Compute content hash
    let md5 = {
        let mut h = Sha256::new();
        h.update(&body);
        let hex = format!("{:x}", h.finalize());
        hex[..32.min(hex.len())].to_string()
    };

    match store.save_terraform_state(project_id, workspace, serial, &lineage, body.to_vec(), &md5).await {
        Ok(_)  => StatusCode::OK.into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("different content") {
                (StatusCode::CONFLICT, Json(json!({"error": msg}))).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": msg}))).into_response()
            }
        }
    }
}

async fn handle_delete(
    store:      &(dyn crate::db::Store + Send + Sync),
    project_id: i32,
    workspace:  &str,
) -> axum::response::Response {
    match store.delete_terraform_state(project_id, workspace).await {
        Ok(_)  => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

async fn handle_lock(
    store:      &(dyn crate::db::Store + Send + Sync),
    project_id: i32,
    workspace:  &str,
    body:       Bytes,
) -> axum::response::Response {
    let lock_info: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v)  => v,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({"error": "invalid lock JSON"}))).into_response(),
    };

    let lock_id   = lock_info.get("ID").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let operation = lock_info.get("Operation").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let info      = lock_info.get("Info").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let who       = lock_info.get("Who").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let version   = lock_info.get("Version").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let path      = lock_info.get("Path").and_then(|v| v.as_str()).unwrap_or("").to_string();

    match store.lock_terraform_state(project_id, workspace, &lock_id, &operation, &info, &who, &version, &path).await {
        Ok(_) => (StatusCode::OK, Json(lock_info)).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if let Some(json_str) = msg.strip_prefix("locked:") {
                let existing: serde_json::Value = serde_json::from_str(json_str).unwrap_or(json!({}));
                (StatusCode::LOCKED, Json(existing)).into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": msg}))).into_response()
            }
        }
    }
}

async fn handle_unlock(
    store:      &(dyn crate::db::Store + Send + Sync),
    project_id: i32,
    workspace:  &str,
    query:      &str,
    body:       Bytes,
) -> axum::response::Response {
    // Lock ID can come from query ?ID=<id> or body JSON
    let lock_id = parse_query_id(query).unwrap_or_else(|| {
        serde_json::from_slice::<serde_json::Value>(&body)
            .ok()
            .and_then(|v| v.get("ID").and_then(|id| id.as_str()).map(|s| s.to_string()))
            .unwrap_or_default()
    });

    match store.unlock_terraform_state(project_id, workspace, &lock_id).await {
        Ok(_)  => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

// ─── UI helper endpoints ───────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i64>,
}

/// GET /api/project/{pid}/terraform/workspaces
pub async fn list_workspaces(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.list_terraform_workspaces(project_id).await {
        Ok(ws)  => (StatusCode::OK, Json(json!(ws))).into_response(),
        Err(e)  => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/project/{pid}/terraform/state/{ws}/history
pub async fn list_state_history(
    State(state): State<Arc<AppState>>,
    Path((project_id, workspace)): Path<(i32, String)>,
    Query(q): Query<HistoryQuery>,
) -> impl IntoResponse {
    let store = state.store.store();
    let limit = q.limit.unwrap_or(50).min(200);
    match store.list_terraform_state_history(project_id, &workspace, limit).await {
        Ok(list) => (StatusCode::OK, Json(json!(list))).into_response(),
        Err(e)   => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

/// GET /api/project/{pid}/terraform/state/{ws}/lock
pub async fn get_lock_info(
    State(state): State<Arc<AppState>>,
    Path((project_id, workspace)): Path<(i32, String)>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.get_terraform_lock(project_id, &workspace).await {
        Ok(Some(lock)) => (StatusCode::OK, Json(json!(crate::models::LockInfo::from_lock(&lock)))).into_response(),
        Ok(None)       => StatusCode::NO_CONTENT.into_response(),
        Err(e)         => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

/// DELETE /api/project/{pid}/terraform/state/{ws}/lock  (force unlock)
pub async fn force_unlock(
    State(state): State<Arc<AppState>>,
    Path((project_id, workspace)): Path<(i32, String)>,
) -> impl IntoResponse {
    let store = state.store.store();
    match store.unlock_terraform_state(project_id, &workspace, "").await {
        Ok(_)  => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

// ─── helpers ──────────────────────────────────────────────────────────────

fn extract_serial_lineage(body: &[u8]) -> (i32, String) {
    let val: serde_json::Value = serde_json::from_slice(body).unwrap_or(json!({}));
    let serial  = val.get("serial").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let lineage = val.get("lineage").and_then(|v| v.as_str()).unwrap_or("").to_string();
    (serial, lineage)
}

fn parse_query_id(query: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        if k.eq_ignore_ascii_case("id") { Some(v.to_string()) } else { None }
    })
}
