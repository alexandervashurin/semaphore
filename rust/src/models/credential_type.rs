use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// A field in a custom credential type input schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialField {
    pub id: String,         // e.g. "username", "password", "token"
    pub label: String,      // Display label for user
    pub field_type: String, // "string" | "password" | "boolean" | "integer"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_field_serialization() {
        let field = CredentialField {
            id: "username".to_string(),
            label: "Username".to_string(),
            field_type: "string".to_string(),
            required: true,
            default_value: None,
            help_text: Some("Enter your username".to_string()),
        };
        let json = serde_json::to_string(&field).unwrap();
        assert!(json.contains("\"id\":\"username\""));
        assert!(json.contains("\"required\":true"));
        assert!(json.contains("\"help_text\":\"Enter your username\""));
    }

    #[test]
    fn test_credential_field_skip_nulls() {
        let field = CredentialField {
            id: "token".to_string(),
            label: "Token".to_string(),
            field_type: "password".to_string(),
            required: false,
            default_value: None,
            help_text: None,
        };
        let json = serde_json::to_string(&field).unwrap();
        assert!(!json.contains("default_value"));
        assert!(!json.contains("help_text"));
    }

    #[test]
    fn test_credential_injector_serialization() {
        let injector = CredentialInjector {
            injector_type: "env".to_string(),
            key: "API_TOKEN".to_string(),
            value_template: "{{ token }}".to_string(),
        };
        let json = serde_json::to_string(&injector).unwrap();
        assert!(json.contains("\"injector_type\":\"env\""));
        assert!(json.contains("\"key\":\"API_TOKEN\""));
    }

    #[test]
    fn test_credential_type_create_serialization() {
        let create = CredentialTypeCreate {
            name: "API Credentials".to_string(),
            description: Some("Custom API credentials".to_string()),
            input_schema: serde_json::json!([]),
            injectors: serde_json::json!([]),
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"name\":\"API Credentials\""));
        assert!(json.contains("\"description\":\"Custom API credentials\""));
    }

    #[test]
    fn test_credential_instance_serialization() {
        let instance = CredentialInstance {
            id: 1,
            project_id: 10,
            credential_type_id: 5,
            name: "My API Key".to_string(),
            values: r#"{"token":"encrypted_value"}"#.to_string(),
            description: Some("API key for production".to_string()),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&instance).unwrap();
        assert!(json.contains("\"name\":\"My API Key\""));
        assert!(json.contains("\"project_id\":10"));
    }

    #[test]
    fn test_credential_instance_create_serialization() {
        let create = CredentialInstanceCreate {
            credential_type_id: 5,
            name: "New Credential".to_string(),
            values: serde_json::json!({"token": "secret"}),
            description: None,
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"credential_type_id\":5"));
        assert!(json.contains("\"name\":\"New Credential\""));
    }
}
