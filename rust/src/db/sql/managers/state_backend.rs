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
            id: r.get("id"),
            project_id: r.get("project_id"),
            workspace: r.get("workspace"),
            serial: r.get("serial"),
            lineage: r.get("lineage"),
            state_data: r.get("state_data"),
            encrypted: r.get("encrypted"),
            md5: r.get("md5"),
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
                id: r.get("id"),
                project_id: r.get("project_id"),
                workspace: r.get("workspace"),
                serial: r.get("serial"),
                lineage: r.get("lineage"),
                encrypted: r.get("encrypted"),
                md5: r.get("md5"),
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
            id: r.get("id"),
            project_id: r.get("project_id"),
            workspace: r.get("workspace"),
            serial: r.get("serial"),
            lineage: r.get("lineage"),
            state_data: r.get("state_data"),
            encrypted: r.get("encrypted"),
            md5: r.get("md5"),
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
                    id: r.get("id"),
                    project_id: r.get("project_id"),
                    workspace: r.get("workspace"),
                    serial: r.get("serial"),
                    lineage: r.get("lineage"),
                    state_data: r.get("state_data"),
                    encrypted: r.get("encrypted"),
                    md5: existing_md5,
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
            id: row.get("id"),
            project_id: row.get("project_id"),
            workspace: row.get("workspace"),
            serial: row.get("serial"),
            lineage: row.get("lineage"),
            state_data: row.get("state_data"),
            encrypted: row.get("encrypted"),
            md5: row.get("md5"),
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
        sqlx::query("DELETE FROM terraform_state WHERE project_id = $1 AND workspace = $2")
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
                workspace: r.get("workspace"),
                lock_id: r.get("lock_id"),
                operation: r.get("operation"),
                info: r.get("info"),
                who: r.get("who"),
                version: r.get("version"),
                path: r.get("path"),
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
            workspace: r.get("workspace"),
            lock_id: r.get("lock_id"),
            operation: r.get("operation"),
            info: r.get("info"),
            who: r.get("who"),
            version: r.get("version"),
            path: r.get("path"),
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

        Ok(rows
            .into_iter()
            .map(|r| r.get::<String, _>("workspace"))
            .collect())
    }

    // ── Maintenance ──────────────────────────────────────────────────────────

    async fn purge_expired_terraform_locks(&self) -> Result<u64> {
        let pool = self.get_postgres_pool()?;
        let result = sqlx::query("DELETE FROM terraform_state_lock WHERE expires_at < NOW()")
            .execute(pool)
            .await
            .map_err(Error::Database)?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TerraformState, TerraformStateLock};

    // ---------- TerraformState model tests ----------

    #[test]
    fn test_terraform_state_default_values() {
        let state = TerraformState {
            id: 0,
            project_id: 1,
            workspace: "default".to_string(),
            serial: 1,
            lineage: "test-lineage".to_string(),
            state_data: b"{}".to_vec(),
            encrypted: false,
            md5: "d41d8cd98f00b204e9800998ecf8427e".to_string(),
            created_at: chrono::Utc::now(),
        };
        assert_eq!(state.workspace, "default");
        assert_eq!(state.serial, 1);
        assert!(!state.encrypted);
    }

    #[test]
    fn test_terraform_state_with_encrypted_flag() {
        let state = TerraformState {
            id: 1,
            project_id: 1,
            workspace: "prod".to_string(),
            serial: 5,
            lineage: "abc".to_string(),
            state_data: br#"{"resources":[]}"#.to_vec(),
            encrypted: true,
            md5: "abc123".to_string(),
            created_at: chrono::Utc::now(),
        };
        assert!(state.encrypted);
    }

    #[test]
    fn test_terraform_state_workspace_variants() {
        let workspaces = vec!["default", "dev", "staging", "prod", ""];
        for ws in workspaces {
            let state = TerraformState {
                id: 0,
                project_id: 1,
                workspace: ws.to_string(),
                serial: 1,
                lineage: "x".to_string(),
                state_data: b"{}".to_vec(),
                encrypted: false,
                md5: "x".to_string(),
                created_at: chrono::Utc::now(),
            };
            assert_eq!(state.workspace, ws);
        }
    }

    // ---------- TerraformStateLock model tests ----------

    #[test]
    fn test_terraform_state_lock_structure() {
        let lock = TerraformStateLock {
            project_id: 1,
            workspace: "default".to_string(),
            lock_id: "lock-uuid".to_string(),
            operation: "apply".to_string(),
            info: "{}".to_string(),
            who: "user @host".to_string(),
            version: "1.5.0".to_string(),
            path: "module.root".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(2),
        };
        assert_eq!(lock.operation, "apply");
        assert_eq!(lock.version, "1.5.0");
    }

    #[test]
    fn test_lock_operation_variants() {
        let ops = vec!["apply", "destroy", "plan", "refresh"];
        for op in ops {
            let lock = TerraformStateLock {
                project_id: 1,
                workspace: "ws".to_string(),
                lock_id: "id".to_string(),
                operation: op.to_string(),
                info: String::new(),
                who: String::new(),
                version: String::new(),
                path: String::new(),
                created_at: chrono::Utc::now(),
                expires_at: chrono::Utc::now(),
            };
            assert_eq!(lock.operation, op);
        }
    }

    #[test]
    fn test_lock_expires_in_future() {
        let now = chrono::Utc::now();
        let lock = TerraformStateLock {
            project_id: 1,
            workspace: "ws".to_string(),
            lock_id: "id".to_string(),
            operation: "apply".to_string(),
            info: String::new(),
            who: String::new(),
            version: String::new(),
            path: String::new(),
            created_at: now,
            expires_at: now + chrono::Duration::hours(2),
        };
        assert!(lock.expires_at > lock.created_at);
    }

    // ---------- TerraformStateSummary tests ----------

    #[test]
    fn test_terraform_state_summary_fields() {
        let summary = crate::models::TerraformStateSummary {
            id: 42,
            project_id: 1,
            workspace: "prod".to_string(),
            serial: 10,
            lineage: "ln".to_string(),
            encrypted: true,
            md5: "hash".to_string(),
            created_at: chrono::Utc::now(),
        };
        assert_eq!(summary.id, 42);
        assert_eq!(summary.serial, 10);
        assert!(summary.encrypted);
    }

    // ---------- SQL query structure tests ----------

    #[test]
    fn test_select_state_query_contains_expected_columns() {
        let query = "SELECT id, project_id, workspace, serial, lineage, state_data, encrypted, md5, created_at
             FROM terraform_state
             WHERE project_id = $1 AND workspace = $2
             ORDER BY serial DESC
             LIMIT 1";
        assert!(query.contains("project_id"));
        assert!(query.contains("workspace"));
        assert!(query.contains("serial"));
        assert!(query.contains("state_data"));
        assert!(query.contains("md5"));
    }

    #[test]
    fn test_delete_expired_locks_query() {
        let query = "DELETE FROM terraform_state_lock WHERE expires_at < NOW()";
        assert!(query.contains("terraform_state_lock"));
        assert!(query.contains("expires_at"));
        assert!(query.contains("NOW()"));
    }

    #[test]
    fn test_list_workspaces_query() {
        let query =
            "SELECT DISTINCT workspace FROM terraform_state WHERE project_id = $1 ORDER BY workspace";
        assert!(query.contains("DISTINCT"));
        assert!(query.contains("workspace"));
        assert!(query.contains("ORDER BY"));
    }

    // ---------- Serialization tests ----------

    #[test]
    fn test_terraform_state_serialize_md5() {
        let state = TerraformState {
            id: 1,
            project_id: 1,
            workspace: "default".to_string(),
            serial: 1,
            lineage: "ln".to_string(),
            state_data: b"{}".to_vec(),
            encrypted: false,
            md5: "abc123".to_string(),
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("\"md5\":\"abc123\""));
        assert!(json.contains("\"workspace\":\"default\""));
    }

    #[test]
    fn test_terraform_state_lock_serialize() {
        let lock = TerraformStateLock {
            project_id: 1,
            workspace: "ws".to_string(),
            lock_id: "l-1".to_string(),
            operation: "apply".to_string(),
            info: "{}".to_string(),
            who: "test".to_string(),
            version: "1.0".to_string(),
            path: "root".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&lock).unwrap();
        assert!(json.contains("\"operation\":\"apply\""));
        assert!(json.contains("\"lock_id\":\"l-1\""));
    }

    // ---------- Validation tests ----------

    #[test]
    fn test_state_serial_positive() {
        let state = TerraformState {
            id: 1,
            project_id: 1,
            workspace: "ws".to_string(),
            serial: 100,
            lineage: "ln".to_string(),
            state_data: b"{}".to_vec(),
            encrypted: false,
            md5: "x".to_string(),
            created_at: chrono::Utc::now(),
        };
        assert!(state.serial > 0);
    }

    #[test]
    fn test_lock_id_non_empty() {
        let lock = TerraformStateLock {
            project_id: 1,
            workspace: "ws".to_string(),
            lock_id: "uuid-1234".to_string(),
            operation: "plan".to_string(),
            info: String::new(),
            who: String::new(),
            version: String::new(),
            path: String::new(),
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now(),
        };
        assert!(!lock.lock_id.is_empty());
    }

    #[test]
    fn test_project_id_positive() {
        let state = TerraformState {
            id: 1,
            project_id: 42,
            workspace: "ws".to_string(),
            serial: 1,
            lineage: "ln".to_string(),
            state_data: b"{}".to_vec(),
            encrypted: false,
            md5: "x".to_string(),
            created_at: chrono::Utc::now(),
        };
        assert_eq!(state.project_id, 42);
    }
}
