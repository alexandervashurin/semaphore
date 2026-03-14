//! MigrationManager - управление миграциями БД

use crate::db::sql::SqlStore;
use crate::db::sql::SqlDialect;
use crate::db::store::*;
use crate::error::{Error, Result};
use async_trait::async_trait;

#[async_trait]
impl MigrationManager for SqlStore {
    fn get_dialect(&self) -> &str {
        match self.db.get_dialect() {
            SqlDialect::SQLite => "sqlite",
            SqlDialect::MySQL => "mysql",
            SqlDialect::PostgreSQL => "postgresql",
        }
    }

    async fn is_initialized(&self) -> Result<bool> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT name FROM sqlite_master WHERE type='table' AND name='migration'";
                let result = sqlx::query(query)
                    .fetch_optional(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(result.is_some())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT table_name FROM information_schema.tables WHERE table_type = 'BASE TABLE' AND table_name = 'migration'";
                let result = sqlx::query(query)
                    .fetch_optional(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(result.is_some())
            }
            SqlDialect::MySQL => {
                let query = "SELECT table_name FROM information_schema.tables WHERE table_type = 'BASE TABLE' AND table_name = 'migration'";
                let result = sqlx::query(query)
                    .fetch_optional(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(result.is_some())
            }
        }
    }

    async fn apply_migration(&self, version: i64, name: String) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO migration (version, name) VALUES (?, ?)";
                sqlx::query(query)
                    .bind(version)
                    .bind(name)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO migration (version, name) VALUES ($1, $2)";
                sqlx::query(query)
                    .bind(version)
                    .bind(name)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO migration (version, name) VALUES (?, ?)";
                sqlx::query(query)
                    .bind(version)
                    .bind(name)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn is_migration_applied(&self, version: i64) -> Result<bool> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT COUNT(*) FROM migration WHERE version = ?";
                let count: i64 = sqlx::query_scalar(query)
                    .bind(version)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(count > 0)
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT COUNT(*) FROM migration WHERE version = $1";
                let count: i64 = sqlx::query_scalar(query)
                    .bind(version)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(count > 0)
            }
            SqlDialect::MySQL => {
                let query = "SELECT COUNT(*) FROM migration WHERE version = ?";
                let count: i64 = sqlx::query_scalar(query)
                    .bind(version)
                    .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                Ok(count > 0)
            }
        }
    }
}

