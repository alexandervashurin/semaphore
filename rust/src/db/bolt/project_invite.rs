//! ProjectInvite - операции с приглашениями в проекты в BoltDB
//!
//! Аналог db/bolt/project_invite.go из Go версии

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::Result;
use crate::models::{ProjectInvite, ProjectInviteWithUser, User, RetrieveQueryParams};

impl BoltStore {
    /// Получает приглашения проекта
    pub async fn get_project_invites(&self, project_id: i32, params: RetrieveQueryParams) -> Result<Vec<ProjectInviteWithUser>> {
        let mut invites = Vec::new();
        
        // Получаем все приглашения проекта
        let project_invites = self.get_objects::<ProjectInvite>(project_id, "project_invites", params).await?;
        
        for invite in project_invites {
            let mut invite_with_user = ProjectInviteWithUser {
                project_invite: invite.clone(),
                invited_by_user: None,
                user: None,
            };

            // Получаем информацию о пригласившем пользователе
            if let Ok(invited_by_user) = self.get_user(invite.inviter_user_id).await {
                invite_with_user.invited_by_user = Some(invited_by_user);
            }

            // Получаем информацию о пользователе, если есть
            if let Some(user_id) = invite.user_id {
                if let Ok(user) = self.get_user(user_id).await {
                    invite_with_user.user = Some(user);
                }
            }

            invites.push(invite_with_user);
        }

        Ok(invites)
    }

    /// Создаёт приглашение в проект
    pub async fn create_project_invite(&self, invite: ProjectInvite) -> Result<ProjectInvite> {
        let new_invite = self.create_object(invite.project_id, "project_invites", invite).await?;
        Ok(new_invite)
    }

    /// Получает приглашение по ID
    pub async fn get_project_invite(&self, project_id: i32, invite_id: i32) -> Result<ProjectInvite> {
        self.get_object(project_id, "project_invites", invite_id).await
    }

    /// Получает приглашение по токену
    pub async fn get_project_invite_by_token(&self, token: &str) -> Result<ProjectInvite> {
        // Получаем все проекты для поиска по всем приглашениям
        let projects = self.get_all_projects().await?;
        
        for project in projects {
            let params = RetrieveQueryParams {
                offset: 0,
                count: 1000,
                filter: String::new(),
            };
            
            let project_invites = self.get_objects::<ProjectInvite>(project.id, "project_invites", params).await?;
            
            for invite in project_invites {
                if invite.token == token {
                    return Ok(invite);
                }
            }
        }

        Err(crate::error::Error::NotFound("Приглашение не найдено".to_string()))
    }

    /// Обновляет приглашение
    pub async fn update_project_invite(&self, invite: ProjectInvite) -> Result<()> {
        self.update_object(invite.project_id, "project_invites", invite).await
    }

    /// Удаляет приглашение
    pub async fn delete_project_invite(&self, project_id: i32, invite_id: i32) -> Result<()> {
        self.delete_object(project_id, "project_invites", invite_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc, Duration};
    use std::path::PathBuf;
    use uuid::Uuid;

    fn create_test_bolt_db() -> BoltStore {
        let path = PathBuf::from("/tmp/test_invites.db");
        BoltStore::new(path).unwrap()
    }

    fn create_test_invite(project_id: i32, user_id: i32) -> ProjectInvite {
        ProjectInvite {
            id: 0,
            project_id,
            user_id: Some(user_id),
            inviter_user_id: 1,
            role_id: 2,
            token: Uuid::new_v4().to_string(),
            created: Utc::now(),
            expires: Utc::now() + Duration::days(7),
        }
    }

    #[tokio::test]
    async fn test_create_project_invite() {
        let db = create_test_bolt_db();
        let invite = create_test_invite(1, 2);
        
        let result = db.create_project_invite(invite).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_project_invite() {
        let db = create_test_bolt_db();
        let invite = create_test_invite(1, 2);
        let created = db.create_project_invite(invite).await.unwrap();
        
        let retrieved = db.get_project_invite(1, created.id).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().token, created.token);
    }

    #[tokio::test]
    async fn test_get_project_invite_by_token() {
        let db = create_test_bolt_db();
        let invite = create_test_invite(1, 2);
        let token = invite.token.clone();
        db.create_project_invite(invite).await.unwrap();
        
        let retrieved = db.get_project_invite_by_token(&token).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().token, token);
    }

    #[tokio::test]
    async fn test_update_project_invite() {
        let db = create_test_bolt_db();
        let mut invite = create_test_invite(1, 2);
        let created = db.create_project_invite(invite.clone()).await.unwrap();
        
        invite.id = created.id;
        invite.role_id = 3; // Изменяем роль
        
        let result = db.update_project_invite(invite).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_project_invite() {
        let db = create_test_bolt_db();
        let invite = create_test_invite(1, 2);
        let created = db.create_project_invite(invite).await.unwrap();
        
        let result = db.delete_project_invite(1, created.id).await;
        assert!(result.is_ok());
        
        let retrieved = db.get_project_invite(1, created.id).await;
        assert!(retrieved.is_err());
    }

    #[tokio::test]
    async fn test_get_project_invites() {
        let db = create_test_bolt_db();
        
        // Создаём несколько приглашений
        for i in 0..3 {
            let invite = create_test_invite(1, i + 10);
            db.create_project_invite(invite).await.unwrap();
        }
        
        let params = RetrieveQueryParams {
            offset: 0,
            count: 10,
            filter: String::new(),
        };
        
        let invites = db.get_project_invites(1, params).await;
        assert!(invites.is_ok());
        assert!(invites.unwrap().len() >= 3);
    }
}
