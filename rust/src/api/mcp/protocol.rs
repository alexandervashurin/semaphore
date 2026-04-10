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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── McpResponse::ok ──────────────────────────────────────────────────

    #[test]
    fn test_response_ok_has_result() {
        let resp = McpResponse::ok(Some(json!(1)), json!({"status": "ok"}));
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
        assert_eq!(resp.jsonrpc, "2.0");
    }

    #[test]
    fn test_response_ok_serializes_correctly() {
        let resp = McpResponse::ok(Some(json!("abc")), json!(42));
        let serialized = serde_json::to_string(&resp).unwrap();
        assert!(serialized.contains("\"result\":42"));
        assert!(!serialized.contains("error"));
    }

    // ── McpResponse::err ─────────────────────────────────────────────────

    #[test]
    fn test_response_err_has_error() {
        let resp = McpResponse::err(Some(json!(2)), -32601, "Method not found");
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[test]
    fn test_response_err_serializes() {
        let resp = McpResponse::err(None, -32600, "Invalid Request");
        let serialized = serde_json::to_string(&resp).unwrap();
        assert!(serialized.contains("\"code\":-32600"));
        assert!(serialized.contains("\"message\":\"Invalid Request\""));
    }

    // ── RpcError ─────────────────────────────────────────────────────────

    #[test]
    fn test_rpc_error_data_is_optional() {
        let err = RpcError {
            code: -32603,
            message: "Internal error".into(),
            data: None,
        };
        let serialized = serde_json::to_string(&err).unwrap();
        assert!(!serialized.contains("data"));
    }

    #[test]
    fn test_rpc_error_with_data() {
        let err = RpcError {
            code: -32602,
            message: "Invalid params".into(),
            data: Some(json!({"detail": "missing field"})),
        };
        let serialized = serde_json::to_string(&err).unwrap();
        assert!(serialized.contains("\"data\""));
    }

    // ── ToolContent ──────────────────────────────────────────────────────

    #[test]
    fn test_tool_content_text() {
        let c = ToolContent::text("hello");
        assert_eq!(c.kind, "text");
        assert_eq!(c.text, "hello");
    }

    #[test]
    fn test_tool_content_json_pretty() {
        let v = json!({"key": "value"});
        let c = ToolContent::json(&v);
        assert_eq!(c.kind, "text");
        assert!(c.text.contains("\"key\""));
        assert!(c.text.contains("\"value\""));
    }

    // ── Property helpers ─────────────────────────────────────────────────

    #[test]
    fn test_prop_int() {
        let p = prop_int("Project ID");
        assert_eq!(p["type"], "integer");
        assert_eq!(p["description"], "Project ID");
    }

    #[test]
    fn test_prop_str() {
        let p = prop_str("Name");
        assert_eq!(p["type"], "string");
        assert_eq!(p["description"], "Name");
    }

    #[test]
    fn test_prop_bool() {
        let p = prop_bool("Enable flag");
        assert_eq!(p["type"], "boolean");
        assert_eq!(p["description"], "Enable flag");
    }

    #[test]
    fn test_prop_int_opt_is_nullable() {
        let p = prop_int_opt("Optional limit");
        assert!(p["type"].is_array());
        assert_eq!(p["description"], "Optional limit");
    }

    #[test]
    fn test_prop_str_opt_is_nullable() {
        let p = prop_str_opt("Optional name");
        assert!(p["type"].is_array());
        let types: Vec<&str> = p["type"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        assert!(types.contains(&"string"));
        assert!(types.contains(&"null"));
    }

    // ── McpRequest deserialization ───────────────────────────────────────

    #[test]
    fn test_mcp_request_deserialize() {
        let raw = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {"project_id": 5}
        });
        let req: McpRequest = serde_json::from_value(raw).unwrap();
        assert_eq!(req.method, "tools/list");
        assert!(req.params.is_some());
    }

    #[test]
    fn test_mcp_request_without_params() {
        let raw = json!({
            "jsonrpc": "2.0",
            "id": null,
            "method": "ping",
            "params": null
        });
        let req: McpRequest = serde_json::from_value(raw).unwrap();
        assert_eq!(req.method, "ping");
        assert!(req.params.is_none());
    }
}
