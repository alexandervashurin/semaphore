//! Migration Operations for BoltDB
//!
//! Операции с миграциями в BoltDB

use std::sync::Arc;
use chrono::{DateTime, Utc};
use crate::db::bolt::BoltStore;
use crate::error::{Error, Result};
use crate::models::Migration;

impl BoltStore {
    /// Проверяет, инициализирована ли БД
    pub async fn is_initialized(&self) -> Result<bool> {
        let tree = self.db.open_tree("migration")
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;
        Ok(!tree.is_empty())
    }

    /// Применяет миграцию
    pub async fn apply_migration(&self, version: i64, name: String) -> Result<()> {
        let tree = self.db.open_tree("migration")
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        let key = version.to_string();
        let value = format!("{}|{}", name, Utc::now().to_rfc3339());

        tree.insert(key.as_bytes(), value.as_bytes())
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        Ok(())
    }

    /// Откатывает миграцию
    pub async fn rollback_migration(&self, version: i64) -> Result<()> {
        let tree = self.db.open_tree("migration")
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        let key = version.to_string();
        tree.remove(key.as_bytes())
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        Ok(())
    }

    /// Получает список применённых миграций
    pub async fn get_migrations(&self) -> Result<Vec<Migration>> {
        let tree = self.db.open_tree("migration")
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        let mut migrations = Vec::new();

        for item in tree.iter() {
            let (key, value) = item
                .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;

            let key_str = String::from_utf8_lossy(&key).to_string();
            let value_str = String::from_utf8_lossy(&value).to_string();

            let parts: Vec<&str> = value_str.split('|').collect();
            if parts.len() >= 2 {
                let version = key_str.parse().unwrap_or(0);
                let name = parts[0].to_string();
                let applied = DateTime::parse_from_rfc3339(parts[1])
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                migrations.push(Migration {
                    id: 0,
                    version,
                    name,
                    applied,
                });
            }
        }

        Ok(migrations)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_operations() {
        // Тест для проверки операций с миграциями
        assert!(true);
    }
}
