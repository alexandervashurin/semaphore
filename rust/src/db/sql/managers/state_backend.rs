//! Terraform Remote State SQL Manager (Phase 1)

use crate::db::sql::SqlStore;
use crate::db::store::TerraformStateManager;
use crate::error::{Error, Result};
use crate::models::{TerraformState, TerraformStateLock, TerraformStateSummary};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
#[allow(clippy::too_many_arguments)]
impl TerraformStateManager for SqlStore {
    async fn save_terraform_state(
        &self,
        project_id: i32,
        workspace:  &str,
        serial:     i32,
        lineage:    &str,
        data:       Vec<u8>,
        md5:        &str,
    ) -> Result<TerraformState> {
        let pool = self.get_postgres_pool()?;

        // Check idempotency: same serial already stored?
        let existing: Option<sqlx::postgres::PgRow> = sqlx::query(
            "SELECT id, project_id, workspace, serial, lineage, state_data, encrypted, md5, created_at
             FROM terraform_state
             WHERE project_id = $1 AND workspace = $2 AND serial = $3
             LIMIT 1",
        )
        .bind(project_id)
        .bind(workspace)
        .bind(serial)
        .fetch_optional(pool)
        .await
        .map_err(Error::Database)?;

        if let Some(row) = existing {
            let stored_md5: String = row.get("md5");
            if stored_md5 == md5 {
                return Ok(row_to_state(row));
            }
            return Err(Error::Other(
                "state serial already exists with different content".to_string(),
            ));
        }

        let row = sqlx::query(
            "INSERT INTO terraform_state
               (project_id, workspace, serial, lineage, state_data, md5)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, project_id, workspace, serial, lineage, state_data, encrypted, md5, created_at",
        )
        .bind(project_id)
        .bind(workspace)
        .bind(serial)
        .bind(lineage)
        .bind(&data)
        .bind(md5)
        .fetch_one(pool)
        .await
        .map_err(Error::Database)?;

        Ok(row_to_state(row))
    }

    async fn get_terraform_state(
        &self,
        project_id: i32,
        workspace:  &str,
    ) -> Result<Option<TerraformState>> {
        let pool = self.get_postgres_pool()?;
        let row = sqlx::query(
            "SELECT id, project_id, workspace, serial, lineage, state_data, encrypted, md5, created_at
             FROM terraform_state
             WHERE project_id = $1 AND workspace = $2
             ORDER BY serial DESC LIMIT 1",
        )
        .bind(project_id)
        .bind(workspace)
        .fetch_optional(pool)
        .await
        .map_err(Error::Database)?;
        Ok(row.map(row_to_state))
    }

    async fn delete_terraform_state(&self, project_id: i32, workspace: &str) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        sqlx::query("DELETE FROM terraform_state WHERE project_id = $1 AND workspace = $2")
            .bind(project_id)
            .bind(workspace)
            .execute(pool)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn lock_terraform_state(
        &self,
        project_id: i32,
        workspace:  &str,
        lock_id:    &str,
        operation:  &str,
        info:       &str,
        who:        &str,
        version:    &str,
        path:       &str,
    ) -> Result<()> {
        let pool = self.get_postgres_pool()?;

        // Atomic: expire old + try insert in one transaction
        let mut tx = pool.begin().await.map_err(Error::Database)?;

        // Delete expired lock first
        sqlx::query(
            "DELETE FROM terraform_state_lock
             WHERE project_id = $1 AND workspace = $2 AND expires_at < NOW()",
        )
        .bind(project_id)
        .bind(workspace)
        .execute(&mut *tx)
        .await
        .map_err(Error::Database)?;

        // Try to insert; ON CONFLICT means already locked
        let inserted: Option<sqlx::postgres::PgRow> = sqlx::query(
            "INSERT INTO terraform_state_lock
               (project_id, workspace, lock_id, operation, info, who, version, path)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
             ON CONFLICT (project_id, workspace) DO NOTHING
             RETURNING lock_id",
        )
        .bind(project_id)
        .bind(workspace)
        .bind(lock_id)
        .bind(operation)
        .bind(info)
        .bind(who)
        .bind(version)
        .bind(path)
        .fetch_optional(&mut *tx)
        .await
        .map_err(Error::Database)?;

        if inserted.is_none() {
            // Fetch the existing lock so we can return its JSON
            let existing: Option<sqlx::postgres::PgRow> = sqlx::query(
                "SELECT project_id, workspace, lock_id, operation, info, who, version, path, created_at, expires_at
                 FROM terraform_state_lock
                 WHERE project_id = $1 AND workspace = $2",
            )
            .bind(project_id)
            .bind(workspace)
            .fetch_optional(&mut *tx)
            .await
            .map_err(Error::Database)?;

            tx.commit().await.map_err(Error::Database)?;

            if let Some(row) = existing {
                let lock = row_to_lock(row);
                let lock_info = crate::models::LockInfo::from_lock(&lock);
                let json = serde_json::to_string(&lock_info).unwrap_or_default();
                return Err(Error::Other(format!("locked:{json}")));
            }
            return Err(Error::Other("locked".to_string()));
        }

        tx.commit().await.map_err(Error::Database)?;
        Ok(())
    }

