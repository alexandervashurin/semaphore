//! ProjectInvite - операции с приглашениями проекта в SQL
//!
//! Аналог db/sql/project_invite.go из Go версии

use crate::db::sql::types::SqlDb;
use crate::db::store::RetrieveQueryParams;
use crate::error::{Error, Result};
use crate::models::{ProjectInvite, ProjectInviteWithUser};
use sqlx::Row;

impl SqlDb {
    fn pg_pool_invite(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает приглашения проекта
    pub async fn get_project_invites(&self, project_id: i32, params: RetrieveQueryParams) -> Result<Vec<ProjectInviteWithUser>> {
        let limit = params.count.unwrap_or(100) as i64;
        let offset = params.offset as i64;

        let rows = sqlx::query(
            "SELECT pi.*, u.name as user_name, u.email as user_email \
             FROM project_invite pi \
             LEFT JOIN \"user\" u ON pi.user_id = u.id \
             WHERE pi.project_id = $1 \
             ORDER BY pi.created DESC \
             LIMIT $2 OFFSET $3"
        )
        .bind(project_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pg_pool_invite()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(|row| ProjectInviteWithUser {
            id: row.get("id"),
            project_id: row.get("project_id"),
            user_id: row.get("user_id"),
            role: row.get("role"),
            created: row.get("created"),
            updated: row.get("updated"),
            token: row.try_get("token").ok().unwrap_or_default(),
            inviter_user_id: row.try_get("inviter_user_id").ok().unwrap_or(0),
            user_name: row.try_get("user_name").ok().unwrap_or_default(),
            user_email: row.try_get("user_email").ok().unwrap_or_default(),
        }).collect())
    }

    /// Создаёт приглашение проекта
    pub async fn create_project_invite(&self, mut invite: ProjectInvite) -> Result<ProjectInvite> {
        let now = chrono::Utc::now();
        invite.created = now;
        invite.updated = now;

        if invite.token.is_empty() {
            invite.token = uuid::Uuid::new_v4().to_string();
        }

        let id: i32 = sqlx::query_scalar(
            "INSERT INTO project_invite (project_id, user_id, role, created, updated, token, inviter_user_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id"
        )
        .bind(invite.project_id)
        .bind(invite.user_id)
        .bind(&invite.role)
        .bind(invite.created)
        .bind(invite.updated)
        .bind(&invite.token)
        .bind(invite.inviter_user_id)
        .fetch_one(self.pg_pool_invite()?)
        .await
        .map_err(Error::Database)?;

        invite.id = id;
        Ok(invite)
    }

    /// Получает приглашение по ID
    pub async fn get_project_invite(&self, project_id: i32, invite_id: i32) -> Result<ProjectInvite> {
        let row = sqlx::query(
            "SELECT * FROM project_invite WHERE id = $1 AND project_id = $2"
        )
        .bind(invite_id)
        .bind(project_id)
        .fetch_one(self.pg_pool_invite()?)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound("Приглашение не найдено".to_string()),
            _ => Error::Database(e),
        })?;

        Ok(ProjectInvite {
            id: row.get("id"),
            project_id: row.get("project_id"),
            user_id: row.get("user_id"),
            role: row.get("role"),
            created: row.get("created"),
            updated: row.get("updated"),
            token: row.try_get("token").ok().unwrap_or_default(),
            inviter_user_id: row.try_get("inviter_user_id").ok().unwrap_or(0),
        })
    }

    /// Получает приглашение по токену
    pub async fn get_project_invite_by_token(&self, token: &str) -> Result<ProjectInvite> {
        let row = sqlx::query(
            "SELECT * FROM project_invite WHERE token = $1"
        )
        .bind(token)
        .fetch_one(self.pg_pool_invite()?)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound("Приглашение не найдено".to_string()),
            _ => Error::Database(e),
        })?;

        Ok(ProjectInvite {
            id: row.get("id"),
            project_id: row.get("project_id"),
            user_id: row.get("user_id"),
            role: row.get("role"),
            created: row.get("created"),
            updated: row.get("updated"),
            token: row.try_get("token").ok().unwrap_or_default(),
            inviter_user_id: row.try_get("inviter_user_id").ok().unwrap_or(0),
        })
    }

    /// Обновляет приглашение
    pub async fn update_project_invite(&self, invite: ProjectInvite) -> Result<()> {
        sqlx::query(
            "UPDATE project_invite SET user_id = $1, role = $2, updated = $3, \
             token = $4, inviter_user_id = $5 WHERE id = $6 AND project_id = $7"
        )
        .bind(invite.user_id)
        .bind(&invite.role)
        .bind(chrono::Utc::now())
        .bind(&invite.token)
        .bind(invite.inviter_user_id)
        .bind(invite.id)
        .bind(invite.project_id)
        .execute(self.pg_pool_invite()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Удаляет приглашение
    pub async fn delete_project_invite(&self, project_id: i32, invite_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM project_invite WHERE id = $1 AND project_id = $2")
            .bind(invite_id)
            .bind(project_id)
            .execute(self.pg_pool_invite()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
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
            token: String::new(),
            inviter_user_id: 1,
        };
        assert_eq!(invite.id, 1);
        assert_eq!(invite.project_id, 1);
    }
}
