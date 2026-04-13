//! Alias Model
//!
//! Модель псевдонима

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Псевдоним
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Alias {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта
    pub project_id: i32,

    /// Псевдоним
    pub alias: String,

    /// ID владельца
    pub owner_id: i32,

    /// Тип владельца
    pub owner_type: String,

    /// Дата создания
    pub created: DateTime<Utc>,
}

impl Alias {
    /// Создаёт новый псевдоним
    pub fn new(project_id: i32, alias: String, owner_id: i32, owner_type: String) -> Self {
        Self {
            id: 0,
            project_id,
            alias,
            owner_id,
            owner_type,
            created: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_new() {
        let alias = Alias::new(10, "deploy-prod".to_string(), 1, "template".to_string());
        assert_eq!(alias.id, 0);
        assert_eq!(alias.project_id, 10);
        assert_eq!(alias.alias, "deploy-prod");
        assert_eq!(alias.owner_id, 1);
        assert_eq!(alias.owner_type, "template");
    }

    #[test]
    fn test_alias_serialization() {
        let alias = Alias {
            id: 1,
            project_id: 5,
            alias: "my-alias".to_string(),
            owner_id: 2,
            owner_type: "template".to_string(),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&alias).unwrap();
        assert!(json.contains("\"alias\":\"my-alias\""));
        assert!(json.contains("\"project_id\":5"));
        assert!(json.contains("\"owner_type\":\"template\""));
    }

    #[test]
    fn test_alias_deserialization() {
        let json = r#"{"id":3,"project_id":7,"alias":"test-alias","owner_id":4,"owner_type":"playbook","created":"2024-01-01T00:00:00Z"}"#;
        let alias: Alias = serde_json::from_str(json).unwrap();
        assert_eq!(alias.id, 3);
        assert_eq!(alias.alias, "test-alias");
        assert_eq!(alias.owner_type, "playbook");
    }

    #[test]
    fn test_alias_clone() {
        let alias = Alias::new(1, "clone-test".to_string(), 1, "template".to_string());
        let cloned = alias.clone();
        assert_eq!(cloned.alias, alias.alias);
        assert_eq!(cloned.owner_id, alias.owner_id);
    }

    #[test]
    fn test_alias_serialization_all_fields() {
        let alias = Alias {
            id: 5,
            project_id: 10,
            alias: "deploy-prod".to_string(),
            owner_id: 3,
            owner_type: "template".to_string(),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&alias).unwrap();
        assert!(json.contains("\"id\":5"));
        assert!(json.contains("\"owner_type\":\"template\""));
        assert!(json.contains("\"alias\":\"deploy-prod\""));
    }

    #[test]
    fn test_alias_deserialization_full() {
        let json = r#"{"id":10,"project_id":20,"alias":"my-alias","owner_id":5,"owner_type":"playbook","created":"2024-01-01T00:00:00Z"}"#;
        let alias: Alias = serde_json::from_str(json).unwrap();
        assert_eq!(alias.id, 10);
        assert_eq!(alias.alias, "my-alias");
        assert_eq!(alias.owner_type, "playbook");
    }

    #[test]
    fn test_alias_debug() {
        let alias = Alias::new(1, "debug-alias".to_string(), 1, "template".to_string());
        let debug_str = format!("{:?}", alias);
        assert!(debug_str.contains("Alias"));
        assert!(debug_str.contains("debug-alias"));
    }

    #[test]
    fn test_alias_empty_string() {
        let alias = Alias::new(1, "".to_string(), 0, "".to_string());
        assert!(alias.alias.is_empty());
        assert!(alias.owner_type.is_empty());
        assert_eq!(alias.owner_id, 0);
    }

    #[test]
    fn test_alias_special_characters() {
        let alias = Alias::new(1, "alias&<special>".to_string(), 1, "type\"quoted\"".to_string());
        let json = serde_json::to_string(&alias).unwrap();
        let restored: Alias = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.alias, "alias&<special>");
        assert_eq!(restored.owner_type, "type\"quoted\"");
    }

    #[test]
    fn test_alias_large_ids() {
        let alias = Alias::new(i32::MAX, "large-id".to_string(), i32::MAX, "template".to_string());
        assert_eq!(alias.project_id, i32::MAX);
        assert_eq!(alias.owner_id, i32::MAX);
    }

    #[test]
    fn test_alias_different_owner_types() {
        let owner_types = ["template", "playbook", "workflow", "custom"];
        for ot in owner_types {
            let alias = Alias::new(1, "test".to_string(), 1, ot.to_string());
            let json = serde_json::to_string(&alias).unwrap();
            assert!(json.contains(&format!("\"owner_type\":\"{}\"", ot)));
        }
    }

    #[test]
    fn test_alias_unicode_alias() {
        let alias = Alias::new(1, "алиас".to_string(), 1, "шаблон".to_string());
        let json = serde_json::to_string(&alias).unwrap();
        let restored: Alias = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.alias, "алиас");
        assert_eq!(restored.owner_type, "шаблон");
    }

    #[test]
    fn test_alias_clone_independence() {
        let mut alias = Alias::new(1, "original".to_string(), 1, "template".to_string());
        let cloned = alias.clone();
        alias.alias = "modified".to_string();
        assert_eq!(cloned.alias, "original");
    }

    #[test]
    fn test_alias_roundtrip() {
        let original = Alias {
            id: 77, project_id: 33, alias: "roundtrip".to_string(),
            owner_id: 11, owner_type: "workflow".to_string(), created: Utc::now(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: Alias = serde_json::from_str(&json).unwrap();
        assert_eq!(original.id, restored.id);
        assert_eq!(original.alias, restored.alias);
        assert_eq!(original.owner_id, restored.owner_id);
    }

    #[test]
    fn test_alias_debug_contains_all_fields() {
        let alias = Alias {
            id: 42, project_id: 10, alias: "debug-alias".to_string(),
            owner_id: 5, owner_type: "template".to_string(), created: Utc::now(),
        };
        let debug_str = format!("{:?}", alias);
        assert!(debug_str.contains("42"));
        assert!(debug_str.contains("debug-alias"));
        assert!(debug_str.contains("Alias"));
    }
}
