//! Модель приглашения в проект (ProjectInvite)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Приглашение в проект
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectInvite {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта
    pub project_id: i32,

    /// ID пользователя
    pub user_id: i32,

    /// Роль пользователя в проекте
    pub role: String,

    /// Дата создания
    pub created: DateTime<Utc>,

    /// Дата обновления
    pub updated: DateTime<Utc>,
}

/// Приглашение в проект с информацией о пользователе
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectInviteWithUser {
    #[serde(flatten)]
    pub invite: ProjectInvite,

    /// Имя пользователя
    pub user_name: String,

    /// Email пользователя
    pub user_email: String,
}
