//! Projects API - Custom Roles Handler (B-BE-09/10/11)
//!
//! Обработчики запросов для управления кастомными ролями

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::models::Role;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;

/// Встроенные роли проекта (всегда доступны)
fn builtin_roles(project_id: i32) -> Vec<Role> {
    vec![
        Role {
            id: -1,
            project_id,
            slug: "owner".to_string(),
            name: "Owner".to_string(),
            description: Some("Full project control".to_string()),
            permissions: Some(0x7FFFFFFF),
        },
        Role {
            id: -2,
            project_id,
            slug: "manager".to_string(),
            name: "Manager".to_string(),
            description: Some("Manage project resources".to_string()),
            permissions: Some(0x0FFFFFFF),
        },
        Role {
            id: -3,
            project_id,
            slug: "task_runner".to_string(),
            name: "Task Runner".to_string(),
            description: Some("Run tasks".to_string()),
            permissions: Some(0x00000001),
        },
        Role {
            id: -4,
            project_id,
            slug: "guest".to_string(),
            name: "Guest".to_string(),
            description: Some("View only".to_string()),
            permissions: Some(0),
        },
    ]
}

/// Получить все роли проекта (включая built-in)
///
/// GET /api/project/{project_id}/roles/all
pub async fn get_all_roles(
    State(_state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> Result<Json<Vec<Role>>, (StatusCode, Json<ErrorResponse>)> {
    let roles = builtin_roles(project_id);
    Ok(Json(roles))
}

/// Получить кастомные роли проекта
///
/// GET /api/project/{project_id}/roles
pub async fn get_roles(
    State(_state): State<Arc<AppState>>,
    Path(_project_id): Path<i32>,
) -> Result<Json<Vec<Role>>, (StatusCode, Json<ErrorResponse>)> {
    // Временная заглушка - возвращаем пустой список
    // TODO: Реализовать получение ролей из БД
    Ok(Json(vec![]))
}

/// Создать роль
///
/// POST /api/project/{project_id}/roles
pub async fn create_role(
    State(_state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<RoleCreatePayload>,
) -> Result<(StatusCode, Json<Role>), (StatusCode, Json<ErrorResponse>)> {
    // Временная заглушка - возвращаем mock роль
    // TODO: Реализовать создание роли в БД
    let role = Role {
        id: 0,
        project_id,
        slug: payload.slug,
        name: payload.name,
        description: payload.description,
        permissions: payload.permissions,
    };

    Ok((StatusCode::CREATED, Json(role)))
}

/// Получить роль по ID
///
/// GET /api/project/{project_id}/roles/{role_id}
pub async fn get_role(
    State(_state): State<Arc<AppState>>,
    Path((_project_id, role_id)): Path<(i32, i32)>,
) -> Result<Json<Role>, (StatusCode, Json<ErrorResponse>)> {
    // Временная заглушка - возвращаем ошибку 404
    // TODO: Реализовать получение роли из БД
    Err(Error::NotFound(format!("Role {} not found", role_id)))
        .map_err(|e| (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new(e.to_string()))
        ))
}

/// Обновить роль
///
/// PUT /api/project/{project_id}/roles/{role_id}
pub async fn update_role(
    State(_state): State<Arc<AppState>>,
    Path((_project_id, _role_id)): Path<(i32, i32)>,
    Json(_payload): Json<RoleUpdatePayload>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Временная заглушка - возвращаем OK
    // TODO: Реализовать обновление роли в БД
    Ok(StatusCode::OK)
}

/// Удалить роль
///
/// DELETE /api/project/{project_id}/roles/{role_id}
pub async fn delete_role(
    State(_state): State<Arc<AppState>>,
    Path((_project_id, _role_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Временная заглушка - возвращаем OK
    // TODO: Реализовать удаление роли из БД
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Types
// ============================================================================

/// Payload для создания роли
#[derive(Debug, Serialize, Deserialize)]
pub struct RoleCreatePayload {
    pub slug: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<i32>,
}

/// Payload для обновления роли
#[derive(Debug, Serialize, Deserialize)]
pub struct RoleUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<i32>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::role::RolePermissions;

    #[test]
    fn test_role_permissions_bitmask() {
        let perms = RolePermissions::default();
        assert_eq!(perms.to_bitmask(), 1); // Только run_tasks

        let admin = RolePermissions::admin();
        assert_eq!(admin.to_bitmask(), 0b1111_1111); // Все права
    }

    #[test]
    fn test_role_permissions_from_bitmask() {
        let perms = RolePermissions::from_bitmask(0b0000_0101);
        assert!(perms.run_tasks);
        assert!(!perms.update_resources);
        assert!(perms.manage_project);
        assert!(!perms.manage_users);
    }

    #[test]
    fn test_role_create_payload_deserialize() {
        let json = r#"{
            "slug": "developer",
            "name": "Developer",
            "description": "Can run tasks and update resources",
            "permissions": 3
        }"#;
        let payload: RoleCreatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.slug, "developer");
        assert_eq!(payload.name, "Developer");
        assert_eq!(payload.permissions, Some(3));
    }

    #[test]
    fn test_role_update_payload_deserialize() {
        let json = r#"{"name": "Updated", "permissions": 7}"#;
        let payload: RoleUpdatePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, Some("Updated".to_string()));
        assert_eq!(payload.permissions, Some(7));
    }
}
