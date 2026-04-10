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
                   ORDER BY c.id DESC LIMIT $2"#,
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)
    }

    async fn get_cost_estimate_for_task(
        &self,
        project_id: i32,
        task_id: i32,
    ) -> Result<Option<CostEstimate>> {
        sqlx::query_as::<_, CostEstimate>(
            r#"SELECT c.*, COALESCE(t.name,'') AS template_name
                   FROM cost_estimate c
                   LEFT JOIN template t ON t.id = c.template_id
                   WHERE c.project_id = $1 AND c.task_id = $2
                   LIMIT 1"#,
        )
        .bind(project_id)
        .bind(task_id)
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
                   RETURNING *, '' AS template_name"#,
        )
        .bind(payload.project_id)
        .bind(payload.task_id)
        .bind(payload.template_id)
        .bind(currency)
        .bind(payload.monthly_cost)
        .bind(payload.monthly_cost_diff)
        .bind(resource_count)
        .bind(resources_added)
        .bind(resources_changed)
        .bind(resources_deleted)
        .bind(&payload.breakdown_json)
        .bind(&payload.infracost_version)
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
                   ORDER BY last_run_at DESC"#,
        )
        .bind(project_id)
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::cost_estimate::{CostEstimate, CostEstimateCreate, CostSummary};

    #[test]
    fn test_cost_estimate_serialization() {
        let estimate = CostEstimate {
            id: 1,
            project_id: 10,
            task_id: 100,
            template_id: 5,
            currency: "USD".to_string(),
            monthly_cost: Some(150.50),
            monthly_cost_diff: Some(25.0),
            resource_count: 10,
            resources_added: 3,
            resources_changed: 2,
            resources_deleted: 1,
            breakdown_json: Some(r#"{"aws_instance": 100}"#.to_string()),
            infracost_version: Some("v0.10.0".to_string()),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            template_name: "Deploy AWS".to_string(),
        };
        let json = serde_json::to_string(&estimate).unwrap();
        assert!(json.contains("\"currency\":\"USD\""));
        assert!(json.contains("\"monthly_cost\":150.5"));
    }

    #[test]
    fn test_cost_estimate_create_serialization() {
        let create = CostEstimateCreate {
            project_id: 10,
            task_id: 100,
            template_id: 5,
            currency: Some("USD".to_string()),
            monthly_cost: Some(100.0),
            monthly_cost_diff: None,
            resource_count: Some(5),
            resources_added: Some(2),
            resources_changed: Some(1),
            resources_deleted: Some(0),
            breakdown_json: None,
            infracost_version: None,
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"project_id\":10"));
        assert!(json.contains("\"currency\":\"USD\""));
    }

    #[test]
    fn test_cost_summary_serialization() {
        let summary = CostSummary {
            template_id: 5,
            template_name: "Deploy Infra".to_string(),
            latest_monthly_cost: Some(200.0),
            runs_with_cost: 15,
            last_run_at: "2024-01-15T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"template_name\":\"Deploy Infra\""));
        assert!(json.contains("\"runs_with_cost\":15"));
    }

    #[test]
    fn test_cost_estimate_null_fields() {
        let estimate = CostEstimate {
            id: 1,
            project_id: 1,
            task_id: 1,
            template_id: 1,
            currency: "USD".to_string(),
            monthly_cost: None,
            monthly_cost_diff: None,
            resource_count: 0,
            resources_added: 0,
            resources_changed: 0,
            resources_deleted: 0,
            breakdown_json: None,
            infracost_version: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            template_name: String::new(),
        };
        let json = serde_json::to_string(&estimate).unwrap();
        assert!(json.contains("\"monthly_cost\":null"));
        assert!(json.contains("\"breakdown_json\":null"));
    }

    #[test]
    fn test_cost_estimate_clone() {
        let estimate = CostEstimate {
            id: 1, project_id: 1, task_id: 1, template_id: 1,
            currency: "EUR".to_string(),
            monthly_cost: Some(50.0), monthly_cost_diff: Some(10.0),
            resource_count: 5, resources_added: 1, resources_changed: 0,
            resources_deleted: 0, breakdown_json: None,
            infracost_version: None, created_at: "2024-01-01".to_string(),
            template_name: "Test".to_string(),
        };
        let cloned = estimate.clone();
        assert_eq!(cloned.currency, estimate.currency);
        assert_eq!(cloned.resource_count, estimate.resource_count);
    }

    #[test]
    fn test_cost_estimate_create_clone() {
        let create = CostEstimateCreate {
            project_id: 2, task_id: 20, template_id: 3,
            currency: Some("RUB".to_string()), monthly_cost: Some(1000.0),
            monthly_cost_diff: None, resource_count: Some(1),
            resources_added: None, resources_changed: None, resources_deleted: None,
            breakdown_json: None, infracost_version: None,
        };
        let cloned = create.clone();
        assert_eq!(cloned.currency, create.currency);
    }

    #[test]
    fn test_cost_summary_clone() {
        let summary = CostSummary {
            template_id: 1, template_name: "Summary".to_string(),
            latest_monthly_cost: Some(0.0), runs_with_cost: 0,
            last_run_at: String::new(),
        };
        let cloned = summary.clone();
        assert_eq!(cloned.template_name, summary.template_name);
    }

    #[test]
    fn test_cost_estimate_deserialize() {
        let json = r#"{"id":5,"project_id":10,"task_id":20,"template_id":3,"currency":"USD","monthly_cost":null,"monthly_cost_diff":null,"resource_count":0,"resources_added":0,"resources_changed":0,"resources_deleted":0,"breakdown_json":null,"infracost_version":null,"created_at":"2024-01-01","template_name":""}"#;
        let estimate: CostEstimate = serde_json::from_str(json).unwrap();
        assert_eq!(estimate.id, 5);
        assert_eq!(estimate.currency, "USD");
    }
}
