//! PlaybookManager - управление playbook

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::playbook::{Playbook, PlaybookCreate, PlaybookUpdate};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl PlaybookManager for SqlStore {
    async fn get_playbooks(&self, project_id: i32) -> Result<Vec<Playbook>> {
        let playbooks = sqlx::query_as::<_, Playbook>(
                "SELECT * FROM playbook WHERE project_id = $1 ORDER BY name"
            )
            .bind(project_id)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

            Ok(playbooks)
    }

    async fn get_playbook(&self, id: i32, project_id: i32) -> Result<Playbook> {
        let playbook = sqlx::query_as::<_, Playbook>(
                "SELECT * FROM playbook WHERE id = $1 AND project_id = $2"
            )
            .bind(id)
            .bind(project_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

            Ok(playbook)
    }

    async fn create_playbook(&self, project_id: i32, payload: PlaybookCreate) -> Result<Playbook> {
        let playbook = sqlx::query_as::<_, Playbook>(
                "INSERT INTO playbook (project_id, name, content, description, playbook_type, repository_id, created, updated)
                 VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW()) RETURNING *"
            )
            .bind(project_id)
            .bind(&payload.name)
            .bind(&payload.content)
            .bind(&payload.description)
            .bind(&payload.playbook_type)
            .bind(payload.repository_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

            Ok(playbook)
    }

    async fn update_playbook(&self, id: i32, project_id: i32, payload: PlaybookUpdate) -> Result<Playbook> {
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
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

            Ok(playbook)
    }

    async fn delete_playbook(&self, id: i32, project_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM playbook WHERE id = $1 AND project_id = $2")
                .bind(id)
                .bind(project_id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;

        Ok(())
    }
}
