//! Integration CRUD - операции с интеграциями
//!
//! Аналог db/sql/integration.go из Go версии (часть 1: CRUD)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    fn pg_pool_integration(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает все интеграции проекта
    pub async fn get_integrations(&self, project_id: i32) -> Result<Vec<Integration>> {
        let rows = sqlx::query(
            "SELECT * FROM integration WHERE project_id = $1 ORDER BY name"
        )
        .bind(project_id)
        .fetch_all(self.pg_pool_integration()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(|row| Integration {
            id: row.get("id"),
            project_id: row.get("project_id"),
            name: row.get("name"),
            template_id: row.get("template_id"),
            auth_method: row.try_get("auth_method").ok().unwrap_or_default(),
            auth_header: row.try_get("auth_header").ok().flatten(),
            auth_secret_id: row.try_get("auth_secret_id").ok().flatten(),
        }).collect())
    }

    /// Получает интеграцию по ID
    pub async fn get_integration(&self, project_id: i32, integration_id: i32) -> Result<Integration> {
        let row = sqlx::query(
            "SELECT * FROM integration WHERE id = $1 AND project_id = $2"
        )
        .bind(integration_id)
        .bind(project_id)
        .fetch_one(self.pg_pool_integration()?)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound("Интеграция не найдена".to_string()),
            _ => Error::Database(e),
        })?;

        Ok(Integration {
            id: row.get("id"),
            project_id: row.get("project_id"),
            name: row.get("name"),
            template_id: row.get("template_id"),
            auth_method: row.try_get("auth_method").ok().unwrap_or_default(),
            auth_header: row.try_get("auth_header").ok().flatten(),
            auth_secret_id: row.try_get("auth_secret_id").ok().flatten(),
        })
    }

    /// Создаёт новую интеграцию
    pub async fn create_integration(&self, mut integration: Integration) -> Result<Integration> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO integration (project_id, name, template_id, auth_method, auth_header, auth_secret_id) \
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"
        )
        .bind(integration.project_id)
        .bind(&integration.name)
        .bind(integration.template_id)
        .bind(&integration.auth_method)
        .bind(&integration.auth_header)
        .bind(integration.auth_secret_id)
        .fetch_one(self.pg_pool_integration()?)
        .await
        .map_err(Error::Database)?;

        integration.id = id;
        Ok(integration)
    }

    /// Обновляет интеграцию
    pub async fn update_integration(&self, integration: Integration) -> Result<()> {
        sqlx::query(
            "UPDATE integration SET name = $1, template_id = $2, auth_method = $3, \
             auth_header = $4, auth_secret_id = $5 WHERE id = $6 AND project_id = $7"
        )
        .bind(&integration.name)
        .bind(integration.template_id)
        .bind(&integration.auth_method)
        .bind(&integration.auth_header)
        .bind(integration.auth_secret_id)
        .bind(integration.id)
        .bind(integration.project_id)
        .execute(self.pg_pool_integration()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Удаляет интеграцию
    pub async fn delete_integration(&self, project_id: i32, integration_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM integration WHERE id = $1 AND project_id = $2")
            .bind(integration_id)
            .bind(project_id)
            .execute(self.pg_pool_integration()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

