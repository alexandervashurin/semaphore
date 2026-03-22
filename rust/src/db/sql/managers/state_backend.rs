//! Terraform Remote State Backend — SQL manager (PostgreSQL)
//!
//! Implements TerraformStateManager for SqlStore.
//! Lock acquisition uses an atomic BEGIN/DELETE-expired/INSERT ON CONFLICT DO NOTHING/COMMIT
//! to prevent the TOCTOU race where two concurrent requests both see an expired lock,
//! delete it, and both succeed.

use crate::db::sql::SqlStore;
use crate::db::store::TerraformStateManager;
use crate::error::{Error, Result};
use crate::models::{TerraformState, TerraformStateLock, TerraformStateSummary};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl TerraformStateManager for SqlStore {
    // ── Read ─────────────────────────────────────────────────────────────────

    async fn get_terraform_state(
        &self,
        project_id: i32,
        workspace: &str,
    ) -> Result<Option<TerraformState>> {
        let pool = self.get_postgres_pool()?;
        let row = sqlx::query(
            "SELECT id, project_id, workspace, serial, lineage, state_data, encrypted, md5, created_at
             FROM terraform_state
             WHERE project_id = $1 AND workspace = $2
             ORDER BY serial DESC
             LIMIT 1",
        )
        .bind(project_id)
        .bind(workspace)
        .fetch_optional(pool)
        .await
        .map_err(Error::Database)?;

        Ok(row.map(|r| TerraformState {
            id:         r.get("id"),
            project_id: r.get("project_id"),
            workspace:  r.get("workspace"),
            serial:     r.get("serial"),
            lineage:    r.get("lineage"),
            state_data: r.get("state_data"),
            encrypted:  r.get("encrypted"),
            md5:        r.get("md5"),
            created_at: r.get("created_at"),
        }))
    }

    async fn list_terraform_states(
        &self,
        project_id: i32,
        workspace: &str,
    ) -> Result<Vec<TerraformStateSummary>> {
        let pool = self.get_postgres_pool()?;
        let rows = sqlx::query(
            "SELECT id, project_id, workspace, serial, lineage, encrypted, md5, created_at
             FROM terraform_state
             WHERE project_id = $1 AND workspace = $2
             ORDER BY serial DESC",
        )
        .bind(project_id)
        .bind(workspace)
        .fetch_all(pool)
        .await
        .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|r| TerraformStateSummary {
                id:         r.get("id"),
                project_id: r.get("project_id"),
                workspace:  r.get("workspace"),
                serial:     r.get("serial"),
                lineage:    r.get("lineage"),
                encrypted:  r.get("encrypted"),
                md5:        r.get("md5"),
                created_at: r.get("created_at"),
            })
            .collect())
    }

    async fn get_terraform_state_by_serial(
        &self,
        project_id: i32,
        workspace: &str,
        serial: i32,
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

        Ok(row.map(|r| TerraformState {
            id:         r.get("id"),
            project_id: r.get("project_id"),
            workspace:  r.get("workspace"),
            serial:     r.get("serial"),
            lineage:    r.get("lineage"),
            state_data: r.get("state_data"),
            encrypted:  r.get("encrypted"),
            md5:        r.get("md5"),
            created_at: r.get("created_at"),
        }))
    }

    // ── Write ────────────────────────────────────────────────────────────────

    async fn create_terraform_state(&self, state: TerraformState) -> Result<TerraformState> {
        let pool = self.get_postgres_pool()?;

        // Idempotency: same serial + same md5 → return existing row.
        let existing = sqlx::query(
            "SELECT id, project_id, workspace, serial, lineage, state_data, encrypted, md5, created_at
             FROM terraform_state
             WHERE project_id = $1 AND workspace = $2 AND serial = $3",
        )
        .bind(state.project_id)
        .bind(&state.workspace)
        .bind(state.serial)
        .fetch_optional(pool)
        .await
        .map_err(Error::Database)?;

        if let Some(r) = existing {
            let existing_md5: String = r.get("md5");
            if existing_md5 == state.md5 {
                // Idempotent retry — same content.
                return Ok(TerraformState {
                    id:         r.get("id"),
                    project_id: r.get("project_id"),
                    workspace:  r.get("workspace"),
                    serial:     r.get("serial"),
                    lineage:    r.get("lineage"),
                    state_data: r.get("state_data"),
                    encrypted:  r.get("encrypted"),
                    md5:        existing_md5,
                    created_at: r.get("created_at"),
                });
            }
            // Same serial, different content → conflict.
            return Err(Error::Other(format!(
                "serial {} already exists with different content",
                state.serial
            )));
        }

        let row = sqlx::query(
            "INSERT INTO terraform_state
               (project_id, workspace, serial, lineage, state_data, encrypted, md5)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, project_id, workspace, serial, lineage, state_data, encrypted, md5, created_at",
        )
        .bind(state.project_id)
        .bind(&state.workspace)
        .bind(state.serial)
        .bind(&state.lineage)
        .bind(&state.state_data)
        .bind(state.encrypted)
        .bind(&state.md5)
        .fetch_one(pool)
        .await
        .map_err(Error::Database)?;

        Ok(TerraformState {
            id:         row.get("id"),
            project_id: row.get("project_id"),
            workspace:  row.get("workspace"),
            serial:     row.get("serial"),
            lineage:    row.get("lineage"),
            state_data: row.get("state_data"),
            encrypted:  row.get("encrypted"),
            md5:        row.get("md5"),
            created_at: row.get("created_at"),
        })
    }

    async fn delete_terraform_state(&self, project_id: i32, workspace: &str) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        // Delete only the latest version.
        sqlx::query(
            "DELETE FROM terraform_state
             WHERE id = (
               SELECT id FROM terraform_state
               WHERE project_id = $1 AND workspace = $2
               ORDER BY serial DESC LIMIT 1
             )",
        )
        .bind(project_id)
        .bind(workspace)
        .execute(pool)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_all_terraform_states(&self, project_id: i32, workspace: &str) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        sqlx::query(
            "DELETE FROM terraform_state WHERE project_id = $1 AND workspace = $2",
        )
        .bind(project_id)
        .bind(workspace)
        .execute(pool)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    // ── Locking ──────────────────────────────────────────────────────────────

    /// Atomic lock acquisition:
    ///   BEGIN
    ///   DELETE expired locks for this workspace
    ///   INSERT … ON CONFLICT DO NOTHING RETURNING *
    ///   COMMIT
    ///
    /// If INSERT returns nothing the workspace is already locked by someone else.
    async fn lock_terraform_state(
        &self,
        project_id: i32,
        workspace: &str,
        lock: TerraformStateLock,
    ) -> Result<TerraformStateLock> {
        let pool = self.get_postgres_pool()?;
        let mut tx = pool.begin().await.map_err(Error::Database)?;

        // 1. Purge expired lock for this workspace only.
        sqlx::query(
            "DELETE FROM terraform_state_lock
             WHERE project_id = $1 AND workspace = $2 AND expires_at < NOW()",
        )
        .bind(project_id)
        .bind(workspace)
        .execute(&mut *tx)
        .await
        .map_err(Error::Database)?;

        // 2. Try to insert our lock — ON CONFLICT means already locked.
        let row = sqlx::query(
            "INSERT INTO terraform_state_lock
               (project_id, workspace, lock_id, operation, info, who, version, path, expires_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW() + INTERVAL '2 hours')
             ON CONFLICT (project_id, workspace) DO NOTHING
             RETURNING project_id, workspace, lock_id, operation, info, who, version, path, created_at, expires_at",
        )
        .bind(project_id)
        .bind(workspace)
        .bind(&lock.lock_id)
        .bind(&lock.operation)
        .bind(&lock.info)
        .bind(&lock.who)
        .bind(&lock.version)
        .bind(&lock.path)
        .fetch_optional(&mut *tx)
        .await
        .map_err(Error::Database)?;

        tx.commit().await.map_err(Error::Database)?;

        match row {
            Some(r) => Ok(TerraformStateLock {
                project_id: r.get("project_id"),
                workspace:  r.get("workspace"),
                lock_id:    r.get("lock_id"),
                operation:  r.get("operation"),
                info:       r.get("info"),
                who:        r.get("who"),
                version:    r.get("version"),
                path:       r.get("path"),
                created_at: r.get("created_at"),
                expires_at: r.get("expires_at"),
            }),
            None => {
                // Workspace is locked — fetch the current lock for 423 response body.
                let existing = self.get_terraform_lock(project_id, workspace).await?;
                Err(Error::Other(format!(
                    "locked:{}",
                    serde_json::to_string(
                        &existing.map(|l| crate::models::LockInfo::from_lock(&l))
                    )
                    .unwrap_or_default()
                )))
            }
        }
    }

    async fn unlock_terraform_state(
        &self,
        project_id: i32,
        workspace: &str,
        lock_id: &str,
    ) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        let result = sqlx::query(
            "DELETE FROM terraform_state_lock
             WHERE project_id = $1 AND workspace = $2 AND lock_id = $3",
        )
        .bind(project_id)
        .bind(workspace)
        .bind(lock_id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound(format!(
                "lock {} not found for workspace {}",
                lock_id, workspace
            )));
        }
        Ok(())
    }

    async fn get_terraform_lock(
        &self,
        project_id: i32,
        workspace: &str,
    ) -> Result<Option<TerraformStateLock>> {
        let pool = self.get_postgres_pool()?;
        let row = sqlx::query(
            "SELECT project_id, workspace, lock_id, operation, info, who, version, path, created_at, expires_at
             FROM terraform_state_lock
             WHERE project_id = $1 AND workspace = $2 AND expires_at > NOW()",
        )
        .bind(project_id)
        .bind(workspace)
        .fetch_optional(pool)
        .await
        .map_err(Error::Database)?;

        Ok(row.map(|r| TerraformStateLock {
            project_id: r.get("project_id"),
            workspace:  r.get("workspace"),
            lock_id:    r.get("lock_id"),
            operation:  r.get("operation"),
            info:       r.get("info"),
            who:        r.get("who"),
            version:    r.get("version"),
            path:       r.get("path"),
            created_at: r.get("created_at"),
            expires_at: r.get("expires_at"),
        }))
    }

    // ── Workspaces ───────────────────────────────────────────────────────────

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

    // ── Maintenance ──────────────────────────────────────────────────────────

    async fn purge_expired_terraform_locks(&self) -> Result<u64> {
        let pool = self.get_postgres_pool()?;
        let result = sqlx::query(
            "DELETE FROM terraform_state_lock WHERE expires_at < NOW()",
        )
        .execute(pool)
        .await
        .map_err(Error::Database)?;
        Ok(result.rows_affected())
    }
}
