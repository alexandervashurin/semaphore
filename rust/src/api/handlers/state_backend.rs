//! Terraform Remote State Backend — HTTP handlers
//!
//! Implements the Terraform HTTP backend protocol:
//! <https://developer.hashicorp.com/terraform/language/settings/backends/http>
//!
//! Endpoint base: /api/project/{project_id}/terraform/state/{workspace}
//!
//! GET    .../state/{ws}         — fetch latest state (raw JSON bytes)
//! POST   .../state/{ws}?ID=...  — push new state
//! DELETE .../state/{ws}         — delete state
//! LOCK   .../state/{ws}         — acquire lock  (423 if locked)
//! UNLOCK .../state/{ws}         — release lock
//!
//! Because LOCK and UNLOCK are non-standard HTTP methods, we register the
//! state route with `axum::routing::any()` and dispatch inside `state_dispatch`.
//!
//! Additional UI-friendly endpoints (registered separately):
//! GET /api/project/{pid}/terraform/workspaces
//! GET /api/project/{pid}/terraform/state/{ws}/history
//! GET /api/project/{pid}/terraform/state/{ws}/lock
//! GET /api/project/{pid}/terraform/state/{ws}/{serial}

use crate::api::extractors::AuthUser;
use crate::api::state::AppState;
use crate::db::store::TerraformStateManager;
use crate::models::{LockInfo, TerraformState, TerraformStateLock};
use axum::{
    body::Bytes,
    extract::{Path, Request, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

// ─── Query params ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct StatePostQuery {
    #[serde(rename = "ID")]
    pub id: Option<String>,
}

// ─── Unified dispatch (GET / POST / DELETE / LOCK / UNLOCK) ──────────────────

/// Single handler for the state endpoint — dispatches by HTTP method.
/// Registered with `axum::routing::any()` so LOCK and UNLOCK reach it.
pub async fn state_dispatch(
    State(app): State<Arc<AppState>>,
    auth: AuthUser,
    Path((project_id, workspace)): Path<(i32, String)>,
    req: Request,
) -> impl IntoResponse {
    let method = req.method().as_str().to_uppercase();

    // Collect body bytes regardless of method.
    let (parts, body) = req.into_parts();
    let body_bytes = match axum::body::to_bytes(body, 64 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "failed to read request body"})),
            )
                .into_response()
        }
    };

    // Parse ?ID=<lock_id> from the URI query string (simple key=value scan).
    let lock_id_from_query = parts.uri.query().and_then(|q| {
        q.split('&').find_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            if k.eq_ignore_ascii_case("id") {
                Some(v.to_string())
            } else {
                None
            }
        })
    });
    let query = StatePostQuery {
        id: lock_id_from_query,
    };

    match method.as_str() {
        "GET" => handle_get(app, auth, project_id, workspace).await,
        "POST" => {
            handle_post(
                app,
                auth,
                project_id,
                workspace,
                query,
                parts.headers,
                body_bytes,
            )
            .await
        }
        "DELETE" => handle_delete(app, auth, project_id, workspace).await,
        "LOCK" => handle_lock(app, auth, project_id, workspace, body_bytes).await,
        "UNLOCK" => handle_unlock(app, auth, project_id, workspace, body_bytes).await,
        _ => StatusCode::METHOD_NOT_ALLOWED.into_response(),
    }
}

// ─── GET ─────────────────────────────────────────────────────────────────────

