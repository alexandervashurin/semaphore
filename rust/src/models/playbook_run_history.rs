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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playbook_run_status_display() {
        assert_eq!(PlaybookRunStatus::Waiting.to_string(), "waiting");
        assert_eq!(PlaybookRunStatus::Running.to_string(), "running");
        assert_eq!(PlaybookRunStatus::Success.to_string(), "success");
        assert_eq!(PlaybookRunStatus::Failed.to_string(), "failed");
        assert_eq!(PlaybookRunStatus::Cancelled.to_string(), "cancelled");
    }

    #[test]
    fn test_playbook_run_serialization() {
        let run = PlaybookRun {
            id: 1,
            project_id: 10,
            playbook_id: 5,
            task_id: Some(100),
            template_id: Some(3),
            status: PlaybookRunStatus::Success,
            inventory_id: None,
            environment_id: None,
            extra_vars: None,
            limit_hosts: None,
            tags: None,
            skip_tags: None,
            start_time: Some(Utc::now()),
            end_time: Some(Utc::now()),
            duration_seconds: Some(120),
            hosts_total: Some(5),
            hosts_changed: Some(2),
            hosts_unreachable: Some(0),
            hosts_failed: Some(0),
            output: Some("PLAY RECAP *****".to_string()),
            error_message: None,
            user_id: Some(1),
            created: Utc::now(),
            updated: Utc::now(),
        };
        let json = serde_json::to_string(&run).unwrap();
        // PlaybookRunStatus uses sqlx::Type, not serde rename
        assert!(json.contains("\"duration_seconds\":120"));
        assert!(json.contains("\"hosts_changed\":2"));
    }

    #[test]
    fn test_playbook_run_create_serialization() {
        let create = PlaybookRunCreate {
            project_id: 10,
            playbook_id: 5,
            task_id: None,
            template_id: None,
            inventory_id: Some(3),
            environment_id: Some(2),
            extra_vars: None,
            limit_hosts: None,
            tags: Some("deploy".to_string()),
            skip_tags: None,
            user_id: Some(1),
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"project_id\":10"));
        assert!(json.contains("\"tags\":\"deploy\""));
    }

    #[test]
    fn test_playbook_run_update_skip_nulls() {
        let update = PlaybookRunUpdate {
            status: Some(PlaybookRunStatus::Failed),
            start_time: None,
            end_time: Some(Utc::now()),
            duration_seconds: None,
            hosts_total: Some(10),
            hosts_changed: None,
            hosts_unreachable: None,
            hosts_failed: Some(3),
            output: Some("Error output".to_string()),
            error_message: Some("Task failed".to_string()),
        };
        let json = serde_json::to_string(&update).unwrap();
        // PlaybookRunStatus doesn't have serde rename, so status serializes differently
        assert!(json.contains("\"hosts_total\":10"));
        assert!(json.contains("\"hosts_failed\":3"));
        assert!(!json.contains("\"hosts_changed\":"));
    }

    #[test]
    fn test_playbook_run_stats_serialization() {
        let stats = PlaybookRunStats {
            total_runs: 100,
            success_runs: 85,
            failed_runs: 10,
            avg_duration_seconds: Some(95.5),
            last_run: Some(Utc::now()),
        };
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"total_runs\":100"));
        assert!(json.contains("\"avg_duration_seconds\":95.5"));
    }

    #[test]
    fn test_playbook_run_filter_default() {
        let filter = PlaybookRunFilter::default();
        assert!(filter.project_id.is_none());
        assert!(filter.playbook_id.is_none());
        assert!(filter.status.is_none());
        assert!(filter.user_id.is_none());
        assert!(filter.limit.is_none());
        assert!(filter.offset.is_none());
    }

    #[test]
    fn test_playbook_run_update_default() {
        let update = PlaybookRunUpdate::default();
        assert!(update.status.is_none());
        assert!(update.start_time.is_none());
        assert!(update.end_time.is_none());
        assert!(update.duration_seconds.is_none());
        assert!(update.hosts_total.is_none());
    }

    #[test]
    fn test_playbook_run_status_equality() {
        assert_eq!(PlaybookRunStatus::Success, PlaybookRunStatus::Success);
        assert_ne!(PlaybookRunStatus::Success, PlaybookRunStatus::Failed);
    }

    #[test]
    fn test_playbook_run_status_serialize_all() {
        let statuses = [
            PlaybookRunStatus::Waiting,
            PlaybookRunStatus::Running,
            PlaybookRunStatus::Success,
            PlaybookRunStatus::Failed,
            PlaybookRunStatus::Cancelled,
        ];
        for status in &statuses {
            // All should serialize without error
            let result = serde_json::to_string(status);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_playbook_run_clone() {
        let run = PlaybookRun {
            id: 1,
            project_id: 10,
            playbook_id: 5,
            task_id: Some(100),
            template_id: None,
            status: PlaybookRunStatus::Running,
            inventory_id: None,
            environment_id: None,
            extra_vars: None,
            limit_hosts: None,
            tags: None,
            skip_tags: None,
            start_time: Some(Utc::now()),
            end_time: None,
            duration_seconds: None,
            hosts_total: None,
            hosts_changed: None,
            hosts_unreachable: None,
            hosts_failed: None,
            output: None,
            error_message: None,
            user_id: None,
            created: Utc::now(),
            updated: Utc::now(),
        };
        let cloned = run.clone();
        assert_eq!(cloned.id, run.id);
        assert_eq!(cloned.status, run.status);
        assert_eq!(cloned.task_id, run.task_id);
    }

    #[test]
    fn test_playbook_run_create_clone() {
        let create = PlaybookRunCreate {
            project_id: 1,
            playbook_id: 1,
            task_id: None,
            template_id: None,
            inventory_id: Some(1),
            environment_id: None,
            extra_vars: None,
            limit_hosts: Some("web".to_string()),
            tags: Some("deploy".to_string()),
            skip_tags: None,
            user_id: Some(1),
        };
        let cloned = create.clone();
        assert_eq!(cloned.project_id, create.project_id);
        assert_eq!(cloned.limit_hosts, create.limit_hosts);
    }

    #[test]
    fn test_playbook_run_stats_default_duration() {
        let stats = PlaybookRunStats {
            total_runs: 10,
            success_runs: 8,
            failed_runs: 2,
            avg_duration_seconds: None,
            last_run: None,
        };
        assert!(stats.avg_duration_seconds.is_none());
        assert!(stats.last_run.is_none());
    }

    #[test]
    fn test_playbook_run_status_all_variants_display() {
        for (status, expected) in &[
            (PlaybookRunStatus::Waiting, "waiting"),
            (PlaybookRunStatus::Running, "running"),
            (PlaybookRunStatus::Success, "success"),
            (PlaybookRunStatus::Failed, "failed"),
            (PlaybookRunStatus::Cancelled, "cancelled"),
        ] {
            assert_eq!(status.to_string(), *expected);
        }
    }

    #[test]
    fn test_playbook_run_unicode_output() {
        let run = PlaybookRun {
            id: 1, project_id: 1, playbook_id: 1, task_id: None, template_id: None,
            status: PlaybookRunStatus::Success, inventory_id: None, environment_id: None,
            extra_vars: None, limit_hosts: None, tags: None, skip_tags: None,
            start_time: Some(Utc::now()), end_time: Some(Utc::now()),
            duration_seconds: Some(60), hosts_total: Some(1), hosts_changed: Some(0),
            hosts_unreachable: Some(0), hosts_failed: Some(0),
            output: Some("Вывод с русским текстом".to_string()),
            error_message: None, user_id: None,
            created: Utc::now(), updated: Utc::now(),
        };
        let json = serde_json::to_string(&run).unwrap();
        let restored: PlaybookRun = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.output, Some("Вывод с русским текстом".to_string()));
    }

    #[test]
    fn test_playbook_run_clone_independence() {
        let mut run = PlaybookRun {
            id: 1, project_id: 1, playbook_id: 1, task_id: None, template_id: None,
            status: PlaybookRunStatus::Running, inventory_id: None, environment_id: None,
            extra_vars: None, limit_hosts: None, tags: None, skip_tags: None,
            start_time: None, end_time: None, duration_seconds: None,
            hosts_total: None, hosts_changed: None, hosts_unreachable: None, hosts_failed: None,
            output: None, error_message: None, user_id: None,
            created: Utc::now(), updated: Utc::now(),
        };
        let cloned = run.clone();
        run.status = PlaybookRunStatus::Failed;
        assert_eq!(cloned.status, PlaybookRunStatus::Running);
    }

    #[test]
    fn test_playbook_run_stats_roundtrip() {
        let original = PlaybookRunStats {
            total_runs: 500, success_runs: 450, failed_runs: 30,
            avg_duration_seconds: Some(120.5), last_run: Some(Utc::now()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: PlaybookRunStats = serde_json::from_str(&json).unwrap();
        assert_eq!(original.total_runs, restored.total_runs);
        assert_eq!(original.avg_duration_seconds, restored.avg_duration_seconds);
    }

    #[test]
    fn test_playbook_run_update_all_fields() {
        let update = PlaybookRunUpdate {
            status: Some(PlaybookRunStatus::Success),
            start_time: Some(Utc::now()),
            end_time: Some(Utc::now()),
            duration_seconds: Some(300),
            hosts_total: Some(10),
            hosts_changed: Some(5),
            hosts_unreachable: Some(1),
            hosts_failed: Some(2),
            output: Some("Full output".to_string()),
            error_message: Some("Some error".to_string()),
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"duration_seconds\":300"));
        assert!(json.contains("\"hosts_total\":10"));
    }
}
