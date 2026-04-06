//! Migration Model
//!
//! Модель миграции БД

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Миграция БД
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Migration {
    /// Уникальный идентификатор
    pub id: i32,

    /// Версия миграции
    pub version: i64,

    /// Название миграции
    pub name: String,

    /// Дата применения
    pub applied: DateTime<Utc>,
}

impl Migration {
    /// Создаёт новую миграцию
    pub fn new(version: i64, name: String) -> Self {
        Self {
            id: 0,
            version,
            name,
            applied: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_new() {
        let migration = Migration::new(1, "create_users_table".to_string());
        assert_eq!(migration.id, 0);
        assert_eq!(migration.version, 1);
        assert_eq!(migration.name, "create_users_table");
    }

    #[test]
    fn test_migration_serialization() {
        let migration = Migration {
            id: 1,
            version: 20240101000000,
            name: "add_templates".to_string(),
            applied: Utc::now(),
        };
        let json = serde_json::to_string(&migration).unwrap();
        assert!(json.contains("\"version\":20240101000000"));
        assert!(json.contains("\"name\":\"add_templates\""));
    }
}