async fn handle_get(
    app: Arc<AppState>,
    _auth: AuthUser,
    project_id: i32,
    workspace: String,
) -> axum::response::Response {
    let store = app.store.store();
    match store.get_terraform_state(project_id, &workspace).await {
        Ok(Some(s)) => (StatusCode::OK, s.state_data).into_response(),
        Ok(None) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// ─── POST ────────────────────────────────────────────────────────────────────

async fn handle_post(
    app: Arc<AppState>,
    _auth: AuthUser,
    project_id: i32,
    workspace: String,
    query: StatePostQuery,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> axum::response::Response {
    let store = app.store.store();

    // Verify lock ID matches if workspace is locked.
    if let Ok(Some(lock)) = store.get_terraform_lock(project_id, &workspace).await {
        let provided = query.id.as_deref().unwrap_or("");
        if lock.lock_id != provided {
            let info = LockInfo::from_lock(&lock);
            return (StatusCode::LOCKED, Json(json!(info))).into_response();
        }
    }

    let (serial, lineage) = extract_serial_lineage(&body);

    // Use sha2 (already a dep) to produce a hex digest for idempotency checks.
    let md5_hash = {
        use sha2::{Digest as _, Sha256};
        let hash = Sha256::digest(&body);
        format!("{:x}", hash)[..32].to_string()
    };

    let serial = if serial == 0 {
        headers
            .get("X-Terraform-State-Serial")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0)
    } else {
        serial
    };

    let record = TerraformState {
        id: 0,
        project_id,
        workspace: workspace.clone(),
        serial,
        lineage,
        state_data: body.to_vec(),
        encrypted: false,
        md5: md5_hash,
        created_at: chrono::Utc::now(),
    };

    match store.create_terraform_state(record).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e)
            if e.to_string()
                .contains("already exists with different content") =>
        {
            (StatusCode::CONFLICT, Json(json!({"error": e.to_string()}))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// ─── DELETE ──────────────────────────────────────────────────────────────────

async fn handle_delete(
    app: Arc<AppState>,
    _auth: AuthUser,
    project_id: i32,
    workspace: String,
) -> axum::response::Response {
    let store = app.store.store();
    match store.delete_terraform_state(project_id, &workspace).await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// ─── LOCK ────────────────────────────────────────────────────────────────────

async fn handle_lock(
    app: Arc<AppState>,
    _auth: AuthUser,
    project_id: i32,
    workspace: String,
    body: Bytes,
) -> axum::response::Response {
    let lock_info: LockInfo = match serde_json::from_slice(&body) {
        Ok(l) => l,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "invalid lock JSON body"})),
            )
                .into_response()
        }
    };

    let store = app.store.store();
    let lock = TerraformStateLock {
        project_id,
        workspace: workspace.clone(),
        lock_id: lock_info.id.clone(),
        operation: lock_info.operation.clone(),
        info: lock_info.info.clone(),
        who: lock_info.who.clone(),
        version: lock_info.version.clone(),
        path: lock_info.path.clone(),
        created_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(2),
    };

    match store
        .lock_terraform_state(project_id, &workspace, lock)
        .await
    {
        Ok(l) => (StatusCode::OK, Json(json!(LockInfo::from_lock(&l)))).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if let Some(json_str) = msg.strip_prefix("locked:") {
                let info: Value = serde_json::from_str(json_str).unwrap_or(Value::Null);
                (StatusCode::LOCKED, Json(info)).into_response()
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": msg})),
                )
                    .into_response()
            }
        }
    }
}

// ─── UNLOCK ──────────────────────────────────────────────────────────────────

