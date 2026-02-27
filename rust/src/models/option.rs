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
