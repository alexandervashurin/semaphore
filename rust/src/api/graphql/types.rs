//! GraphQL типы

use async_graphql::{InputObject, SimpleObject};

/// Пользователь
#[derive(SimpleObject, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub name: String,
    pub email: String,
    pub admin: bool,
}

/// Проект
#[derive(SimpleObject, Debug, Clone)]
pub struct Project {
    pub id: i32,
    pub name: String,
}

/// Шаблон (Template)
#[derive(SimpleObject, Debug, Clone)]
pub struct Template {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub playbook: String,
}

/// Задача (Task)
#[derive(SimpleObject, Debug, Clone)]
pub struct Task {
    pub id: i32,
    pub template_id: i32,
    pub project_id: i32,
    pub status: String,
}

/// Строка лога задачи — для subscription taskOutput
#[derive(SimpleObject, Debug, Clone)]
pub struct TaskOutputLine {
    pub task_id: i32,
    pub line: String,
    pub timestamp: String,
    /// Уровень: "info" | "warning" | "error" | "debug"
    pub level: String,
}

/// Изменение статуса задачи — для subscription taskStatus
#[derive(SimpleObject, Debug, Clone)]
pub struct TaskStatusEvent {
    pub task_id: i32,
    pub project_id: i32,
    pub status: String,
    pub updated_at: String,
}
