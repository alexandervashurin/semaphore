//! Модель репозитория

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Тип репозитория
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryType {
    Git,
    Http,
    Https,
    File,
}

/// Репозиторий - хранилище кода (Git, HTTP, файл)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Repository {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта
    pub project_id: i32,

    /// Название репозитория
    pub name: String,

    /// URL репозитория
    pub git_url: String,

    /// Тип репозитория
    pub git_type: RepositoryType,

    /// ID ключа доступа
    pub key_id: i32,

    /// Путь к файлу (для file-типа)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_path: Option<String>,
}

impl Repository {
    /// Создаёт новый репозиторий
    pub fn new(project_id: i32, name: String, git_url: String) -> Self {
        Self {
            id: 0,
            project_id,
            name,
            git_url,
            git_type: RepositoryType::Git,
            key_id: 0,
            git_path: None,
        }
    }

    /// Получает URL для клонирования
    pub fn get_clone_url(&self) -> &str {
        &self.git_url
    }
}
