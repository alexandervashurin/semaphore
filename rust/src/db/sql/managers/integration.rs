//! IntegrationManager - управление интеграциями

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::Integration;
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl IntegrationManager for SqlStore {
    async fn get_integrations(&self, project_id: i32) -> Result<Vec<Integration>> {
        let pool = self.get_postgres_pool()?;
        let rows = sqlx::query(
            "SELECT id, project_id, name, template_id, auth_method, auth_header, auth_secret_id FROM integration WHERE project_id = $1 ORDER BY name"
        )
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(|row| Integration {
            id: row.get("id"),
            project_id: row.get("project_id"),
            name: row.get("name"),
            template_id: row.try_get("template_id").unwrap_or(0),
            auth_method: row.try_get("auth_method").unwrap_or_default(),
            auth_header: row.try_get("auth_header").ok().flatten(),
            auth_secret_id: row.try_get("auth_secret_id").ok().flatten(),
        }).collect())
    }

    async fn get_integration(&self, project_id: i32, integration_id: i32) -> Result<Integration> {
        let pool = self.get_postgres_pool()?;
        let row = sqlx::query(
            "SELECT id, project_id, name, template_id, auth_method, auth_header, auth_secret_id FROM integration WHERE id = $1 AND project_id = $2"
        )
        .bind(integration_id)
        .bind(project_id)
        .fetch_optional(pool)
        .await
        .map_err(Error::Database)?
        .ok_or_else(|| Error::NotFound("Интеграция не найдена".to_string()))?;

        Ok(Integration {
            id: row.get("id"),
            project_id: row.get("project_id"),
            name: row.get("name"),
            template_id: row.try_get("template_id").unwrap_or(0),
            auth_method: row.try_get("auth_method").unwrap_or_default(),
            auth_header: row.try_get("auth_header").ok().flatten(),
            auth_secret_id: row.try_get("auth_secret_id").ok().flatten(),
        })
    }

    async fn create_integration(&self, mut integration: Integration) -> Result<Integration> {
        let pool = self.get_postgres_pool()?;
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO integration (project_id, name, template_id, auth_method, auth_header, auth_secret_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"
        )
        .bind(integration.project_id)
        .bind(&integration.name)
        .bind(integration.template_id)
        .bind(&integration.auth_method)
        .bind(&integration.auth_header)
        .bind(integration.auth_secret_id)
        .fetch_one(pool)
        .await
        .map_err(Error::Database)?;

        integration.id = id;
        Ok(integration)
    }

    async fn update_integration(&self, integration: Integration) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        sqlx::query(
            "UPDATE integration SET name = $1, template_id = $2, auth_method = $3, auth_header = $4, auth_secret_id = $5 WHERE id = $6 AND project_id = $7"
        )
        .bind(&integration.name)
        .bind(integration.template_id)
        .bind(&integration.auth_method)
        .bind(&integration.auth_header)
        .bind(integration.auth_secret_id)
        .bind(integration.id)
        .bind(integration.project_id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_integration(&self, project_id: i32, integration_id: i32) -> Result<()> {
        let pool = self.get_postgres_pool()?;
        sqlx::query("DELETE FROM integration WHERE id = $1 AND project_id = $2")
            .bind(integration_id)
            .bind(project_id)
            .execute(pool)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}
