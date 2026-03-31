//! Axum handlers for the embedded MCP endpoint and MCP settings API.

use super::protocol::{McpRequest, McpResponse};
use super::tools;
use crate::api::extractors::AuthUser;
use crate::api::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

const MCP_VERSION: &str = "2024-11-05";
const SERVER_NAME: &str = "velum-mcp";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

// ── POST /mcp ─────────────────────────────────────────────────────────────────

/// Main MCP JSON-RPC 2.0 endpoint. Requires Bearer JWT auth.
pub async fn mcp_endpoint(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser, // enforce authentication
    Json(req): Json<McpRequest>,
) -> impl IntoResponse {
    let id = req.id.clone();
    let resp = handle(req, &state).await;
    // Notification responses (Value::Null) → 204 No Content
    if resp == Value::Null {
        (StatusCode::NO_CONTENT, Json(Value::Null))
    } else {
        (StatusCode::OK, Json(resp))
    }
}

async fn handle(req: McpRequest, state: &Arc<AppState>) -> Value {
    let id = req.id.clone();
    match req.method.as_str() {
        "initialize" => serde_json::to_value(McpResponse::ok(
            id,
            json!({
                "protocolVersion": MCP_VERSION,
                "capabilities": { "tools": {} },
                "serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION }
            }),
        ))
        .unwrap_or(Value::Null),

        "notifications/initialized" => Value::Null,

        "tools/list" => {
            let tool_list = tools::all_definitions();
            serde_json::to_value(McpResponse::ok(id, json!({ "tools": tool_list })))
                .unwrap_or(Value::Null)
        }

        "tools/call" => {
            let params = req.params.unwrap_or(Value::Null);
            let tool_name = params["name"].as_str().unwrap_or("").to_string();
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| Value::Object(Default::default()));

            let result = tools::dispatch(&tool_name, &args, state).await;
            let content: Vec<Value> = result
                .content
                .iter()
                .map(|c| json!({ "type": c.kind, "text": c.text }))
                .collect();

            serde_json::to_value(McpResponse::ok(
                id,
                json!({ "content": content, "isError": result.is_error }),
            ))
            .unwrap_or(Value::Null)
        }

        "ping" => serde_json::to_value(McpResponse::ok(id, json!({}))).unwrap_or(Value::Null),

        other => serde_json::to_value(McpResponse::err(
            id,
            -32601,
            format!("Method not found: {other}"),
        ))
        .unwrap_or(Value::Null),
    }
}

// ── GET /api/mcp/settings ─────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct McpSettings {
    pub enabled: bool,
    pub tool_count: usize,
    pub endpoint: &'static str,
    pub transport: &'static str,
    pub version: &'static str,
}

pub async fn get_mcp_settings(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let store = state.store.store();
    let enabled = store
        .get_option("mcp.enabled")
        .await
        .unwrap_or(Some("true".into()))
        .unwrap_or_else(|| "true".into())
        == "true";

    Json(json!({
        "enabled": enabled,
        "endpoint": "/mcp",
        "transport": "http (JSON-RPC 2.0)",
        "version": MCP_VERSION,
        "server_version": SERVER_VERSION,
        "tool_count": tools::all_definitions().len(),
        "auth": "Bearer JWT (same token used for the Velum API)",
        "claude_desktop_config": {
            "mcpServers": {
                "velum": {
                    "url": "http://<your-velum-host>/mcp",
                    "headers": { "Authorization": "Bearer <your-jwt-token>" }
                }
            }
        },
        "claude_code_cmd": "claude mcp add-json velum '{\"type\":\"http\",\"url\":\"http://localhost:3000/mcp\",\"headers\":{\"Authorization\":\"Bearer <token>\"}}'",
    }))
}

// ── PUT /api/mcp/settings ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct McpSettingsUpdate {
    pub enabled: Option<bool>,
}

pub async fn update_mcp_settings(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Json(body): Json<McpSettingsUpdate>,
) -> impl IntoResponse {
    let store = state.store.store();
    if let Some(enabled) = body.enabled {
        let _ = store
            .set_option("mcp.enabled", if enabled { "true" } else { "false" })
            .await;
    }
    (StatusCode::NO_CONTENT, Json(Value::Null))
}

// ── GET /api/mcp/tools ────────────────────────────────────────────────────────

pub async fn get_mcp_tools(_auth: AuthUser) -> impl IntoResponse {
    Json(json!({ "tools": tools::all_definitions() }))
}
