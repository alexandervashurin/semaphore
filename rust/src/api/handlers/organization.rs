//! Organization handlers — Multi-Tenancy API

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
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
                    .into_response()
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
                    .into_response()
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
                .into_response()
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
                    .into_response()
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
