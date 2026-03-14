//! OptionsManager - управление опциями

use crate::db::sql::SqlStore;
use crate::db::sql::types::SqlDialect;
use crate::db::store::*;
use crate::error::{Error, Result};
use async_trait::async_trait;
use sqlx::Row;
use std::collections::HashMap;

#[async_trait]
impl OptionsManager for SqlStore {
    async fn get_options(&self) -> Result<HashMap<String, String>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT key, value FROM option";
                let rows = sqlx::query(query)
                    .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| {
                    let key: String = row.get("key");
                    let value: String = row.get("value");
                    (key, value)
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT key, value FROM option";
                let rows = sqlx::query(query)
                    .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| {
                    let key: String = row.get("key");
                    let value: String = row.get("value");
                    (key, value)
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = "SELECT key, value FROM option";
                let rows = sqlx::query(query)
                    .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| {
                    let key: String = row.get("key");
                    let value: String = row.get("value");
                    (key, value)
                }).collect())
            }
        }
    }

    async fn get_option(&self, key: &str) -> Result<Option<String>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT value FROM option WHERE key = ?";
                let result = sqlx::query_scalar::<_, String>(query)
                    .bind(key)
                    .fetch_optional(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(result)
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT value FROM option WHERE key = $1";
                let result = sqlx::query_scalar::<_, String>(query)
                    .bind(key)
                    .fetch_optional(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(result)
            }
            SqlDialect::MySQL => {
                let query = "SELECT value FROM option WHERE key = ?";
                let result = sqlx::query_scalar::<_, String>(query)
                    .bind(key)
                    .fetch_optional(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(result)
            }
        }
    }

    async fn set_option(&self, key: &str, value: &str) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT OR REPLACE INTO option (key, value) VALUES (?, ?)";
                sqlx::query(query)
                    .bind(key)
                    .bind(value)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO option (key, value) VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value";
                sqlx::query(query)
                    .bind(key)
                    .bind(value)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO option (key, value) VALUES (?, ?) ON DUPLICATE KEY UPDATE value = VALUES(value)";
                sqlx::query(query)
                    .bind(key)
                    .bind(value)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_option(&self, key: &str) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "DELETE FROM option WHERE key = ?";
                sqlx::query(query)
                    .bind(key)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "DELETE FROM option WHERE key = $1";
                sqlx::query(query)
                    .bind(key)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "DELETE FROM option WHERE key = ?";
                sqlx::query(query)
                    .bind(key)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

