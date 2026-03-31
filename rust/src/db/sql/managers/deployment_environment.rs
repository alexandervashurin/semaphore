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
