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
        
        // Убираем ведущие слэши из URL-пути (sqlite:///C:/path -> C:/path)
        let path_str = database_path.trim_start_matches('/');
        let path = Path::new(path_str);
        
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).await
                    .map_err(|e| Error::Other(format!("Failed to create database directory: {}", e)))?;
            }
        }
        
        Ok(())
    }
}

/// Создаёт подключение к БД на основе строки подключения
pub async fn create_database_connection(database_url: &str) -> Result<SqlDb> {
    // Определяем тип БД по префиксу
    if database_url.starts_with("sqlite:") || database_url.ends_with(".db") || database_url.ends_with(".sqlite") {
        let path = database_url.trim_start_matches("sqlite:");
        if !path.starts_with(":memory") {
            SqlDb::create_database_if_not_exists(path).await?;
        }
        let url = if database_url.starts_with("sqlite:") {
            database_url.to_string()
        } else {
            format!("sqlite:///{}", path.replace('\\', "/"))
        };
        SqlDb::connect_sqlite(&url).await
    } else if database_url.starts_with("mysql:") {
        // Парсим MySQL URL
        Err(Error::Other("MySQL connection not fully implemented yet".to_string()))
    } else if database_url.starts_with("postgres:") || database_url.starts_with("postgresql:") {
        // Парсим PostgreSQL URL
        // Формат: postgres://user:pass@host:port/dbname?options
        parse_and_connect_postgres(database_url).await
    } else {
        Err(Error::Other(format!("Unknown database type: {}", database_url)))
    }
}

/// Парсит PostgreSQL URL и подключается к БД
async fn parse_and_connect_postgres(database_url: &str) -> Result<SqlDb> {
    use std::collections::HashMap;
    
    // Удаляем префикс
    let url_without_prefix = database_url
        .trim_start_matches("postgres://")
        .trim_start_matches("postgresql://");
    
    // Простой парсинг: user:pass@host:port/dbname?options
    let (auth_part, rest) = url_without_prefix
        .split_once('@')
        .ok_or_else(|| Error::Other("Invalid PostgreSQL URL: missing @".to_string()))?;
    
    let (username, password) = auth_part
        .split_once(':')
        .ok_or_else(|| Error::Other("Invalid PostgreSQL URL: missing password".to_string()))?;
    
    let (host_port, db_with_options) = rest
        .split_once('/')
        .ok_or_else(|| Error::Other("Invalid PostgreSQL URL: missing database name".to_string()))?;
    
    let (host, port_str) = host_port
        .split_once(':')
        .ok_or_else(|| Error::Other("Invalid PostgreSQL URL: missing port".to_string()))?;
    
    let port = port_str
        .split_once('?')
        .map(|(p, _)| p)
        .unwrap_or(port_str)
        .parse::<u16>()
        .map_err(|_| Error::Other("Invalid PostgreSQL URL: invalid port".to_string()))?;
    
    // Извлекаем имя БД
    let db_name = db_with_options
        .split_once('?')
        .map(|(db, _)| db)
        .unwrap_or(db_with_options);
    
    // Создаём конфиг
    let mut config = DbConnectionConfig {
        host: host.to_string(),
        port,
        username: username.to_string(),
        password: password.to_string(),
        db_name: db_name.to_string(),
        options: HashMap::new(),
    };
    
    // Парсим опции если есть
    if let Some(options_str) = db_with_options.split_once('?').map(|(_, o)| o) {
        for pair in options_str.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                config.options.insert(key.to_string(), value.to_string());
            }
        }
    }
    
    // Подключаемся
    SqlDb::connect_postgres(&config).await
}

/// Создаёт URL для тестовой SQLite БД (уникальный файл, корректный формат для Windows)
#[cfg(test)]
pub fn test_sqlite_url() -> (String, tempfile::NamedTempFile) {
    let temp = tempfile::NamedTempFile::new().unwrap();
    let path = temp.path().to_string_lossy().replace('\\', "/");
    let url = format!("sqlite:///{}", path);
    (url, temp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_connection() {
        let (db_path, _temp) = test_sqlite_url();
        
        let result = SqlDb::connect_sqlite(&db_path).await;
        assert!(result.is_ok());
        
        let db = result.unwrap();
        assert!(db.is_connected());
        assert_eq!(db.get_dialect(), SqlDialect::SQLite);
        
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_sqlite_ping() {
        let (db_path, _temp) = test_sqlite_url();
        
        let db = SqlDb::connect_sqlite(&db_path).await.unwrap();
        let result = db.ping().await;
        assert!(result.is_ok());
        
        let _ = db.close().await;
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
