use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CostEstimate {
    pub id: i32,
    pub project_id: i32,
    pub task_id: i32,
    pub template_id: i32,
    pub currency: String,
    pub monthly_cost: Option<f64>,
    pub monthly_cost_diff: Option<f64>,
    pub resource_count: i32,
    pub resources_added: i32,
    pub resources_changed: i32,
    pub resources_deleted: i32,
    pub breakdown_json: Option<String>,
    pub infracost_version: Option<String>,
    pub created_at: String,
    #[sqlx(default)]
    pub template_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimateCreate {
    pub project_id: i32,
    pub task_id: i32,
    pub template_id: i32,
    pub currency: Option<String>,
    pub monthly_cost: Option<f64>,
    pub monthly_cost_diff: Option<f64>,
    pub resource_count: Option<i32>,
    pub resources_added: Option<i32>,
    pub resources_changed: Option<i32>,
    pub resources_deleted: Option<i32>,
    pub breakdown_json: Option<String>,
    pub infracost_version: Option<String>,
}

/// Summary row for dashboard (aggregated per template)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CostSummary {
    pub template_id: i32,
    pub template_name: String,
    pub latest_monthly_cost: Option<f64>,
    pub runs_with_cost: i64,
    pub last_run_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(json.contains("\"resource_count\":10"));
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
        assert!(json.contains("\"latest_monthly_cost\":200.0"));
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
}
