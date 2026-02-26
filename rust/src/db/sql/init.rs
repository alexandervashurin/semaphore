//! SQL DB Init - инициализация подключения к БД
//!
//! Аналог db/sql/SqlDb.go из Go версии (часть 2: инициализация)

use crate::db::sql::types::{SqlDb, SqlDialect, DbConnectionConfig};
use crate::error::{Error, Result};
use sqlx::{sqlite::SqlitePoolOptions, mysql::MySqlPoolOptions, postgres::PgPoolOptions};

impl SqlDb {
    /// Подключается к SQLite БД
    pub async fn connect_sqlite(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| Error::Database(e))?;
        
        let mut db = Self::new(SqlDialect::SQLite);
        db.sqlite_pool = Some(pool);
        
        Ok(db)
    }
    
    /// Подключается к MySQL БД
    pub async fn connect_mysql(config: &DbConnectionConfig) -> Result<Self> {
        let pool = MySqlPoolOptions::new()
            .max_connections(10)
            .connect(&config.mysql_connection_string())
            .await
            .map_err(|e| Error::Database(e))?;
        
        let mut db = Self::new(SqlDialect::MySQL);
        db.mysql_pool = Some(pool);
        
        Ok(db)
    }
    
    /// Подключается к PostgreSQL БД
    pub async fn connect_postgres(config: &DbConnectionConfig) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&config.postgres_connection_string())
            .await
            .map_err(|e| Error::Database(e))?;
        
        let mut db = Self::new(SqlDialect::PostgreSQL);
        db.postgres_pool = Some(pool);
        
        Ok(db)
    }
    
    /// Подключается к БД на основе конфигурации
    pub async fn connect(dialect: SqlDialect, config: &DbConnectionConfig) -> Result<Self> {
        match dialect {
            SqlDialect::SQLite => {
                // Для SQLite используем db_name как путь к файлу
                Self::connect_sqlite(&config.db_name).await
            }
            SqlDialect::MySQL => Self::connect_mysql(config).await,
            SqlDialect::PostgreSQL => Self::connect_postgres(config).await,
        }
    }
    
    /// Проверяет подключение к БД
    pub async fn ping(&self) -> Result<()> {
        match self.dialect {
            SqlDialect::SQLite => {
                if let Some(pool) = &self.sqlite_pool {
                    pool.acquire().await
                        .map_err(|e| Error::Database(e))?;
                }
            }
            SqlDialect::MySQL => {
                if let Some(pool) = &self.mysql_pool {
                    pool.acquire().await
                        .map_err(|e| Error::Database(e))?;
                }
            }
            SqlDialect::PostgreSQL => {
                if let Some(pool) = &self.postgres_pool {
                    pool.acquire().await
                        .map_err(|e| Error::Database(e))?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Закрывает подключение к БД
    pub async fn close(&self) -> Result<()> {
        match self.dialect {
            SqlDialect::SQLite => {
                if let Some(pool) = &self.sqlite_pool {
                    pool.close().await;
                }
            }
            SqlDialect::MySQL => {
                if let Some(pool) = &self.mysql_pool {
                    pool.close().await;
                }
            }
            SqlDialect::PostgreSQL => {
                if let Some(pool) = &self.postgres_pool {
                    pool.close().await;
                }
            }
        }
        
        Ok(())
    }
    
    /// Создаёт БД если она не существует (для SQLite)
    pub async fn create_database_if_not_exists(database_path: &str) -> Result<()> {
        use std::path::Path;
        use tokio::fs;
        
        let path = Path::new(database_path);
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| Error::Other(format!("Failed to create database directory: {}", e)))?;
        }
        
        Ok(())
    }
}

/// Создаёт подключение к БД на основе строки подключения
pub async fn create_database_connection(database_url: &str) -> Result<SqlDb> {
    // Определяем тип БД по префиксу
    if database_url.starts_with("sqlite:") || database_url.ends_with(".db") || database_url.ends_with(".sqlite") {
        let path = database_url.trim_start_matches("sqlite:");
        SqlDb::create_database_if_not_exists(path).await?;
        SqlDb::connect_sqlite(path).await
    } else if database_url.starts_with("mysql:") {
        // Парсим MySQL URL
        Err(Error::Other("MySQL connection not fully implemented yet".to_string()))
    } else if database_url.starts_with("postgres:") {
        // Парсим PostgreSQL URL
        Err(Error::Other("PostgreSQL connection not fully implemented yet".to_string()))
    } else {
        Err(Error::Other(format!("Unknown database type: {}", database_url)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_sqlite_connection() {
        let temp_db = env::temp_dir().join("test.db");
        let db_path = temp_db.to_string_lossy().to_string();
        
        let result = SqlDb::connect_sqlite(&db_path).await;
        assert!(result.is_ok());
        
        let db = result.unwrap();
        assert!(db.is_connected());
        assert_eq!(db.get_dialect(), SqlDialect::SQLite);
        
        // Cleanup
        let _ = db.close().await;
        let _ = std::fs::remove_file(&temp_db);
    }

    #[tokio::test]
    async fn test_sqlite_ping() {
        let temp_db = env::temp_dir().join("test_ping.db");
        let db_path = temp_db.to_string_lossy().to_string();
        
        let db = SqlDb::connect_sqlite(&db_path).await.unwrap();
        let result = db.ping().await;
        assert!(result.is_ok());
        
        // Cleanup
        let _ = db.close().await;
        let _ = std::fs::remove_file(&temp_db);
    }

    #[test]
    fn test_db_connection_config_mysql() {
        let config = DbConnectionConfig {
            host: "localhost".to_string(),
            port: 3306,
            username: "user".to_string(),
            password: "pass".to_string(),
            db_name: "test".to_string(),
            ..Default::default()
        };
        
        let conn_str = config.mysql_connection_string();
        assert!(conn_str.contains("mysql://"));
        assert!(conn_str.contains("localhost:3306"));
    }

    #[test]
    fn test_db_connection_config_postgres() {
        let config = DbConnectionConfig {
            host: "localhost".to_string(),
            port: 5432,
            username: "user".to_string(),
            password: "pass".to_string(),
            db_name: "test".to_string(),
            ..Default::default()
        };
        
        let conn_str = config.postgres_connection_string();
        assert!(conn_str.contains("postgres://"));
        assert!(conn_str.contains("localhost:5432"));
    }
}
