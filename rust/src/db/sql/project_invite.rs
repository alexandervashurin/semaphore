//! ProjectInvite - операции с приглашениями проекта в SQL
//!
//! Аналог db/sql/project_invite.go из Go версии

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::{ProjectInvite, ProjectInviteWithUser, RetrieveQueryParams};
use sqlx::Row;

impl SqlDb {
    /// Получает приглашения проекта
    pub async fn get_project_invites(&self, project_id: i32, params: RetrieveQueryParams) -> Result<Vec<ProjectInviteWithUser>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let invites = sqlx::query_as::<_, ProjectInviteWithUser>(
                    r#"SELECT pi.*, u.username as user_name, u.email as user_email 
                       FROM project_invite pi
                       JOIN user u ON pi.user_id = u.id
                       WHERE pi.project_id = ?
                       LIMIT ? OFFSET ?"#
                )
                .bind(project_id)
                .bind(params.count.unwrap_or(100) as i64)
                .bind(params.offset as i64)
                .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(invites)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Создаёт приглашение проекта
    pub async fn create_project_invite(&self, mut invite: ProjectInvite) -> Result<ProjectInvite> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO project_invite (project_id, user_id, role, created, updated)
                     VALUES (?, ?, ?, ?, ?)"
                )
                .bind(invite.project_id)
                .bind(invite.user_id)
                .bind(&invite.role)
                .bind(invite.created)
                .bind(invite.updated)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                invite.id = result.last_insert_rowid() as i32;
                Ok(invite)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Получает приглашение по ID
    pub async fn get_project_invite(&self, project_id: i32, invite_id: i32) -> Result<ProjectInvite> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let invite = sqlx::query_as::<_, ProjectInvite>(
                    "SELECT * FROM project_invite WHERE id = ? AND project_id = ?"
                )
                .bind(invite_id)
                .bind(project_id)
                .fetch_optional(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                invite.ok_or(Error::NotFound("Project invite not found".to_string()))
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Получает приглашение по токену
    pub async fn get_project_invite_by_token(&self, token: &str) -> Result<ProjectInvite> {
        // TODO: Реализовать когда будет поле token в ProjectInvite
        Err(Error::NotImplemented("get_project_invite_by_token not implemented".to_string()))
    }

    /// Обновляет приглашение
    pub async fn update_project_invite(&self, invite: ProjectInvite) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE project_invite SET role = ?, updated = ?
                     WHERE id = ? AND project_id = ?"
                )
                .bind(&invite.role)
                .bind(invite.updated)
                .bind(invite.id)
                .bind(invite.project_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Удаляет приглашение
    pub async fn delete_project_invite(&self, project_id: i32, invite_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("DELETE FROM project_invite WHERE id = ? AND project_id = ?")
                    .bind(invite_id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_invite_struct() {
        let invite = ProjectInvite {
            id: 1,
            project_id: 1,
            user_id: 1,
            role: "owner".to_string(),
            created: chrono::Utc::now(),
            updated: chrono::Utc::now(),
        };
        assert_eq!(invite.id, 1);
        assert_eq!(invite.project_id, 1);
    }
}
