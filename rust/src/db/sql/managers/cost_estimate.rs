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
            id: 1,
            project_id: 1,
            task_id: 1,
            template_id: 1,
            currency: "EUR".to_string(),
            monthly_cost: Some(50.0),
            monthly_cost_diff: Some(10.0),
            resource_count: 5,
            resources_added: 1,
            resources_changed: 0,
            resources_deleted: 0,
            breakdown_json: None,
            infracost_version: None,
            created_at: "2024-01-01".to_string(),
            template_name: "Test".to_string(),
        };
        let cloned = estimate.clone();
        assert_eq!(cloned.currency, estimate.currency);
        assert_eq!(cloned.resource_count, estimate.resource_count);
    }

    #[test]
    fn test_cost_estimate_create_clone() {
        let create = CostEstimateCreate {
            project_id: 2,
            task_id: 20,
            template_id: 3,
            currency: Some("RUB".to_string()),
            monthly_cost: Some(1000.0),
            monthly_cost_diff: None,
            resource_count: Some(1),
            resources_added: None,
            resources_changed: None,
            resources_deleted: None,
            breakdown_json: None,
            infracost_version: None,
        };
        let cloned = create.clone();
        assert_eq!(cloned.currency, create.currency);
    }

    #[test]
    fn test_cost_summary_clone() {
        let summary = CostSummary {
            template_id: 1,
            template_name: "Summary".to_string(),
            latest_monthly_cost: Some(0.0),
            runs_with_cost: 0,
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

    #[test]
    fn test_cost_estimate_debug_format() {
        let estimate = CostEstimate {
            id: 1,
            project_id: 10,
            task_id: 100,
            template_id: 5,
            currency: "GBP".to_string(),
            monthly_cost: Some(300.0),
            monthly_cost_diff: Some(50.0),
            resource_count: 15,
            resources_added: 5,
            resources_changed: 3,
            resources_deleted: 2,
            breakdown_json: Some(r#"{"aws": 200}"#.to_string()),
            infracost_version: Some("v1.0.0".to_string()),
            created_at: "2024-02-01".to_string(),
            template_name: "AWS Deploy".to_string(),
        };
        let debug = format!("{:?}", estimate);
        assert!(debug.contains("CostEstimate"));
        assert!(debug.contains("GBP"));
    }

    #[test]
    fn test_cost_estimate_unicode_currency() {
        // While unusual, test that unicode works in currency field
        let estimate = CostEstimate {
            id: 1,
            project_id: 1,
            task_id: 1,
            template_id: 1,
            currency: "доллар".to_string(),
            monthly_cost: Some(100.0),
            monthly_cost_diff: None,
            resource_count: 0,
            resources_added: 0,
            resources_changed: 0,
            resources_deleted: 0,
            breakdown_json: None,
            infracost_version: None,
            created_at: "2024-01-01".to_string(),
            template_name: "Тест".to_string(),
        };
        let json = serde_json::to_string(&estimate).unwrap();
        assert!(json.contains("доллар"));

        let deserialized: CostEstimate = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.currency, "доллар");
    }

    #[test]
    fn test_cost_estimate_create_all_fields() {
        let create = CostEstimateCreate {
            project_id: 100,
            task_id: 1000,
            template_id: 50,
            currency: Some("EUR".to_string()),
            monthly_cost: Some(500.0),
            monthly_cost_diff: Some(100.0),
            resource_count: Some(25),
            resources_added: Some(10),
            resources_changed: Some(5),
            resources_deleted: Some(3),
            breakdown_json: Some(r#"{"azure_vm": 300}"#.to_string()),
            infracost_version: Some("v0.11.0".to_string()),
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"project_id\":100"));
        assert!(json.contains("\"monthly_cost\":500.0"));
        assert!(json.contains("\"resources_added\":10"));
    }

    #[test]
    fn test_cost_estimate_negative_values() {
        let estimate = CostEstimate {
            id: 1,
            project_id: 1,
            task_id: 1,
            template_id: 1,
            currency: "USD".to_string(),
            monthly_cost: Some(-50.0),
            monthly_cost_diff: Some(-25.0),
            resource_count: 0,
            resources_added: 0,
            resources_changed: 0,
            resources_deleted: 5,
            breakdown_json: None,
            infracost_version: None,
            created_at: "2024-01-01".to_string(),
            template_name: "Negative Test".to_string(),
        };
        let json = serde_json::to_string(&estimate).unwrap();
        assert!(json.contains("-50.0"));
        assert!(json.contains("-25.0"));
    }

    #[test]
    fn test_cost_summary_deserialization() {
        let json = r#"{
            "template_id": 10,
            "template_name": "Production Deploy",
            "latest_monthly_cost": 750.25,
            "runs_with_cost": 42,
            "last_run_at": "2024-03-15T14:30:00Z"
        }"#;
        let summary: CostSummary = serde_json::from_str(json).unwrap();
        assert_eq!(summary.template_id, 10);
        assert_eq!(summary.template_name, "Production Deploy");
        assert_eq!(summary.latest_monthly_cost, Some(750.25));
        assert_eq!(summary.runs_with_cost, 42);
    }

    #[test]
    fn test_cost_summary_null_cost() {
        let json = r#"{
            "template_id": 1,
            "template_name": "No Cost Template",
            "latest_monthly_cost": null,
            "runs_with_cost": 0,
            "last_run_at": ""
        }"#;
        let summary: CostSummary = serde_json::from_str(json).unwrap();
        assert_eq!(summary.latest_monthly_cost, None);
        assert_eq!(summary.runs_with_cost, 0);
    }

    #[test]
    fn test_cost_estimate_create_deserialization() {
        let json = r#"{
            "project_id": 5,
            "task_id": 50,
            "template_id": 2,
            "currency": "JPY",
            "monthly_cost": 10000.0,
            "monthly_cost_diff": 2000.0,
            "resource_count": 10,
            "resources_added": 4,
            "resources_changed": 2,
            "resources_deleted": 1,
            "breakdown_json": "{\"gcp\": 5000}",
            "infracost_version": "v0.12.0"
        }"#;
        let create: CostEstimateCreate = serde_json::from_str(json).unwrap();
        assert_eq!(create.project_id, 5);
        assert_eq!(create.currency, Some("JPY".to_string()));
        assert_eq!(create.resource_count, Some(10));
    }

    #[test]
    fn test_cost_estimate_create_minimal() {
        let create = CostEstimateCreate {
            project_id: 1,
            task_id: 1,
            template_id: 1,
            currency: None,
            monthly_cost: None,
            monthly_cost_diff: None,
            resource_count: None,
            resources_added: None,
            resources_changed: None,
            resources_deleted: None,
            breakdown_json: None,
            infracost_version: None,
        };
        let json = serde_json::to_string(&create).unwrap();
        assert!(json.contains("\"project_id\":1"));
        assert!(json.contains("\"currency\":null"));
    }

    #[test]
    fn test_cost_estimate_json_roundtrip() {
        let original = CostEstimate {
            id: 88,
            project_id: 8,
            task_id: 888,
            template_id: 88,
            currency: "CHF".to_string(),
            monthly_cost: Some(999.99),
            monthly_cost_diff: Some(99.99),
            resource_count: 88,
            resources_added: 8,
            resources_changed: 8,
            resources_deleted: 8,
            breakdown_json: Some(r#"{"gcp": 500}"#.to_string()),
            infracost_version: Some("v0.15.0".to_string()),
            created_at: "2024-08-08T08:08:08Z".to_string(),
            template_name: "Roundtrip Test".to_string(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: CostEstimate = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.id, original.id);
        assert_eq!(restored.currency, original.currency);
        assert_eq!(restored.monthly_cost, original.monthly_cost);
    }

    #[test]
    fn test_cost_estimate_create_json_roundtrip() {
        let original = CostEstimateCreate {
            project_id: 33,
            task_id: 333,
            template_id: 33,
            currency: Some("CAD".to_string()),
            monthly_cost: Some(333.33),
            monthly_cost_diff: Some(33.33),
            resource_count: Some(33),
            resources_added: Some(3),
            resources_changed: Some(3),
            resources_deleted: Some(3),
            breakdown_json: Some(r#"{"aws": 100}"#.to_string()),
            infracost_version: Some("v0.13.0".to_string()),
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: CostEstimateCreate = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.project_id, original.project_id);
        assert_eq!(restored.currency, original.currency);
    }

    #[test]
    fn test_cost_summary_json_roundtrip() {
        let original = CostSummary {
            template_id: 22,
            template_name: "Summary Roundtrip".to_string(),
            latest_monthly_cost: Some(222.22),
            runs_with_cost: 22,
            last_run_at: "2024-02-22T22:22:22Z".to_string(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: CostSummary = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.template_id, original.template_id);
        assert_eq!(restored.template_name, original.template_name);
        assert_eq!(restored.runs_with_cost, original.runs_with_cost);
    }

    #[test]
    fn test_cost_estimate_zero_values() {
        let estimate = CostEstimate {
            id: 0,
            project_id: 0,
            task_id: 0,
            template_id: 0,
            currency: String::new(),
            monthly_cost: Some(0.0),
            monthly_cost_diff: Some(0.0),
            resource_count: 0,
            resources_added: 0,
            resources_changed: 0,
            resources_deleted: 0,
            breakdown_json: Some("{}".to_string()),
            infracost_version: Some(String::new()),
            created_at: String::new(),
            template_name: String::new(),
        };
        assert_eq!(estimate.monthly_cost, Some(0.0));
        assert!(estimate.currency.is_empty());
    }

    #[test]
    fn test_cost_estimate_large_breakdown_json() {
        let large_json = r#"{"aws_instances": [{"type": "t3.large", "count": 10, "cost": 500}], "azure_vms": [{"type": "Standard_B2s", "count": 5, "cost": 200}], "gcp_instances": [{"type": "n1-standard-1", "count": 3, "cost": 150}]}"#;
        let estimate = CostEstimate {
            id: 1,
            project_id: 1,
            task_id: 1,
            template_id: 1,
            currency: "USD".to_string(),
            monthly_cost: Some(850.0),
            monthly_cost_diff: Some(100.0),
            resource_count: 18,
            resources_added: 18,
            resources_changed: 0,
            resources_deleted: 0,
            breakdown_json: Some(large_json.to_string()),
            infracost_version: Some("v0.20.0".to_string()),
            created_at: "2024-01-01".to_string(),
            template_name: "Multi-Cloud".to_string(),
        };

        let json = serde_json::to_string(&estimate).unwrap();
        let restored: CostEstimate = serde_json::from_str(&json).unwrap();
        assert!(restored.breakdown_json.unwrap().contains("aws_instances"));
    }
}
