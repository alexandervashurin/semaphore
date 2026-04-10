//! ProjectInviteManager - управление приглашениями в проект

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::{ProjectInvite, ProjectInviteWithUser};
use async_trait::async_trait;

#[async_trait]
impl ProjectInviteManager for SqlStore {
    async fn get_project_invites(
        &self,
        project_id: i32,
        params: RetrieveQueryParams,
    ) -> Result<Vec<ProjectInviteWithUser>> {
        self.get_project_invites(project_id, params).await
    }

    async fn create_project_invite(&self, invite: ProjectInvite) -> Result<ProjectInvite> {
        self.db.create_project_invite(invite).await
    }

    async fn get_project_invite(&self, project_id: i32, invite_id: i32) -> Result<ProjectInvite> {
        self.db.get_project_invite(project_id, invite_id).await
    }

    async fn get_project_invite_by_token(&self, token: &str) -> Result<ProjectInvite> {
        self.db.get_project_invite_by_token(token).await
    }

    async fn update_project_invite(&self, invite: ProjectInvite) -> Result<()> {
        self.db.update_project_invite(invite).await
    }

    async fn delete_project_invite(&self, project_id: i32, invite_id: i32) -> Result<()> {
        self.db.delete_project_invite(project_id, invite_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{ProjectInvite, ProjectInviteWithUser};
    use chrono::Utc;

    #[test]
    fn test_project_invite_serialization() {
        let invite = ProjectInvite {
            id: 1, project_id: 10, user_id: 5, role: "manager".to_string(),
            created: Utc::now(), updated: Utc::now(),
            token: "invite-token-123".to_string(), inviter_user_id: 1,
        };
        let json = serde_json::to_string(&invite).unwrap();
        assert!(json.contains("\"role\":\"manager\""));
        assert!(json.contains("\"token\":\"invite-token-123\""));
    }

    #[test]
    fn test_project_invite_with_user_serialization() {
        let invite = ProjectInviteWithUser {
            id: 1, project_id: 10, user_id: 5, role: "task_runner".to_string(),
            created: Utc::now(), updated: Utc::now(),
            token: "token-456".to_string(), inviter_user_id: 1,
            user_name: "John Doe".to_string(), user_email: "john@example.com".to_string(),
        };
        let json = serde_json::to_string(&invite).unwrap();
        assert!(json.contains("\"user_name\":\"John Doe\""));
        assert!(json.contains("\"user_email\":\"john@example.com\""));
    }

    #[test]
    fn test_project_invite_clone() {
        let invite = ProjectInvite {
            id: 1, project_id: 10, user_id: 5, role: "manager".to_string(),
            created: Utc::now(), updated: Utc::now(),
            token: "clone-token".to_string(), inviter_user_id: 1,
        };
        let cloned = invite.clone();
        assert_eq!(cloned.token, invite.token);
        assert_eq!(cloned.role, invite.role);
    }

    #[test]
    fn test_project_invite_with_user_clone() {
        let invite = ProjectInviteWithUser {
            id: 1, project_id: 10, user_id: 5, role: "owner".to_string(),
            created: Utc::now(), updated: Utc::now(),
            token: "tok".to_string(), inviter_user_id: 1,
            user_name: "Clone User".to_string(), user_email: "clone@example.com".to_string(),
        };
        let cloned = invite.clone();
        assert_eq!(cloned.user_name, invite.user_name);
    }

    #[test]
    fn test_project_invite_deserialize() {
        let json = r#"{"id":5,"project_id":20,"user_id":10,"role":"viewer","created":"2024-01-01T00:00:00Z","updated":"2024-01-01T00:00:00Z","token":"deser-token","inviter_user_id":2}"#;
        let invite: ProjectInvite = serde_json::from_str(json).unwrap();
        assert_eq!(invite.id, 5);
        assert_eq!(invite.role, "viewer");
    }

    #[test]
    fn test_project_invite_with_user_deserialize() {
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
                id: 1, project_id: 1, user_id: 1, role: role.to_string(),
                created: Utc::now(), updated: Utc::now(),
                token: "token".to_string(), inviter_user_id: 1,
            };
            let json = serde_json::to_string(&invite).unwrap();
            assert!(json.contains(&format!("\"role\":\"{}\"", role)));
        }
    }

    #[test]
    fn test_project_invite_empty_user_info() {
        let invite = ProjectInviteWithUser {
            id: 1, project_id: 1, user_id: 1, role: "owner".to_string(),
            created: Utc::now(), updated: Utc::now(),
            token: "tok".to_string(), inviter_user_id: 1,
            user_name: String::new(), user_email: String::new(),
        };
        assert!(invite.user_name.is_empty());
        assert!(invite.user_email.is_empty());
    }
}