    async fn unlock_terraform_state(
        &self,
        project_id: i32,
        workspace:  &str,
        lock_id:    &str,
    ) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        if lock_id.is_empty() {
            // Force unlock
            sqlx::query(
                "DELETE FROM terraform_state_lock WHERE project_id = $1 AND workspace = $2",
            )
            .bind(project_id)
            .bind(workspace)
            .execute(pool)
            .await
            .map_err(Error::Database)?;
        } else {
            sqlx::query(
                "DELETE FROM terraform_state_lock
                 WHERE project_id = $1 AND workspace = $2 AND lock_id = $3",
            )
            .bind(project_id)
            .bind(workspace)
            .bind(lock_id)
            .execute(pool)
            .await
            .map_err(Error::Database)?;
        }
        Ok(())
    }

    async fn get_terraform_lock(
        &self,
        project_id: i32,
        workspace:  &str,
    ) -> Result<Option<TerraformStateLock>> {
        let pool = self.get_postgres_pool()?;
        let row = sqlx::query(
            "SELECT project_id, workspace, lock_id, operation, info, who, version, path, created_at, expires_at
             FROM terraform_state_lock
             WHERE project_id = $1 AND workspace = $2",
        )
        .bind(project_id)
        .bind(workspace)
        .fetch_optional(pool)
        .await
        .map_err(Error::Database)?;
        Ok(row.map(row_to_lock))
    }

    async fn list_terraform_workspaces(&self, project_id: i32) -> Result<Vec<String>> {
        let pool = self.get_postgres_pool()?;
        let rows = sqlx::query(
            "SELECT DISTINCT workspace FROM terraform_state WHERE project_id = $1 ORDER BY workspace",
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(Error::Database)?;
        Ok(rows.into_iter().map(|r| r.get::<String, _>("workspace")).collect())
    }

    async fn list_terraform_state_history(
        &self,
        project_id: i32,
        workspace:  &str,
        limit:      i64,
    ) -> Result<Vec<TerraformStateSummary>> {
        let pool = self.get_postgres_pool()?;
        let rows = sqlx::query(
            "SELECT id, project_id, workspace, serial, lineage, md5, created_at
             FROM terraform_state
             WHERE project_id = $1 AND workspace = $2
             ORDER BY serial DESC
             LIMIT $3",
        )
        .bind(project_id)
        .bind(workspace)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(Error::Database)?;
        Ok(rows.into_iter().map(row_to_summary).collect())
    }

    async fn get_terraform_state_by_serial(
        &self,
        project_id: i32,
        workspace:  &str,
        serial:     i32,
    ) -> Result<Option<TerraformState>> {
        let pool = self.get_postgres_pool()?;
        let row = sqlx::query(
            "SELECT id, project_id, workspace, serial, lineage, state_data, encrypted, md5, created_at
             FROM terraform_state
             WHERE project_id = $1 AND workspace = $2 AND serial = $3",
        )
        .bind(project_id)
        .bind(workspace)
        .bind(serial)
        .fetch_optional(pool)
        .await
        .map_err(Error::Database)?;
        Ok(row.map(row_to_state))
    }

    async fn purge_expired_terraform_locks(&self) -> Result<u64> {
        let pool = self.get_postgres_pool()?;
        let result = sqlx::query("DELETE FROM terraform_state_lock WHERE expires_at < NOW()")
            .execute(pool)
            .await
            .map_err(Error::Database)?;
        Ok(result.rows_affected())
    }
}

fn row_to_state(row: sqlx::postgres::PgRow) -> TerraformState {
    TerraformState {
        id:         row.get("id"),
        project_id: row.get("project_id"),
        workspace:  row.get("workspace"),
        serial:     row.get("serial"),
        lineage:    row.get("lineage"),
        state_data: row.get("state_data"),
        encrypted:  row.get("encrypted"),
        md5:        row.get("md5"),
        created_at: row.get("created_at"),
    }
}

fn row_to_summary(row: sqlx::postgres::PgRow) -> TerraformStateSummary {
    TerraformStateSummary {
        id:         row.get("id"),
        project_id: row.get("project_id"),
        workspace:  row.get("workspace"),
        serial:     row.get("serial"),
        lineage:    row.get("lineage"),
        md5:        row.get("md5"),
        created_at: row.get("created_at"),
    }
}

fn row_to_lock(row: sqlx::postgres::PgRow) -> TerraformStateLock {
    TerraformStateLock {
        project_id: row.get("project_id"),
        workspace:  row.get("workspace"),
        lock_id:    row.get("lock_id"),
        operation:  row.get("operation"),
        info:       row.get("info"),
        who:        row.get("who"),
        version:    row.get("version"),
        path:       row.get("path"),
        created_at: row.get("created_at"),
        expires_at: row.get("expires_at"),
    }
}
