//! Integration Matcher - операции с IntegrationMatcher
//!
//! Аналог db/sql/integration.go из Go версии (часть 2: IntegrationMatcher)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    fn pg_pool_matcher(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает все matcher'ы для интеграции
    pub async fn get_integration_matchers(&self, project_id: i32, integration_id: i32) -> Result<Vec<IntegrationMatcher>> {
        let rows = sqlx::query(
            "SELECT * FROM integration_matcher WHERE integration_id = $1 AND project_id = $2"
        )
        .bind(integration_id)
        .bind(project_id)
        .fetch_all(self.pg_pool_matcher()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(|row| IntegrationMatcher {
            id: row.get("id"),
            integration_id: row.get("integration_id"),
            project_id: row.get("project_id"),
            name: row.try_get("name").ok().unwrap_or_default(),
            body_data_type: row.try_get("body_data_type").ok().unwrap_or_default(),
            key: row.try_get("key").ok().flatten(),
            matcher_type: row.get("matcher_type"),
            matcher_value: row.get("matcher_value"),
            method: row.try_get("method").ok().unwrap_or_default(),
        }).collect())
    }

    /// Создаёт IntegrationMatcher
    pub async fn create_integration_matcher(&self, mut matcher: IntegrationMatcher) -> Result<IntegrationMatcher> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO integration_matcher \
             (integration_id, project_id, name, body_data_type, key, matcher_type, matcher_value, method) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id"
        )
        .bind(matcher.integration_id)
        .bind(matcher.project_id)
        .bind(&matcher.name)
        .bind(&matcher.body_data_type)
        .bind(&matcher.key)
        .bind(&matcher.matcher_type)
        .bind(&matcher.matcher_value)
        .bind(&matcher.method)
        .fetch_one(self.pg_pool_matcher()?)
        .await
        .map_err(Error::Database)?;

        matcher.id = id;
        Ok(matcher)
    }

    /// Обновляет IntegrationMatcher
    pub async fn update_integration_matcher(&self, matcher: IntegrationMatcher) -> Result<()> {
        sqlx::query(
            "UPDATE integration_matcher SET name = $1, body_data_type = $2, key = $3, \
             matcher_type = $4, matcher_value = $5, method = $6 \
             WHERE id = $7 AND integration_id = $8 AND project_id = $9"
        )
        .bind(&matcher.name)
        .bind(&matcher.body_data_type)
        .bind(&matcher.key)
        .bind(&matcher.matcher_type)
        .bind(&matcher.matcher_value)
        .bind(&matcher.method)
        .bind(matcher.id)
        .bind(matcher.integration_id)
        .bind(matcher.project_id)
        .execute(self.pg_pool_matcher()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Удаляет IntegrationMatcher
    pub async fn delete_integration_matcher(&self, project_id: i32, integration_id: i32, matcher_id: i32) -> Result<()> {
        sqlx::query(
            "DELETE FROM integration_matcher WHERE id = $1 AND integration_id = $2 AND project_id = $3"
        )
        .bind(matcher_id)
        .bind(integration_id)
        .bind(project_id)
        .execute(self.pg_pool_matcher()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }
}

