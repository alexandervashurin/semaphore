//! Running Job
//!
//! Запущенная задача

use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::services::task_logger::{TaskLogger, TaskStatus, StatusListener, LogListener};
use super::types::{LogRecord, CommitInfo, JobData};

/// Запущенная задача
pub struct RunningJob {
    pub status: TaskStatus,
    pub log_records: Arc<Mutex<Vec<LogRecord>>>,
    pub job: JobData,
    pub commit: Option<CommitInfo>,
    status_listeners: Arc<Mutex<Vec<StatusListener>>>,
    log_listeners: Arc<Mutex<Vec<LogListener>>>,
}

impl RunningJob {
    /// Создаёт новую запущенную задачу
    pub fn new(job: JobData) -> Self {
        Self {
            status: TaskStatus::Waiting,
            log_records: Arc::new(Mutex::new(Vec::new())),
            job,
            commit: None,
            status_listeners: Arc::new(Mutex::new(Vec::new())),
            log_listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Добавляет слушателя статуса
    pub fn add_status_listener(&self, listener: StatusListener) {
        let mut listeners = self.status_listeners.lock().unwrap();
        listeners.push(listener);
    }

    /// Добавляет слушателя логов
    pub fn add_log_listener(&self, listener: LogListener) {
        let mut listeners = self.log_listeners.lock().unwrap();
        listeners.push(listener);
    }

    /// Логирует сообщение
    pub fn log(&self, msg: &str) {
        self.log_with_time(Utc::now(), msg);
    }

    /// Логирует сообщение с временем
    pub fn log_with_time(&self, now: DateTime<Utc>, msg: &str) {
        let mut records = self.log_records.lock().unwrap();
        records.push(LogRecord {
            time: now,
            message: msg.to_string(),
        });

        let listeners = self.log_listeners.lock().unwrap();
        for listener in listeners.iter() {
            listener(now, msg.to_string());
        }
    }

    /// Устанавливает статус
    pub fn set_status(&self, status: TaskStatus) {
        if self.status == status {
            return;
        }

        let mut current_status = self.status.clone();
        current_status = status.clone();

        let listeners = self.status_listeners.lock().unwrap();
        for listener in listeners.iter() {
            listener(status.clone());
        }
    }

    /// Устанавливает информацию о коммите
    pub fn set_commit(&mut self, hash: String, message: String) {
        self.commit = Some(CommitInfo { hash, message });
    }

    /// Ждёт завершения обработки логов
    pub async fn wait_log(&self) {
        // В базовой версии просто возвращаем
    }

    /// Логирует вывод команды
    pub async fn log_cmd(&self, cmd: &Command) {
        // TODO: Реализовать логирование вывода команды
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_running_job_creation() {
        let job = JobData {
            username: "test".to_string(),
            incoming_version: None,
            alias: None,
            task: crate::models::Task {
                id: 1,
                template_id: 1,
                project_id: 1,
                status: TaskStatus::Waiting,
                playbook: None,
                environment: None,
                secret: None,
                arguments: None,
                git_branch: None,
                user_id: None,
                integration_id: None,
                schedule_id: None,
                created: Utc::now(),
                start: None,
                end: None,
                message: None,
                commit_hash: None,
                commit_message: None,
                build_task_id: None,
                version: None,
                inventory_id: None,
                params: None,
            },
            template: crate::models::Template {
                id: 1,
                project_id: 1,
                name: "Test".to_string(),
                playbook: "test.yml".to_string(),
                description: String::new(),
                inventory_id: 1,
                repository_id: 1,
                environment_id: 1,
                r#type: crate::models::template::TemplateType::Default,
                app: crate::models::template::TemplateApp::Ansible,
                git_branch: "main".to_string(),
                deleted: false,
                created: Utc::now(),
            },
            inventory: crate::models::Inventory {
                id: 1,
                project_id: 1,
                name: "Test".to_string(),
                inventory_type: crate::models::inventory::InventoryType::Static,
                inventory_data: String::new(),
                ssh_key_id: None,
                become_key_id: None,
                repository_id: None,
            },
            repository: crate::models::Repository {
                id: 1,
                project_id: 1,
                name: "Test".to_string(),
                git_url: "https://example.com/test.git".to_string(),
                ssh_key_id: None,
            },
            environment: crate::models::Environment {
                id: 1,
                project_id: 1,
                name: "Test".to_string(),
                json: "{}".to_string(),
                secret_storage_id: None,
            },
        };

        let running_job = RunningJob::new(job);
        assert_eq!(running_job.status, TaskStatus::Waiting);
    }
}
