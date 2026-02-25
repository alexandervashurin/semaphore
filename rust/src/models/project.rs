//! Модель проекта

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Проект - верхнеуровневая структура в Semaphore
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    /// Уникальный идентификатор
    pub id: i32,

    /// Дата создания
    pub created: DateTime<Utc>,

    /// Название проекта
    pub name: String,

    /// Включить уведомления
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert: Option<bool>,

    /// Chat ID для уведомлений
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_chat: Option<String>,

    /// Максимальное количество параллельных задач
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_parallel_tasks: Option<i32>,

    /// Тип проекта
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// ID хранилища секретов по умолчанию
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_secret_storage_id: Option<i32>,
}

impl Project {
    /// Создаёт новый проект
    pub fn new(name: String) -> Self {
        Self {
            id: 0, // Будет установлен базой данных
            created: Utc::now(),
            name,
            alert: None,
            alert_chat: None,
            max_parallel_tasks: None,
            r#type: None,
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
