//! Organization handlers — Multi-Tenancy API

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use std::sync::Arc;

use crate::api::extractors::AuthUser;
use crate::api::state::AppState;
use crate::db::store::OrganizationManager;
use crate::models::{OrganizationCreate, OrganizationUpdate, OrganizationUserCreate};

/// GET /api/organizations — список всех организаций (только admin)
pub async fn get_organizations(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> impl IntoResponse {
    if !auth.admin {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Admin required"})),
        )
            .into_response();
    }
    match state.store.get_organizations().await {
        Ok(orgs) => Json(orgs).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/organizations — создать организацию (только admin)
pub async fn create_organization(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<OrganizationCreate>,
) -> impl IntoResponse {
    if !auth.admin {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Admin required"})),
        )
            .into_response();
    }
    match state.store.create_organization(payload).await {
        Ok(org) => (StatusCode::CREATED, Json(org)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/organizations/:id — получить организацию
pub async fn get_organization(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    // admin видит любую; member — только свою
    if !auth.admin {
        match state.store.get_user_organizations(auth.user_id).await {
            Ok(orgs) if orgs.iter().any(|o| o.id == id) => {}
            _ => {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({"error": "Access denied"})),
                )
                    .into_response();
            }
        }
    }
    match state.store.get_organization(id).await {
        Ok(org) => Json(org).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

/// PUT /api/organizations/:id — обновить организацию (только admin)
pub async fn update_organization(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<OrganizationUpdate>,
) -> impl IntoResponse {
    if !auth.admin {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Admin required"})),
        )
            .into_response();
    }
    match state.store.update_organization(id, payload).await {
        Ok(org) => Json(org).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// DELETE /api/organizations/:id — удалить организацию (только admin)
pub async fn delete_organization(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    if !auth.admin {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Admin required"})),
        )
            .into_response();
    }
    match state.store.delete_organization(id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/organizations/:id/users — пользователи организации
pub async fn get_organization_users(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    if !auth.admin {
        match state.store.get_user_organizations(auth.user_id).await {
            Ok(orgs) if orgs.iter().any(|o| o.id == id) => {}
            _ => {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({"error": "Access denied"})),
                )
                    .into_response();
            }
        }
    }
    match state.store.get_organization_users(id).await {
        Ok(users) => Json(users).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/organizations/:id/users — добавить пользователя в организацию (только admin)
pub async fn add_organization_user(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(mut payload): Json<OrganizationUserCreate>,
) -> impl IntoResponse {
    if !auth.admin {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Admin required"})),
        )
            .into_response();
    }
    payload.org_id = id;
    match state.store.add_user_to_organization(payload).await {
        Ok(ou) => (StatusCode::CREATED, Json(ou)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// DELETE /api/organizations/:id/users/:user_id — удалить пользователя из организации (только admin)
pub async fn remove_organization_user(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((id, user_id)): Path<(i32, i32)>,
) -> impl IntoResponse {
    if !auth.admin {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Admin required"})),
        )
            .into_response();
    }
    match state.store.remove_user_from_organization(id, user_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// PUT /api/organizations/:id/users/:user_id/role — изменить роль пользователя (только admin)
pub async fn update_organization_user_role(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((id, user_id)): Path<(i32, i32)>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    if !auth.admin {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Admin required"})),
        )
            .into_response();
    }
    let role = match body.get("role").and_then(|r| r.as_str()) {
        Some(r) => r.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "role is required"})),
            )
                .into_response();
        }
    };
    match state
        .store
        .update_user_organization_role(id, user_id, &role)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/user/organizations — организации текущего пользователя
pub async fn get_my_organizations(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> impl IntoResponse {
    match state.store.get_user_organizations(auth.user_id).await {
        Ok(orgs) => Json(orgs).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/organizations/:id/branding — получить branding организации (публичный, без auth)
/// Используется на login page для кастомизации UI под организацию
pub async fn get_organization_branding(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match state.store.get_organization(id).await {
        Ok(org) => {
            let branding = org.settings.unwrap_or_else(|| serde_json::json!({}));
            Json(serde_json::json!({
                "org_id": org.id,
                "org_name": org.name,
                "slug": org.slug,
                "logo_url": branding.get("logo_url").and_then(|v| v.as_str()),
                "primary_color": branding.get("primary_color").and_then(|v| v.as_str()).unwrap_or("#005057"),
                "app_name": branding.get("app_name").and_then(|v| v.as_str()).unwrap_or("Velum"),
                "favicon_url": branding.get("favicon_url").and_then(|v| v.as_str()),
                "custom_css": branding.get("custom_css").and_then(|v| v.as_str()),
            })).into_response()
        }
        Err(e) => (StatusCode::NOT_FOUND, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

/// PUT /api/organizations/:id/branding — обновить branding (только admin)
pub async fn update_organization_branding(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(branding): Json<serde_json::Value>,
) -> impl IntoResponse {
    if !auth.admin {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Admin required"})),
        )
            .into_response();
    }
    // Обновляем поле settings через OrganizationUpdate
    let payload = crate::models::OrganizationUpdate {
        settings: Some(branding),
        ..Default::default()
    };
    match state.store.update_organization(id, payload).await {
        Ok(org) => Json(org).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/organizations/:id/quota — проверить квоты организации
pub async fn check_organization_quota(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((id, quota_type)): Path<(i32, String)>,
) -> impl IntoResponse {
    if !auth.admin {
        match state.store.get_user_organizations(auth.user_id).await {
            Ok(orgs) if orgs.iter().any(|o| o.id == id) => {}
            _ => {
                return (
                    StatusCode::FORBIDDEN,
                    Json(json!({"error": "Access denied"})),
                )
                    .into_response();
            }
        }
    }
    match state.store.check_organization_quota(id, &quota_type).await {
        Ok(ok) => Json(json!({"quota_type": quota_type, "within_limit": ok})).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{OrganizationCreate, OrganizationUpdate, OrganizationUserCreate};
    use serde_json;

    // --- OrganizationCreate validation tests ---

    #[test]
    fn test_organization_create_valid() {
        let create = OrganizationCreate {
            name: "Acme Corp".to_string(),
            slug: Some("acme".to_string()),
            description: Some("Test org".to_string()),
            settings: None,
            quota_max_projects: Some(10),
            quota_max_users: Some(50),
            quota_max_tasks_per_month: Some(1000),
        };
        assert_eq!(create.name, "Acme Corp");
        assert_eq!(create.slug.as_deref(), Some("acme"));
        assert!(create.quota_max_projects.is_some());
    }

    #[test]
    fn test_organization_create_minimal() {
        let create = OrganizationCreate {
            name: "Minimal".to_string(),
            slug: None,
            description: None,
            settings: None,
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
        };
        assert_eq!(create.name, "Minimal");
        assert!(create.slug.is_none());
        assert!(create.quota_max_projects.is_none());
    }

    #[test]
    fn test_organization_create_with_settings_json() {
        let settings = serde_json::json!({
            "logo_url": "https://example.com/logo.png",
            "primary_color": "#FF5733",
            "app_name": "CustomApp"
        });
        let create = OrganizationCreate {
            name: "Branded Org".to_string(),
            slug: Some("branded".to_string()),
            description: None,
            settings: Some(settings.clone()),
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
        };
        assert!(create.settings.is_some());
        let s = create.settings.unwrap();
        assert_eq!(s["primary_color"].as_str(), Some("#FF5733"));
        assert_eq!(s["app_name"].as_str(), Some("CustomApp"));
    }

    #[test]
    fn test_organization_create_name_max_length() {
        let long_name = "a".repeat(255);
        let create = OrganizationCreate {
            name: long_name.clone(),
            slug: Some("long-org".to_string()),
            description: None,
            settings: None,
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
        };
        assert_eq!(create.name.len(), 255);
    }

    // --- OrganizationUpdate tests ---

    #[test]
    fn test_organization_update_default() {
        let update = OrganizationUpdate::default();
        assert!(update.name.is_none());
        assert!(update.description.is_none());
        assert!(update.settings.is_none());
        assert!(update.active.is_none());
    }

    #[test]
    fn test_organization_update_partial() {
        let update = OrganizationUpdate {
            name: Some("New Name".to_string()),
            description: Some("Updated description".to_string()),
            settings: None,
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
            active: Some(true),
        };
        assert_eq!(update.name.as_deref(), Some("New Name"));
        assert!(update.active.is_some());
        assert!(update.settings.is_none());
    }

    #[test]
    fn test_organization_update_skip_serialization_nulls() {
        let update = OrganizationUpdate {
            name: Some("Only Name".to_string()),
            description: None,
            settings: None,
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
            active: None,
        };
        let serialized = serde_json::to_string(&update).unwrap();
        assert!(serialized.contains("\"name\":\"Only Name\""));
        assert!(!serialized.contains("description"));
        assert!(!serialized.contains("settings"));
        assert!(!serialized.contains("active"));
    }

    // --- OrganizationUserCreate tests ---

    #[test]
    fn test_organization_user_create() {
        let create = OrganizationUserCreate {
            org_id: 1,
            user_id: 42,
            role: "admin".to_string(),
        };
        assert_eq!(create.org_id, 1);
        assert_eq!(create.user_id, 42);
        assert_eq!(create.role, "admin");
    }

    #[test]
    fn test_organization_user_role_variants() {
        let roles = vec!["owner", "admin", "member"];
        for role in roles {
            let create = OrganizationUserCreate {
                org_id: 1,
                user_id: 1,
                role: role.to_string(),
            };
            assert_eq!(create.role, role);
        }
    }

    // --- Admin check logic tests ---

    #[test]
    fn test_admin_check_forbidden_logic() {
        // Моделируем проверку admin: если admin=false, должен быть Forbidden
        let admin = false;
        let should_forbid = !admin;
        assert!(should_forbid);

        let admin = true;
        let should_forbid = !admin;
        assert!(!should_forbid);
    }

    #[test]
    fn test_admin_check_response_structure() {
        // Проверяем, что JSON ошибки имеет ожидаемую структуру
        let error_response = json!({"error": "Admin required"});
        assert!(error_response.get("error").is_some());
        assert_eq!(error_response["error"].as_str(), Some("Admin required"));
    }

    // --- JSON payload parsing tests ---

    #[test]
    fn test_parse_organization_create_from_json() {
        let json_str = r#"{
            "name": "Parsed Org",
            "slug": "parsed-org",
            "description": "Parsed description",
            "quota_max_projects": 20,
            "quota_max_users": 100
        }"#;
        let parsed: OrganizationCreate = serde_json::from_str(json_str).unwrap();
        assert_eq!(parsed.name, "Parsed Org");
        assert_eq!(parsed.slug.as_deref(), Some("parsed-org"));
        assert_eq!(parsed.quota_max_projects, Some(20));
        assert_eq!(parsed.quota_max_users, Some(100));
    }

    #[test]
    fn test_parse_organization_update_from_json() {
        let json_str = r#"{
            "name": "Updated Org",
            "active": false,
            "quota_max_projects": 50
        }"#;
        let parsed: OrganizationUpdate = serde_json::from_str(json_str).unwrap();
        assert_eq!(parsed.name.as_deref(), Some("Updated Org"));
        assert_eq!(parsed.active, Some(false));
        assert_eq!(parsed.quota_max_projects, Some(50));
        assert!(parsed.description.is_none());
    }

    #[test]
    fn test_parse_organization_user_create_from_json() {
        let json_str = r#"{
            "org_id": 5,
            "user_id": 10,
            "role": "member"
        }"#;
        let parsed: OrganizationUserCreate = serde_json::from_str(json_str).unwrap();
        assert_eq!(parsed.org_id, 5);
        assert_eq!(parsed.user_id, 10);
        assert_eq!(parsed.role, "member");
    }

    // --- Role update body parsing tests ---

    #[test]
    fn test_role_update_body_parsing() {
        let body = serde_json::json!({"role": "admin"});
        let role = body.get("role").and_then(|r| r.as_str());
        assert_eq!(role, Some("admin"));
    }

    #[test]
    fn test_role_update_body_missing_role() {
        let body = serde_json::json!({"something_else": "value"});
        let role = body.get("role").and_then(|r| r.as_str());
        assert!(role.is_none());
    }

    #[test]
    fn test_role_update_body_empty_string() {
        let body = serde_json::json!({"role": ""});
        let role = body.get("role").and_then(|r| r.as_str());
        assert_eq!(role, Some(""));
    }

    // --- Branding JSON structure tests ---

    #[test]
    fn test_branding_json_structure() {
        let branding = serde_json::json!({
            "logo_url": "https://example.com/logo.png",
            "primary_color": "#005057",
            "app_name": "Velum",
            "favicon_url": "https://example.com/favicon.ico",
            "custom_css": "body { color: red; }"
        });
        assert!(branding.get("logo_url").is_some());
        assert!(branding.get("primary_color").is_some());
        assert!(branding.get("app_name").is_some());
        assert_eq!(branding["primary_color"].as_str(), Some("#005057"));
    }

    #[test]
    fn test_branding_json_defaults() {
        let branding = serde_json::json!({});
        let primary_color = branding
            .get("primary_color")
            .and_then(|v| v.as_str())
            .unwrap_or("#005057");
        let app_name = branding
            .get("app_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Velum");
        assert_eq!(primary_color, "#005057");
        assert_eq!(app_name, "Velum");
    }

    #[test]
    fn test_branding_json_partial() {
        let branding = serde_json::json!({
            "logo_url": "https://example.com/logo.png"
        });
        assert!(branding.get("logo_url").is_some());
        assert!(branding.get("primary_color").is_none());
        assert!(branding.get("favicon_url").is_none());
    }

    #[test]
    fn test_organization_create_serialize() {
        let create = OrganizationCreate {
            name: "Serialize Test".to_string(),
            slug: Some("serialize-test".to_string()),
            description: None,
            settings: None,
            quota_max_projects: Some(15),
            quota_max_users: None,
            quota_max_tasks_per_month: None,
        };
        let serialized = serde_json::to_string(&create).unwrap();
        assert!(serialized.contains("\"name\":\"Serialize Test\""));
        assert!(serialized.contains("\"slug\":\"serialize-test\""));
        assert!(serialized.contains("\"quota_max_projects\":15"));
        assert!(serialized.contains("\"slug\":null") == false);
    }

    #[test]
    fn test_organization_user_create_serialize() {
        let create = OrganizationUserCreate {
            org_id: 7,
            user_id: 21,
            role: "owner".to_string(),
        };
        let serialized = serde_json::to_string(&create).unwrap();
        assert!(serialized.contains("\"org_id\":7"));
        assert!(serialized.contains("\"user_id\":21"));
        assert!(serialized.contains("\"role\":\"owner\""));
    }
}
