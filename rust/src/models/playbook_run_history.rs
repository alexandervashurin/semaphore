//! Модель PlaybookRun - история запусков playbook

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Статус запуска playbook
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum PlaybookRunStatus {
    Waiting,
    Running,
    Success,
    Failed,
    Cancelled,
}

impl std::fmt::Display for PlaybookRunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaybookRunStatus::Waiting => write!(f, "waiting"),
            PlaybookRunStatus::Running => write!(f, "running"),
            PlaybookRunStatus::Success => write!(f, "success"),
            PlaybookRunStatus::Failed => write!(f, "failed"),
            PlaybookRunStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// История запуска playbook
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PlaybookRun {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта
    pub project_id: i32,

    /// ID playbook
    pub playbook_id: i32,

    /// ID задачи (task)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<i32>,

    /// ID шаблона
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<i32>,

    /// Статус выполнения
    pub status: PlaybookRunStatus,

    /// ID инвентаря
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_id: Option<i32>,

    /// ID окружения
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<i32>,

    /// Дополнительные переменные (JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_vars: Option<String>,

    /// Ограничение по хостам
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_hosts: Option<String>,

    /// Теги для запуска
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,

    /// Пропускаемые теги
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_tags: Option<String>,

    /// Время начала выполнения
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Utc>>,

    /// Время завершения
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,

    /// Длительность в секундах
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<i32>,

    /// Всего хостов
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts_total: Option<i32>,

    /// Изменено хостов
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts_changed: Option<i32>,

    /// Недоступных хостов
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts_unreachable: Option<i32>,

    /// Хостов с ошибками
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts_failed: Option<i32>,

    /// Вывод playbook
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,

    /// Сообщение об ошибке
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    /// ID пользователя
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i32>,

    /// Дата создания
    pub created: DateTime<Utc>,

    /// Дата обновления
    pub updated: DateTime<Utc>,
}

/// Создание записи playbook_run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookRunCreate {
    pub project_id: i32,
    pub playbook_id: i32,
    pub task_id: Option<i32>,
    pub template_id: Option<i32>,
    pub inventory_id: Option<i32>,
    pub environment_id: Option<i32>,
    pub extra_vars: Option<String>,
    pub limit_hosts: Option<String>,
    pub tags: Option<String>,
    pub skip_tags: Option<String>,
    pub user_id: Option<i32>,
}

/// Обновление записи playbook_run
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaybookRunUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<PlaybookRunStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts_total: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts_changed: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts_unreachable: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts_failed: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// Статистика запусков playbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookRunStats {
    pub total_runs: i64,
    pub success_runs: i64,
    pub failed_runs: i64,
    pub avg_duration_seconds: Option<f64>,
    pub last_run: Option<DateTime<Utc>>,
}

/// Фильтр для поиска запусков
#[derive(Debug, Clone, Default)]
pub struct PlaybookRunFilter {
    pub project_id: Option<i32>,
    pub playbook_id: Option<i32>,
    pub status: Option<PlaybookRunStatus>,
    pub user_id: Option<i32>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
