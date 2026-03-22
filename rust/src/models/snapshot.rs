//! Task Snapshot Model
//!
//! Снапшот успешного запуска шаблона — зафиксированные параметры деплоя.
//! Позволяет "откатиться" к предыдущей версии одним кликом.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Снапшот успешного запуска
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskSnapshot {
    pub id: i32,
    pub project_id: i32,
    pub template_id: i32,
    /// ID исходной задачи
    pub task_id: i32,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    pub arguments: Option<String>,
    pub inventory_id: Option<i32>,
    pub environment_id: Option<i32>,
    pub message: Option<String>,
    /// Пользовательская метка (необязательно)
    pub label: Option<String>,
    pub created_at: String,
    /// Название шаблона (joined)
    #[sqlx(default)]
    pub template_name: String,
}

/// Создание нового снапшота (обычно автоматически после успешной задачи)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSnapshotCreate {
    pub template_id: i32,
    pub task_id: i32,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    pub arguments: Option<String>,
    pub inventory_id: Option<i32>,
    pub environment_id: Option<i32>,
    pub message: Option<String>,
    pub label: Option<String>,
}

/// Запрос на откат (создание новой задачи из снапшота)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRequest {
    /// Пользовательское сообщение для задачи отката
    pub message: Option<String>,
}
