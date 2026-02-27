//! Модель Hook - хуки для задач

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Тип хука
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HookType {
    /// HTTP запрос
    Http,
    /// Bash скрипт
    Bash,
    /// Python скрипт
    Python,
}

/// Хук для задачи
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Hook {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта
    pub project_id: i32,

    /// ID шаблона
    pub template_id: i32,

    /// Название хука
    pub name: String,

    /// Тип хука
    pub r#type: HookType,

    /// URL (для HTTP)
    pub url: Option<String>,

    /// Скрипт (для Bash/Python)
    pub script: Option<String>,

    /// Метод HTTP (GET, POST, etc.)
    pub http_method: Option<String>,

    /// Тело запроса (для HTTP)
    pub http_body: Option<String>,

    /// Таймаут в секундах
    pub timeout_secs: Option<i32>,
}

impl Hook {
    /// Создаёт новый хук
    pub fn new(project_id: i32, template_id: i32, name: String, hook_type: HookType) -> Self {
        Self {
            id: 0,
            project_id,
            template_id,
            name,
            r#type: hook_type,
            url: None,
            script: None,
            http_method: None,
            http_body: None,
            timeout_secs: None,
        }
    }
}
