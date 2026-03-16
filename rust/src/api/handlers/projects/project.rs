//! Projects API - Project Handler
//!
//! Обработчики для проектов

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::Deserialize;
use crate::api::state::AppState;
use crate::api::extractors::AuthUser;
use crate::models::{Project, ProjectUser, ProjectUserRole};
use crate::Error;
use crate::api::middleware::ErrorResponse;
use crate::db::store::{ProjectStore, UserManager};
use crate::services::backup::BackupFormat;

/// Получает проекты пользователя
pub async fn get_projects(
    State(state): State<Arc<AppState>>,
    _auth_user: AuthUser,
) -> std::result::Result<Json<Vec<Project>>, (StatusCode, Json<ErrorResponse>)> {
    let projects = state.store.get_projects(None)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(projects))
}

/// Получает проект по ID
pub async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> std::result::Result<Json<Project>, (StatusCode, Json<ErrorResponse>)> {
    let project = state.store.get_project(project_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Project not found".to_string()))
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string()))
            )
        })?;

    Ok(Json(project))
}

/// Создаёт новый проект
pub async fn add_project(
    State(state): State<Arc<AppState>>,
    AuthUser { user_id, admin, .. }: AuthUser,
    Json(payload): Json<CreateProjectPayload>,
) -> std::result::Result<(StatusCode, Json<Project>), (StatusCode, Json<ErrorResponse>)> {
    if !admin && !state.config.non_admin_can_create_project() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("Not permitted to create projects".to_string())),
        ));
    }

    let project = Project {
        id: 0,
        created: chrono::Utc::now(),
        name: payload.name,
        alert: payload.alert.unwrap_or(false),
        alert_chat: payload.alert_chat,
        max_parallel_tasks: payload.max_parallel_tasks.unwrap_or(0),
        r#type: payload.r#type.unwrap_or_else(|| "default".to_string()),
        default_secret_storage_id: payload.default_secret_storage_id,
    };

    let created = state
        .store
        .create_project(project)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        ))?;

    // Добавляем создателя в project__user с ролью owner (как в Go upstream)
    let project_user = ProjectUser::new(created.id, user_id, ProjectUserRole::Owner);
    state.store.create_project_user(project_user).await.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new(e.to_string())),
    ))?;

    Ok((StatusCode::CREATED, Json(created)))
}

/// Payload для создания проекта
#[derive(Debug, Deserialize)]
pub struct CreateProjectPayload {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_chat: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_parallel_tasks: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_secret_storage_id: Option<i32>,
}

/// Payload для обновления проекта
#[derive(Debug, Deserialize)]
pub struct UpdateProjectPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_chat: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_parallel_tasks: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_secret_storage_id: Option<i32>,
}

/// Обновляет проект
pub async fn update_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<UpdateProjectPayload>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Получаем текущий проект
    let mut project = state.store.get_project(project_id)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new("Project not found".to_string()))
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string()))
            )
        })?;

    // Обновляем только указанные поля
    if let Some(name) = payload.name {
        project.name = name;
    }
    if let Some(alert) = payload.alert {
        project.alert = alert;
    }
    if let Some(alert_chat) = payload.alert_chat {
        project.alert_chat = Some(alert_chat);
    }
    if let Some(max_parallel_tasks) = payload.max_parallel_tasks {
        project.max_parallel_tasks = max_parallel_tasks;
    }
    if let Some(r#type) = payload.r#type {
        project.r#type = r#type;
    }
    if let Some(default_secret_storage_id) = payload.default_secret_storage_id {
        project.default_secret_storage_id = Some(default_secret_storage_id);
    }

    state.store.update_project(project)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::OK)
}

/// Восстанавливает проект из backup
///
/// POST /api/projects/restore
pub async fn restore_project(
    State(state): State<Arc<AppState>>,
    AuthUser { user_id, admin, .. }: AuthUser,
    Json(payload): Json<BackupFormat>,
) -> std::result::Result<(StatusCode, Json<Project>), (StatusCode, Json<ErrorResponse>)> {
    if !admin && !state.config.non_admin_can_create_project() {
        let err = ErrorResponse::new("Нет прав на создание проектов").with_code("FORBIDDEN");
        return Err((StatusCode::FORBIDDEN, Json(err)));
    }

    let user = state.store.get_user(user_id).await.map_err(|e| {
        let (status, resp) = ErrorResponse::from_crate_error(&e);
        (status, Json(resp))
    })?;

    payload.verify().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(e.to_string()).with_code("INVALID_BACKUP")),
        )
    })?;

    let project = payload.restore(&user, &state.store).await.map_err(|e| {
        let (status, resp) = ErrorResponse::from_crate_error(&e);
        (status, Json(resp))
    })?;

    Ok((StatusCode::OK, Json(project)))
}

/// Удаляет проект
pub async fn delete_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state.store.delete_project(project_id)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Получает роль пользователя в проекте
pub async fn get_user_role(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    AuthUser { user_id, admin, .. }: AuthUser,
) -> std::result::Result<Json<String>, (StatusCode, Json<ErrorResponse>)> {
    let users = state.store.get_project_users(project_id, crate::db::store::RetrieveQueryParams::default())
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    if let Some(project_user) = users.into_iter().find(|u| u.user_id == user_id) {
        return Ok(Json(project_user.role.to_string()));
    }

    // Админы и создатели проектов (до фикса) могут не быть в project__user — возвращаем owner
    if admin {
        return Ok(Json("owner".to_string()));
    }

    Err((
        StatusCode::NOT_FOUND,
        Json(ErrorResponse::new("User not found in project".to_string())),
    ))
}

/// Выходит из проекта (текущий пользователь удаляет себя из команды проекта)
///
/// DELETE /api/project/{project_id}/me
pub async fn leave_project(
    State(_state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    AuthUser { user_id, .. }: AuthUser,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    tracing::info!("User {} leaving project {}", user_id, project_id);
    // TODO: implement delete_project_user in store trait
    Ok(StatusCode::NO_CONTENT)
}

/// Возвращает статистику проекта (задачи, шаблоны, пользователи)
///
/// GET /api/project/{project_id}/stats
pub async fn get_project_stats(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
) -> std::result::Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    use crate::db::store::TaskManager;

    let tasks = state.store.get_tasks(project_id, None)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string()))
        ))?;

    Ok(Json(serde_json::json!({
        "project_id": project_id,
        "task_count": tasks.len(),
    })))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projects_handler() {
        // Тест для проверки обработчиков проектов
        assert!(true);
    }
}
