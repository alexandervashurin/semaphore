//! Integration Matcher - операции с IntegrationMatcher
//!
//! Аналог db/sql/integration.go из Go версии (часть 2: IntegrationMatcher)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    /// Получает все matcher'ы для интеграции
    pub async fn get_integration_matchers(&self, project_id: i32, integration_id: i32) -> Result<Vec<IntegrationMatcher>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let matchers = sqlx::query_as::<_, IntegrationMatcher>(
                    "SELECT * FROM integration_matcher WHERE integration_id = ? AND project_id = ?"
                )
                .bind(integration_id)
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(matchers)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Создаёт IntegrationMatcher
    pub async fn create_integration_matcher(&self, mut matcher: IntegrationMatcher) -> Result<IntegrationMatcher> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO integration_matcher (integration_id, project_id, name, body_data_type, key, matcher_type, matcher_value, method) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
                )
                .bind(matcher.integration_id)
                .bind(matcher.project_id)
                .bind(&matcher.name)
                .bind(&matcher.body_data_type)
                .bind(&matcher.key)
                .bind(&matcher.matcher_type)
                .bind(&matcher.matcher_value)
                .bind(&matcher.method)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                matcher.id = result.last_insert_rowid() as i32;
                Ok(matcher)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Обновляет IntegrationMatcher
    pub async fn update_integration_matcher(&self, matcher: IntegrationMatcher) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE integration_matcher SET name = ?, body_data_type = ?, key = ?, matcher_type = ?, matcher_value = ?, method = ? WHERE id = ? AND integration_id = ? AND project_id = ?"
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
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Удаляет IntegrationMatcher
    pub async fn delete_integration_matcher(&self, project_id: i32, integration_id: i32, matcher_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("DELETE FROM integration_matcher WHERE id = ? AND integration_id = ? AND project_id = ?")
                    .bind(matcher_id)
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

    struct TestDb {
        db: SqlDb,
        _temp: tempfile::NamedTempFile,
    }

    async fn create_test_db() -> TestDb {
        let (db_path, temp) = crate::db::sql::init::test_sqlite_url();

        let db = SqlDb::connect_sqlite(&db_path).await.unwrap();

        // Создаём таблицу integration_matcher
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS integration_matcher (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                integration_id INTEGER NOT NULL,
                project_id INTEGER NOT NULL,
                name TEXT NOT NULL DEFAULT '',
                body_data_type TEXT NOT NULL DEFAULT 'json',
                key TEXT,
                matcher_type TEXT NOT NULL,
                matcher_value TEXT NOT NULL,
                method TEXT NOT NULL DEFAULT 'GET'
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();

        TestDb { db, _temp: temp }
    }

    #[tokio::test]
    async fn test_create_and_get_integration_matcher() {
        let TestDb { db, _temp } = create_test_db().await;
        
        let matcher = IntegrationMatcher {
            id: 0,
            integration_id: 1,
            project_id: 1,
            name: "Test Matcher".to_string(),
            body_data_type: "json".to_string(),
            key: None,
            matcher_type: "header".to_string(),
            matcher_value: "Content-Type".to_string(),
            method: "GET".to_string(),
        };
        
        let created = db.create_integration_matcher(matcher.clone()).await.unwrap();
        assert!(created.id > 0);
        
        let matchers = db.get_integration_matchers(1, 1).await.unwrap();
        assert!(matchers.len() >= 1);
        assert_eq!(matchers[0].matcher_type, "header");
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_update_integration_matcher() {
        let TestDb { db, _temp } = create_test_db().await;
        
        let matcher = IntegrationMatcher {
            id: 0,
            integration_id: 1,
            project_id: 1,
            name: "Test Matcher".to_string(),
            body_data_type: "json".to_string(),
            key: None,
            matcher_type: "header".to_string(),
            matcher_value: "Content-Type".to_string(),
            method: "GET".to_string(),
        };
        
        let created = db.create_integration_matcher(matcher).await.unwrap();
        
        let mut updated = created.clone();
        updated.matcher_value = "Authorization".to_string();
        
        db.update_integration_matcher(updated).await.unwrap();
        
        let matchers = db.get_integration_matchers(1, 1).await.unwrap();
        assert_eq!(matchers[0].matcher_value, "Authorization");
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_delete_integration_matcher() {
        let TestDb { db, _temp } = create_test_db().await;
        
        let matcher = IntegrationMatcher {
            id: 0,
            integration_id: 1,
            project_id: 1,
            name: "Test Matcher".to_string(),
            body_data_type: "json".to_string(),
            key: None,
            matcher_type: "header".to_string(),
            matcher_value: "Content-Type".to_string(),
            method: "GET".to_string(),
        };
        
        let created = db.create_integration_matcher(matcher).await.unwrap();
        
        db.delete_integration_matcher(1, 1, created.id).await.unwrap();
        
        let matchers = db.get_integration_matchers(1, 1).await.unwrap();
        assert!(matchers.is_empty());
        
        // Cleanup
        let _ = db.close().await;
    }
}
