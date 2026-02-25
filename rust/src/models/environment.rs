//! Модель окружения (Environment)

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Окружение - переменные окружения для задач
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Environment {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта
    pub project_id: i32,

    /// Название окружения
    pub name: String,

    /// JSON с переменными окружения
    pub json: String,

    /// ID хранилища секретов
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_storage_id: Option<i32>,
}

impl Environment {
    /// Создаёт новое окружение
    pub fn new(project_id: i32, name: String, json: String) -> Self {
        Self {
            id: 0,
            project_id,
            name,
            json,
            secret_storage_id: None,
        }
    }

    /// Парсит JSON с переменными окружения
    pub fn parse_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.json)
    }
}
