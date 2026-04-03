//! Velum MCP (Model Context Protocol) — embedded server
//!
//! Exposes the following routes (added to the main Axum router):
//!
//!   POST /mcp             — JSON-RPC 2.0 MCP endpoint (Claude connects here)
//!   GET  /api/mcp/settings — Read MCP settings
//!   PUT  /api/mcp/settings — Update MCP settings
//!   GET  /api/mcp/tools   — List available tools (for the settings UI)
//!
//! Authentication: same JWT/Bearer token used by the rest of the API.
//! Claude Desktop/Code config example:
//! ```json
//! { "mcpServers": { "velum": { "url": "http://localhost:3000/mcp",
//!   "headers": { "Authorization": "Bearer ` + "<token>" + `" } } } }
//! ```

pub mod handler;
pub mod protocol;
pub mod tools;

pub use handler::{get_mcp_settings, get_mcp_tools, mcp_endpoint, update_mcp_settings};
