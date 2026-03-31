//! Projects API - Invites Handler
//!
//! Обработчики для приглашений в проект

use crate::api::extractors::AuthUser;
use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::{ProjectInviteManager, ProjectStore, RetrieveQueryParams};
use crate::error::{Error, Result};
use crate::models::ProjectInvite;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Payload для создания приглашения
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvitePayload {
    pub user_id: i32,
    pub role: String,
}

/// Ответ с приглашением
#[derive(Debug, Serialize, Deserialize)]
pub struct InviteResponse {
    pub id: i32,
    pub project_id: i32,
    pub user_id: i32,
    pub role: String,
    pub token: String,
}

/// Получает приглашения проекта
pub async fn get_invites(
    State(state): State<Arc<AppState>>,
    AuthUser {
        user_id: _user_id, ..
    }: AuthUser,
    Path(project_id): Path<i32>,
) -> std::result::Result<
    Json<Vec<crate::models::ProjectInviteWithUser>>,
    (StatusCode, Json<ErrorResponse>),
> {
    let invites = state
        .store
        .get_project_invites(project_id, RetrieveQueryParams::default())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(Json(invites))
}

/// Создаёт приглашение в проект
pub async fn create_invite(
    State(state): State<Arc<AppState>>,
    AuthUser { user_id, .. }: AuthUser,
    Path(project_id): Path<i32>,
    Json(payload): Json<CreateInvitePayload>,
) -> std::result::Result<(StatusCode, Json<InviteResponse>), (StatusCode, Json<ErrorResponse>)> {
    let token = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    let invite = ProjectInvite {
        id: 0,
        project_id,
        user_id: payload.user_id,
        role: payload.role,
        created: now,
        updated: now,
        token: token.clone(),
        inviter_user_id: user_id,
    };

    let created = state
        .store
        .create_project_invite(invite)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok((
        StatusCode::CREATED,
        Json(InviteResponse {
            id: created.id,
            project_id: created.project_id,
            user_id: created.user_id,
            role: created.role,
            token: created.token,
        }),
    ))
}

/// Удаляет приглашение
pub async fn delete_invite(
    State(state): State<Arc<AppState>>,
    AuthUser { .. }: AuthUser,
    Path((project_id, invite_id)): Path<(i32, i32)>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    state
        .store
        .delete_project_invite(project_id, invite_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Принимает приглашение по токену
pub async fn accept_invite(
    State(state): State<Arc<AppState>>,
    AuthUser { user_id, .. }: AuthUser,
    Path(token): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let invite = state
        .store
        .get_project_invite_by_token(&token)
        .await
        .map_err(|e| match e {
            Error::NotFound(_) => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(
                    "Invite not found or expired".to_string(),
                )),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ),
        })?;

    // Проверяем что текущий пользователь - приглашённый
    if invite.user_id != user_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "Invite belongs to another user".to_string(),
            )),
        ));
    }

    // Добавляем пользователя в проект с указанной ролью
    let project_user = crate::models::ProjectUser {
        id: 0,
        project_id: invite.project_id,
        user_id: invite.user_id,
        role: parse_project_role(&invite.role),
        created: chrono::Utc::now(),
        username: String::new(),
        name: String::new(),
    };

    if let Err(e) = state.store.create_project_user(project_user).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(e.to_string())),
        ));
    }

    // Удаляем приглашение после принятия
    let _ = state
        .store
        .delete_project_invite(invite.project_id, invite.id)
        .await;

    Ok(Json(serde_json::json!({
        "project_id": invite.project_id,
        "role": invite.role,
        "accepted": true
    })))
}

fn parse_project_role(s: &str) -> crate::models::ProjectUserRole {
    use crate::models::ProjectUserRole;
    match s.to_lowercase().as_str() {
        "owner" => ProjectUserRole::Owner,
        "manager" => ProjectUserRole::Manager,
        "task_runner" => ProjectUserRole::TaskRunner,
        "guest" => ProjectUserRole::Guest,
        _ => ProjectUserRole::Guest,
    }
}
