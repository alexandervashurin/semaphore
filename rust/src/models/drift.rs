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
