//! Integration Extract Value - операции с IntegrationExtractValue
//!
//! Аналог db/sql/integration.go из Go версии (часть 3: IntegrationExtractValue)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    fn pg_pool_extract(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает все extract values для интеграции
    pub async fn get_integration_extract_values(&self, project_id: i32, integration_id: i32) -> Result<Vec<IntegrationExtractValue>> {
        let rows = sqlx::query(
            "SELECT * FROM integration_extract_value WHERE integration_id = $1 AND project_id = $2"
        )
        .bind(integration_id)
        .bind(project_id)
        .fetch_all(self.pg_pool_extract()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(|row| IntegrationExtractValue {
            id: row.get("id"),
            integration_id: row.get("integration_id"),
            project_id: row.get("project_id"),
            name: row.try_get("name").ok().unwrap_or_default(),
            value_source: row.get("value_source"),
            body_data_type: row.try_get("body_data_type").ok().unwrap_or_default(),
            key: row.try_get("key").ok().flatten(),
            variable: row.try_get("variable").ok().flatten(),
            value_name: row.get("value_name"),
            value_type: row.get("value_type"),
        }).collect())
    }

    /// Создаёт IntegrationExtractValue
    pub async fn create_integration_extract_value(&self, mut value: IntegrationExtractValue) -> Result<IntegrationExtractValue> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO integration_extract_value \
             (integration_id, project_id, name, value_source, body_data_type, key, variable, value_name, value_type) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id"
        )
        .bind(value.integration_id)
        .bind(value.project_id)
        .bind(&value.name)
        .bind(&value.value_source)
        .bind(&value.body_data_type)
        .bind(&value.key)
        .bind(&value.variable)
        .bind(&value.value_name)
        .bind(&value.value_type)
        .fetch_one(self.pg_pool_extract()?)
        .await
        .map_err(Error::Database)?;

        value.id = id;
        Ok(value)
    }

    /// Обновляет IntegrationExtractValue
    pub async fn update_integration_extract_value(&self, value: IntegrationExtractValue) -> Result<()> {
        sqlx::query(
            "UPDATE integration_extract_value SET name = $1, value_source = $2, \
             body_data_type = $3, key = $4, variable = $5, value_name = $6, value_type = $7 \
             WHERE id = $8 AND integration_id = $9 AND project_id = $10"
        )
        .bind(&value.name)
        .bind(&value.value_source)
        .bind(&value.body_data_type)
        .bind(&value.key)
        .bind(&value.variable)
        .bind(&value.value_name)
        .bind(&value.value_type)
        .bind(value.id)
        .bind(value.integration_id)
        .bind(value.project_id)
        .execute(self.pg_pool_extract()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Удаляет IntegrationExtractValue
    pub async fn delete_integration_extract_value(&self, project_id: i32, integration_id: i32, value_id: i32) -> Result<()> {
        sqlx::query(
            "DELETE FROM integration_extract_value WHERE id = $1 AND integration_id = $2 AND project_id = $3"
        )
        .bind(value_id)
        .bind(integration_id)
        .bind(project_id)
        .execute(self.pg_pool_extract()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDb {
        db: SqlDb,
        _temp: tempfile::NamedTempFile,
    }

    async fn create_test_db() -> TestDb {
        let (db_path, temp) = crate::db::sql::init::test_sqlite_url();

        let db = SqlDb::connect_sqlite(&db_path).await.unwrap();

        // Создаём таблицу integration_extract_value
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS integration_extract_value (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                integration_id INTEGER NOT NULL,
                project_id INTEGER NOT NULL,
                name TEXT NOT NULL DEFAULT '',
                value_source TEXT NOT NULL,
                body_data_type TEXT NOT NULL DEFAULT 'json',
                key TEXT,
                variable TEXT,
                value_name TEXT NOT NULL,
                value_type TEXT NOT NULL
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();

        TestDb { db, _temp: temp }
    }

    #[tokio::test]
    async fn test_create_and_get_integration_extract_value() {
        let TestDb { db, _temp } = create_test_db().await;

        let value = IntegrationExtractValue {
            id: 0,
            integration_id: 1,
            project_id: 1,
            name: "Test Value".to_string(),
            value_source: "body".to_string(),
            body_data_type: "json".to_string(),
            key: None,
            variable: None,
            value_name: "task_id".to_string(),
            value_type: "json".to_string(),
        };

        let created = db.create_integration_extract_value(value.clone()).await.unwrap();
        assert!(created.id > 0);

        let values = db.get_integration_extract_values(1, 1).await.unwrap();
        assert!(values.len() >= 1);
        assert_eq!(values[0].value_name, "task_id");

        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_update_integration_extract_value() {
        let TestDb { db, _temp } = create_test_db().await;

        let value = IntegrationExtractValue {
            id: 0,
            integration_id: 1,
            project_id: 1,
            name: "Test Value".to_string(),
            value_source: "body".to_string(),
            body_data_type: "json".to_string(),
            key: None,
            variable: None,
            value_name: "task_id".to_string(),
            value_type: "json".to_string(),
        };

        let created = db.create_integration_extract_value(value).await.unwrap();

        let mut updated = created.clone();
        updated.value_name = "job_id".to_string();

        db.update_integration_extract_value(updated).await.unwrap();

        let values = db.get_integration_extract_values(1, 1).await.unwrap();
        assert_eq!(values[0].value_name, "job_id");

        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_delete_integration_extract_value() {
        let TestDb { db, _temp } = create_test_db().await;

        let value = IntegrationExtractValue {
            id: 0,
            integration_id: 1,
            project_id: 1,
            name: "Test Value".to_string(),
            value_source: "body".to_string(),
            body_data_type: "json".to_string(),
            key: None,
            variable: None,
            value_name: "task_id".to_string(),
            value_type: "json".to_string(),
        };

        let created = db.create_integration_extract_value(value).await.unwrap();

        db.delete_integration_extract_value(1, 1, created.id).await.unwrap();

        let values = db.get_integration_extract_values(1, 1).await.unwrap();
        assert!(values.is_empty());

        // Cleanup
        let _ = db.close().await;
    }
}
