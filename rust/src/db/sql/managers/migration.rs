//! MigrationManager - управление миграциями БД

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use async_trait::async_trait;

#[async_trait]
impl MigrationManager for SqlStore {
    fn get_dialect(&self) -> &str {
        "postgresql"
    }

    async fn is_initialized(&self) -> Result<bool> {
        let query = "SELECT table_name FROM information_schema.tables WHERE table_type = 'BASE TABLE' AND table_name = 'migration'";
        let result = sqlx::query(query)
            .fetch_optional(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(result.is_some())
    }

    async fn apply_migration(&self, version: i64, name: String) -> Result<()> {
        let query = "INSERT INTO migration (version, name) VALUES ($1, $2)";
        sqlx::query(query)
            .bind(version)
            .bind(name)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn is_migration_applied(&self, version: i64) -> Result<bool> {
        let query = "SELECT COUNT(*) FROM migration WHERE version = $1";
        let count: i64 = sqlx::query_scalar(query)
            .bind(version)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(count > 0)
    }
}
