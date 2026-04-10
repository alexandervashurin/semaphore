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

    /// Токен приглашения
    pub token: String,

    /// ID пригласившего пользователя
    pub inviter_user_id: i32,
}

/// Приглашение в проект с информацией о пользователе
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectInviteWithUser {
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

    /// Токен приглашения
    pub token: String,

    /// ID пригласившего пользователя
    pub inviter_user_id: i32,

    /// Имя пользователя
    #[sqlx(default)]
    pub user_name: String,

    /// Email пользователя
    #[sqlx(default)]
    pub user_email: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_invite_serialization() {
        let invite = ProjectInvite {
            id: 1,
            project_id: 10,
            user_id: 5,
            role: "manager".to_string(),
            created: Utc::now(),
            updated: Utc::now(),
            token: "invite-token-123".to_string(),
            inviter_user_id: 1,
        };
        let json = serde_json::to_string(&invite).unwrap();
        assert!(json.contains("\"role\":\"manager\""));
        assert!(json.contains("\"token\":\"invite-token-123\""));
        assert!(json.contains("\"inviter_user_id\":1"));
    }

    #[test]
    fn test_project_invite_with_user_serialization() {
        let invite = ProjectInviteWithUser {
            id: 1,
            project_id: 10,
            user_id: 5,
            role: "task_runner".to_string(),
            created: Utc::now(),
            updated: Utc::now(),
            token: "token-456".to_string(),
            inviter_user_id: 1,
            user_name: "John Doe".to_string(),
            user_email: "john@example.com".to_string(),
        };
        let json = serde_json::to_string(&invite).unwrap();
        assert!(json.contains("\"user_name\":\"John Doe\""));
        assert!(json.contains("\"user_email\":\"john@example.com\""));
    }

    #[test]
    fn test_project_invite_clone() {
        let invite = ProjectInvite {
            id: 1,
            project_id: 10,
            user_id: 5,
            role: "manager".to_string(),
            created: Utc::now(),
            updated: Utc::now(),
            token: "clone-token".to_string(),
            inviter_user_id: 1,
        };
        let cloned = invite.clone();
        assert_eq!(cloned.token, invite.token);
        assert_eq!(cloned.role, invite.role);
    }

    #[test]
    fn test_project_invite_with_user_clone() {
        let invite = ProjectInviteWithUser {
            id: 1,
            project_id: 10,
            user_id: 5,
            role: "owner".to_string(),
            created: Utc::now(),
            updated: Utc::now(),
            token: "clone-token-2".to_string(),
            inviter_user_id: 1,
            user_name: "Clone User".to_string(),
            user_email: "clone@example.com".to_string(),
        };
        let cloned = invite.clone();
        assert_eq!(cloned.user_name, invite.user_name);
        assert_eq!(cloned.user_email, invite.user_email);
    }

    #[test]
    fn test_project_invite_deserialization() {
        let json = r#"{"id":5,"project_id":20,"user_id":10,"role":"viewer","created":"2024-01-01T00:00:00Z","updated":"2024-01-01T00:00:00Z","token":"deser-token","inviter_user_id":2}"#;
        let invite: ProjectInvite = serde_json::from_str(json).unwrap();
        assert_eq!(invite.id, 5);
        assert_eq!(invite.project_id, 20);
        assert_eq!(invite.user_id, 10);
        assert_eq!(invite.role, "viewer");
        assert_eq!(invite.token, "deser-token");
    }

    #[test]
    fn test_project_invite_with_user_deserialization() {
        let json = r#"{"id":3,"project_id":15,"user_id":8,"role":"admin","created":"2024-01-01T00:00:00Z","updated":"2024-01-01T00:00:00Z","token":"tok","inviter_user_id":1,"user_name":"Test User","user_email":"test@test.com"}"#;
        let invite: ProjectInviteWithUser = serde_json::from_str(json).unwrap();
        assert_eq!(invite.user_name, "Test User");
        assert_eq!(invite.user_email, "test@test.com");
    }

    #[test]
    fn test_project_invite_all_roles() {
        let roles = ["owner", "manager", "task_runner", "viewer"];
        for role in roles {
            let invite = ProjectInvite {
                id: 1,
                project_id: 1,
                user_id: 1,
                role: role.to_string(),
                created: Utc::now(),
                updated: Utc::now(),
                token: "token".to_string(),
                inviter_user_id: 1,
            };
            let json = serde_json::to_string(&invite).unwrap();
            assert!(json.contains(&format!("\"role\":\"{}\"", role)));
        }
    }

    #[test]
    fn test_project_invite_debug() {
        let invite = ProjectInvite {
            id: 1, project_id: 10, user_id: 5, role: "manager".to_string(),
            created: Utc::now(), updated: Utc::now(), token: "debug".to_string(),
            inviter_user_id: 1,
        };
        let debug_str = format!("{:?}", invite);
        assert!(debug_str.contains("ProjectInvite"));
    }

    #[test]
    fn test_project_invite_with_user_debug() {
        let invite = ProjectInviteWithUser {
            id: 1, project_id: 10, user_id: 5, role: "manager".to_string(),
            created: Utc::now(), updated: Utc::now(), token: "debug".to_string(),
            inviter_user_id: 1, user_name: "Debug User".to_string(),
            user_email: "debug@test.com".to_string(),
        };
        let debug_str = format!("{:?}", invite);
        assert!(debug_str.contains("ProjectInviteWithUser"));
        assert!(debug_str.contains("Debug User"));
    }

    #[test]
    fn test_project_invite_empty_token() {
        let invite = ProjectInvite {
            id: 1, project_id: 1, user_id: 1, role: "viewer".to_string(),
            created: Utc::now(), updated: Utc::now(), token: "".to_string(),
            inviter_user_id: 1,
        };
        assert!(invite.token.is_empty());
    }

    #[test]
    fn test_project_invite_same_inviter() {
        let invite = ProjectInvite {
            id: 1, project_id: 10, user_id: 5, role: "manager".to_string(),
            created: Utc::now(), updated: Utc::now(), token: "self-invite".to_string(),
            inviter_user_id: 5, // same as user_id
        };
        assert_eq!(invite.user_id, invite.inviter_user_id);
    }

    #[test]
    fn test_project_invite_with_user_empty_fields() {
        let invite = ProjectInviteWithUser {
            id: 0, project_id: 0, user_id: 0, role: String::new(),
            created: Utc::now(), updated: Utc::now(), token: String::new(),
            inviter_user_id: 0, user_name: String::new(), user_email: String::new(),
        };
        assert!(invite.role.is_empty());
        assert!(invite.token.is_empty());
        assert!(invite.user_name.is_empty());
    }

    #[test]
    fn test_project_invite_deserialization_minimal() {
        let json = r#"{"id":1,"project_id":1,"user_id":1,"role":"viewer","created":"2024-01-01T00:00:00Z","updated":"2024-01-01T00:00:00Z","token":"t","inviter_user_id":1}"#;
        let invite: ProjectInvite = serde_json::from_str(json).unwrap();
        assert_eq!(invite.token, "t");
        assert_eq!(invite.role, "viewer");
    }

    #[test]
    fn test_project_invite_with_user_deserialization_minimal() {
        let json = r#"{"id":1,"project_id":1,"user_id":1,"role":"owner","created":"2024-01-01T00:00:00Z","updated":"2024-01-01T00:00:00Z","token":"t","inviter_user_id":1,"user_name":"N","user_email":"e@t.com"}"#;
        let invite: ProjectInviteWithUser = serde_json::from_str(json).unwrap();
        assert_eq!(invite.user_name, "N");
        assert_eq!(invite.user_email, "e@t.com");
    }
}
