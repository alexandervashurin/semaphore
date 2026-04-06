use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Drift check configuration for a template
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DriftConfig {
    pub id: i32,
    pub project_id: i32,
    pub template_id: i32,
    pub enabled: bool,
    /// Cron expression for auto-check schedule (NULL = manual only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<String>,
    pub created: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftConfigCreate {
    pub template_id: i32,
    pub enabled: Option<bool>,
    pub schedule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftConfigUpdate {
    pub enabled: Option<bool>,
    pub schedule: Option<String>,
}

/// Result of a drift check run
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DriftResult {
    pub id: i32,
    pub drift_config_id: i32,
    pub project_id: i32,
    pub template_id: i32,
    /// "clean" | "drifted" | "error" | "pending"
    pub status: String,
    /// Summary of detected changes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// task_id of the check run
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<i32>,
    pub checked_at: DateTime<Utc>,
}

/// DriftConfig with latest result for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftConfigWithStatus {
    #[serde(flatten)]
    pub config: DriftConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_result: Option<DriftResult>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_config_serialization() {
        let config = DriftConfig {
            id: 1,
            project_id: 10,
            template_id: 5,
            enabled: true,
            schedule: Some("0 * * * *".to_string()),
            created: Utc::now(),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"schedule\":\"0 * * * *\""));
    }

    #[test]
    fn test_drift_config_create_serialization() {
        let create = DriftConfigCreate {
            template_id: 5,
            enabled: Some(true),
            schedule: Some("daily".to_string()),
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"template_id\":5"));
        assert!(json.contains("\"enabled\":true"));
    }

    #[test]
    fn test_drift_result_serialization() {
        let result = DriftResult {
            id: 1,
            drift_config_id: 10,
            project_id: 5,
            template_id: 3,
            status: "drifted".to_string(),
            summary: Some("3 resources changed".to_string()),
            task_id: Some(100),
            checked_at: Utc::now(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"status\":\"drifted\""));
        assert!(json.contains("\"summary\":\"3 resources changed\""));
    }

    #[test]
    fn test_drift_config_with_status_serialization() {
        let config = DriftConfig {
            id: 1,
            project_id: 10,
            template_id: 5,
            enabled: true,
            schedule: None,
            created: Utc::now(),
        };
        let with_status = DriftConfigWithStatus {
            config,
            latest_result: None,
        };
        let json = serde_json::to_string(&with_status).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(!json.contains("\"latest_result\":"));
    }
}
