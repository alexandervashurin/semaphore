//! NotificationPolicyManager - управление политиками уведомлений

use crate::db::sql::SqlStore;
use crate::db::store::NotificationPolicyManager;
use crate::error::{Error, Result};
use crate::models::notification::{
    NotificationPolicy, NotificationPolicyCreate, NotificationPolicyUpdate,
};
use async_trait::async_trait;

#[async_trait]
impl NotificationPolicyManager for SqlStore {
    async fn get_notification_policies(&self, project_id: i32) -> Result<Vec<NotificationPolicy>> {
        let rows = sqlx::query_as::<_, NotificationPolicy>(
            "SELECT * FROM notification_policy WHERE project_id = $1 ORDER BY name",
        )
        .bind(project_id)
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(rows)
    }

    async fn get_notification_policy(
        &self,
        id: i32,
        project_id: i32,
    ) -> Result<NotificationPolicy> {
        let row = sqlx::query_as::<_, NotificationPolicy>(
            "SELECT * FROM notification_policy WHERE id = $1 AND project_id = $2",
        )
        .bind(id)
        .bind(project_id)
        .fetch_one(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(row)
    }

    async fn create_notification_policy(
        &self,
        project_id: i32,
        payload: NotificationPolicyCreate,
    ) -> Result<NotificationPolicy> {
        let enabled = payload.enabled.unwrap_or(true);
        let row = sqlx::query_as::<_, NotificationPolicy>(
                "INSERT INTO notification_policy (project_id, name, channel_type, webhook_url, trigger, template_id, enabled, created)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, NOW()) RETURNING *"
            )
            .bind(project_id)
            .bind(&payload.name)
            .bind(&payload.channel_type)
            .bind(&payload.webhook_url)
            .bind(&payload.trigger)
            .bind(payload.template_id)
            .bind(enabled)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(row)
    }

    async fn update_notification_policy(
        &self,
        id: i32,
        project_id: i32,
        payload: NotificationPolicyUpdate,
    ) -> Result<NotificationPolicy> {
        let row = sqlx::query_as::<_, NotificationPolicy>(
                "UPDATE notification_policy SET name = $1, channel_type = $2, webhook_url = $3, trigger = $4, template_id = $5, enabled = $6
                 WHERE id = $7 AND project_id = $8 RETURNING *"
            )
            .bind(&payload.name)
            .bind(&payload.channel_type)
            .bind(&payload.webhook_url)
            .bind(&payload.trigger)
            .bind(payload.template_id)
            .bind(payload.enabled)
            .bind(id)
            .bind(project_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(row)
    }

    async fn delete_notification_policy(&self, id: i32, project_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM notification_policy WHERE id = $1 AND project_id = $2")
            .bind(id)
            .bind(project_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn get_matching_policies(
        &self,
        project_id: i32,
        trigger: &str,
        template_id: Option<i32>,
    ) -> Result<Vec<NotificationPolicy>> {
        let rows = sqlx::query_as::<_, NotificationPolicy>(
            "SELECT * FROM notification_policy
                 WHERE project_id = $1 AND enabled = TRUE
                   AND (trigger = $2 OR trigger = 'always')
                   AND (template_id IS NULL OR template_id = $3)",
        )
        .bind(project_id)
        .bind(trigger)
        .bind(template_id)
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::notification::{
        NotificationChannelType, NotificationPolicy, NotificationPolicyCreate,
        NotificationPolicyUpdate,
    };
    use chrono::Utc;

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

        let json = serde_json::to_string(&NotificationChannelType::Teams).unwrap();
        assert_eq!(json, "\"teams\"");
    }

    #[test]
    fn test_notification_policy_create_serialization() {
        let payload = NotificationPolicyCreate {
            name: "On Failure".to_string(),
            channel_type: "slack".to_string(),
            webhook_url: "https://hooks.slack.com/xxx".to_string(),
            trigger: "on_failure".to_string(),
            template_id: Some(5),
            enabled: Some(true),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"name\":\"On Failure\""));
        assert!(json.contains("\"channel_type\":\"slack\""));
        assert!(json.contains("\"trigger\":\"on_failure\""));
        assert!(json.contains("\"template_id\":5"));
    }

    #[test]
    fn test_notification_policy_create_with_nulls() {
        let payload = NotificationPolicyCreate {
            name: "Always Notify".to_string(),
            channel_type: "generic".to_string(),
            webhook_url: "https://example.com/hook".to_string(),
            trigger: "always".to_string(),
            template_id: None,
            enabled: None,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"template_id\":null"));
        assert!(json.contains("\"enabled\":null"));
    }

    #[test]
    fn test_notification_policy_update_serialization() {
        let payload = NotificationPolicyUpdate {
            name: "Updated Policy".to_string(),
            channel_type: "teams".to_string(),
            webhook_url: "https://outlook.office.com/hook".to_string(),
            trigger: "on_success".to_string(),
            template_id: Some(10),
            enabled: false,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"name\":\"Updated Policy\""));
        assert!(json.contains("\"channel_type\":\"teams\""));
        assert!(json.contains("\"enabled\":false"));
    }

    #[test]
    fn test_notification_policy_create_deserialize() {
        let json = r#"{"name":"Test","channel_type":"teams","webhook_url":"https://example.com","trigger":"on_success","template_id":null,"enabled":true}"#;
        let create: NotificationPolicyCreate = serde_json::from_str(json).unwrap();
        assert_eq!(create.name, "Test");
        assert_eq!(create.channel_type, "teams");
        assert_eq!(create.trigger, "on_success");
        assert!(create.template_id.is_none());
        assert_eq!(create.enabled, Some(true));
    }

    #[test]
    fn test_notification_policy_update_deserialize() {
        let json = r#"{"name":"Upd","channel_type":"generic","webhook_url":"https://example.com","trigger":"always","template_id":null,"enabled":false}"#;
        let update: NotificationPolicyUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(update.name, "Upd");
        assert!(!update.enabled);
    }

    #[test]
    fn test_notification_channel_type_all_variants() {
        let variants = vec![
            (NotificationChannelType::Slack, "slack"),
            (NotificationChannelType::Teams, "teams"),
            (NotificationChannelType::PagerDuty, "pagerduty"),
            (NotificationChannelType::Generic, "generic"),
        ];
        for (variant, expected) in variants {
            assert_eq!(variant.to_string(), expected);
        }
    }

    #[test]
    fn test_notification_policy_create_clone() {
        let payload = NotificationPolicyCreate {
            name: "Clone Test".to_string(),
            channel_type: "slack".to_string(),
            webhook_url: "https://hooks.slack.com/xxx".to_string(),
            trigger: "on_start".to_string(),
            template_id: None,
            enabled: Some(true),
        };
        let cloned = payload.clone();
        assert_eq!(cloned.name, payload.name);
        assert_eq!(cloned.channel_type, payload.channel_type);
    }

    #[test]
    fn test_notification_policy_update_clone() {
        let payload = NotificationPolicyUpdate {
            name: "Clone Update".to_string(),
            channel_type: "pagerduty".to_string(),
            webhook_url: "https://events.pagerduty.com/xxx".to_string(),
            trigger: "on_failure".to_string(),
            template_id: Some(1),
            enabled: true,
        };
        let cloned = payload.clone();
        assert_eq!(cloned.name, payload.name);
        assert_eq!(cloned.enabled, payload.enabled);
    }
}
