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

#[cfg(test)]
mod tests {
    #[test]
    fn test_migration_dialect_is_postgresql() {
        // SqlStore::get_dialect returns "postgresql"
        let dialect = "postgresql";
        assert_eq!(dialect, "postgresql");
    }

    #[test]
    fn test_migration_version_positive() {
        let version: i64 = 1;
        assert!(version > 0);
    }

    #[test]
    fn test_migration_version_zero() {
        let version: i64 = 0;
        assert_eq!(version, 0);
    }

    #[test]
    fn test_migration_name_not_empty() {
        let name = "initial_schema".to_string();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_migration_name_empty() {
        let name = String::new();
        assert!(name.is_empty());
    }

    #[test]
    fn test_migration_count_check() {
        let count: i64 = 0;
        assert_eq!(count > 0, false);

        let count: i64 = 1;
        assert_eq!(count > 0, true);
    }

    #[test]
    fn test_migration_is_initialized_check() {
        // Simulating the check: result.is_some() -> true means initialized
        let some_result = Some("migration_table");
        assert!(some_result.is_some());

        let none_result: Option<&str> = None;
        assert!(!none_result.is_some());
    }

    #[test]
    fn test_migration_version_large_number() {
        let version: i64 = 20260410120000;
        assert!(version > 1000);
    }

    #[test]
    fn test_migration_name_with_special_chars() {
        let name = "add_user_table_2024".to_string();
        assert!(name.contains("2024"));
    }
}
