//! Integration CRUD - операции с интеграциями
//!
//! Аналог db/sql/integration.go из Go версии (часть 1: CRUD)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    /// Получает все интеграции проекта
    pub async fn get_integrations(&self, project_id: i32) -> Result<Vec<Integration>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let integrations = sqlx::query_as::<_, Integration>(
                    "SELECT * FROM integration WHERE project_id = ?"
                )
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(integrations)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Получает интеграцию по ID
    pub async fn get_integration(&self, project_id: i32, integration_id: i32) -> Result<Integration> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let integration = sqlx::query_as::<_, Integration>(
                    "SELECT * FROM integration WHERE project_id = ? AND id = ?"
                )
                .bind(project_id)
                .bind(integration_id)
                .fetch_one(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(integration)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Создаёт новую интеграцию
    pub async fn create_integration(&self, mut integration: Integration) -> Result<Integration> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO integration (project_id, name, template_id) VALUES (?, ?, ?)"
                )
                .bind(integration.project_id)
                .bind(&integration.name)
                .bind(integration.template_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                integration.id = result.last_insert_rowid() as i32;
                Ok(integration)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Обновляет интеграцию
    pub async fn update_integration(&self, integration: Integration) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE integration SET name = ?, template_id = ? WHERE id = ? AND project_id = ?"
                )
                .bind(&integration.name)
                .bind(integration.template_id)
                .bind(integration.id)
                .bind(integration.project_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Удаляет интеграцию
    pub async fn delete_integration(&self, project_id: i32, integration_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("DELETE FROM integration WHERE id = ? AND project_id = ?")
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
        let temp_db = env::temp_dir().join("test_integration.db");
        let db_path = temp_db.to_string_lossy().to_string();
        
        let db = SqlDb::connect_sqlite(&db_path).await.unwrap();
        
        // Создаём таблицу integration
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS integration (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                template_id INTEGER
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();
        
        db
    }

    #[tokio::test]
    async fn test_create_and_get_integration() {
        let db = create_test_db().await;
        
        let integration = Integration {
            id: 0,
            project_id: 1,
            name: "Test Integration".to_string(),
            template_id: Some(1),
        };
        
        let created = db.create_integration(integration.clone()).await.unwrap();
        assert!(created.id > 0);
        
        let retrieved = db.get_integration(1, created.id).await.unwrap();
        assert_eq!(retrieved.name, "Test Integration");
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_get_integrations() {
        let db = create_test_db().await;
        
        // Создаём несколько интеграций
        for i in 0..5 {
            let integration = Integration {
                id: 0,
                project_id: 1,
                name: format!("Integration {}", i),
                template_id: Some(1),
            };
            db.create_integration(integration).await.unwrap();
        }
        
        let integrations = db.get_integrations(1).await.unwrap();
        assert!(integrations.len() >= 5);
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_update_integration() {
        let db = create_test_db().await;
        
        let integration = Integration {
            id: 0,
            project_id: 1,
            name: "Test Integration".to_string(),
            template_id: Some(1),
        };
        
        let created = db.create_integration(integration).await.unwrap();
        
        let mut updated = created.clone();
        updated.name = "Updated Integration".to_string();
        
        db.update_integration(updated).await.unwrap();
        
        let retrieved = db.get_integration(1, created.id).await.unwrap();
        assert_eq!(retrieved.name, "Updated Integration");
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_delete_integration() {
        let db = create_test_db().await;
        
        let integration = Integration {
            id: 0,
            project_id: 1,
            name: "Test Integration".to_string(),
            template_id: Some(1),
        };
        
        let created = db.create_integration(integration).await.unwrap();
        
        db.delete_integration(1, created.id).await.unwrap();
        
        let result = db.get_integration(1, created.id).await;
        assert!(result.is_err());
        
        // Cleanup
        let _ = db.close().await;
    }
}
