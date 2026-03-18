//! Модель проекта

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Проект - верхнеуровневая структура в Velum
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    /// Уникальный идентификатор
    pub id: i32,

    /// Дата создания
    pub created: DateTime<Utc>,

    /// Название проекта
    pub name: String,

    /// Включить уведомления
    #[serde(default)]
    pub alert: bool,

    /// Chat ID для уведомлений
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_chat: Option<String>,

    /// Максимальное количество параллельных задач
    #[serde(default)]
    pub max_parallel_tasks: i32,

    /// Тип проекта
    #[serde(default)]
    pub r#type: String,

    /// ID хранилища секретов по умолчанию
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_secret_storage_id: Option<i32>,
}

#[cfg(test)]
impl Default for Project {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}

impl Project {
    /// Создаёт новый проект
    pub fn new(name: String) -> Self {
        Self {
            id: 0, // Будет установлен базой данных
            created: Utc::now(),
            name,
            alert: false,
            alert_chat: None,
            max_parallel_tasks: 0,
            r#type: "default".to_string(),
            default_secret_storage_id: None,
        }
    }

    /// Проверяет валидность проекта
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Название проекта не может быть пустым".to_string());
        }
        Ok(())
    }
}
