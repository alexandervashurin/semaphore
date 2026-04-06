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
}
