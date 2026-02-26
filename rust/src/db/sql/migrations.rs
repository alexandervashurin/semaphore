//! SQL DB Migrations - миграции схемы БД
//!
//! Аналог db/sql/SqlDb.go из Go версии (часть 3: миграции)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use sqlx::Row;

/// Менеджер миграций
pub struct MigrationManager {
    /// Таблица миграций
    pub table_name: String,
}

impl MigrationManager {
    /// Создаёт новый менеджер миграций
    pub fn new() -> Self {
        Self {
            table_name: "migration".to_string(),
        }
    }
    
    /// Создаёт таблицу миграций если не существует
    pub async fn ensure_migration_table(&self, db: &SqlDb) -> Result<()> {
        match db.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    &format!("CREATE TABLE IF NOT EXISTS {} (version INTEGER PRIMARY KEY, name TEXT)", self.table_name)
                )
                .execute(db.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                sqlx::query(
                    &format!("CREATE TABLE IF NOT EXISTS {} (version BIGINT PRIMARY KEY, name VARCHAR(255))", self.table_name)
                )
                .execute(db.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                sqlx::query(
                    &format!("CREATE TABLE IF NOT EXISTS {} (version BIGINT PRIMARY KEY, name VARCHAR(255))", self.table_name)
                )
                .execute(db.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
            }
        }
        
        Ok(())
    }
    
    /// Проверяет применена ли миграция
    pub async fn is_migration_applied(&self, db: &SqlDb, version: i64) -> Result<bool> {
        match db.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    &format!("SELECT COUNT(*) FROM {} WHERE version = ?", self.table_name)
                )
                .bind(version)
                .fetch_one(db.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                let count: i64 = result.get(0);
                Ok(count > 0)
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                let result = sqlx::query(
                    &format!("SELECT COUNT(*) FROM {} WHERE version = ?", self.table_name)
                )
                .bind(version)
                .fetch_one(db.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                let count: i64 = result.get(0);
                Ok(count > 0)
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                let result = sqlx::query(
                    &format!("SELECT COUNT(*) FROM {} WHERE version = $1", self.table_name)
                )
                .bind(version)
                .fetch_one(db.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                let count: i64 = result.get(0);
                Ok(count > 0)
            }
        }
    }
    
    /// Применяет миграцию
    pub async fn apply_migration(&self, db: &SqlDb, version: i64, name: &str) -> Result<()> {
        match db.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    &format!("INSERT OR REPLACE INTO {} (version, name) VALUES (?, ?)", self.table_name)
                )
                .bind(version)
                .bind(name)
                .execute(db.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                sqlx::query(
                    &format!("INSERT INTO {} (version, name) VALUES (?, ?) ON DUPLICATE KEY UPDATE name = VALUES(name)", self.table_name)
                )
                .bind(version)
                .bind(name)
                .execute(db.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                sqlx::query(
                    &format!("INSERT INTO {} (version, name) VALUES ($1, $2) ON CONFLICT (version) DO UPDATE SET name = EXCLUDED.name", self.table_name)
                )
                .bind(version)
                .bind(name)
                .execute(db.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
            }
        }
        
        Ok(())
    }
    
    /// Получает последнюю версию миграции
    pub async fn get_latest_version(&self, db: &SqlDb) -> Result<Option<i64>> {
        match db.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    &format!("SELECT MAX(version) FROM {}", self.table_name)
                )
                .fetch_optional(db.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                if let Some(row) = result {
                    let version: Option<i64> = row.get(0);
                    Ok(version)
                } else {
                    Ok(None)
                }
            }
            crate::db::sql::types::SqlDialect::MySQL => {
                let result = sqlx::query(
                    &format!("SELECT MAX(version) FROM {}", self.table_name)
                )
                .fetch_optional(db.get_mysql_pool().ok_or(Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                if let Some(row) = result {
                    let version: Option<i64> = row.get(0);
                    Ok(version)
                } else {
                    Ok(None)
                }
            }
            crate::db::sql::types::SqlDialect::PostgreSQL => {
                let result = sqlx::query(
                    &format!("SELECT MAX(version) FROM {}", self.table_name)
                )
                .fetch_optional(db.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                if let Some(row) = result {
                    let version: Option<i64> = row.get(0);
                    Ok(version)
                } else {
                    Ok(None)
                }
            }
        }
    }
}

impl Default for MigrationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_migration_manager_sqlite() {
        let temp_db = env::temp_dir().join("test_migration.db");
        let db_path = temp_db.to_string_lossy().to_string();
        
        let db = SqlDb::connect_sqlite(&db_path).await.unwrap();
        let manager = MigrationManager::new();
        
        // Создаём таблицу миграций
        manager.ensure_migration_table(&db).await.unwrap();
        
        // Проверяем что миграция не применена
        let is_applied = manager.is_migration_applied(&db, 1).await.unwrap();
        assert!(!is_applied);
        
        // Применяем миграцию
        manager.apply_migration(&db, 1, "test_migration").await.unwrap();
        
        // Проверяем что миграция применена
        let is_applied = manager.is_migration_applied(&db, 1).await.unwrap();
        assert!(is_applied);
        
        // Получаем последнюю версию
        let version = manager.get_latest_version(&db).await.unwrap();
        assert_eq!(version, Some(1));
        
        // Cleanup
        let _ = db.close().await;
        let _ = std::fs::remove_file(&temp_db);
    }

    #[test]
    fn test_migration_manager_creation() {
        let manager = MigrationManager::new();
        assert_eq!(manager.table_name, "migration");
    }
}
