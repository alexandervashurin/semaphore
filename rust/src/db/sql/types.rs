//! SQL DB Types - типы для SQL хранилища
//!
//! PostgreSQL-only implementation

use sqlx::PgPool;

/// SQL хранилище данных
pub struct SqlDb {
    /// Connection pool для PostgreSQL
    pub postgres_pool: Option<PgPool>,
}

impl SqlDb {
    /// Создаёт новое SQL хранилище
    pub fn new() -> Self {
        Self {
            postgres_pool: None,
        }
    }

    /// Проверяет подключено ли хранилище
    pub fn is_connected(&self) -> bool {
        self.postgres_pool.is_some()
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
    /// PostgreSQL транзакция
    pub postgres_txn: Option<sqlx::Transaction<'static, sqlx::Postgres>>,
}

impl SqlTransaction {
    /// Создаёт новую транзакцию
    pub fn new() -> Self {
        Self {
            postgres_txn: None,
        }
    }

    /// Начинает транзакцию
    pub async fn begin(&mut self, db: &SqlDb) -> Result<(), crate::error::Error> {
        let pool = db.get_postgres_pool()
            .ok_or_else(|| crate::error::Error::Other("PostgreSQL pool not found".to_string()))?;
        self.postgres_txn = Some(pool.begin().await
            .map_err(crate::error::Error::Database)?);
        Ok(())
    }

    /// Фиксирует транзакцию
    pub async fn commit(&mut self) -> Result<(), crate::error::Error> {
        if let Some(txn) = self.postgres_txn.take() {
            txn.commit().await
                .map_err(crate::error::Error::Database)?;
        }
        Ok(())
    }

    /// Откатывает транзакцию
    pub async fn rollback(&mut self) -> Result<(), crate::error::Error> {
        if let Some(txn) = self.postgres_txn.take() {
            txn.rollback().await
                .map_err(crate::error::Error::Database)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_db_creation() {
        let db = SqlDb::new();
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
        let txn = SqlTransaction::new();
        assert!(txn.postgres_txn.is_none());
    }
}
