//! MCP (Model Context Protocol) JSON-RPC 2.0 types — embedded in Velum

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Incoming JSON-RPC request
#[derive(Debug, Deserialize)]
pub struct McpRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

/// Outgoing JSON-RPC response
#[derive(Debug, Serialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

impl McpResponse {
    pub fn ok(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }
    pub fn err(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// A single tool result content item
#[derive(Debug, Serialize)]
pub struct ToolContent {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub text: String,
}

impl ToolContent {
    pub fn text(s: impl Into<String>) -> Self {
        Self {
            kind: "text",
            text: s.into(),
        }
    }
    pub fn json(v: &Value) -> Self {
        Self {
            kind: "text",
            text: serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string()),
        }
    }
}

/// Helper: build a required integer property schema
pub fn prop_int(desc: &str) -> Value {
    serde_json::json!({ "type": "integer", "description": desc })
}
/// Helper: build a required string property schema
pub fn prop_str(desc: &str) -> Value {
    serde_json::json!({ "type": "string", "description": desc })
}
/// Helper: optional integer
pub fn prop_int_opt(desc: &str) -> Value {
    serde_json::json!({ "type": ["integer","null"], "description": desc })
}
/// Helper: optional string
pub fn prop_str_opt(desc: &str) -> Value {
    serde_json::json!({ "type": ["string","null"], "description": desc })
}
/// Helper: boolean
pub fn prop_bool(desc: &str) -> Value {
    serde_json::json!({ "type": "boolean", "description": desc })
}
