//! Handlers для Custom Credential Types API

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::CredentialTypeManager;
use crate::models::credential_type::{
    CredentialInstanceCreate, CredentialTypeCreate, CredentialTypeUpdate,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

/// GET /api/credential-types
pub async fn list_credential_types(
    State(state): State<Arc<AppState>>,
) -> Result<
    Json<Vec<crate::models::credential_type::CredentialType>>,
    (StatusCode, Json<ErrorResponse>),
> {
    let items = state.store.get_credential_types().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;
    Ok(Json(items))
}

/// POST /api/credential-types
pub async fn create_credential_type(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CredentialTypeCreate>,
) -> Result<
    (
        StatusCode,
        Json<crate::models::credential_type::CredentialType>,
    ),
    (StatusCode, Json<ErrorResponse>),
> {
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "Credential type name is required".to_string(),
            )),
        ));
    }
    let item = state
        .store
        .create_credential_type(payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    Ok((StatusCode::CREATED, Json(item)))
}

/// GET /api/credential-types/:id
pub async fn get_credential_type(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<crate::models::credential_type::CredentialType>, (StatusCode, Json<ErrorResponse>)>
{
    let item = state.store.get_credential_type(id).await.map_err(|e| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;
    Ok(Json(item))
}

/// PUT /api/credential-types/:id
pub async fn update_credential_type(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<CredentialTypeUpdate>,
) -> Result<Json<crate::models::credential_type::CredentialType>, (StatusCode, Json<ErrorResponse>)>
{
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "Credential type name is required".to_string(),
            )),
        ));
    }
    let item = state
        .store
        .update_credential_type(id, payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    Ok(Json(item))
}

/// DELETE /api/credential-types/:id
pub async fn delete_credential_type(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_credential_type(id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/project/:project_id/credentials
pub async fn list_credential_instances(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<
    Json<Vec<crate::models::credential_type::CredentialInstance>>,
    (StatusCode, Json<ErrorResponse>),
> {
    let items = state
        .store
        .get_credential_instances(project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    Ok(Json(items))
}

/// POST /api/project/:project_id/credentials
pub async fn create_credential_instance(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<CredentialInstanceCreate>,
) -> Result<
    (
        StatusCode,
        Json<crate::models::credential_type::CredentialInstance>,
    ),
    (StatusCode, Json<ErrorResponse>),
> {
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "Credential name is required".to_string(),
            )),
        ));
    }
    let item = state
        .store
        .create_credential_instance(project_id, payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    Ok((StatusCode::CREATED, Json(item)))
}

