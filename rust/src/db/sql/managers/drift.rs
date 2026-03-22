//! DriftManager - управление GitOps Drift Detection

use crate::db::sql::SqlStore;
use crate::db::store::DriftManager;
use crate::error::{Error, Result};
use crate::models::drift::{DriftConfig, DriftConfigCreate, DriftResult};
use async_trait::async_trait;

#[async_trait]
impl DriftManager for SqlStore {
    async fn get_drift_configs(&self, project_id: i32) -> Result<Vec<DriftConfig>> {
        let rows = sqlx::query_as::<_, DriftConfig>(
                "SELECT * FROM drift_config WHERE project_id = $1 ORDER BY id"
            )
            .bind(project_id)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(rows)
    }

    async fn get_drift_config(&self, id: i32, project_id: i32) -> Result<DriftConfig> {
        let row = sqlx::query_as::<_, DriftConfig>(
                "SELECT * FROM drift_config WHERE id = $1 AND project_id = $2"
            )
            .bind(id)
            .bind(project_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(row)
    }

    async fn create_drift_config(&self, project_id: i32, payload: DriftConfigCreate) -> Result<DriftConfig> {
        let enabled = payload.enabled.unwrap_or(true);
        let row = sqlx::query_as::<_, DriftConfig>(
                "INSERT INTO drift_config (project_id, template_id, enabled, schedule, created)
                 VALUES ($1, $2, $3, $4, NOW()) RETURNING *"
            )
            .bind(project_id)
            .bind(payload.template_id)
            .bind(enabled)
            .bind(&payload.schedule)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(row)
    }

    async fn update_drift_config_enabled(&self, id: i32, project_id: i32, enabled: bool) -> Result<()> {
        sqlx::query(
                "UPDATE drift_config SET enabled = $1 WHERE id = $2 AND project_id = $3"
            )
            .bind(enabled)
            .bind(id)
            .bind(project_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_drift_config(&self, id: i32, project_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM drift_config WHERE id = $1 AND project_id = $2")
                .bind(id)
                .bind(project_id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }

    async fn get_drift_results(&self, drift_config_id: i32, limit: i64) -> Result<Vec<DriftResult>> {
        let rows = sqlx::query_as::<_, DriftResult>(
                "SELECT * FROM drift_result WHERE drift_config_id = $1 ORDER BY checked_at DESC LIMIT $2"
            )
            .bind(drift_config_id)
            .bind(limit)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(rows)
    }

    async fn create_drift_result(
        &self,
        project_id: i32,
        drift_config_id: i32,
        template_id: i32,
        status: &str,
        summary: Option<String>,
        task_id: Option<i32>,
    ) -> Result<DriftResult> {
        let row = sqlx::query_as::<_, DriftResult>(
                "INSERT INTO drift_result (drift_config_id, project_id, template_id, status, summary, task_id, checked_at)
                 VALUES ($1, $2, $3, $4, $5, $6, NOW()) RETURNING *"
            )
            .bind(drift_config_id)
            .bind(project_id)
            .bind(template_id)
            .bind(status)
            .bind(&summary)
            .bind(task_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(row)
    }

    async fn get_latest_drift_results(&self, project_id: i32) -> Result<Vec<DriftResult>> {
        let rows = sqlx::query_as::<_, DriftResult>(
                "SELECT DISTINCT ON (drift_config_id) *
                 FROM drift_result
                 WHERE project_id = $1
                 ORDER BY drift_config_id, checked_at DESC"
            )
            .bind(project_id)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(rows)
    }
}
