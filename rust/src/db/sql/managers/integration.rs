//! IntegrationManager - управление интеграциями

use crate::db::sql::SqlStore;
use crate::db::sql::types::SqlDialect;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::Integration;
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl IntegrationManager for SqlStore {
    async fn get_integrations(&self, project_id: i32) -> Result<Vec<Integration>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?;
                let rows = sqlx::query(
                    "SELECT id, project_id, name, template_id, auth_method, auth_header, auth_secret_id FROM integration WHERE project_id = ? ORDER BY name"
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
            _ => Ok(vec![]),
        }
    }

    async fn get_integration(&self, project_id: i32, integration_id: i32) -> Result<Integration> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?;
                let row = sqlx::query(
                    "SELECT id, project_id, name, template_id, auth_method, auth_header, auth_secret_id FROM integration WHERE id = ? AND project_id = ?"
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
            _ => Err(Error::NotFound("Интеграция не найдена".to_string())),
        }
    }

    async fn create_integration(&self, mut integration: Integration) -> Result<Integration> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?;
                let result = sqlx::query(
                    "INSERT INTO integration (project_id, name, template_id, auth_method, auth_header, auth_secret_id) VALUES (?, ?, ?, ?, ?, ?)"
                )
                .bind(integration.project_id)
                .bind(&integration.name)
                .bind(integration.template_id)
                .bind(&integration.auth_method)
                .bind(&integration.auth_header)
                .bind(integration.auth_secret_id)
                .execute(pool)
                .await
                .map_err(Error::Database)?;

                integration.id = result.last_insert_rowid() as i32;
                Ok(integration)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string())),
        }
    }

    async fn update_integration(&self, integration: Integration) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?;
                sqlx::query(
                    "UPDATE integration SET name = ?, template_id = ?, auth_method = ?, auth_header = ?, auth_secret_id = ? WHERE id = ? AND project_id = ?"
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
            _ => Err(Error::Other("Only SQLite supported for now".to_string())),
        }
    }

    async fn delete_integration(&self, project_id: i32, integration_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?;
                sqlx::query("DELETE FROM integration WHERE id = ? AND project_id = ?")
                    .bind(integration_id)
                    .bind(project_id)
                    .execute(pool)
                    .await
                    .map_err(Error::Database)?;
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string())),
        }
    }
}
