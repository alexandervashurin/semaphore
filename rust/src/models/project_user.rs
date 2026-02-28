//! Project User Model
//!
//! Пользователь проекта

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::models::user::ProjectUserRole;

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
        }
    }
}
