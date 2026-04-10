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

    #[test]
    fn test_project_user_clone() {
        let pu = ProjectUser::new(10, 5, ProjectUserRole::Owner);
        let cloned = pu.clone();
        assert_eq!(cloned.project_id, pu.project_id);
        assert_eq!(cloned.user_id, pu.user_id);
        assert_eq!(cloned.role, pu.role);
    }

    #[test]
    fn test_project_user_all_roles() {
        let roles = [ProjectUserRole::Owner, ProjectUserRole::Manager, ProjectUserRole::TaskRunner];
        for role in roles.iter() {
            let pu = ProjectUser::new(1, 1, role.clone());
            let json = serde_json::to_string(&pu).unwrap();
            assert!(json.contains("\"role\":"));
        }
    }

    #[test]
    fn test_project_user_deserialization() {
        let json = r#"{"id":5,"project_id":20,"user_id":10,"role":"manager","created":"2024-01-01T00:00:00Z","username":"admin","name":"Admin User"}"#;
        let pu: ProjectUser = serde_json::from_str(json).unwrap();
        assert_eq!(pu.id, 5);
        assert_eq!(pu.username, "admin");
        assert_eq!(pu.name, "Admin User");
    }

    #[test]
    fn test_project_user_debug() {
        let pu = ProjectUser::new(1, 1, ProjectUserRole::Owner);
        let debug_str = format!("{:?}", pu);
        assert!(debug_str.contains("ProjectUser"));
    }

    #[test]
    fn test_project_user_empty_strings() {
        let pu = ProjectUser::new(1, 1, ProjectUserRole::TaskRunner);
        assert!(pu.username.is_empty());
        assert!(pu.name.is_empty());
    }

    #[test]
    fn test_project_user_new_with_manager_role() {
        let pu = ProjectUser::new(100, 50, ProjectUserRole::Manager);
        assert_eq!(pu.role, ProjectUserRole::Manager);
        assert_eq!(pu.project_id, 100);
        assert_eq!(pu.user_id, 50);
    }

    #[test]
    fn test_project_user_serialization_full() {
        let pu = ProjectUser {
            id: 10,
            project_id: 100,
            user_id: 50,
            role: ProjectUserRole::Owner,
            created: Utc::now(),
            username: "owner_user".to_string(),
            name: "Owner User".to_string(),
        };
        let json = serde_json::to_string(&pu).unwrap();
        assert!(json.contains("\"id\":10"));
        assert!(json.contains("\"project_id\":100"));
        assert!(json.contains("\"user_id\":50"));
        assert!(json.contains("\"name\":\"Owner User\""));
    }

    #[test]
    fn test_project_user_roundtrip() {
        let original = ProjectUser {
            id: 7, project_id: 42, user_id: 21, role: ProjectUserRole::Manager,
            created: Utc::now(), username: "roundtrip".to_string(), name: "Round Trip".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: ProjectUser = serde_json::from_str(&json).unwrap();
        assert_eq!(original.id, restored.id);
        assert_eq!(original.role, restored.role);
        assert_eq!(original.username, restored.username);
    }
}
