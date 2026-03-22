//! Models for Terraform Remote State Backend (Phase 1)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Stored Terraform state version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerraformState {
    pub id:         i64,
    pub project_id: i32,
    pub workspace:  String,
    pub serial:     i32,
    pub lineage:    String,
    #[serde(skip)]
    pub state_data: Vec<u8>,
    pub encrypted:  bool,
    pub md5:        String,
    pub created_at: DateTime<Utc>,
}

/// Summary without raw bytes (for listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerraformStateSummary {
    pub id:         i64,
    pub project_id: i32,
    pub workspace:  String,
    pub serial:     i32,
    pub lineage:    String,
    pub md5:        String,
    pub created_at: DateTime<Utc>,
}

/// Active workspace lock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerraformStateLock {
    pub project_id: i32,
    pub workspace:  String,
    pub lock_id:    String,
    pub operation:  String,
    pub info:       String,
    pub who:        String,
    pub version:    String,
    pub path:       String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Terraform LockInfo JSON (matches Terraform's wire format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    #[serde(rename = "ID")]
    pub id:        String,
    #[serde(rename = "Operation")]
    pub operation: String,
    #[serde(rename = "Info")]
    pub info:      String,
    #[serde(rename = "Who")]
    pub who:       String,
    #[serde(rename = "Version")]
    pub version:   String,
    #[serde(rename = "Created")]
    pub created:   String,
    #[serde(rename = "Path")]
    pub path:      String,
}

impl LockInfo {
    pub fn from_lock(lock: &TerraformStateLock) -> Self {
        Self {
            id:        lock.lock_id.clone(),
            operation: lock.operation.clone(),
            info:      lock.info.clone(),
            who:       lock.who.clone(),
            version:   lock.version.clone(),
            created:   lock.created_at.to_rfc3339(),
            path:      lock.path.clone(),
        }
    }
}

/// Diff between two state versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiff {
    pub from_serial: i32,
    pub to_serial:   i32,
    pub added:       Vec<StateDiffResource>,
    pub changed:     Vec<StateDiffResource>,
    pub removed:     Vec<StateDiffResource>,
}

/// Single resource in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiffResource {
    pub address:       String,
    pub resource_type: String,
    pub name:          String,
}
