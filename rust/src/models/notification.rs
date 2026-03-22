use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Notification channel type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationChannelType {
    Slack,
    Teams,
    PagerDuty,
    Generic,
}

impl std::fmt::Display for NotificationChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            NotificationChannelType::Slack => "slack",
            NotificationChannelType::Teams => "teams",
            NotificationChannelType::PagerDuty => "pagerduty",
            NotificationChannelType::Generic => "generic",
        };
        write!(f, "{}", s)
    }
}

/// Notification policy: when to fire and where
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NotificationPolicy {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    /// Channel type: slack, teams, pagerduty, generic
    pub channel_type: String,
    /// Webhook URL
    pub webhook_url: String,
    /// Trigger: "on_failure" | "on_success" | "on_start" | "always"
    pub trigger: String,
    /// Optional: only fire for this template_id (NULL = all templates)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<i32>,
    /// Whether policy is enabled
    pub enabled: bool,
    pub created: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPolicyCreate {
    pub name: String,
    pub channel_type: String,
    pub webhook_url: String,
    pub trigger: String,
    pub template_id: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPolicyUpdate {
    pub name: String,
    pub channel_type: String,
    pub webhook_url: String,
    pub trigger: String,
    pub template_id: Option<i32>,
    pub enabled: bool,
}
