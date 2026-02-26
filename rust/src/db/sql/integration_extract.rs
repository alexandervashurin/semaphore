//! Integration Extract Value - операции с IntegrationExtractValue
//!
//! Аналог db/sql/integration.go из Go версии (часть 3: IntegrationExtractValue)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    /// Получает все extract values для интеграции
    pub async fn get_integration_extract_values(&self, project_id: i32, integration_id: i32) -> Result<Vec<IntegrationExtractValue>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let values = sqlx::query_as::<_, IntegrationExtractValue>(
                    "SELECT * FROM integration_extract_value WHERE integration_id = ? AND project_id = ?"
                )
                .bind(integration_id)
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(values)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Создаёт IntegrationExtractValue
    pub async fn create_integration_extract_value(&self, mut value: IntegrationExtractValue) -> Result<IntegrationExtractValue> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO integration_extract_value (integration_id, project_id, value_source, value_name, value_type) VALUES (?, ?, ?, ?, ?)"
                )
                .bind(value.integration_id)
                .bind(value.project_id)
                .bind(&value.value_source)
                .bind(&value.value_name)
                .bind(&value.value_type)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                value.id = result.last_insert_rowid() as i32;
                Ok(value)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Обновляет IntegrationExtractValue
    pub async fn update_integration_extract_value(&self, value: IntegrationExtractValue) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE integration_extract_value SET value_source = ?, value_name = ?, value_type = ? WHERE id = ? AND integration_id = ? AND project_id = ?"
                )
                .bind(&value.value_source)
                .bind(&value.value_name)
                .bind(&value.value_type)
                .bind(value.id)
                .bind(value.integration_id)
                .bind(value.project_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Удаляет IntegrationExtractValue
    pub async fn delete_integration_extract_value(&self, project_id: i32, integration_id: i32, value_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("DELETE FROM integration_extract_value WHERE id = ? AND integration_id = ? AND project_id = ?")
                    .bind(value_id)
                    .bind(integration_id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    async fn create_test_db() -> SqlDb {
        let temp_db = env::temp_dir().join("test_integration_extract.db");
        let db_path = temp_db.to_string_lossy().to_string();
        
        let db = SqlDb::connect_sqlite(&db_path).await.unwrap();
        
        // Создаём таблицу integration_extract_value
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS integration_extract_value (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                integration_id INTEGER NOT NULL,
                project_id INTEGER NOT NULL,
                value_source TEXT NOT NULL,
                value_name TEXT NOT NULL,
                value_type TEXT NOT NULL
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();
        
        db
    }

    #[tokio::test]
    async fn test_create_and_get_integration_extract_value() {
        let db = create_test_db().await;
        
        let value = IntegrationExtractValue {
            id: 0,
            integration_id: 1,
            project_id: 1,
            value_source: "body".to_string(),
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
        let db = create_test_db().await;
        
        let value = IntegrationExtractValue {
            id: 0,
            integration_id: 1,
            project_id: 1,
            value_source: "body".to_string(),
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
        let db = create_test_db().await;
        
        let value = IntegrationExtractValue {
            id: 0,
            integration_id: 1,
            project_id: 1,
            value_source: "body".to_string(),
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
