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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_status_display() {
        assert_eq!(PlanStatus::Pending.to_string(), "pending");
        assert_eq!(PlanStatus::Approved.to_string(), "approved");
        assert_eq!(PlanStatus::Rejected.to_string(), "rejected");
    }

    #[test]
    fn test_plan_status_from_str() {
        assert_eq!("approved".parse::<PlanStatus>().unwrap(), PlanStatus::Approved);
        assert_eq!("rejected".parse::<PlanStatus>().unwrap(), PlanStatus::Rejected);
        assert_eq!("unknown".parse::<PlanStatus>().unwrap(), PlanStatus::Pending);
    }

    #[test]
    fn test_terraform_plan_serialization() {
        let plan = TerraformPlan {
            id: 1,
            task_id: 100,
            project_id: 10,
            plan_output: "Plan: 1 to add, 0 to change, 0 to destroy.".to_string(),
            plan_json: None,
            resources_added: 1,
            resources_changed: 0,
            resources_removed: 0,
            status: "pending".to_string(),
            created_at: Utc::now(),
            reviewed_at: None,
            reviewed_by: None,
            review_comment: None,
        };
        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains("\"task_id\":100"));
        assert!(json.contains("\"resources_added\":1"));
        assert!(json.contains("\"status\":\"pending\""));
    }

    #[test]
    fn test_plan_review_payload_serialization() {
        let payload = PlanReviewPayload {
            comment: Some("Looks good to me".to_string()),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"comment\":\"Looks good to me\""));
    }

    #[test]
    fn test_plan_review_payload_empty() {
        let payload = PlanReviewPayload { comment: None };
        let json = serde_json::to_string(&payload).unwrap();
        // PlanReviewPayload doesn't have skip_serializing_if
        assert!(json.contains("\"comment\":null"));
    }

    #[test]
    fn test_plan_status_clone() {
        let status = PlanStatus::Approved;
        let cloned = status.clone();
        assert_eq!(cloned, status);
    }

    #[test]
    fn test_plan_status_equality() {
        assert_eq!(PlanStatus::Pending, PlanStatus::Pending);
        assert_ne!(PlanStatus::Approved, PlanStatus::Rejected);
    }

    #[test]
    fn test_terraform_plan_clone() {
        let plan = TerraformPlan {
            id: 1, task_id: 100, project_id: 10,
            plan_output: "Plan output".to_string(), plan_json: None,
            resources_added: 1, resources_changed: 0, resources_removed: 0,
            status: "pending".to_string(), created_at: Utc::now(),
            reviewed_at: None, reviewed_by: None, review_comment: None,
        };
        let cloned = plan.clone();
        assert_eq!(cloned.plan_output, plan.plan_output);
        assert_eq!(cloned.resources_added, plan.resources_added);
    }

    #[test]
    fn test_terraform_plan_with_review() {
        let plan = TerraformPlan {
            id: 2, task_id: 200, project_id: 20,
            plan_output: "Reviewed plan".to_string(), plan_json: Some("{}" .to_string()),
            resources_added: 3, resources_changed: 1, resources_removed: 2,
            status: "approved".to_string(), created_at: Utc::now(),
            reviewed_at: Some(Utc::now()), reviewed_by: Some(5),
            review_comment: Some("LGTM".to_string()),
        };
        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains("\"status\":\"approved\""));
        assert!(json.contains("\"review_comment\":\"LGTM\""));
    }

    #[test]
    fn test_plan_review_payload_clone() {
        let payload = PlanReviewPayload { comment: Some("Test".to_string()) };
        let cloned = payload.clone();
        assert_eq!(cloned.comment, payload.comment);
    }

    #[test]
    fn test_terraform_plan_deserialization() {
        let json = r#"{"id":10,"task_id":50,"project_id":5,"plan_output":"diff","plan_json":null,"resources_added":0,"resources_changed":0,"resources_removed":0,"status":"pending","created_at":"2024-01-01T00:00:00Z","reviewed_at":null,"reviewed_by":null,"review_comment":null}"#;
        let plan: TerraformPlan = serde_json::from_str(json).unwrap();
        assert_eq!(plan.id, 10);
        assert_eq!(plan.task_id, 50);
        assert_eq!(plan.status, "pending");
    }

    #[test]
    fn test_terraform_plan_debug() {
        let plan = TerraformPlan {
            id: 1, task_id: 1, project_id: 1,
            plan_output: "Debug".to_string(), plan_json: None,
            resources_added: 0, resources_changed: 0, resources_removed: 0,
            status: "pending".to_string(), created_at: Utc::now(),
            reviewed_at: None, reviewed_by: None, review_comment: None,
        };
        let debug_str = format!("{:?}", plan);
        assert!(debug_str.contains("TerraformPlan"));
    }

    #[test]
    fn test_plan_status_from_str_default() {
        let result = "invalid_status".parse::<PlanStatus>().unwrap();
        assert_eq!(result, PlanStatus::Pending);
    }
}
