//! Models for Terraform Plan Approval Workflow (Phase 2)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Plan review status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Pending,
    Approved,
    Rejected,
}

impl std::fmt::Display for PlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanStatus::Pending => write!(f, "pending"),
            PlanStatus::Approved => write!(f, "approved"),
            PlanStatus::Rejected => write!(f, "rejected"),
        }
    }
}

impl std::str::FromStr for PlanStatus {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "approved" => PlanStatus::Approved,
            "rejected" => PlanStatus::Rejected,
            _ => PlanStatus::Pending,
        })
    }
}

/// Stored terraform plan awaiting review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerraformPlan {
    pub id: i64,
    pub task_id: i32,
    pub project_id: i32,
    pub plan_output: String,
    pub plan_json: Option<String>,
    pub resources_added: i32,
    pub resources_changed: i32,
    pub resources_removed: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<i32>,
    pub review_comment: Option<String>,
}

/// Payload for approve/reject
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanReviewPayload {
    pub comment: Option<String>,
}
