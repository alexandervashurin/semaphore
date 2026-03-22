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
