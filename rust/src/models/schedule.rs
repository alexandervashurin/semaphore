//! Модель расписания

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Расписание - автоматический запуск задач
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Schedule {
    #[serde(default)]
    pub id: i32,
    pub template_id: i32,
    #[serde(default)]
    pub project_id: i32,
    #[serde(default)]
    pub cron: String,
    #[serde(default)]
    pub cron_format: Option<String>,
    pub name: String,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub last_commit_hash: Option<String>,
    #[serde(default)]
    pub repository_id: Option<i32>,
    #[serde(default)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_serialization() {
        let schedule = Schedule {
            id: 1,
            template_id: 10,
            project_id: 5,
            cron: "0 * * * *".to_string(),
            cron_format: Some("standard".to_string()),
            name: "Hourly Deploy".to_string(),
            active: true,
            last_commit_hash: Some("abc123".to_string()),
            repository_id: Some(3),
            created: Some("2024-01-01T00:00:00Z".to_string()),
            run_at: None,
            delete_after_run: false,
        };
        let json = serde_json::to_string(&schedule).unwrap();
        assert!(json.contains("\"name\":\"Hourly Deploy\""));
        assert!(json.contains("\"cron\":\"0 * * * *\""));
        assert!(json.contains("\"active\":true"));
    }

    #[test]
    fn test_schedule_skip_nulls() {
        let schedule = Schedule {
            id: 1,
            template_id: 10,
            project_id: 5,
            cron: "0 0 * * *".to_string(),
            cron_format: None,
            name: "Daily".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        let json = serde_json::to_string(&schedule).unwrap();
        // run_at uses skip_serializing_if so it's omitted when None
        assert!(!json.contains("\"run_at\":"));
        // last_commit_hash and repository_id use #[serde(default)] so they serialize
        assert!(json.contains("\"last_commit_hash\":null"));
        assert!(json.contains("\"repository_id\":null"));
    }

    #[test]
    fn test_schedule_run_at_serialization() {
        let schedule = Schedule {
            id: 1,
            template_id: 10,
            project_id: 5,
            cron: String::new(),
            cron_format: Some("run_at".to_string()),
            name: "One-time deploy".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: Some("2024-06-15T10:00:00Z".to_string()),
            delete_after_run: true,
        };
        let json = serde_json::to_string(&schedule).unwrap();
        assert!(json.contains("\"run_at\":\"2024-06-15T10:00:00Z\""));
        assert!(json.contains("\"delete_after_run\":true"));
    }

    #[test]
    fn test_schedule_with_tpl_serialization() {
        let schedule = Schedule {
            id: 1,
            template_id: 10,
            project_id: 5,
            cron: "*/5 * * * *".to_string(),
            cron_format: None,
            name: "Frequent".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        let with_tpl = ScheduleWithTpl {
            schedule,
            tpl_playbook: Some("deploy.yml".to_string()),
        };
        let json = serde_json::to_string(&with_tpl).unwrap();
        assert!(json.contains("\"tpl_playbook\":\"deploy.yml\""));
        assert!(json.contains("\"name\":\"Frequent\""));
    }

    #[test]
    fn test_schedule_clone() {
        let schedule = Schedule {
            id: 1,
            template_id: 10,
            project_id: 5,
            cron: "0 * * * *".to_string(),
            cron_format: None,
            name: "Clone Test".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        let cloned = schedule.clone();
        assert_eq!(cloned.id, schedule.id);
        assert_eq!(cloned.name, schedule.name);
        assert_eq!(cloned.active, schedule.active);
    }

    #[test]
    fn test_schedule_default_values() {
        let schedule = Schedule {
            id: 0,
            template_id: 0,
            project_id: 0,
            cron: String::new(),
            cron_format: None,
            name: String::new(),
            active: false,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        assert!(schedule.cron.is_empty());
        assert!(schedule.name.is_empty());
        assert!(!schedule.active);
        assert!(!schedule.delete_after_run);
    }
}
