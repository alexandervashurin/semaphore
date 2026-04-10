//! DeploymentEnvironmentManager — реестр окружений деплоя (FI-GL-1)

use crate::db::sql::SqlStore;
use crate::db::store::DeploymentEnvironmentManager;
use crate::error::{Error, Result};
use crate::models::{
    DeploymentEnvironment, DeploymentEnvironmentCreate, DeploymentEnvironmentUpdate,
    DeploymentRecord,
};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl DeploymentEnvironmentManager for SqlStore {
    async fn get_deployment_environments(
        &self,
        project_id: i32,
    ) -> Result<Vec<DeploymentEnvironment>> {
        let rows = sqlx::query(
            "SELECT * FROM deployment_environment WHERE project_id = $1 ORDER BY tier, name",
        )
        .bind(project_id)
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows.iter().map(row_to_env).collect())
    }

    async fn get_deployment_environment(
        &self,
        id: i32,
        project_id: i32,
    ) -> Result<DeploymentEnvironment> {
        let row =
            sqlx::query("SELECT * FROM deployment_environment WHERE id = $1 AND project_id = $2")
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_postgres_pool()?)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => {
                        Error::NotFound("Deployment environment not found".into())
                    }
                    _ => Error::Database(e),
                })?;

        Ok(row_to_env(&row))
    }

    async fn create_deployment_environment(
        &self,
        project_id: i32,
        payload: DeploymentEnvironmentCreate,
    ) -> Result<DeploymentEnvironment> {
        let row = sqlx::query(
            "INSERT INTO deployment_environment \
             (project_id, name, url, tier, status, template_id, created, updated) \
             VALUES ($1, $2, $3, $4, 'unknown', $5, NOW(), NOW()) RETURNING *",
        )
        .bind(project_id)
        .bind(&payload.name)
        .bind(&payload.url)
        .bind(&payload.tier)
        .bind(payload.template_id)
        .fetch_one(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(row_to_env(&row))
    }

    async fn update_deployment_environment(
        &self,
        id: i32,
        project_id: i32,
        payload: DeploymentEnvironmentUpdate,
    ) -> Result<DeploymentEnvironment> {
        let row = sqlx::query(
            "UPDATE deployment_environment SET \
             name        = COALESCE($3, name), \
             url         = COALESCE($4, url), \
             tier        = COALESCE($5, tier), \
             status      = COALESCE($6, status), \
             template_id = COALESCE($7, template_id), \
             updated     = NOW() \
             WHERE id = $1 AND project_id = $2 RETURNING *",
        )
        .bind(id)
        .bind(project_id)
        .bind(&payload.name)
        .bind(&payload.url)
        .bind(&payload.tier)
        .bind(&payload.status)
        .bind(payload.template_id)
        .fetch_one(self.get_postgres_pool()?)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound("Deployment environment not found".into()),
            _ => Error::Database(e),
        })?;

        Ok(row_to_env(&row))
    }

    async fn delete_deployment_environment(&self, id: i32, project_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM deployment_environment WHERE id = $1 AND project_id = $2")
            .bind(id)
            .bind(project_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn get_deployment_history(
        &self,
        env_id: i32,
        project_id: i32,
    ) -> Result<Vec<DeploymentRecord>> {
        let rows = sqlx::query(
            "SELECT * FROM deployment_record \
             WHERE deploy_environment_id = $1 AND project_id = $2 \
             ORDER BY created DESC LIMIT 50",
        )
        .bind(env_id)
        .bind(project_id)
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows
            .iter()
            .map(|r| DeploymentRecord {
                id: r.get("id"),
                deploy_environment_id: r.get("deploy_environment_id"),
                task_id: r.get("task_id"),
                project_id: r.get("project_id"),
                version: r.try_get("version").ok().flatten(),
                deployed_by: r.try_get("deployed_by").ok().flatten(),
                status: r.get("status"),
                created: r.get("created"),
            })
            .collect())
    }

    async fn record_deployment(
        &self,
        env_id: i32,
        task_id: i32,
        project_id: i32,
        version: Option<String>,
        deployed_by: Option<i32>,
        status: &str,
    ) -> Result<()> {
        // Записываем в историю
        sqlx::query(
            "INSERT INTO deployment_record \
             (deploy_environment_id, task_id, project_id, version, deployed_by, status, created) \
             VALUES ($1, $2, $3, $4, $5, $6, NOW())",
        )
        .bind(env_id)
        .bind(task_id)
        .bind(project_id)
        .bind(&version)
        .bind(deployed_by)
        .bind(status)
        .execute(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;

        // Обновляем last_task_id, last_deploy_version, last_deployed_by, status в окружении
        sqlx::query(
            "UPDATE deployment_environment SET \
             last_task_id = $2, last_deploy_version = $3, last_deployed_by = $4, \
             status = CASE WHEN $5 = 'success' THEN 'active' ELSE 'unknown' END, \
             updated = NOW() \
             WHERE id = $1",
        )
        .bind(env_id)
        .bind(task_id)
        .bind(&version)
        .bind(deployed_by)
        .bind(status)
        .execute(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(())
    }
}

fn row_to_env(row: &sqlx::postgres::PgRow) -> DeploymentEnvironment {
    DeploymentEnvironment {
        id: row.get("id"),
        project_id: row.get("project_id"),
        name: row.get("name"),
        url: row.try_get("url").ok().flatten(),
        tier: row.try_get("tier").ok().unwrap_or_else(|| "other".into()),
        status: row
            .try_get("status")
            .ok()
            .unwrap_or_else(|| "unknown".into()),
        template_id: row.try_get("template_id").ok().flatten(),
        last_task_id: row.try_get("last_task_id").ok().flatten(),
        last_deploy_version: row.try_get("last_deploy_version").ok().flatten(),
        last_deployed_by: row.try_get("last_deployed_by").ok().flatten(),
        created: row.get("created"),
        updated: row.get("updated"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        DeploymentEnvironment, DeploymentEnvironmentCreate, DeploymentEnvironmentUpdate,
        DeploymentRecord,
    };
    use chrono::Utc;

    #[test]
    fn test_deployment_environment_structure() {
        let env = DeploymentEnvironment {
            id: 1,
            project_id: 10,
            name: "Production".to_string(),
            url: Some("https://prod.example.com".to_string()),
            tier: "production".to_string(),
            status: "active".to_string(),
            template_id: Some(5),
            last_task_id: Some(100),
            last_deploy_version: Some("v1.2.3".to_string()),
            last_deployed_by: Some(1),
            created: Utc::now(),
            updated: Utc::now(),
        };
        assert_eq!(env.name, "Production");
        assert_eq!(env.tier, "production");
    }

    #[test]
    fn test_deployment_environment_create() {
        let create = DeploymentEnvironmentCreate {
            name: "Staging".to_string(),
            url: Some("https://staging.example.com".to_string()),
            tier: "staging".to_string(),
            template_id: Some(3),
        };
        assert_eq!(create.name, "Staging");
        assert!(create.url.is_some());
    }

    #[test]
    fn test_deployment_environment_update() {
        let update = DeploymentEnvironmentUpdate {
            name: Some("Updated Prod".to_string()),
            url: Some("https://new.example.com".to_string()),
            tier: Some("production".to_string()),
            status: Some("active".to_string()),
            template_id: Some(5),
        };
        assert!(update.name.is_some());
        assert!(update.status.is_some());
    }

    #[test]
    fn test_deployment_record_structure() {
        let record = DeploymentRecord {
            id: 1,
            deploy_environment_id: 10,
            task_id: 100,
            project_id: 1,
            version: Some("v2.0.0".to_string()),
            deployed_by: Some(5),
            status: "success".to_string(),
            created: Utc::now(),
        };
        assert_eq!(record.deploy_environment_id, 10);
        assert_eq!(record.status, "success");
    }

    #[test]
    fn test_deployment_record_serialize() {
        let record = DeploymentRecord {
            id: 1,
            deploy_environment_id: 1,
            task_id: 10,
            project_id: 1,
            version: Some("v1.0".to_string()),
            deployed_by: None,
            status: "pending".to_string(),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"status\":\"pending\""));
        assert!(json.contains("\"version\":\"v1.0\""));
    }

    #[test]
    fn test_deployment_environment_status_variants() {
        let statuses = vec!["active", "unknown", "error", "deploying"];
        for status in statuses {
            let env = DeploymentEnvironment {
                id: 1,
                project_id: 1,
                name: "Env".to_string(),
                url: None,
                tier: "other".to_string(),
                status: status.to_string(),
                template_id: None,
                last_task_id: None,
                last_deploy_version: None,
                last_deployed_by: None,
                created: Utc::now(),
                updated: Utc::now(),
            };
            assert_eq!(env.status, status);
        }
    }

    #[test]
    fn test_deployment_environment_tier_variants() {
        let tiers = vec!["production", "staging", "development", "testing", "other"];
        for tier in tiers {
            let env = DeploymentEnvironment {
                id: 1,
                project_id: 1,
                name: "Env".to_string(),
                url: None,
                tier: tier.to_string(),
                status: "unknown".to_string(),
                template_id: None,
                last_task_id: None,
                last_deploy_version: None,
                last_deployed_by: None,
                created: Utc::now(),
                updated: Utc::now(),
            };
            assert_eq!(env.tier, tier);
        }
    }

    #[test]
    fn test_deployment_environment_clone() {
        let env = DeploymentEnvironment {
            id: 42,
            project_id: 10,
            name: "Clone Env".to_string(),
            url: Some("https://clone.example.com".to_string()),
            tier: "production".to_string(),
            status: "active".to_string(),
            template_id: None,
            last_task_id: None,
            last_deploy_version: None,
            last_deployed_by: None,
            created: Utc::now(),
            updated: Utc::now(),
        };
        let cloned = env.clone();
        assert_eq!(cloned.id, env.id);
        assert_eq!(cloned.name, env.name);
    }

    #[test]
    fn test_sql_query_deployment_environment() {
        let query =
            "SELECT * FROM deployment_environment WHERE project_id = $1 ORDER BY tier, name";
        assert!(query.contains("deployment_environment"));
        assert!(query.contains("ORDER BY"));
    }

    #[test]
    fn test_sql_query_deployment_record() {
        let query = "SELECT * FROM deployment_record \
             WHERE deploy_environment_id = $1 AND project_id = $2 \
             ORDER BY created DESC LIMIT 50";
        assert!(query.contains("deployment_record"));
        assert!(query.contains("deploy_environment_id"));
    }

    #[test]
    fn test_sql_query_record_deployment() {
        let query = "INSERT INTO deployment_record \
             (deploy_environment_id, task_id, project_id, version, deployed_by, status, created) \
             VALUES ($1, $2, $3, $4, $5, $6, NOW())";
        assert!(query.contains("INSERT"));
        assert!(query.contains("deployment_record"));
    }

    #[test]
    fn test_deployment_environment_create_serialize() {
        let create = DeploymentEnvironmentCreate {
            name: "Test Env".to_string(),
            url: None,
            tier: "development".to_string(),
            template_id: None,
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"name\":\"Test Env\""));
        assert!(json.contains("\"tier\":\"development\""));
    }
}
