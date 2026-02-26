//! SQL DB Types - типы для SQL хранилища
//!
//! Аналог db/sql/SqlDb.go из Go версии (часть 1: типы)

use sqlx::{SqlitePool, MySqlPool, PgPool};
use std::sync::Arc;

/// Тип SQL диалекта
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SqlDialect {
    SQLite,
    MySQL,
    PostgreSQL,
}

/// SQL хранилище данных
pub struct SqlDb {
    /// SQL диалект
    pub dialect: SqlDialect,
    
    /// Connection pool для SQLite
    pub sqlite_pool: Option<SqlitePool>,
    
    /// Connection pool для MySQL
    pub mysql_pool: Option<MySqlPool>,
    
    /// Connection pool для PostgreSQL
    pub postgres_pool: Option<PgPool>,
}

impl SqlDb {
    /// Создаёт новое SQL хранилище
    pub fn new(dialect: SqlDialect) -> Self {
        Self {
            dialect,
            sqlite_pool: None,
            mysql_pool: None,
            postgres_pool: None,
        }
    }
    
    /// Получает SQL диалект
    pub fn get_dialect(&self) -> SqlDialect {
        self.dialect
    }
    
    /// Проверяет подключено ли хранилище
    pub fn is_connected(&self) -> bool {
        match self.dialect {
            SqlDialect::SQLite => self.sqlite_pool.is_some(),
            SqlDialect::MySQL => self.mysql_pool.is_some(),
            SqlDialect::PostgreSQL => self.postgres_pool.is_some(),
        }
    }
    
    /// Получает SQLite pool
    pub fn get_sqlite_pool(&self) -> Option<&SqlitePool> {
        self.sqlite_pool.as_ref()
    }
    
    /// Получает MySQL pool
    pub fn get_mysql_pool(&self) -> Option<&MySqlPool> {
        self.mysql_pool.as_ref()
    }
    
    /// Получает PostgreSQL pool
    pub fn get_postgres_pool(&self) -> Option<&PgPool> {
        self.postgres_pool.as_ref()
    }
}

/// Конфигурация подключения к БД
#[derive(Debug, Clone)]
pub struct DbConnectionConfig {
    /// Хост
    pub host: String,
    
    /// Порт
    pub port: u16,
    
    /// Имя пользователя
    pub username: String,
    
    /// Пароль
    pub password: String,
    
    /// Имя БД
    pub db_name: String,
    
    /// Дополнительные опции
    pub options: std::collections::HashMap<String, String>,
}

impl Default for DbConnectionConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 0,
            username: String::new(),
            password: String::new(),
            db_name: "semaphore".to_string(),
            options: std::collections::HashMap::new(),
        }
    }
}

impl DbConnectionConfig {
    /// Создаёт новую конфигурацию подключения
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Создаёт connection string для MySQL
    pub fn mysql_connection_string(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}?{}",
            self.username,
            self.password,
            self.host,
            self.port,
            self.db_name,
            self.options_to_query_string()
        )
    }
    
    /// Создаёт connection string для PostgreSQL
    pub fn postgres_connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?{}",
            self.username,
            self.password,
            self.host,
            self.port,
            self.db_name,
            self.options_to_query_string()
        )
    }
    
    /// Преобразует опции в query string
    fn options_to_query_string(&self) -> String {
        self.options
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
    }
}

/// Транзакция SQL
pub struct SqlTransaction {
    /// Диалект
    pub dialect: SqlDialect,
    
    /// SQLite транзакция
    pub sqlite_txn: Option<sqlx::Transaction<'static, sqlx::Sqlite>>,
    
    /// MySQL транзакция
    pub mysql_txn: Option<sqlx::Transaction<'static, sqlx::MySql>>,
    
    /// PostgreSQL транзакция
    pub postgres_txn: Option<sqlx::Transaction<'static, sqlx::Postgres>>,
}

impl SqlTransaction {
    /// Создаёт новую транзакцию
    pub fn new(dialect: SqlDialect) -> Self {
        Self {
            dialect,
            sqlite_txn: None,
            mysql_txn: None,
            postgres_txn: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_db_creation() {
        let db = SqlDb::new(SqlDialect::SQLite);
        assert_eq!(db.get_dialect(), SqlDialect::SQLite);
        assert!(!db.is_connected());
    }

    #[test]
    fn test_db_connection_config_default() {
        let config = DbConnectionConfig::new();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.db_name, "semaphore");
        assert_eq!(config.port, 0);
    }

    #[test]
    fn test_mysql_connection_string() {
        let config = DbConnectionConfig {
            host: "localhost".to_string(),
            port: 3306,
            username: "user".to_string(),
            password: "pass".to_string(),
            db_name: "test".to_string(),
            options: std::collections::HashMap::new(),
        };
        
        let conn_str = config.mysql_connection_string();
        assert!(conn_str.starts_with("mysql://"));
        assert!(conn_str.contains("localhost"));
        assert!(conn_str.contains("3306"));
    }

    #[test]
    fn test_postgres_connection_string() {
        let config = DbConnectionConfig {
            host: "localhost".to_string(),
            port: 5432,
            username: "user".to_string(),
            password: "pass".to_string(),
            db_name: "test".to_string(),
            options: std::collections::HashMap::new(),
        };
        
        let conn_str = config.postgres_connection_string();
        assert!(conn_str.starts_with("postgres://"));
        assert!(conn_str.contains("localhost"));
        assert!(conn_str.contains("5432"));
    }

    #[test]
    fn test_sql_transaction_creation() {
        let txn = SqlTransaction::new(SqlDialect::SQLite);
        assert_eq!(txn.dialect, SqlDialect::SQLite);
    }
}
