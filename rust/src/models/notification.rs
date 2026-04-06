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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_channel_type_display() {
        assert_eq!(NotificationChannelType::Slack.to_string(), "slack");
        assert_eq!(NotificationChannelType::Teams.to_string(), "teams");
        assert_eq!(NotificationChannelType::PagerDuty.to_string(), "pagerduty");
        assert_eq!(NotificationChannelType::Generic.to_string(), "generic");
    }

    #[test]
    fn test_notification_channel_type_serialization() {
        let json = serde_json::to_string(&NotificationChannelType::Slack).unwrap();
        assert_eq!(json, "\"slack\"");
    }

    #[test]
    fn test_notification_policy_serialization() {
        let policy = NotificationPolicy {
            id: 1,
            project_id: 10,
            name: "On Failure".to_string(),
            channel_type: "slack".to_string(),
            webhook_url: "https://hooks.slack.com/xxx".to_string(),
            trigger: "on_failure".to_string(),
            template_id: None,
            enabled: true,
            created: Utc::now(),
        };
        let json = serde_json::to_string(&policy).unwrap();
        assert!(json.contains("\"name\":\"On Failure\""));
        assert!(json.contains("\"channel_type\":\"slack\""));
        assert!(json.contains("\"enabled\":true"));
    }

    #[test]
    fn test_notification_policy_create_serialization() {
        let create = NotificationPolicyCreate {
            name: "New Policy".to_string(),
            channel_type: "teams".to_string(),
            webhook_url: "https://outlook.office.com/xxx".to_string(),
            trigger: "on_success".to_string(),
            template_id: Some(5),
            enabled: Some(true),
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"name\":\"New Policy\""));
        assert!(json.contains("\"template_id\":5"));
    }

    #[test]
    fn test_notification_policy_update_serialization() {
        let update = NotificationPolicyUpdate {
            name: "Updated Policy".to_string(),
            channel_type: "generic".to_string(),
            webhook_url: "https://example.com/webhook".to_string(),
            trigger: "always".to_string(),
            template_id: None,
            enabled: false,
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"enabled\":false"));
        assert!(json.contains("\"trigger\":\"always\""));
    }
}
