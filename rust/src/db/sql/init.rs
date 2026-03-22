//! SQL DB Init - инициализация подключения к БД (PostgreSQL)
//!
//! Аналог db/sql/SqlDb.go из Go версии (часть 2: инициализация)

use crate::db::sql::types::{SqlDb, DbConnectionConfig};
use crate::error::{Error, Result};
use sqlx::postgres::PgPoolOptions;

impl SqlDb {
    /// Подключается к PostgreSQL БД
    pub async fn connect_postgres(config: &DbConnectionConfig) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&config.postgres_connection_string())
            .await
            .map_err(Error::Database)?;

        let mut db = Self::new();
        db.postgres_pool = Some(pool);

        Ok(db)
    }

    /// Подключается к PostgreSQL по URL напрямую
    pub async fn connect_postgres_url(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .map_err(Error::Database)?;

        let mut db = Self::new();
        db.postgres_pool = Some(pool);

        Ok(db)
    }

    /// Проверяет подключение к БД
    pub async fn ping(&self) -> Result<()> {
        if let Some(pool) = &self.postgres_pool {
            pool.acquire().await
                .map_err(Error::Database)?;
        }
        Ok(())
    }

    /// Закрывает подключение к БД
    pub async fn close(&self) -> Result<()> {
        if let Some(pool) = &self.postgres_pool {
            pool.close().await;
        }
        Ok(())
    }
}

/// Создаёт подключение к БД на основе строки подключения
pub async fn create_database_connection(database_url: &str) -> Result<SqlDb> {
    tracing::info!("Creating database connection: {}", database_url);

    if database_url.starts_with("postgres:") || database_url.starts_with("postgresql:") {
        parse_and_connect_postgres(database_url).await
    } else {
        Err(Error::Other(format!(
            "Unsupported database type: {}. Only PostgreSQL is supported.",
            database_url
        )))
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

    SqlDb::connect_postgres(&config).await
}

#[cfg(test)]
mod tests {
    use super::*;

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
