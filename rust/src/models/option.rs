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
}
