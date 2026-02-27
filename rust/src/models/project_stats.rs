//! Project Stats Model
//!
//! Статистика проекта

use serde::{Deserialize, Serialize};

/// Статистика проекта
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectStats {
    /// Количество задач
    pub task_count: i32,

    /// Количество успешных задач
    pub success_count: i32,

    /// Количество неудачных задач
    pub fail_count: i32,

    /// Количество остановленных задач
    pub stopped_count: i32,

    /// Количество активных пользователей
    pub active_user_count: i32,

    /// Количество шаблонов
    pub template_count: i32,

    /// Количество инвентарей
    pub inventory_count: i32,

    /// Количество репозиториев
    pub repository_count: i32,
}

impl ProjectStats {
    /// Создаёт новую статистику
    pub fn new() -> Self {
        Self::default()
    }

    /// Процент успешных задач
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.fail_count + self.stopped_count;
        if total == 0 {
            0.0
        } else {
            (self.success_count as f64 / total as f64) * 100.0
        }
    }
}
