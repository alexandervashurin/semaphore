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

    #[test]
    fn test_migration_clone() {
        let migration = Migration::new(5, "add_indexes".to_string());
        let cloned = migration.clone();
        assert_eq!(cloned.version, migration.version);
        assert_eq!(cloned.name, migration.name);
    }

    #[test]
    fn test_migration_deserialization() {
        let json = r#"{"id":10,"version":20240201000000,"name":"add_users","applied":"2024-02-01T00:00:00Z"}"#;
        let migration: Migration = serde_json::from_str(json).unwrap();
        assert_eq!(migration.id, 10);
        assert_eq!(migration.version, 20240201000000);
        assert_eq!(migration.name, "add_users");
    }

    #[test]
    fn test_migration_large_version() {
        let migration = Migration::new(20260407120000, "add_organizations".to_string());
        let json = serde_json::to_string(&migration).unwrap();
        assert!(json.contains("\"version\":20260407120000"));
        assert!(json.contains("\"name\":\"add_organizations\""));
    }

    #[test]
    fn test_migration_zero_version() {
        let migration = Migration::new(0, "initial".to_string());
        assert_eq!(migration.version, 0);
        assert_eq!(migration.name, "initial");
    }
}
