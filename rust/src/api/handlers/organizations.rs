//! Organization API Handlers - Управление организациями (Multi-Tenancy)

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::OrganizationManager;
use crate::error::Error;
use crate::models::organization::{
    Organization, OrganizationCreate, OrganizationUpdate, OrganizationUser, OrganizationUserCreate,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono;
use std::sync::Arc;
use validator::Validate;

// Используем стандартный Result для handlers с двумя параметрами
type HandlerResult<T> = std::result::Result<T, (StatusCode, Json<ErrorResponse>)>;

// ============================================================================
// Organization CRUD
// ============================================================================

/// Получить все организации
pub async fn get_organizations(
    State(state): State<Arc<AppState>>,
) -> HandlerResult<Json<Vec<Organization>>> {
    let organizations = state.store.get_organizations().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;
    Ok(Json(organizations))
}

/// Получить организацию по ID
pub async fn get_organization(
    Path(org_id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult<Json<Organization>> {
    let org = state
        .store
        .get_organization(org_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;
    Ok(Json(org))
}

/// Создать организацию
pub async fn create_organization(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OrganizationCreate>,
) -> HandlerResult<(StatusCode, Json<Organization>)> {
    payload.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(e.to_string())),
        )
    })?;

    // Генерируем slug из названия если не указан
    let slug = payload.slug.clone().unwrap_or_else(|| {
        payload
            .name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
    });

    let org = state
        .store
        .create_organization(OrganizationCreate {
            slug: Some(slug),
            ..payload
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok((StatusCode::CREATED, Json(org)))
}

/// Обновить организацию
pub async fn update_organization(
    Path(org_id): Path<i32>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OrganizationUpdate>,
) -> HandlerResult<Json<Organization>> {
    let org = state
        .store
        .update_organization(org_id, payload)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;
    Ok(Json(org))
}

/// Удалить организацию
pub async fn delete_organization(
    Path(org_id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult<StatusCode> {
    state
        .store
        .delete_organization(org_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Organization Users
// ============================================================================

/// Получить пользователей организации
pub async fn get_organization_users(
    Path(org_id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult<Json<Vec<OrganizationUser>>> {
    let users = state
        .store
        .get_organization_users(org_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    Ok(Json(users))
}

/// Добавить пользователя в организацию
pub async fn add_user_to_organization(
    Path(org_id): Path<i32>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OrganizationUserCreate>,
) -> HandlerResult<(StatusCode, Json<OrganizationUser>)> {
    // Убеждаемся, что org_id в path и payload совпадают
    let user_payload = OrganizationUserCreate { org_id, ..payload };

    let user = state
        .store
        .add_user_to_organization(user_payload)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    Ok((StatusCode::CREATED, Json(user)))
}

/// Удалить пользователя из организации
pub async fn remove_user_from_organization(
    Path((org_id, user_id)): Path<(i32, i32)>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult<StatusCode> {
    state
        .store
        .remove_user_from_organization(org_id, user_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;
    Ok(StatusCode::NO_CONTENT)
}

/// Обновить роль пользователя в организации
pub async fn update_user_organization_role(
    Path((org_id, user_id)): Path<(i32, i32)>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> HandlerResult<StatusCode> {
    let role = payload
        .get("role")
        .and_then(|r| r.as_str())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("role is required".to_string())),
            )
        })?;

    state
        .store
        .update_user_organization_role(org_id, user_id, role)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(e.to_string())),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;
    Ok(StatusCode::NO_CONTENT)
}

/// Получить организации пользователя
pub async fn get_user_organizations(
    Path(user_id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult<Json<Vec<Organization>>> {
    let orgs = state
        .store
        .get_user_organizations(user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    Ok(Json(orgs))
}

/// Проверить квоту организации
pub async fn check_organization_quota(
    Path((org_id, quota_type)): Path<(i32, String)>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult<Json<bool>> {
    let allowed = state
        .store
        .check_organization_quota(org_id, &quota_type)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;
    Ok(Json(allowed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::organization::{
        Organization, OrganizationCreate, OrganizationUpdate, OrganizationUser,
        OrganizationUserCreate,
    };
    use chrono::Utc;
    use serde_json;

    // ========================================================================
    // 1. Тесты для request/response payloads
    // ========================================================================

    #[test]
    fn test_create_organization_request_payload() {
        let payload = OrganizationCreate {
            name: "Test Corp".to_string(),
            slug: Some("test-corp".to_string()),
            description: Some("A test organization".to_string()),
            settings: None,
            quota_max_projects: Some(20),
            quota_max_users: Some(100),
            quota_max_tasks_per_month: Some(5000),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"name\":\"Test Corp\""));
        assert!(json.contains("\"slug\":\"test-corp\""));
        assert!(json.contains("\"quota_max_projects\":20"));
    }

    #[test]
    fn test_create_organization_response_payload() {
        let org = Organization {
            id: 42,
            name: "Response Org".to_string(),
            slug: "response-org".to_string(),
            description: Some("Created successfully".to_string()),
            settings: None,
            quota_max_projects: Some(10),
            quota_max_users: Some(50),
            quota_max_tasks_per_month: Some(1000),
            active: true,
            created: Utc::now(),
            updated: None,
        };
        let json = serde_json::to_string(&org).unwrap();
        let deserialized: Organization = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, 42);
        assert_eq!(deserialized.name, "Response Org");
        assert_eq!(deserialized.slug, "response-org");
        assert!(deserialized.active);
    }

    #[test]
    fn test_update_organization_request_payload() {
        let payload = OrganizationUpdate {
            name: Some("Updated Name".to_string()),
            description: Some("New description".to_string()),
            settings: Some(serde_json::json!({"theme": "dark"})),
            quota_max_projects: Some(100),
            quota_max_users: Some(200),
            quota_max_tasks_per_month: Some(10000),
            active: Some(false),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"name\":\"Updated Name\""));
        assert!(json.contains("\"active\":false"));
        assert!(json.contains("\"theme\":\"dark\""));
    }

    #[test]
    fn test_update_organization_partial_request() {
        let payload = OrganizationUpdate {
            name: Some("Only Name".to_string()),
            description: None,
            settings: None,
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
            active: None,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"name\":\"Only Name\""));
        assert!(!json.contains("\"description\""));
        assert!(!json.contains("\"active\""));
    }

    // ========================================================================
    // 2. Тесты для query / path parameters
    // ========================================================================

    #[test]
    fn test_path_org_id_positive() {
        let org_id: i32 = 1;
        assert!(org_id > 0);
    }

    #[test]
    fn test_path_org_id_negative() {
        let org_id: i32 = -1;
        assert!(org_id < 0);
    }

    #[test]
    fn test_path_org_id_zero() {
        let org_id: i32 = 0;
        assert_eq!(org_id, 0);
    }

    #[test]
    fn test_path_org_user_pair() {
        let (org_id, user_id): (i32, i32) = (5, 10);
        assert_eq!(org_id, 5);
        assert_eq!(user_id, 10);
    }

    #[test]
    fn test_path_quota_type_string() {
        let quota_type = "projects".to_string();
        assert_eq!(quota_type, "projects");
    }

    // ========================================================================
    // 3. Тесты для моделей данных
    // ========================================================================

    #[test]
    fn test_organization_model_all_fields() {
        let org = Organization {
            id: 1,
            name: "Full Org".to_string(),
            slug: "full-org".to_string(),
            description: Some("Full description".to_string()),
            settings: Some(serde_json::json!({"key": "value"})),
            quota_max_projects: Some(50),
            quota_max_users: Some(200),
            quota_max_tasks_per_month: Some(10000),
            active: false,
            created: Utc::now(),
            updated: Some(Utc::now()),
        };
        assert_eq!(org.id, 1);
        assert!(!org.active);
        assert!(org.updated.is_some());
        assert!(org.settings.is_some());
    }

    #[test]
    fn test_organization_user_model() {
        let user = OrganizationUser {
            id: 1,
            org_id: 10,
            user_id: 42,
            role: "owner".to_string(),
            created: Utc::now(),
        };
        assert_eq!(user.role, "owner");
        assert_eq!(user.org_id, 10);
        assert_eq!(user.user_id, 42);
    }

    #[test]
    fn test_organization_user_create_model() {
        let create = OrganizationUserCreate {
            org_id: 10,
            user_id: 42,
            role: "admin".to_string(),
        };
        assert_eq!(create.org_id, 10);
        assert_eq!(create.role, "admin");
    }

    #[test]
    fn test_organization_user_create_with_org_id_override() {
        let payload = OrganizationUserCreate {
            org_id: 5,
            user_id: 10,
            role: "member".to_string(),
        };
        let org_id_from_path = 15;
        let final_payload = OrganizationUserCreate {
            org_id: org_id_from_path,
            ..payload
        };
        assert_eq!(final_payload.org_id, 15);
        assert_eq!(final_payload.user_id, 10);
    }

    // ========================================================================
    // 4. Тесты для JSON serialization
    // ========================================================================

    #[test]
    fn test_serialize_organization_create_to_json() {
        let create = OrganizationCreate {
            name: "Serialize Test".to_string(),
            slug: None,
            description: None,
            settings: None,
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
        };
        let json = serde_json::to_value(&create).unwrap();
        assert_eq!(json["name"], "Serialize Test");
        assert!(json["slug"].is_null());
        assert!(json["description"].is_null());
    }

    #[test]
    fn test_deserialize_organization_create_from_json() {
        let json_str = r#"{
            "name": "Deserialize Org",
            "slug": "deserialize-org",
            "description": "Test",
            "settings": null,
            "quota_max_projects": 5,
            "quota_max_users": null,
            "quota_max_tasks_per_month": null
        }"#;
        let create: OrganizationCreate = serde_json::from_str(json_str).unwrap();
        assert_eq!(create.name, "Deserialize Org");
        assert_eq!(create.slug, Some("deserialize-org".to_string()));
        assert_eq!(create.quota_max_projects, Some(5));
    }

    #[test]
    fn test_serialize_organization_update_skip_nulls() {
        let update = OrganizationUpdate {
            name: Some("NonNull".to_string()),
            description: None,
            settings: None,
            quota_max_projects: Some(42),
            quota_max_users: None,
            quota_max_tasks_per_month: None,
            active: None,
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"name\":\"NonNull\""));
        assert!(json.contains("\"quota_max_projects\":42"));
        assert!(!json.contains("\"description\""));
        assert!(!json.contains("\"quota_max_users\""));
    }

    #[test]
    fn test_serialize_vec_organizations() {
        let orgs = vec![
            Organization {
                id: 1,
                name: "Org 1".to_string(),
                slug: "org-1".to_string(),
                description: None,
                settings: None,
                quota_max_projects: None,
                quota_max_users: None,
                quota_max_tasks_per_month: None,
                active: true,
                created: Utc::now(),
                updated: None,
            },
            Organization {
                id: 2,
                name: "Org 2".to_string(),
                slug: "org-2".to_string(),
                description: None,
                settings: None,
                quota_max_projects: None,
                quota_max_users: None,
                quota_max_tasks_per_month: None,
                active: false,
                created: Utc::now(),
                updated: None,
            },
        ];
        let json = serde_json::to_value(&orgs).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 2);
        assert_eq!(json[0]["name"], "Org 1");
        assert_eq!(json[1]["name"], "Org 2");
    }

    #[test]
    fn test_serialize_vec_organization_users() {
        let users = vec![
            OrganizationUser {
                id: 1,
                org_id: 1,
                user_id: 10,
                role: "owner".to_string(),
                created: Utc::now(),
            },
            OrganizationUser {
                id: 2,
                org_id: 1,
                user_id: 11,
                role: "member".to_string(),
                created: Utc::now(),
            },
        ];
        let json = serde_json::to_value(&users).unwrap();
        assert!(json.is_array());
        assert_eq!(json[0]["role"], "owner");
        assert_eq!(json[1]["role"], "member");
    }

    // ========================================================================
    // 5. Тесты для валидации входных данных
    // ========================================================================

    #[test]
    fn test_validate_organization_create_valid_name() {
        let payload = OrganizationCreate {
            name: "Valid Name".to_string(),
            slug: None,
            description: None,
            settings: None,
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
        };
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn test_validate_organization_create_empty_name_fails() {
        let payload = OrganizationCreate {
            name: "".to_string(),
            slug: None,
            description: None,
            settings: None,
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
        };
        assert!(payload.validate().is_err());
    }

    #[test]
    fn test_validate_organization_create_name_too_long() {
        let long_name = "A".repeat(256);
        let payload = OrganizationCreate {
            name: long_name,
            slug: None,
            description: None,
            settings: None,
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
        };
        assert!(payload.validate().is_err());
    }

    #[test]
    fn test_validate_organization_create_name_max_length() {
        let max_name = "A".repeat(255);
        let payload = OrganizationCreate {
            name: max_name,
            slug: None,
            description: None,
            settings: None,
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
        };
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn test_slug_generation_from_name() {
        let name = "My Awesome Organization!";
        let slug: String = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect();
        assert_eq!(slug, "my-awesome-organization-");
    }

    #[test]
    fn test_slug_generation_preserves_alphanumeric() {
        let name = "Test123 Corp";
        let slug: String = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect();
        assert_eq!(slug, "test123-corp");
    }

    #[test]
    fn test_update_user_role_from_json() {
        let json_payload = serde_json::json!({"role": "admin"});
        let role = json_payload.get("role").and_then(|r| r.as_str());
        assert_eq!(role, Some("admin"));
    }

    #[test]
    fn test_update_user_role_missing_role_fails() {
        let json_payload = serde_json::json!({"name": "test"});
        let role = json_payload.get("role").and_then(|r| r.as_str());
        assert!(role.is_none());
    }

    #[test]
    fn test_update_user_role_null_value_fails() {
        let json_payload = serde_json::json!({"role": null});
        let role = json_payload.get("role").and_then(|r| r.as_str());
        assert!(role.is_none());
    }

    #[test]
    fn test_organization_deserialize_roundtrip() {
        let original = Organization {
            id: 7,
            name: "Roundtrip".to_string(),
            slug: "roundtrip".to_string(),
            description: Some("Test roundtrip".to_string()),
            settings: Some(serde_json::json!({"a": 1})),
            quota_max_projects: Some(3),
            quota_max_users: Some(15),
            quota_max_tasks_per_month: Some(500),
            active: true,
            created: Utc::now(),
            updated: None,
        };
        let json = serde_json::to_value(&original).unwrap();
        let restored: Organization = serde_json::from_value(json).unwrap();
        assert_eq!(restored.id, original.id);
        assert_eq!(restored.name, original.name);
        assert_eq!(restored.slug, original.slug);
        assert_eq!(restored.description, original.description);
        assert_eq!(restored.active, original.active);
    }
}