/// DELETE /api/project/:project_id/credentials/:id
pub async fn delete_credential_instance(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .delete_credential_instance(id, project_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::credential_type::{
        CredentialField, CredentialInjector, CredentialInstance, CredentialInstanceCreate,
        CredentialType, CredentialTypeCreate, CredentialTypeUpdate,
    };
    use chrono::Utc;

    // ===== Тесты для CredentialTypeCreate =====

    #[test]
    fn test_credential_type_create_valid() {
        let payload = CredentialTypeCreate {
            name: "API Credentials".to_string(),
            description: Some("Custom API credentials".to_string()),
            input_schema: serde_json::json!([
                {
                    "id": "api_key",
                    "label": "API Key",
                    "field_type": "password",
                    "required": true
                }
            ]),
            injectors: serde_json::json!([
                {
                    "injector_type": "env",
                    "key": "API_KEY",
                    "value_template": "{{ api_key }}"
                }
            ]),
        };
        assert_eq!(payload.name, "API Credentials");
        assert!(payload.description.is_some());
        assert!(payload.input_schema.is_array());
        assert!(payload.injectors.is_array());
    }

    #[test]
    fn test_credential_type_create_minimal() {
        let payload = CredentialTypeCreate {
            name: "Simple Auth".to_string(),
            description: None,
            input_schema: serde_json::json!([]),
            injectors: serde_json::json!([]),
        };
        assert_eq!(payload.name, "Simple Auth");
        assert!(payload.description.is_none());
    }

    #[test]
    fn test_credential_type_create_empty_name_validation() {
        let payload = CredentialTypeCreate {
            name: "   ".to_string(),
            description: None,
            input_schema: serde_json::json!([]),
            injectors: serde_json::json!([]),
        };
        // Проверяем, что name состоит только из пробелов
        assert!(payload.name.trim().is_empty());
    }

    #[test]
    fn test_credential_type_create_json_serialization() {
        let payload = CredentialTypeCreate {
            name: "Database Credentials".to_string(),
            description: Some("For PostgreSQL".to_string()),
            input_schema: serde_json::json!([{"id": "host", "label": "Host", "field_type": "string", "required": true}]),
            injectors: serde_json::json!([{"injector_type": "env", "key": "DB_HOST", "value_template": "{{ host }}"}]),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"name\":\"Database Credentials\""));
        assert!(json.contains("\"description\":\"For PostgreSQL\""));
    }

    #[test]
    fn test_credential_type_create_json_deserialization() {
        let json_str = r#"{
            "name": "Test Type",
            "description": null,
            "input_schema": [],
            "injectors": []
        }"#;
        let payload: CredentialTypeCreate = serde_json::from_str(json_str).unwrap();
        assert_eq!(payload.name, "Test Type");
        assert!(payload.description.is_none());
    }

    // ===== Тесты для CredentialTypeUpdate =====

    #[test]
    fn test_credential_type_update_valid() {
        let payload = CredentialTypeUpdate {
            name: "Updated Name".to_string(),
            description: Some("Updated description".to_string()),
            input_schema: serde_json::json!([
                {"id": "token", "label": "Token", "field_type": "password", "required": true}
            ]),
            injectors: serde_json::json!([
                {"injector_type": "file", "key": "/tmp/cred", "value_template": "{{ token }}"}
            ]),
        };
        assert_eq!(payload.name, "Updated Name");
        assert_eq!(payload.description.unwrap(), "Updated description");
    }

    #[test]
    fn test_credential_type_update_empty_name_validation() {
        let payload = CredentialTypeUpdate {
            name: "".to_string(),
            description: None,
            input_schema: serde_json::json!([]),
            injectors: serde_json::json!([]),
        };
        assert!(payload.name.trim().is_empty());
    }

    #[test]
    fn test_credential_type_update_json_serialization() {
        let payload = CredentialTypeUpdate {
            name: "Modified Type".to_string(),
            description: None,
            input_schema: serde_json::json!([]),
            injectors: serde_json::json!([]),
        };
        let json = serde_json::to_string(&payload).unwrap();
        let deserialized: CredentialTypeUpdate = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Modified Type");
    }

    // ===== Тесты для CredentialInstanceCreate =====

    #[test]
    fn test_credential_instance_create_valid() {
        let payload = CredentialInstanceCreate {
            credential_type_id: 1,
            name: "My API Key".to_string(),
            values: serde_json::json!({
                "api_key": "secret123",
                "api_secret": "super_secret"
            }),
            description: Some("Production API credentials".to_string()),
        };
        assert_eq!(payload.credential_type_id, 1);
        assert_eq!(payload.name, "My API Key");
        assert!(payload.values.is_object());
        assert!(payload.description.is_some());
    }

    #[test]
    fn test_credential_instance_create_minimal() {
        let payload = CredentialInstanceCreate {
            credential_type_id: 5,
            name: "Test Credential".to_string(),
            values: serde_json::json!({}),
            description: None,
        };
        assert_eq!(payload.credential_type_id, 5);
        assert!(payload.description.is_none());
    }

    #[test]
    fn test_credential_instance_create_empty_name_validation() {
        let payload = CredentialInstanceCreate {
            credential_type_id: 1,
            name: "  ".to_string(),
            values: serde_json::json!({}),
            description: None,
        };
        assert!(payload.name.trim().is_empty());
    }

    #[test]
    fn test_credential_instance_create_json_serialization() {
        let payload = CredentialInstanceCreate {
            credential_type_id: 3,
            name: "SSH Key".to_string(),
            values: serde_json::json!({"private_key": "encrypted_data"}),
            description: None,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"credential_type_id\":3"));
        assert!(json.contains("\"name\":\"SSH Key\""));
    }

    // ===== Тесты для CredentialType модели =====

    #[test]
    fn test_credential_type_model() {
        let now = Utc::now();
        let credential_type = CredentialType {
            id: 1,
            name: "OAuth2 Credentials".to_string(),
            description: Some("OAuth2 client credentials".to_string()),
            input_schema: serde_json::json!([
                {"id": "client_id", "label": "Client ID", "field_type": "string", "required": true},
                {"id": "client_secret", "label": "Client Secret", "field_type": "password", "required": true}
            ]).to_string(),
            injectors: serde_json::json!([
                {"injector_type": "env", "key": "CLIENT_ID", "value_template": "{{ client_id }}"}
            ]).to_string(),
            created: now,
            updated: now,
        };
        assert_eq!(credential_type.id, 1);
        assert_eq!(credential_type.name, "OAuth2 Credentials");
        assert!(credential_type.input_schema.contains("client_id"));
        assert!(credential_type.injectors.contains("CLIENT_ID"));
    }

    #[test]
    fn test_credential_type_json_serialization() {
        let now = Utc::now();
        let credential_type = CredentialType {
            id: 42,
            name: "Test".to_string(),
            description: None,
            input_schema: "[]".to_string(),
            injectors: "[]".to_string(),
            created: now,
            updated: now,
        };
        let json = serde_json::to_string(&credential_type).unwrap();
        assert!(json.contains("\"id\":42"));
        assert!(json.contains("\"name\":\"Test\""));
        assert!(!json.contains("null"));
    }

    // ===== Тесты для CredentialInstance модели =====

    #[test]
    fn test_credential_instance_model() {
        let now = Utc::now();
        let instance = CredentialInstance {
            id: 10,
            project_id: 5,
            credential_type_id: 2,
            name: "Production DB".to_string(),
            values: r#"{"username":"admin","password":"encrypted"}"#.to_string(),
            description: Some("Production database credentials".to_string()),
            created: now,
        };
        assert_eq!(instance.id, 10);
        assert_eq!(instance.project_id, 5);
        assert_eq!(instance.credential_type_id, 2);
        assert!(instance.values.contains("username"));
    }

    #[test]
    fn test_credential_instance_skip_none_description() {
        let now = Utc::now();
        let instance = CredentialInstance {
            id: 1,
            project_id: 1,
            credential_type_id: 1,
            name: "Test".to_string(),
            values: "{}".to_string(),
            description: None,
            created: now,
        };
        let json = serde_json::to_string(&instance).unwrap();
        // description с None должен быть пропущен при сериализации
        assert!(!json.contains("\"description\":null"));
    }

    // ===== Тесты для CredentialField =====

    #[test]
    fn test_credential_field_all_types() {
        let field_types = vec!["string", "password", "boolean", "integer"];
        for ft in field_types {
            let field = CredentialField {
                id: format!("field_{}", ft),
                label: format!("Field {}", ft),
                field_type: ft.to_string(),
                required: true,
                default_value: None,
                help_text: None,
            };
            let json = serde_json::to_string(&field).unwrap();
            assert!(json.contains(&format!("\"field_type\":\"{}\"", ft)));
        }
    }

    #[test]
    fn test_credential_field_with_defaults() {
        let field = CredentialField {
            id: "port".to_string(),
            label: "Port".to_string(),
            field_type: "integer".to_string(),
            required: false,
            default_value: Some("5432".to_string()),
            help_text: Some("Database port".to_string()),
        };
        let json = serde_json::to_string(&field).unwrap();
        assert!(json.contains("\"default_value\":\"5432\""));
        assert!(json.contains("\"help_text\":\"Database port\""));
    }

    // ===== Тесты для CredentialInjector =====

    #[test]
    fn test_credential_injector_types() {
        let injector_types = vec!["env", "file", "extra_vars"];
        for it in injector_types {
            let injector = CredentialInjector {
                injector_type: it.to_string(),
                key: format!("KEY_{}", it.to_uppercase()),
                value_template: "{{ value }}".to_string(),
            };
            let json = serde_json::to_string(&injector).unwrap();
            assert!(json.contains(&format!("\"injector_type\":\"{}\"", it)));
        }
    }

    #[test]
    fn test_credential_injector_file_type() {
        let injector = CredentialInjector {
            injector_type: "file".to_string(),
            key: "/tmp/credentials/{{ id }}.json".to_string(),
            value_template: "{{ data }}".to_string(),
        };
        assert_eq!(injector.injector_type, "file");
        assert!(injector.key.contains("{{ id }}"));
    }
}
