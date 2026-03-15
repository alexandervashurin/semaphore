//! Модель расписания

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Расписание - автоматический запуск задач
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Schedule {
    pub id: i32,
    pub template_id: i32,
    pub project_id: i32,
    pub cron: String,
    pub cron_format: Option<String>,
    pub name: String,
    pub active: bool,
    pub last_commit_hash: Option<String>,
    pub repository_id: Option<i32>,
    pub created: Option<String>,
    /// Одноразовый запуск: дата/время ISO 8601 (если cron_format = "run_at")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_at: Option<String>,
    /// Удалить расписание после выполнения (только для run_at)
    #[serde(default)]
    pub delete_after_run: bool,
}

/// Расписание с дополнительными полями
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScheduleWithTpl {
    #[serde(flatten)]
    pub schedule: Schedule,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tpl_playbook: Option<String>,
}
