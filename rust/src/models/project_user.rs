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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_user_new() {
        let pu = ProjectUser::new(10, 5, ProjectUserRole::Manager);
        assert_eq!(pu.id, 0);
        assert_eq!(pu.project_id, 10);
        assert_eq!(pu.user_id, 5);
        assert_eq!(pu.role, ProjectUserRole::Manager);
        assert!(pu.username.is_empty());
    }

    #[test]
    fn test_project_user_serialization() {
        let pu = ProjectUser {
            id: 1,
            project_id: 10,
            user_id: 5,
            role: ProjectUserRole::TaskRunner,
            created: Utc::now(),
            username: "johndoe".to_string(),
            name: "John Doe".to_string(),
        };
        let json = serde_json::to_string(&pu).unwrap();
        assert!(json.contains("\"username\":\"johndoe\""));
        assert!(json.contains("\"role\":\"task_runner\""));
    }
}
