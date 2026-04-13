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

    #[test]
    fn test_running_job_initial_status_is_waiting() {
        let job = RunningJob::new(make_test_job());
        assert_eq!(job.get_status(), TaskStatus::Waiting);
    }

    #[test]
    fn test_running_job_log_records_accumulate() {
        let job = RunningJob::new(make_test_job());
        job.log("first");
        job.log("second");
        job.log("third");
        let records = job.log_records.lock().unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].message, "first");
        assert_eq!(records[1].message, "second");
        assert_eq!(records[2].message, "third");
    }

    #[test]
    fn test_running_job_log_empty_string() {
        let job = RunningJob::new(make_test_job());
        job.log("");
        let records = job.log_records.lock().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].message, "");
    }

    #[test]
    fn test_running_job_status_transitions() {
        let job = RunningJob::new(make_test_job());
        assert_eq!(job.get_status(), TaskStatus::Waiting);
        job.set_status(TaskStatus::Running);
        assert_eq!(job.get_status(), TaskStatus::Running);
        job.set_status(TaskStatus::Success);
        assert_eq!(job.get_status(), TaskStatus::Success);
    }

    #[test]
    fn test_running_job_set_status_same_value_no_listener_call() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let job = RunningJob::new(make_test_job());
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        job.add_status_listener(Box::new(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }));
        // Initial status is Waiting, setting Waiting again should not call listener
        job.set_status(TaskStatus::Waiting);
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_running_job_set_status_different_calls_listener() {
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
    fn test_running_job_multiple_status_listeners() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let job = RunningJob::new(make_test_job());
        let c1 = Arc::new(AtomicUsize::new(0));
        let c2 = Arc::new(AtomicUsize::new(0));
        let c3 = Arc::new(AtomicUsize::new(0));
        job.add_status_listener(Box::new({ let c = c1.clone(); move |_| { c.fetch_add(1, Ordering::SeqCst); } }));
        job.add_status_listener(Box::new({ let c = c2.clone(); move |_| { c.fetch_add(1, Ordering::SeqCst); } }));
        job.add_status_listener(Box::new({ let c = c3.clone(); move |_| { c.fetch_add(1, Ordering::SeqCst); } }));
        job.set_status(TaskStatus::Running);
        assert_eq!(c1.load(Ordering::SeqCst), 1);
        assert_eq!(c2.load(Ordering::SeqCst), 1);
        assert_eq!(c3.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_running_job_log_with_special_characters() {
        let job = RunningJob::new(make_test_job());
        job.log("Line 1\nLine 2\tTab");
        job.log("Special: <>&\"'");
        job.log("Unicode: привет мир");
        let records = job.log_records.lock().unwrap();
        assert_eq!(records.len(), 3);
        assert!(records[0].message.contains('\n'));
        assert!(records[2].message.contains("привет"));
    }

    #[test]
    fn test_running_job_commit_info_none_initially() {
        let job = RunningJob::new(make_test_job());
        assert!(job.commit.is_none());
    }

    #[test]
    fn test_running_job_set_commit_info() {
        let mut job = RunningJob::new(make_test_job());
        job.set_commit("deadbeef".to_string(), "Initial commit".to_string());
        let commit = job.commit.as_ref().unwrap();
        assert_eq!(commit.hash, "deadbeef");
        assert_eq!(commit.message, "Initial commit");
    }

    #[test]
    fn test_running_job_log_cmd_with_no_args() {
        let job = RunningJob::new(make_test_job());
        let cmd = Command::new("ls");
        job.log_cmd(&cmd);
        let records = job.log_records.lock().unwrap();
        assert!(records[0].message.contains("ls"));
    }

    #[test]
    fn test_running_job_log_cmd_with_multiple_args() {
        let job = RunningJob::new(make_test_job());
        let mut cmd = Command::new("grep");
        cmd.arg("-r").arg("pattern").arg("/path");
        job.log_cmd(&cmd);
        let records = job.log_records.lock().unwrap();
        assert!(records[0].message.contains("grep"));
        assert!(records[0].message.contains("-r"));
        assert!(records[0].message.contains("pattern"));
    }

    #[test]
    fn test_running_job_log_records_shared_via_arc() {
        let job = RunningJob::new(make_test_job());
        let records_clone = job.log_records.clone();
        job.log("via original");
        {
            let records = records_clone.lock().unwrap();
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].message, "via original");
        }
    }

    #[test]
    fn test_running_job_status_listener_receives_correct_status() {
        let job = RunningJob::new(make_test_job());
        let received = Arc::new(std::sync::Mutex::new(Vec::new()));
        let received_clone = received.clone();
        job.add_status_listener(Box::new(move |status| {
            received_clone.lock().unwrap().push(status);
        }));
        job.set_status(TaskStatus::Running);
        job.set_status(TaskStatus::Success);
        let statuses = received.lock().unwrap();
        assert_eq!(statuses.len(), 2);
        assert_eq!(statuses[0], TaskStatus::Running);
        assert_eq!(statuses[1], TaskStatus::Success);
    }

    #[test]
    fn test_running_job_wait_log_does_not_panic() {
        let job = RunningJob::new(make_test_job());
        // Just verify it completes
        futures::executor::block_on(job.wait_log());
    }

    #[test]
    fn test_running_job_log_listener_receives_time_and_message() {
        let job = RunningJob::new(make_test_job());
        let received = Arc::new(std::sync::Mutex::new(Vec::new()));
        let received_clone = received.clone();
        job.add_log_listener(Box::new(move |time, msg| {
            received_clone.lock().unwrap().push((time, msg));
        }));
        job.log("test log");
        let records = received.lock().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].1, "test log");
    }

    #[test]
    fn test_running_job_set_status_error() {
        let job = RunningJob::new(make_test_job());
        job.set_status(TaskStatus::Error);
        assert_eq!(job.get_status(), TaskStatus::Error);
    }

    #[test]
    fn test_running_job_set_status_stopped() {
        let job = RunningJob::new(make_test_job());
        job.set_status(TaskStatus::Stopped);
        assert_eq!(job.get_status(), TaskStatus::Stopped);
    }
}
