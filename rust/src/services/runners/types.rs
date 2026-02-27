//! Runners Types
//!
//! Типы для раннеров

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::models::*;
use crate::services::task_logger::TaskStatus;

/// Данные задачи
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobData {
    pub username: String,
    pub incoming_version: Option<String>,
    pub alias: Option<String>,
    pub task: Task,
    pub template: Template,
    pub inventory: Inventory,
    pub inventory_repository: Option<Repository>,
    pub repository: Repository,
    pub environment: Environment,
}

/// Состояние раннера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerState {
    pub current_jobs: Vec<JobState>,
    pub new_jobs: Vec<JobData>,
    pub access_keys: std::collections::HashMap<i32, AccessKey>,
    pub clear_cache: Option<bool>,
    pub cache_clean_project_id: Option<i32>,
}

/// Состояние задачи
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobState {
    pub id: i32,
    pub status: TaskStatus,
}

/// Запись лога
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    pub time: DateTime<Utc>,
    pub message: String,
}

/// Информация о коммите
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
}

/// Прогресс раннера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerProgress {
    pub jobs: Vec<JobProgress>,
}

/// Прогресс задачи
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgress {
    pub id: i32,
    pub status: TaskStatus,
    pub log_records: Vec<LogRecord>,
    pub commit: Option<CommitInfo>,
}

/// Регистрация раннера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerRegistration {
    pub registration_token: String,
    pub webhook: Option<String>,
    pub max_parallel_tasks: i32,
    pub public_key: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_record_creation() {
        let record = LogRecord {
            time: Utc::now(),
            message: "Test".to_string(),
        };
        assert!(!record.message.is_empty());
    }

    #[test]
    fn test_commit_info_creation() {
        let commit = CommitInfo {
            hash: "abc123".to_string(),
            message: "Test commit".to_string(),
        };
        assert_eq!(commit.hash, "abc123");
    }

    #[test]
    fn test_runner_registration_creation() {
        let reg = RunnerRegistration {
            registration_token: "token".to_string(),
            webhook: None,
            max_parallel_tasks: 5,
            public_key: None,
        };
        assert_eq!(reg.registration_token, "token");
    }
}
