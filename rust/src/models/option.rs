//! Модель Option - опции проекта

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Опция проекта
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OptionItem {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта (0 для глобальных опций)
    pub project_id: i32,

    /// Ключ опции
    pub key: String,

    /// Значение опции
    pub value: String,
}

impl OptionItem {
    /// Создаёт новую опцию
    pub fn new(project_id: i32, key: String, value: String) -> Self {
        Self {
            id: 0,
            project_id,
            key,
            value,
        }
    }
}

// Ре-экспорт как Option для совместимости
pub use OptionItem as Option;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_item_new() {
        let opt = OptionItem::new(10, "max_tasks".to_string(), "5".to_string());
        assert_eq!(opt.id, 0);
        assert_eq!(opt.project_id, 10);
        assert_eq!(opt.key, "max_tasks");
        assert_eq!(opt.value, "5");
    }

    #[test]
    fn test_option_item_serialization() {
        let opt = OptionItem {
            id: 1,
            project_id: 10,
            key: "alert_chat".to_string(),
            value: "@team-lead".to_string(),
        };
        let json = serde_json::to_string(&opt).unwrap();
        assert!(json.contains("\"key\":\"alert_chat\""));
        assert!(json.contains("\"value\":\"@team-lead\""));
    }

    #[test]
    fn test_option_alias() {
        // Проверяем что OptionItem alias работает
        let opt = Option::new(5, "test_key".to_string(), "test_value".to_string());
        assert_eq!(opt.key, "test_key");
        assert_eq!(opt.value, "test_value");
    }

    #[test]
    fn test_option_item_clone() {
        let opt = OptionItem::new(1, "clone_key".to_string(), "clone_value".to_string());
        let cloned = opt.clone();
        assert_eq!(cloned.key, opt.key);
        assert_eq!(cloned.value, opt.value);
    }

    #[test]
    fn test_option_item_debug() {
        let opt = OptionItem::new(1, "debug_key".to_string(), "debug_value".to_string());
        let debug_str = format!("{:?}", opt);
        assert!(debug_str.contains("OptionItem"));
        assert!(debug_str.contains("debug_key"));
    }

    #[test]
    fn test_option_item_deserialization() {
        let json = r#"{"id":5,"project_id":10,"key":"max_retries","value":"3"}"#;
        let opt: OptionItem = serde_json::from_str(json).unwrap();
        assert_eq!(opt.id, 5);
        assert_eq!(opt.key, "max_retries");
        assert_eq!(opt.value, "3");
    }

    #[test]
    fn test_option_item_zero_project_id() {
        let opt = OptionItem::new(0, "global_option".to_string(), "true".to_string());
        assert_eq!(opt.project_id, 0);
        assert_eq!(opt.key, "global_option");
    }

    #[test]
    fn test_option_item_empty_values() {
        let opt = OptionItem::new(1, "".to_string(), "".to_string());
        assert!(opt.key.is_empty());
        assert!(opt.value.is_empty());
    }

    #[test]
    fn test_option_item_serialization_all_fields() {
        let opt = OptionItem {
            id: 42,
            project_id: 100,
            key: "notification_channel".to_string(),
            value: "#alerts".to_string(),
        };
        let json = serde_json::to_string(&opt).unwrap();
        assert!(json.contains("\"id\":42"));
        assert!(json.contains("\"project_id\":100"));
        assert!(json.contains("\"key\":\"notification_channel\""));
        assert!(json.contains("\"value\":\"#alerts\""));
    }

    #[test]
    fn test_option_item_special_characters() {
        let opt = OptionItem::new(1, "key:with:colons".to_string(), "value with spaces & special".to_string());
        let json = serde_json::to_string(&opt).unwrap();
        assert!(json.contains("key:with:colons"));
        assert!(json.contains("value with spaces & special"));
    }

    #[test]
    fn test_option_item_unicode() {
        let opt = OptionItem::new(1, "ключ".to_string(), "значение".to_string());
        let json = serde_json::to_string(&opt).unwrap();
        let restored: OptionItem = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.key, "ключ");
        assert_eq!(restored.value, "значение");
    }

    #[test]
    fn test_option_item_clone_independence() {
        let mut opt = OptionItem::new(1, "key".to_string(), "value".to_string());
        let cloned = opt.clone();
        opt.value = "modified".to_string();
        assert_eq!(cloned.value, "value");
    }

    #[test]
    fn test_option_item_roundtrip() {
        let original = OptionItem {
            id: 100,
            project_id: 200,
            key: "roundtrip_key".to_string(),
            value: "roundtrip_value".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: OptionItem = serde_json::from_str(&json).unwrap();
        assert_eq!(original.id, restored.id);
        assert_eq!(original.key, restored.key);
        assert_eq!(original.value, restored.value);
    }

    #[test]
    fn test_option_item_debug_all_fields() {
        let opt = OptionItem::new(42, "debug_key".to_string(), "debug_value".to_string());
        let debug_str = format!("{:?}", opt);
        assert!(debug_str.contains("42"));
        assert!(debug_str.contains("debug_key"));
        assert!(debug_str.contains("debug_value"));
    }

    #[test]
    fn test_option_item_with_negative_project_id() {
        let opt = OptionItem::new(-1, "negative".to_string(), "value".to_string());
        assert_eq!(opt.project_id, -1);
    }
}
