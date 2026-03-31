//! Organization API Handlers - Управление организациями (Multi-Tenancy)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use validator::Validate;
use chrono;
use crate::api::state::AppState;
use crate::db::store::OrganizationManager;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;
use crate::models::organization::{Organization, OrganizationCreate, OrganizationUpdate, OrganizationUser, OrganizationUserCreate};

// Используем стандартный Result для handlers с двумя параметрами
type HandlerResult<T> = std::result::Result<T, (StatusCode, Json<ErrorResponse>)>;

// ============================================================================
// Organization CRUD
// ============================================================================

/// Получить все организации
pub async fn get_organizations(
    State(state): State<Arc<AppState>>,
) -> HandlerResult<Json<Vec<Organization>>> {
    let organizations = state.store.get_organizations().await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;
    Ok(Json(organizations))
}

/// Получить организацию по ID
pub async fn get_organization(
    Path(org_id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult<Json<Organization>> {
    let org = state.store.get_organization(org_id).await
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
    payload.validate().map_err(|e| (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse::new(e.to_string()))
    ))?;
    
    // Генерируем slug из названия если не указан
    let slug = payload.slug.clone().unwrap_or_else(|| {
        payload.name.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
    });

    let org = state.store.create_organization(OrganizationCreate {
        slug: Some(slug),
        ..payload
    }).await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new(e.to_string()))
    ))?;
    
    Ok((StatusCode::CREATED, Json(org)))
}

/// Обновить организацию
pub async fn update_organization(
    Path(org_id): Path<i32>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OrganizationUpdate>,
) -> HandlerResult<Json<Organization>> {
    let org = state.store.update_organization(org_id, payload).await
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
    state.store.delete_organization(org_id).await
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
    let users = state.store.get_organization_users(org_id).await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;
    Ok(Json(users))
}

/// Добавить пользователя в организацию
pub async fn add_user_to_organization(
    Path(org_id): Path<i32>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OrganizationUserCreate>,
) -> HandlerResult<(StatusCode, Json<OrganizationUser>)> {
    // Убеждаемся, что org_id в path и payload совпадают
    let user_payload = OrganizationUserCreate {
        org_id,
        ..payload
    };
    
    let user = state.store.add_user_to_organization(user_payload).await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;
    Ok((StatusCode::CREATED, Json(user)))
}

/// Удалить пользователя из организации
pub async fn remove_user_from_organization(
    Path((org_id, user_id)): Path<(i32, i32)>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult<StatusCode> {
    state.store.remove_user_from_organization(org_id, user_id).await
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
    let role = payload.get("role")
        .and_then(|r| r.as_str())
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("role is required".to_string()))
        ))?;
    
    state.store.update_user_organization_role(org_id, user_id, role).await
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
    let orgs = state.store.get_user_organizations(user_id).await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;
    Ok(Json(orgs))
}

/// Проверить квоту организации
pub async fn check_organization_quota(
    Path((org_id, quota_type)): Path<(i32, String)>,
    State(state): State<Arc<AppState>>,
) -> HandlerResult<Json<bool>> {
    let allowed = state.store.check_organization_quota(org_id, &quota_type).await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;
    Ok(Json(allowed))
}
