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

    #[test]
    fn test_schedule_clone_full() {
        let schedule = Schedule {
            id: 5,
            template_id: 20,
            project_id: 10,
            cron: "*/10 * * * *".to_string(),
            cron_format: Some("standard".to_string()),
            name: "Clone Full".to_string(),
            active: true,
            last_commit_hash: Some("def456".to_string()),
            repository_id: Some(2),
            created: Some("2024-01-01".to_string()),
            run_at: None,
            delete_after_run: false,
        };
        let cloned = schedule.clone();
        assert_eq!(cloned.id, schedule.id);
        assert_eq!(cloned.cron, schedule.cron);
        assert_eq!(cloned.repository_id, schedule.repository_id);
    }

    #[test]
    fn test_schedule_debug() {
        let schedule = Schedule {
            id: 1,
            template_id: 1,
            project_id: 1,
            cron: "0 0 * * *".to_string(),
            cron_format: None,
            name: "Debug Schedule".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        let debug_str = format!("{:?}", schedule);
        assert!(debug_str.contains("Schedule"));
        assert!(debug_str.contains("Debug Schedule"));
    }

    #[test]
    fn test_schedule_deserialization_full() {
        let json = r#"{"id":10,"template_id":50,"project_id":25,"cron":"0 12 * * *","cron_format":"standard","name":"Noon Deploy","active":true,"last_commit_hash":"abc","repository_id":3,"created":"2024-01-01","run_at":null,"delete_after_run":false}"#;
        let schedule: Schedule = serde_json::from_str(json).unwrap();
        assert_eq!(schedule.id, 10);
        assert_eq!(schedule.cron, "0 12 * * *");
        assert_eq!(schedule.name, "Noon Deploy");
    }

    #[test]
    fn test_schedule_with_tpl_clone() {
        let schedule = Schedule {
            id: 1,
            template_id: 10,
            project_id: 5,
            cron: "0 * * * *".to_string(),
            cron_format: None,
            name: "With Tpl".to_string(),
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
        let cloned = with_tpl.clone();
        assert_eq!(cloned.tpl_playbook, with_tpl.tpl_playbook);
    }

    #[test]
    fn test_schedule_run_at_false() {
        let schedule = Schedule {
            id: 1,
            template_id: 1,
            project_id: 1,
            cron: "".to_string(),
            cron_format: None,
            name: "Test".to_string(),
            active: false,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        assert!(!schedule.active);
        assert!(!schedule.delete_after_run);
    }

    #[test]
    fn test_schedule_serialization_all_fields_populated() {
        let schedule = Schedule {
            id: 1,
            template_id: 1,
            project_id: 1,
            cron: "1 2 3 4 5".to_string(),
            cron_format: Some("custom".to_string()),
            name: "All Fields".to_string(),
            active: true,
            last_commit_hash: Some("hash1".to_string()),
            repository_id: Some(1),
            created: Some("2024-01-01T00:00:00Z".to_string()),
            run_at: Some("2024-06-01T12:00:00Z".to_string()),
            delete_after_run: true,
        };
        let json = serde_json::to_string(&schedule).unwrap();
        assert!(json.contains("\"cron\":\"1 2 3 4 5\""));
        assert!(json.contains("\"delete_after_run\":true"));
        assert!(json.contains("\"run_at\":\"2024-06-01T12:00:00Z\""));
    }

    #[test]
    fn test_schedule_unicode_name() {
        let schedule = Schedule {
            id: 1,
            template_id: 1,
            project_id: 1,
            cron: "0 * * * *".to_string(),
            cron_format: None,
            name: "Расписание".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        let json = serde_json::to_string(&schedule).unwrap();
        let restored: Schedule = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, "Расписание");
    }

    #[test]
    fn test_schedule_clone_independence() {
        let mut schedule = Schedule {
            id: 1,
            template_id: 1,
            project_id: 1,
            cron: "0 * * * *".to_string(),
            cron_format: None,
            name: "Original".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        let cloned = schedule.clone();
        schedule.name = "Modified".to_string();
        assert_eq!(cloned.name, "Original");
    }

    #[test]
    fn test_schedule_roundtrip() {
        let original = Schedule {
            id: 77,
            template_id: 33,
            project_id: 11,
            cron: "*/15 * * * *".to_string(),
            cron_format: Some("standard".to_string()),
            name: "Roundtrip".to_string(),
            active: true,
            last_commit_hash: Some("abc123".to_string()),
            repository_id: Some(5),
            created: Some("2024-01-01".to_string()),
            run_at: None,
            delete_after_run: false,
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: Schedule = serde_json::from_str(&json).unwrap();
        assert_eq!(original.id, restored.id);
        assert_eq!(original.cron, restored.cron);
        assert_eq!(original.name, restored.name);
    }

    #[test]
    fn test_schedule_with_tpl_default() {
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
        let with_tpl = ScheduleWithTpl {
            schedule,
            tpl_playbook: None,
        };
        assert!(with_tpl.tpl_playbook.is_none());
    }
}
