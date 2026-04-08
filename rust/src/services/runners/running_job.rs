//! Running Job
//!
//! Запущенная задача

use std::sync::{Arc, Mutex, RwLock};
use chrono::{DateTime, Utc};
use tokio::process::Command;

use crate::services::task_logger::{TaskStatus, StatusListener, LogListener};
use super::types::{LogRecord, CommitInfo, JobData};

/// Запущенная задача
pub struct RunningJob {
    pub status: RwLock<TaskStatus>,
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
            status: RwLock::new(TaskStatus::Waiting),
            log_records: Arc::new(Mutex::new(Vec::new())),
            job,
            commit: None,
            status_listeners: Arc::new(Mutex::new(Vec::new())),
            log_listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Возвращает текущий статус
    pub fn get_status(&self) -> TaskStatus {
        *self.status.read().unwrap()
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
        let current = *self.status.read().unwrap();
        if current == status {
            return;
        }

        *self.status.write().unwrap() = status;

        let listeners = self.status_listeners.lock().unwrap();
        for listener in listeners.iter() {
            listener(status);
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

    /// Логирует имя команды и аргументы (не захватывает вывод)
    pub fn log_cmd(&self, cmd: &Command) {
        let program = cmd.as_std().get_program().to_string_lossy().to_string();
        let args: Vec<String> = cmd
            .as_std()
            .get_args()
            .map(|s| s.to_string_lossy().to_string())
            .collect();
        self.log(&format!("$ {} {}", program, args.join(" ")));
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_job() -> JobData {
        JobData {
            username: "test".to_string(),
            incoming_version: None,
            alias: None,
            task: crate::models::Task::default(),
            template: crate::models::Template::default(),
            inventory: crate::models::Inventory::default(),
            inventory_repository: None,
            repository: crate::models::Repository::default(),
            environment: crate::models::Environment::default(),
        }
    }

    #[test]
    fn test_running_job_creation() {
        let running_job = RunningJob::new(make_test_job());
        assert_eq!(running_job.get_status(), TaskStatus::Waiting);
    }

    #[test]
    fn test_set_status() {
        let job = RunningJob::new(make_test_job());
        job.set_status(TaskStatus::Running);
        assert_eq!(job.get_status(), TaskStatus::Running);
    }

    #[test]
    fn test_log_records() {
        let job = RunningJob::new(make_test_job());
        job.log("hello");
        let records = job.log_records.lock().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].message, "hello");
    }

    #[test]
    fn test_status_listener_called() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let job = RunningJob::new(make_test_job());
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        job.add_status_listener(Box::new(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }));
        job.set_status(TaskStatus::Running);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_status_listener_not_called_on_same_status() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let job = RunningJob::new(make_test_job());
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        job.add_status_listener(Box::new(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }));
        // Status уже Waiting, ставим Waiting again -- listener не должен сработать
        job.set_status(TaskStatus::Waiting);
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_log_listener_called() {
        let job = RunningJob::new(make_test_job());
        let received = Arc::new(std::sync::Mutex::new(Vec::new()));
        let received_clone = received.clone();
        job.add_log_listener(Box::new(move |_time, msg| {
            received_clone.lock().unwrap().push(msg);
        }));
        job.log("test message");
        assert_eq!(received.lock().unwrap()[0], "test message");
    }

    #[test]
    fn test_set_commit() {
        let mut job = RunningJob::new(make_test_job());
        job.set_commit("abc123".to_string(), "fix bug".to_string());
        assert!(job.commit.is_some());
        let c = job.commit.as_ref().unwrap();
        assert_eq!(c.hash, "abc123");
        assert_eq!(c.message, "fix bug");
    }

    #[test]
    fn test_log_with_time() {
        let job = RunningJob::new(make_test_job());
        let now = Utc::now();
        job.log_with_time(now, "timed message");
        let records = job.log_records.lock().unwrap();
        assert_eq!(records[0].message, "timed message");
        assert_eq!(records[0].time, now);
    }

    #[tokio::test]
    async fn test_wait_log_completes() {
        let job = RunningJob::new(make_test_job());
        job.wait_log().await; // должна просто завершиться
    }

    #[test]
    fn test_log_cmd() {
        use tokio::process::Command;
        let job = RunningJob::new(make_test_job());
        let mut cmd = Command::new("echo");
        cmd.arg("hello");
        job.log_cmd(&cmd);
        let records = job.log_records.lock().unwrap();
        assert!(records[0].message.contains("echo"));
        assert!(records[0].message.contains("hello"));
    }
}
