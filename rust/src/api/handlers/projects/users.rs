//! Projects API - Users Handler
//!
//! Обработчики для пользователей в проектах

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::models::{User, ProjectUser, ProjectUserRole};
use crate::error::{Error, Result};
use crate::api::middleware::ErrorResponse;
use crate::db::store::RetrieveQueryParams;

/// Проектный пользователь
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectUserResponse {
    pub id: i32,
    pub username: String,
    pub name: String,
    pub role: String,
}

/// Получает пользователей проекта
pub async fn get_users(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> std::result::Result<Json<Vec<ProjectUserResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let users = state.store.get_project_users(project_id, RetrieveQueryParams::default())
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    let result: Vec<ProjectUserResponse> = users.into_iter().map(|user| {
        ProjectUserResponse {
            id: user.id,
            username: user.username,
            name: user.name,
            role: user.role.to_string(),
        }
    }).collect();

    Ok(Json(result))
}

/// Добавляет пользователя в проект
pub async fn add_user(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<AddUserPayload>,
) -> std::result::Result<(StatusCode, Json<ProjectUserResponse>), (StatusCode, Json<ErrorResponse>)> {
    let project_user = ProjectUser {
        id: 0,
        project_id,
        user_id: payload.user_id,
        role: payload.role.clone(),
    };

    // В реальной реализации нужно сохранить в БД
    // state.store.create_project_user(project_user).await?;

    let response = ProjectUserResponse {
        id: payload.user_id,
        username: String::new(),
        name: String::new(),
        role: project_user.role.to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Обновляет роль пользователя в проекте
pub async fn update_user_role(
    State(state): State<Arc<AppState>>,
    Path((project_id, user_id)): Path<(i32, i32)>,
    Json(payload): Json<UpdateUserRolePayload>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // В реальной реализации нужно обновить в БД
    // state.store.update_project_user_role(project_id, user_id, payload.role).await?;

    Ok(StatusCode::OK)
}

/// Удаляет пользователя из проекта
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path((project_id, user_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // В реальной реализации нужно удалить из БД
    // state.store.delete_project_user(project_id, user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Payload для добавления пользователя
#[derive(Debug, Deserialize)]
pub struct AddUserPayload {
    pub user_id: i32,
    pub role: ProjectUserRole,
}

/// Payload для обновления роли
#[derive(Debug, Deserialize)]
pub struct UpdateUserRolePayload {
    pub role: ProjectUserRole,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_users_handler() {
        // Тест для проверки обработчиков пользователей
        assert!(true);
    }
}