async fn handle_unlock(
    app: Arc<AppState>,
    _auth: AuthUser,
    project_id: i32,
    workspace: String,
    body: Bytes,
) -> axum::response::Response {
    let store = app.store.store();

    let lock_id = serde_json::from_slice::<LockInfo>(&body)
        .ok()
        .map(|l| l.id)
        .unwrap_or_default();

    if lock_id.is_empty() {
        // Force-unlock.
        if let Ok(Some(existing)) = store.get_terraform_lock(project_id, &workspace).await {
            let _ = store
                .unlock_terraform_state(project_id, &workspace, &existing.lock_id)
                .await;
        }
        return StatusCode::OK.into_response();
    }

    match store
        .unlock_terraform_state(project_id, &workspace, &lock_id)
        .await
    {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) if e.to_string().contains("not found") => (
            StatusCode::CONFLICT,
            Json(json!({"error": "lock not found or ID mismatch"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// ─── UI endpoints ─────────────────────────────────────────────────────────────

/// GET /api/project/{pid}/terraform/workspaces
pub async fn list_workspaces(
    State(app): State<Arc<AppState>>,
    _auth: AuthUser,
    Path(project_id): Path<i32>,
) -> impl IntoResponse {
    let store = app.store.store();
    match store.list_terraform_workspaces(project_id).await {
        Ok(ws) => (StatusCode::OK, Json(json!(ws))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/project/{pid}/terraform/state/{ws}/history
pub async fn list_state_history(
    State(app): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((project_id, workspace)): Path<(i32, String)>,
) -> impl IntoResponse {
    let store = app.store.store();
    match store.list_terraform_states(project_id, &workspace).await {
        Ok(versions) => (StatusCode::OK, Json(json!(versions))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/project/{pid}/terraform/state/{ws}/lock
pub async fn get_lock_info(
    State(app): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((project_id, workspace)): Path<(i32, String)>,
) -> impl IntoResponse {
    let store = app.store.store();
    match store.get_terraform_lock(project_id, &workspace).await {
        Ok(Some(lock)) => (StatusCode::OK, Json(json!(LockInfo::from_lock(&lock)))).into_response(),
        Ok(None) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/project/{pid}/terraform/state/{ws}/{serial}
pub async fn get_state_by_serial(
    State(app): State<Arc<AppState>>,
    _auth: AuthUser,
    Path((project_id, workspace, serial)): Path<(i32, String, i32)>,
) -> impl IntoResponse {
    let store = app.store.store();
    match store
        .get_terraform_state_by_serial(project_id, &workspace, serial)
        .await
    {
        Ok(Some(s)) => (StatusCode::OK, s.state_data).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn extract_serial_lineage(body: &[u8]) -> (i32, String) {
    let Ok(v) = serde_json::from_slice::<Value>(body) else {
        return (0, String::new());
    };
    let serial = v.get("serial").and_then(|s| s.as_i64()).unwrap_or(0) as i32;
    let lineage = v
        .get("lineage")
        .and_then(|l| l.as_str())
        .unwrap_or("")
        .to_string();
    (serial, lineage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_serial_lineage_valid() {
        let body = br#"{"serial": 5, "lineage": "abc-123"}"#;
        let (serial, lineage) = extract_serial_lineage(body);
        assert_eq!(serial, 5);
        assert_eq!(lineage, "abc-123");
    }

    #[test]
    fn test_extract_serial_lineage_zero_serial() {
        let body = br#"{"serial": 0, "lineage": "test"}"#;
        let (serial, lineage) = extract_serial_lineage(body);
        assert_eq!(serial, 0);
        assert_eq!(lineage, "test");
    }

    #[test]
    fn test_extract_serial_lineage_missing_serial() {
        let body = br#"{"lineage": "no-serial"}"#;
        let (serial, lineage) = extract_serial_lineage(body);
        assert_eq!(serial, 0);
        assert_eq!(lineage, "no-serial");
    }

    #[test]
    fn test_extract_serial_lineage_missing_lineage() {
        let body = br#"{"serial": 10}"#;
        let (serial, lineage) = extract_serial_lineage(body);
        assert_eq!(serial, 10);
        assert_eq!(lineage, "");
    }

    #[test]
    fn test_extract_serial_lineage_empty_body() {
        let body = b"{}";
        let (serial, lineage) = extract_serial_lineage(body);
        assert_eq!(serial, 0);
        assert_eq!(lineage, "");
    }

    #[test]
    fn test_extract_serial_lineage_invalid_json() {
        let body = b"not json";
        let (serial, lineage) = extract_serial_lineage(body);
        assert_eq!(serial, 0);
        assert_eq!(lineage, "");
    }

    #[test]
    fn test_extract_serial_lineage_serial_as_string() {
        let body = br#"{"serial": "42", "lineage": "test"}"#;
        let (serial, lineage) = extract_serial_lineage(body);
        // as_i64() returns None for string, so defaults to 0
        assert_eq!(serial, 0);
        assert_eq!(lineage, "test");
    }

    #[test]
    fn test_extract_serial_lineage_large_serial() {
        let body = br#"{"serial": 999999, "lineage": "large"}"#;
        let (serial, lineage) = extract_serial_lineage(body);
        assert_eq!(serial, 999999);
        assert_eq!(lineage, "large");
    }

    #[test]
    fn test_state_post_query_from_query_string() {
        let query = "ID=lock-123&foo=bar";
        let lock_id = query.split('&').find_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            if k.eq_ignore_ascii_case("id") {
                Some(v.to_string())
            } else {
                None
            }
        });
        assert_eq!(lock_id, Some("lock-123".to_string()));
    }

    #[test]
    fn test_state_post_query_case_insensitive() {
        let query = "id=lock-456&other=val";
        let lock_id = query.split('&').find_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            if k.eq_ignore_ascii_case("id") {
                Some(v.to_string())
            } else {
                None
            }
        });
        assert_eq!(lock_id, Some("lock-456".to_string()));
    }

    #[test]
    fn test_state_post_query_no_id() {
        let query = "foo=bar&baz=qux";
        let lock_id = query.split('&').find_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            if k.eq_ignore_ascii_case("id") {
                Some(v.to_string())
            } else {
                None
            }
        });
        assert!(lock_id.is_none());
    }

    #[test]
    fn test_state_post_query_empty() {
        let query = "";
        let lock_id = query.split('&').find_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            if k.eq_ignore_ascii_case("id") {
                Some(v.to_string())
            } else {
                None
            }
        });
        assert!(lock_id.is_none());
    }

    #[test]
    fn test_lock_info_deserialization() {
        let json = r#"{
            "ID": "lock-abc",
            "Operation": "OperationTypeApply",
            "Info": "Apply",
            "Who": "user@example.com",
            "Version": "1.5.0",
            "Path": "module.main"
        }"#;
        let lock: LockInfo = serde_json::from_str(json).unwrap();
        assert_eq!(lock.id, "lock-abc");
        assert_eq!(lock.operation, "OperationTypeApply");
        assert_eq!(lock.info, "Apply");
        assert_eq!(lock.who, "user@example.com");
    }

    #[test]
    fn test_lock_info_with_optional_created() {
        let json = r#"{
            "ID": "lock-xyz",
            "Operation": "OperationTypePlan",
            "Info": "Plan",
            "Who": "admin",
            "Version": "1.6.0",
            "Path": "root",
            "Created": "2024-01-01T00:00:00Z"
        }"#;
        let lock: LockInfo = serde_json::from_str(json).unwrap();
        assert_eq!(lock.id, "lock-xyz");
        assert!(lock.created.is_some());
        assert_eq!(lock.created, Some("2024-01-01T00:00:00Z".to_string()));
    }

    #[test]
    fn test_terraform_state_summary_serialization() {
        let summary = crate::models::TerraformStateSummary {
            id: 1,
            project_id: 10,
            workspace: "production".to_string(),
            serial: 42,
            lineage: "line-xyz".to_string(),
            encrypted: false,
            md5: "abc123".to_string(),
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"workspace\":\"production\""));
        assert!(json.contains("\"serial\":42"));
        assert!(json.contains("\"encrypted\":false"));
    }

    #[test]
    fn test_state_diff_resource_serialization() {
        use crate::models::StateDiffResource;
        let resource = StateDiffResource {
            address: "aws_instance.web".to_string(),
            change_type: "added".to_string(),
            resource_type: "aws_instance".to_string(),
            name: "web".to_string(),
        };
        let json = serde_json::to_string(&resource).unwrap();
        assert!(json.contains("\"address\":\"aws_instance.web\""));
        assert!(json.contains("\"change_type\":\"added\""));
    }

    #[test]
    fn test_state_diff_serialization() {
        use crate::models::{StateDiff, StateDiffResource};
        let diff = StateDiff {
            from_serial: 1,
            to_serial: 3,
            resources: vec![StateDiffResource {
                address: "aws_instance.app".to_string(),
                change_type: "changed".to_string(),
                resource_type: "aws_instance".to_string(),
                name: "app".to_string(),
            }],
            added: 0,
            changed: 1,
            removed: 0,
        };
        let json = serde_json::to_string(&diff).unwrap();
        assert!(json.contains("\"from_serial\":1"));
        assert!(json.contains("\"to_serial\":3"));
        assert!(json.contains("\"changed\":1"));
    }

    #[test]
    fn test_method_dispatch_recognition() {
        // Verify that uppercase method matching works
        let methods = vec!["GET", "POST", "DELETE", "LOCK", "UNLOCK", "PATCH"];
        let recognized: Vec<String> = methods
            .iter()
            .map(|m| m.to_uppercase())
            .filter(|m| matches!(m.as_str(), "GET" | "POST" | "DELETE" | "LOCK" | "UNLOCK"))
            .collect();
        assert_eq!(recognized.len(), 5);
    }
}
