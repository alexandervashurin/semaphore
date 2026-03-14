//! Модель Playbook - YAML файл с задачами Ansible/Terraform

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Playbook - YAML файл с автоматизацией
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Playbook {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта
    pub project_id: i32,

    /// Название плейбука
    pub name: String,

    /// YAML содержимое
    pub content: String,

    /// Описание
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Тип (ansible, terraform, shell)
    pub playbook_type: String,

    /// ID репозитория (опционально, если связан с git)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_id: Option<i32>,

    /// Дата создания
    pub created: DateTime<Utc>,

    /// Дата обновления
    pub updated: DateTime<Utc>,
}

/// Playbook для создания
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookCreate {
    pub name: String,
    pub content: String,
    pub description: Option<String>,
    pub playbook_type: String,
    pub repository_id: Option<i32>,
}

/// Playbook для обновления
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookUpdate {
    pub name: String,
    pub content: String,
    pub description: Option<String>,
    pub playbook_type: String,
}
