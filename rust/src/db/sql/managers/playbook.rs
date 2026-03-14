//! PlaybookManager - управление playbook

use crate::db::sql::SqlStore;
use crate::db::sql::types::SqlDialect;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::playbook::{Playbook, PlaybookCreate, PlaybookUpdate};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl PlaybookManager for SqlStore {
    async fn get_playbooks(&self, project_id: i32) -> Result<Vec<Playbook>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let playbooks = sqlx::query_as::<_, Playbook>(
                    "SELECT * FROM playbook WHERE project_id = ? ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

                Ok(playbooks)
            }
            SqlDialect::PostgreSQL => {
                let playbooks = sqlx::query_as::<_, Playbook>(
                    "SELECT * FROM playbook WHERE project_id = $1 ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

                Ok(playbooks)
            }
            SqlDialect::MySQL => {
                let playbooks = sqlx::query_as::<_, Playbook>(
                    "SELECT * FROM `playbook` WHERE project_id = ? ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

                Ok(playbooks)
            }
        }
    }

    async fn get_playbook(&self, id: i32, project_id: i32) -> Result<Playbook> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let playbook = sqlx::query_as::<_, Playbook>(
                    "SELECT * FROM playbook WHERE id = ? AND project_id = ?"
                )
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

                Ok(playbook)
            }
            SqlDialect::PostgreSQL => {
                let playbook = sqlx::query_as::<_, Playbook>(
                    "SELECT * FROM playbook WHERE id = $1 AND project_id = $2"
                )
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

                Ok(playbook)
            }
            SqlDialect::MySQL => {
                let playbook = sqlx::query_as::<_, Playbook>(
                    "SELECT * FROM `playbook` WHERE id = ? AND project_id = ?"
                )
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

                Ok(playbook)
            }
        }
    }

    async fn create_playbook(&self, project_id: i32, payload: PlaybookCreate) -> Result<Playbook> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let playbook = sqlx::query_as::<_, Playbook>(
                    "INSERT INTO playbook (project_id, name, content, description, playbook_type, repository_id, created, updated)
                     VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now')) RETURNING *"
                )
                .bind(project_id)
                .bind(&payload.name)
                .bind(&payload.content)
                .bind(&payload.description)
                .bind(&payload.playbook_type)
                .bind(&payload.repository_id)
                .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

                Ok(playbook)
            }
            SqlDialect::PostgreSQL => {
                let playbook = sqlx::query_as::<_, Playbook>(
                    "INSERT INTO playbook (project_id, name, content, description, playbook_type, repository_id, created, updated)
                     VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW()) RETURNING *"
                )
                .bind(project_id)
                .bind(&payload.name)
                .bind(&payload.content)
                .bind(&payload.description)
                .bind(&payload.playbook_type)
                .bind(&payload.repository_id)
                .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

                Ok(playbook)
            }
            SqlDialect::MySQL => {
                let playbook = sqlx::query_as::<_, Playbook>(
                    "INSERT INTO `playbook` (project_id, name, content, description, playbook_type, repository_id, created, updated)
                     VALUES (?, ?, ?, ?, ?, ?, NOW(), NOW()) RETURNING *"
                )
                .bind(project_id)
                .bind(&payload.name)
                .bind(&payload.content)
                .bind(&payload.description)
                .bind(&payload.playbook_type)
                .bind(&payload.repository_id)
                .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

                Ok(playbook)
            }
        }
    }

    async fn update_playbook(&self, id: i32, project_id: i32, payload: PlaybookUpdate) -> Result<Playbook> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let playbook = sqlx::query_as::<_, Playbook>(
                    "UPDATE playbook SET name = ?, content = ?, description = ?, playbook_type = ?, updated = datetime('now')
                     WHERE id = ? AND project_id = ? RETURNING *"
                )
                .bind(&payload.name)
                .bind(&payload.content)
                .bind(&payload.description)
                .bind(&payload.playbook_type)
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

                Ok(playbook)
            }
            SqlDialect::PostgreSQL => {
                let playbook = sqlx::query_as::<_, Playbook>(
                    "UPDATE playbook SET name = $1, content = $2, description = $3, playbook_type = $4, updated = NOW()
                     WHERE id = $5 AND project_id = $6 RETURNING *"
                )
                .bind(&payload.name)
                .bind(&payload.content)
                .bind(&payload.description)
                .bind(&payload.playbook_type)
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

                Ok(playbook)
            }
            SqlDialect::MySQL => {
                let playbook = sqlx::query_as::<_, Playbook>(
                    "UPDATE `playbook` SET name = ?, content = ?, description = ?, playbook_type = ?, updated = NOW()
                     WHERE id = ? AND project_id = ? RETURNING *"
                )
                .bind(&payload.name)
                .bind(&payload.content)
                .bind(&payload.description)
                .bind(&payload.playbook_type)
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;

                Ok(playbook)
            }
        }
    }

    async fn delete_playbook(&self, id: i32, project_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query("DELETE FROM playbook WHERE id = ? AND project_id = ?")
                    .bind(id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query("DELETE FROM playbook WHERE id = $1 AND project_id = $2")
                    .bind(id)
                    .bind(project_id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                sqlx::query("DELETE FROM `playbook` WHERE id = ? AND project_id = ?")
                    .bind(id)
                    .bind(project_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
        }

        Ok(())
    }
}
