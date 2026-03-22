use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// A field in a custom credential type input schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialField {
    pub id: String,           // e.g. "username", "password", "token"
    pub label: String,        // Display label for user
    pub field_type: String,   // "string" | "password" | "boolean" | "integer"
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_text: Option<String>,
}

/// An injector definition (how to inject credential into tasks)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialInjector {
    /// Type: "env" | "file" | "extra_vars"
    pub injector_type: String,
    /// For "env": environment variable name, e.g. "MY_TOKEN"
    /// For "file": file path template, e.g. "/tmp/cred_{{ id }}"
    pub key: String,
    /// Template using field IDs: "{{ username }}:{{ password }}"
    pub value_template: String,
}

/// Custom credential type definition
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CredentialType {
    pub id: i32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON array of CredentialField
    pub input_schema: String,
    /// JSON array of CredentialInjector
    pub injectors: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialTypeCreate {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
    pub injectors: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialTypeUpdate {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
    pub injectors: serde_json::Value,
}

/// A credential instance: stores values for a specific credential type
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CredentialInstance {
    pub id: i32,
    pub project_id: i32,
    pub credential_type_id: i32,
    pub name: String,
    /// JSON object with field_id -> encrypted value pairs
    pub values: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialInstanceCreate {
    pub credential_type_id: i32,
    pub name: String,
    pub values: serde_json::Value,
    pub description: Option<String>,
}
