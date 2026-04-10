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

    #[test]
    fn test_notification_channel_type_all_variants() {
        let types = [
            NotificationChannelType::Slack,
            NotificationChannelType::Teams,
            NotificationChannelType::PagerDuty,
            NotificationChannelType::Generic,
        ];
        for t in &types {
            let json = serde_json::to_string(t).unwrap();
            assert!(json.starts_with('"') && json.ends_with('"'));
        }
    }

    #[test]
    fn test_notification_policy_clone() {
        let policy = NotificationPolicy {
            id: 1,
            project_id: 10,
            name: "Clone Test".to_string(),
            channel_type: "slack".to_string(),
            webhook_url: "https://hooks.slack.com/xxx".to_string(),
            trigger: "on_failure".to_string(),
            template_id: None,
            enabled: true,
            created: Utc::now(),
        };
        let cloned = policy.clone();
        assert_eq!(cloned.name, policy.name);
        assert_eq!(cloned.id, policy.id);
    }

    #[test]
    fn test_notification_policy_create_deserialize() {
        let json = r#"{"name":"Test","channel_type":"teams","webhook_url":"https://example.com","trigger":"on_success","template_id":null,"enabled":true}"#;
        let create: NotificationPolicyCreate = serde_json::from_str(json).unwrap();
        assert_eq!(create.name, "Test");
        assert_eq!(create.channel_type, "teams");
    }

    #[test]
    fn test_notification_policy_clone_full() {
        let policy = NotificationPolicy {
            id: 5, project_id: 20, name: "Clone Policy".to_string(),
            channel_type: "generic".to_string(), webhook_url: "https://clone.com".to_string(),
            trigger: "always".to_string(), template_id: Some(10), enabled: true,
            created: Utc::now(),
        };
        let cloned = policy.clone();
        assert_eq!(cloned.template_id, policy.template_id);
        assert_eq!(cloned.trigger, policy.trigger);
    }

    #[test]
    fn test_notification_policy_create_clone() {
        let create = NotificationPolicyCreate {
            name: "Clone Create".to_string(), channel_type: "slack".to_string(),
            webhook_url: "https://clone-create.com".to_string(), trigger: "on_failure".to_string(),
            template_id: None, enabled: Some(false),
        };
        let cloned = create.clone();
        assert_eq!(cloned.name, create.name);
    }

    #[test]
    fn test_notification_policy_update_clone() {
        let update = NotificationPolicyUpdate {
            name: "Clone Update".to_string(), channel_type: "teams".to_string(),
            webhook_url: "https://clone-update.com".to_string(), trigger: "on_start".to_string(),
            template_id: Some(5), enabled: true,
        };
        let cloned = update.clone();
        assert_eq!(cloned.enabled, update.enabled);
    }

    #[test]
    fn test_notification_channel_type_clone() {
        let t = NotificationChannelType::PagerDuty;
        let cloned = t.clone();
        assert_eq!(format!("{:?}", t), format!("{:?}", cloned));
    }

    #[test]
    fn test_notification_policy_deserialization() {
        let json = r#"{"id":10,"project_id":50,"name":"Deser Policy","channel_type":"generic","webhook_url":"https://deser.com","trigger":"on_success","template_id":null,"enabled":false,"created":"2024-01-01T00:00:00Z"}"#;
        let policy: NotificationPolicy = serde_json::from_str(json).unwrap();
        assert_eq!(policy.id, 10);
        assert_eq!(policy.channel_type, "generic");
        assert!(!policy.enabled);
    }

    #[test]
    fn test_notification_policy_create_with_template_id() {
        let create = NotificationPolicyCreate {
            name: "With Template".to_string(), channel_type: "slack".to_string(),
            webhook_url: "https://example.com".to_string(), trigger: "on_failure".to_string(),
            template_id: Some(42), enabled: Some(true),
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"template_id\":42"));
    }

    #[test]
    fn test_notification_policy_all_triggers() {
        let triggers = ["on_failure", "on_success", "on_start", "always"];
        for trigger in triggers {
            let policy = NotificationPolicy {
                id: 1, project_id: 1, name: "Trigger Test".to_string(),
                channel_type: "generic".to_string(), webhook_url: "https://example.com".to_string(),
                trigger: trigger.to_string(), template_id: None, enabled: true,
                created: Utc::now(),
            };
            let json = serde_json::to_string(&policy).unwrap();
            assert!(json.contains(&format!("\"trigger\":\"{}\"", trigger)));
        }
    }
}
