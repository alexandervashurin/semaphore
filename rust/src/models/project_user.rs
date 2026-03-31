//! Project User Model
//!
//! Пользователь проекта

use crate::models::user::ProjectUserRole;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Пользователь проекта
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectUser {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта
    pub project_id: i32,

    /// ID пользователя
    pub user_id: i32,

    /// Роль пользователя
    pub role: ProjectUserRole,

    /// Дата создания
    pub created: DateTime<Utc>,

    /// Имя пользователя (логин)
    #[sqlx(default)]
    pub username: String,

    /// Полное имя пользователя
    #[sqlx(default)]
    pub name: String,
}

impl ProjectUser {
    /// Создаёт нового пользователя проекта
    pub fn new(project_id: i32, user_id: i32, role: ProjectUserRole) -> Self {
        Self {
            id: 0,
            project_id,
            user_id,
            role,
            created: Utc::now(),
            username: String::new(),
            name: String::new(),
        }
    }
}
