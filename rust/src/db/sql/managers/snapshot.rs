//! Task Snapshot SQL Manager

use crate::db::sql::SqlStore;
use crate::db::store::SnapshotManager;
use crate::error::{Error, Result};
use crate::models::snapshot::{TaskSnapshot, TaskSnapshotCreate};
use async_trait::async_trait;

#[async_trait]
impl SnapshotManager for SqlStore {
    async fn get_snapshots(&self, project_id: i32, template_id: Option<i32>, limit: i64) -> Result<Vec<TaskSnapshot>> {
        let rows = if let Some(tpl_id) = template_id {
                sqlx::query_as::<_, TaskSnapshot>(
                    r#"SELECT s.*, COALESCE(t.name,'') AS template_name
                       FROM task_snapshot s
                       LEFT JOIN template t ON t.id = s.template_id
                       WHERE s.project_id = $1 AND s.template_id = $2
                       ORDER BY s.id DESC LIMIT $3"#
                )
                .bind(project_id).bind(tpl_id).bind(limit)
                .fetch_all(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?
            } else {
                sqlx::query_as::<_, TaskSnapshot>(
                    r#"SELECT s.*, COALESCE(t.name,'') AS template_name
                       FROM task_snapshot s
                       LEFT JOIN template t ON t.id = s.template_id
                       WHERE s.project_id = $1
                       ORDER BY s.id DESC LIMIT $2"#
                )
                .bind(project_id).bind(limit)
                .fetch_all(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?
            };
            Ok(rows)
    }

    async fn create_snapshot(&self, project_id: i32, payload: TaskSnapshotCreate) -> Result<TaskSnapshot> {
        let row = sqlx::query_as::<_, TaskSnapshot>(
                r#"INSERT INTO task_snapshot (project_id, template_id, task_id, git_branch, git_commit, arguments, inventory_id, environment_id, message, label)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                   RETURNING *, '' AS template_name"#
            )
            .bind(project_id)
            .bind(payload.template_id)
            .bind(payload.task_id)
            .bind(&payload.git_branch)
            .bind(&payload.git_commit)
            .bind(&payload.arguments)
            .bind(payload.inventory_id)
            .bind(payload.environment_id)
            .bind(&payload.message)
            .bind(&payload.label)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(row)
    }

    async fn get_snapshot(&self, id: i32, project_id: i32) -> Result<TaskSnapshot> {
        sqlx::query_as::<_, TaskSnapshot>(
                r#"SELECT s.*, COALESCE(t.name,'') AS template_name
                   FROM task_snapshot s LEFT JOIN template t ON t.id = s.template_id
                   WHERE s.id = $1 AND s.project_id = $2"#
            )
            .bind(id).bind(project_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(|_| Error::NotFound(format!("Snapshot {} not found", id)))
    }

    async fn delete_snapshot(&self, id: i32, project_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM task_snapshot WHERE id = $1 AND project_id = $2")
                .bind(id).bind(project_id)
                .execute(self.get_postgres_pool()?)
                .await.map_err(Error::Database)?;
        Ok(())
    }
}
