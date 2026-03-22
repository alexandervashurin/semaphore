//! SQL DB Migrations - миграции схемы БД (PostgreSQL)

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

    fn pool<'a>(&self, db: &'a SqlDb) -> Result<&'a sqlx::PgPool> {
        db.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Создаёт таблицу миграций если не существует
    pub async fn ensure_migration_table(&self, db: &SqlDb) -> Result<()> {
        sqlx::query(
            &format!("CREATE TABLE IF NOT EXISTS {} (version BIGINT PRIMARY KEY, name VARCHAR(255))", self.table_name)
        )
        .execute(self.pool(db)?)
        .await
        .map_err(Error::Database)?;

        Ok(())
    }

    /// Проверяет применена ли миграция
    pub async fn is_migration_applied(&self, db: &SqlDb, version: i64) -> Result<bool> {
        let result = sqlx::query(
            &format!("SELECT COUNT(*) FROM {} WHERE version = $1", self.table_name)
        )
        .bind(version)
        .fetch_one(self.pool(db)?)
        .await
        .map_err(Error::Database)?;

        let count: i64 = result.get(0);
        Ok(count > 0)
    }

    /// Применяет миграцию
    pub async fn apply_migration(&self, db: &SqlDb, version: i64, name: &str) -> Result<()> {
        sqlx::query(
            &format!("INSERT INTO {} (version, name) VALUES ($1, $2) ON CONFLICT (version) DO UPDATE SET name = EXCLUDED.name", self.table_name)
        )
        .bind(version)
        .bind(name)
        .execute(self.pool(db)?)
        .await
        .map_err(Error::Database)?;

        Ok(())
    }

    /// Получает последнюю версию миграции
    pub async fn get_latest_version(&self, db: &SqlDb) -> Result<Option<i64>> {
        let result = sqlx::query(
            &format!("SELECT MAX(version) FROM {}", self.table_name)
        )
        .fetch_optional(self.pool(db)?)
        .await
        .map_err(Error::Database)?;

        if let Some(row) = result {
            let version: Option<i64> = row.get(0);
            Ok(version)
        } else {
            Ok(None)
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

    #[test]
    fn test_migration_manager_creation() {
        let manager = MigrationManager::new();
        assert_eq!(manager.table_name, "migration");
    }
}
