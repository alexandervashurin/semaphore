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
                "SELECT * FROM notification_policy WHERE project_id = $1 ORDER BY name"
            )
            .bind(project_id)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(rows)
    }

    async fn get_notification_policy(&self, id: i32, project_id: i32) -> Result<NotificationPolicy> {
        let row = sqlx::query_as::<_, NotificationPolicy>(
                "SELECT * FROM notification_policy WHERE id = $1 AND project_id = $2"
            )
            .bind(id)
            .bind(project_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(row)
    }

    async fn create_notification_policy(&self, project_id: i32, payload: NotificationPolicyCreate) -> Result<NotificationPolicy> {
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

    async fn update_notification_policy(&self, id: i32, project_id: i32, payload: NotificationPolicyUpdate) -> Result<NotificationPolicy> {
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

    async fn get_matching_policies(&self, project_id: i32, trigger: &str, template_id: Option<i32>) -> Result<Vec<NotificationPolicy>> {
        let rows = sqlx::query_as::<_, NotificationPolicy>(
                "SELECT * FROM notification_policy
                 WHERE project_id = $1 AND enabled = TRUE
                   AND (trigger = $2 OR trigger = 'always')
                   AND (template_id IS NULL OR template_id = $3)"
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
