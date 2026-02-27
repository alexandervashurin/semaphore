//! Job Pool
//!
//! Пул задач для раннеров

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

use crate::error::{Error, Result};
use crate::services::task_logger::{TaskLogger, TaskStatus};
use super::running_job::RunningJob;
use super::types::{JobData, RunnerState, RunnerProgress, JobProgress, LogRecord, CommitInfo};

/// Логгер задач
pub struct JobLogger {
    pub context: String,
}

impl JobLogger {
    pub fn new(context: &str) -> Self {
        Self {
            context: context.to_string(),
        }
    }

    pub fn info(&self, message: &str) {
        tracing::info!("[{}] {}", self.context, message);
    }

    pub fn debug(&self, message: &str) {
        tracing::debug!("[{}] {}", self.context, message);
    }

    pub fn task_info(&self, message: &str, task_id: i32, status: &str) {
        tracing::info!("[{}] {} - Task {}: {}", self.context, message, task_id, status);
    }
}

/// Пул задач
pub struct JobPool {
    running_jobs: Arc<Mutex<HashMap<i32, RunningJob>>>,
    queue: Arc<Mutex<Vec<Job>>>,
    processing: AtomicU32,
}

impl JobPool {
    /// Создаёт новый пул задач
    pub fn new() -> Self {
        Self {
            running_jobs: Arc::new(Mutex::new(HashMap::new())),
            queue: Arc::new(Mutex::new(Vec::new())),
            processing: AtomicU32::new(0),
        }
    }

    /// Проверяет, есть ли задача в очереди
    pub async fn exists_in_queue(&self, task_id: i32) -> bool {
        let queue = self.queue.lock().await;
        queue.iter().any(|j| j.job.task.id == task_id)
    }

    /// Проверяет, есть ли запущенные задачи
    pub async fn has_running_jobs(&self) -> bool {
        let running = self.running_jobs.lock().await;
        running.values().any(|j| !j.status.is_finished())
    }

    /// Запускает пул задач
    pub async fn run(&self) -> Result<()> {
        let logger = JobLogger::new("running");
        let mut queue_interval = interval(Duration::from_secs(5));
        let mut request_interval = interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                _ = queue_interval.tick() => {
                    self.check_queue(&logger).await;
                }
                _ = request_interval.tick() => {
                    self.check_new_jobs(&logger).await;
                }
            }
        }
    }

    /// Проверяет очередь задач
    async fn check_queue(&self, logger: &JobLogger) {
        logger.debug("Checking queue");

        let mut queue = self.queue.lock().await;
        if queue.is_empty() {
            return;
        }

        let task = queue[0].clone();
        if task.status == TaskStatus::Error {
            queue.remove(0);
            logger.task_info("Task dequeued", task.job.task.id, "failed");
            return;
        }

        // Запускаем задачу
        // TODO: Реализовать запуск задачи
        queue.remove(0);
    }

    /// Проверяет новые задачи
    async fn check_new_jobs(&self, logger: &JobLogger) {
        // TODO: Реализовать проверку новых задач
    }
}

/// Задача в очереди
#[derive(Clone)]
pub struct Job {
    pub username: String,
    pub incoming_version: Option<String>,
    pub alias: Option<String>,
    pub job: JobData,
    pub status: TaskStatus,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_pool_creation() {
        let pool = JobPool::new();
        assert!(true);
    }

    #[test]
    fn test_job_logger_creation() {
        let logger = JobLogger::new("test");
        assert_eq!(logger.context, "test");
    }
}
