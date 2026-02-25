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
    pub name: String,
    pub active: bool,
}

/// Расписание с дополнительными полями
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScheduleWithTpl {
    #[serde(flatten)]
    pub schedule: Schedule,
    pub tpl_playbook: String,
    pub tpl_alias: String,
}
