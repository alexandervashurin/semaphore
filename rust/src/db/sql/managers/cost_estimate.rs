//! Terraform Cost Estimate SQL Manager

use crate::db::sql::SqlStore;
use crate::db::store::CostEstimateManager;
use crate::error::{Error, Result};
use crate::models::cost_estimate::{CostEstimate, CostEstimateCreate, CostSummary};
use async_trait::async_trait;

#[async_trait]
impl CostEstimateManager for SqlStore {
    async fn get_cost_estimates(&self, project_id: i32, limit: i64) -> Result<Vec<CostEstimate>> {
        sqlx::query_as::<_, CostEstimate>(
                r#"SELECT c.*, COALESCE(t.name,'') AS template_name
                   FROM cost_estimate c
                   LEFT JOIN template t ON t.id = c.template_id
                   WHERE c.project_id = $1
                   ORDER BY c.id DESC LIMIT $2"#
            )
            .bind(project_id).bind(limit)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)
    }

    async fn get_cost_estimate_for_task(&self, project_id: i32, task_id: i32) -> Result<Option<CostEstimate>> {
        sqlx::query_as::<_, CostEstimate>(
                r#"SELECT c.*, COALESCE(t.name,'') AS template_name
                   FROM cost_estimate c
                   LEFT JOIN template t ON t.id = c.template_id
                   WHERE c.project_id = $1 AND c.task_id = $2
                   LIMIT 1"#
            )
            .bind(project_id).bind(task_id)
            .fetch_optional(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)
    }

    async fn create_cost_estimate(&self, payload: CostEstimateCreate) -> Result<CostEstimate> {
        let currency = payload.currency.as_deref().unwrap_or("USD");
        let resource_count = payload.resource_count.unwrap_or(0);
        let resources_added = payload.resources_added.unwrap_or(0);
        let resources_changed = payload.resources_changed.unwrap_or(0);
        let resources_deleted = payload.resources_deleted.unwrap_or(0);

        sqlx::query_as::<_, CostEstimate>(
                r#"INSERT INTO cost_estimate
                   (project_id, task_id, template_id, currency, monthly_cost, monthly_cost_diff,
                    resource_count, resources_added, resources_changed, resources_deleted,
                    breakdown_json, infracost_version)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                   RETURNING *, '' AS template_name"#
            )
            .bind(payload.project_id).bind(payload.task_id).bind(payload.template_id)
            .bind(currency).bind(payload.monthly_cost).bind(payload.monthly_cost_diff)
            .bind(resource_count).bind(resources_added).bind(resources_changed).bind(resources_deleted)
            .bind(&payload.breakdown_json).bind(&payload.infracost_version)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)
    }

    async fn get_cost_summaries(&self, project_id: i32) -> Result<Vec<CostSummary>> {
        sqlx::query_as::<_, CostSummary>(
                r#"SELECT
                       c.template_id,
                       COALESCE(t.name, 'Шаблон #' || c.template_id::text) AS template_name,
                       (SELECT monthly_cost FROM cost_estimate
                        WHERE project_id = c.project_id AND template_id = c.template_id
                        ORDER BY id DESC LIMIT 1) AS latest_monthly_cost,
                       COUNT(*) AS runs_with_cost,
                       MAX(c.created_at)::text AS last_run_at
                   FROM cost_estimate c
                   LEFT JOIN template t ON t.id = c.template_id
                   WHERE c.project_id = $1
                   GROUP BY c.template_id, t.name, c.project_id
                   ORDER BY last_run_at DESC"#
            )
            .bind(project_id)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)
    }
}
