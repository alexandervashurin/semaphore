/// MCP (Model Context Protocol) JSON-RPC 2.0 types.
///
/// Spec: https://spec.modelcontextprotocol.io/
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── JSON-RPC base ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Request {
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

impl Response {
    pub fn ok(id: Option<Value>, result: Value) -> Self {
        Self { jsonrpc: "2.0".into(), id, result: Some(result), error: None }
    }
    pub fn err(id: Option<Value>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(RpcError { code, message: message.into(), data: None }),
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

// ── MCP types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

#[derive(Debug, Serialize)]
pub struct ToolResult {
    pub content: Vec<Content>,
    #[serde(rename = "isError")]
    pub is_error: bool,
}

impl ToolResult {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: vec![Content::Text { text: text.into() }],
            is_error: false,
        }
    }
    pub fn json(value: &Value) -> Self {
        Self {
            content: vec![Content::Text {
                text: serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()),
            }],
            is_error: false,
        }
    }
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            content: vec![Content::Text { text: msg.into() }],
            is_error: true,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Content {
    Text { text: String },
}

// ── Schema helpers ────────────────────────────────────────────────────────────

/// Build a JSON Schema object for tool input validation.
#[macro_export]
macro_rules! schema {
    ({ $($key:literal : $val:tt),* $(,)? }) => {{
        use serde_json::json;
        json!({ $($key: $val),* })
    }};
}

pub fn prop_int(description: &str) -> Value {
    serde_json::json!({ "type": "integer", "description": description })
}
pub fn prop_str(description: &str) -> Value {
    serde_json::json!({ "type": "string", "description": description })
}
pub fn prop_bool(description: &str) -> Value {
    serde_json::json!({ "type": "boolean", "description": description })
}
pub fn prop_str_opt(description: &str) -> Value {
    serde_json::json!({ "type": ["string", "null"], "description": description })
}
pub fn prop_int_opt(description: &str) -> Value {
    serde_json::json!({ "type": ["integer", "null"], "description": description })
}
