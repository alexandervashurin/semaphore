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
            "SELECT * FROM drift_config WHERE project_id = $1 ORDER BY id",
        )
        .bind(project_id)
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(rows)
    }

    async fn get_drift_config(&self, id: i32, project_id: i32) -> Result<DriftConfig> {
        let row = sqlx::query_as::<_, DriftConfig>(
            "SELECT * FROM drift_config WHERE id = $1 AND project_id = $2",
        )
        .bind(id)
        .bind(project_id)
        .fetch_one(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(row)
    }

    async fn create_drift_config(
        &self,
        project_id: i32,
        payload: DriftConfigCreate,
    ) -> Result<DriftConfig> {
        let enabled = payload.enabled.unwrap_or(true);
        let row = sqlx::query_as::<_, DriftConfig>(
            "INSERT INTO drift_config (project_id, template_id, enabled, schedule, created)
                 VALUES ($1, $2, $3, $4, NOW()) RETURNING *",
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

    async fn update_drift_config_enabled(
        &self,
        id: i32,
        project_id: i32,
        enabled: bool,
    ) -> Result<()> {
        sqlx::query("UPDATE drift_config SET enabled = $1 WHERE id = $2 AND project_id = $3")
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

    async fn get_drift_results(
        &self,
        drift_config_id: i32,
        limit: i64,
    ) -> Result<Vec<DriftResult>> {
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
                 ORDER BY drift_config_id, checked_at DESC",
        )
        .bind(project_id)
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::drift::{
        DriftConfig, DriftConfigCreate, DriftConfigUpdate, DriftConfigWithStatus, DriftResult,
    };
    use chrono::Utc;

    #[test]
    fn test_drift_config_serialization() {
        let config = DriftConfig {
            id: 1,
            project_id: 10,
            template_id: 5,
            enabled: true,
            schedule: Some("0 * * * *".to_string()),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"schedule\":\"0 * * * *\""));
    }

    #[test]
    fn test_drift_config_skip_null_schedule() {
        let config = DriftConfig {
            id: 1,
            project_id: 10,
            template_id: 5,
            enabled: true,
            schedule: None,
            created: Utc::now(),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(!json.contains("\"schedule\":"));
    }

    #[test]
    fn test_drift_config_create_serialization() {
        let create = DriftConfigCreate {
            template_id: 5,
            enabled: Some(true),
            schedule: Some("daily".to_string()),
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"template_id\":5"));
    }

    #[test]
    fn test_drift_config_create_defaults() {
        let create = DriftConfigCreate {
            template_id: 1,
            enabled: None,
            schedule: None,
        };
        assert!(create.enabled.is_none());
        assert!(create.schedule.is_none());
    }

    #[test]
    fn test_drift_config_update_serialization() {
        let update = DriftConfigUpdate {
            enabled: Some(false),
            schedule: None,
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"enabled\":false"));
        // DriftConfigUpdate doesn't have skip_serializing_if on schedule
        assert!(json.contains("\"schedule\":null"));
    }

    #[test]
    fn test_drift_result_serialization() {
        let result = DriftResult {
            id: 1,
            drift_config_id: 10,
            project_id: 5,
            template_id: 3,
            status: "drifted".to_string(),
            summary: Some("3 resources changed".to_string()),
            task_id: Some(100),
            checked_at: Utc::now(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"status\":\"drifted\""));
        assert!(json.contains("\"summary\":\"3 resources changed\""));
    }

    #[test]
    fn test_drift_result_skip_nulls() {
        let result = DriftResult {
            id: 1,
            drift_config_id: 10,
            project_id: 5,
            template_id: 3,
            status: "clean".to_string(),
            summary: None,
            task_id: None,
            checked_at: Utc::now(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("\"summary\":"));
        assert!(!json.contains("\"task_id\":"));
    }

    #[test]
    fn test_drift_config_with_status_serialization() {
        let config = DriftConfig {
            id: 1,
            project_id: 10,
            template_id: 5,
            enabled: true,
            schedule: None,
            created: Utc::now(),
        };
        let with_status = DriftConfigWithStatus {
            config,
            latest_result: None,
        };
        let json = serde_json::to_string(&with_status).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(!json.contains("\"latest_result\":"));
    }

    #[test]
    fn test_drift_config_with_status_with_result() {
        let config = DriftConfig {
            id: 1,
            project_id: 10,
            template_id: 5,
            enabled: true,
            schedule: None,
            created: Utc::now(),
        };
        let result = DriftResult {
            id: 1,
            drift_config_id: 1,
            project_id: 10,
            template_id: 5,
            status: "drifted".to_string(),
            summary: Some("changed".to_string()),
            task_id: None,
            checked_at: Utc::now(),
        };
        let with_status = DriftConfigWithStatus {
            config,
            latest_result: Some(result),
        };
        let json = serde_json::to_string(&with_status).unwrap();
        assert!(json.contains("\"status\":\"drifted\""));
        assert!(json.contains("\"summary\":\"changed\""));
    }

    #[test]
    fn test_drift_config_clone() {
        let config = DriftConfig {
            id: 1,
            project_id: 10,
            template_id: 5,
            enabled: true,
            schedule: None,
            created: Utc::now(),
        };
        let cloned = config.clone();
        assert_eq!(cloned.id, config.id);
        assert_eq!(cloned.enabled, config.enabled);
    }

    #[test]
    fn test_drift_result_clone() {
        let result = DriftResult {
            id: 1,
            drift_config_id: 10,
            project_id: 5,
            template_id: 3,
            status: "clean".to_string(),
            summary: None,
            task_id: None,
            checked_at: Utc::now(),
        };
        let cloned = result.clone();
        assert_eq!(cloned.status, result.status);
    }

    #[test]
    fn test_drift_status_variants() {
        let statuses = vec!["clean", "drifted", "error", "pending"];
        for status in statuses {
            let result = DriftResult {
                id: 0,
                drift_config_id: 0,
                project_id: 0,
                template_id: 0,
                status: status.to_string(),
                summary: None,
                task_id: None,
                checked_at: Utc::now(),
            };
            let json = serde_json::to_string(&result).unwrap();
            assert!(json.contains(&format!("\"status\":\"{}\"", status)));
        }
    }

    #[test]
    fn test_drift_config_create_enabled_true() {
        let create = DriftConfigCreate {
            template_id: 1,
            enabled: Some(true),
            schedule: Some("*/5 * * * *".to_string()),
        };
        assert_eq!(create.enabled, Some(true));
    }

    #[test]
    fn test_drift_config_update_both_none() {
        let update = DriftConfigUpdate {
            enabled: None,
            schedule: None,
        };
        assert!(update.enabled.is_none());
        assert!(update.schedule.is_none());
    }
}
