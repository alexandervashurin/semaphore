//! Модуль логирования задач
//!
//! Полная реализация интерфейсов из Go версии pkg/task_logger

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::process::Command;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

// ============================================================================
// TaskStatus - статусы задач
// ============================================================================

/// Статус задачи
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Задача ожидает выполнения
    Waiting,
    /// Задача запускается (инициализация)
    Starting,
    /// Задача ожидает подтверждения пользователем
    WaitingConfirmation,
    /// Задача подтверждена пользователем
    Confirmed,
    /// Задача отклонена пользователем
    Rejected,
    /// Задача выполняется
    Running,
    /// Задача останавливается
    Stopping,
    /// Задача остановлена пользователем
    Stopped,
    /// Задача выполнена успешно
    Success,
    /// Задача выполнена с ошибкой
    Error,
    /// Задача не выполнена (отменена)
    NotExecuted,
}

impl FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "waiting" => Ok(TaskStatus::Waiting),
            "starting" => Ok(TaskStatus::Starting),
            "waiting_confirmation" => Ok(TaskStatus::WaitingConfirmation),
            "confirmed" => Ok(TaskStatus::Confirmed),
            "rejected" => Ok(TaskStatus::Rejected),
            "running" => Ok(TaskStatus::Running),
            "stopping" => Ok(TaskStatus::Stopping),
            "stopped" => Ok(TaskStatus::Stopped),
            "success" => Ok(TaskStatus::Success),
            "error" => Ok(TaskStatus::Error),
            "not_executed" => Ok(TaskStatus::NotExecuted),
            _ => Err(format!("Неизвестный статус задачи: {}", s)),
        }
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Waiting => write!(f, "waiting"),
            TaskStatus::Starting => write!(f, "starting"),
            TaskStatus::WaitingConfirmation => write!(f, "waiting_confirmation"),
            TaskStatus::Confirmed => write!(f, "confirmed"),
            TaskStatus::Rejected => write!(f, "rejected"),
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Stopping => write!(f, "stopping"),
            TaskStatus::Stopped => write!(f, "stopped"),
            TaskStatus::Success => write!(f, "success"),
            TaskStatus::Error => write!(f, "error"),
            TaskStatus::NotExecuted => write!(f, "not_executed"),
        }
    }
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Waiting
    }
}

impl TaskStatus {
    /// Проверяет, является ли статус допустимым
    pub fn is_valid(&self) -> bool {
        matches!(
            self,
            TaskStatus::Waiting
                | TaskStatus::Starting
                | TaskStatus::WaitingConfirmation
                | TaskStatus::Confirmed
                | TaskStatus::Rejected
                | TaskStatus::Running
                | TaskStatus::Stopping
                | TaskStatus::Stopped
                | TaskStatus::Success
                | TaskStatus::Error
                | TaskStatus::NotExecuted
        )
    }

    /// Проверяет, завершена ли задача
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            TaskStatus::Success | TaskStatus::Error | TaskStatus::Stopped | TaskStatus::NotExecuted
        )
    }

    /// Проверяет, нужно ли отправлять уведомление для этого статуса
    pub fn is_notifiable(&self) -> bool {
        matches!(
            self,
            TaskStatus::Success | TaskStatus::Error | TaskStatus::WaitingConfirmation
        )
    }

    /// Форматирует статус с эмодзи (как в Go версии)
    pub fn format(&self) -> String {
        let emoji = match self {
            TaskStatus::Error => "❌",
            TaskStatus::Success => "✅",
            TaskStatus::Stopped => "⏹️",
            TaskStatus::WaitingConfirmation => "⚠️",
            _ => "❓",
        };

        let text = match self {
            TaskStatus::Waiting => " WAITING",
            TaskStatus::Starting => " STARTING",
            TaskStatus::WaitingConfirmation => " WAITING_CONFIRMATION",
            TaskStatus::Confirmed => " CONFIRMED",
            TaskStatus::Rejected => " REJECTED",
            TaskStatus::Running => " RUNNING",
            TaskStatus::Stopping => " STOPPING",
            TaskStatus::Stopped => " STOPPED",
            TaskStatus::Success => " SUCCESS",
            TaskStatus::Error => " ERROR",
            TaskStatus::NotExecuted => " NOT_EXECUTED",
        };

        format!("{}{}", emoji, text)
    }
}

/// Возвращает список незавершённых статусов
pub fn unfinished_task_statuses() -> Vec<TaskStatus> {
    vec![
        TaskStatus::Waiting,
        TaskStatus::Starting,
        TaskStatus::WaitingConfirmation,
        TaskStatus::Confirmed,
        TaskStatus::Rejected,
        TaskStatus::Running,
        TaskStatus::Stopping,
    ]
}

// ============================================================================
// Типы слушателей
// ============================================================================

/// Слушатель изменений статуса задачи
pub type StatusListener = Box<dyn Fn(TaskStatus) + Send + Sync>;

/// Слушатель логов задач
pub type LogListener = Box<dyn Fn(DateTime<Utc>, String) + Send + Sync>;

// ============================================================================
// Трейт TaskLogger
// ============================================================================

/// Трейт для логгера задач (полная совместимость с Go интерфейсом)
pub trait TaskLogger: Send + Sync {
    /// Логирует сообщение
    fn log(&self, msg: &str);

    /// Логирует сообщение с форматированием
    fn logf(&self, format: &str, args: fmt::Arguments<'_>);

    /// Логирует сообщение с временной меткой
    fn log_with_time(&self, time: DateTime<Utc>, msg: &str);

    /// Логирует сообщение с форматированием и временной меткой
    fn logf_with_time(&self, time: DateTime<Utc>, format: &str, args: fmt::Arguments<'_>);

    /// Логирует вывод команды
    fn log_cmd(&self, cmd: &Command);

    /// Устанавливает статус задачи
    fn set_status(&self, status: TaskStatus);

    /// Получает текущий статус задачи
    fn get_status(&self) -> TaskStatus;

    /// Добавляет слушателя статуса
    fn add_status_listener(&self, listener: StatusListener);

    /// Добавляет слушателя логов
    fn add_log_listener(&self, listener: LogListener);

    /// Устанавливает информацию о коммите
    fn set_commit(&self, hash: &str, message: &str);

    /// Ждёт завершения обработки всех логов
    fn wait_log(&self);
}

// ============================================================================
// Базовая реализация TaskLogger
// ============================================================================

/// Базовая реализация логгера для использования в приложении
pub struct BasicLogger {
    status: RwLock<TaskStatus>,
    status_listeners: RwLock<Vec<StatusListener>>,
    log_listeners: RwLock<Vec<LogListener>>,
    commit_hash: RwLock<Option<String>>,
    commit_message: RwLock<Option<String>>,
}

impl BasicLogger {
    pub fn new() -> Self {
        Self {
            status: RwLock::new(TaskStatus::Waiting),
            status_listeners: RwLock::new(Vec::new()),
            log_listeners: RwLock::new(Vec::new()),
            commit_hash: RwLock::new(None),
            commit_message: RwLock::new(None),
        }
    }

    fn notify_status_listeners(&self, status: TaskStatus) {
        let listeners = self.status_listeners.read().unwrap();
        for listener in listeners.iter() {
            listener(status);
        }
    }

    fn notify_log_listeners(&self, time: DateTime<Utc>, msg: String) {
        let listeners = self.log_listeners.read().unwrap();
        for listener in listeners.iter() {
            listener(time, msg.clone());
        }
    }
}

impl Default for BasicLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskLogger for BasicLogger {
    fn log(&self, msg: &str) {
        let now = Utc::now();
        self.log_with_time(now, msg);
    }

    fn logf(&self, format: &str, args: fmt::Arguments<'_>) {
        let msg = format!("{}", args);
        self.log(&msg);
    }

    fn log_with_time(&self, time: DateTime<Utc>, msg: &str) {
        println!("[{}] {}", time.format("%H:%M:%S"), msg);
        self.notify_log_listeners(time, msg.to_string());
    }

    fn logf_with_time(&self, time: DateTime<Utc>, format: &str, args: fmt::Arguments<'_>) {
        let msg = format!("{}", args);
        self.log_with_time(time, &msg);
    }

    fn log_cmd(&self, cmd: &Command) {
        let program = cmd.get_program().to_string_lossy();
        let args: Vec<String> = cmd
            .get_args()
            .map(|s| s.to_string_lossy().to_string())
            .collect();
        self.log(&format!("$ {} {}", program, args.join(" ")));
    }

    fn set_status(&self, status: TaskStatus) {
        let mut s = self.status.write().unwrap();
        *s = status;
        drop(s);
        self.notify_status_listeners(status);
    }

    fn get_status(&self) -> TaskStatus {
        *self.status.read().unwrap()
    }

    fn add_status_listener(&self, listener: StatusListener) {
        let mut listeners = self.status_listeners.write().unwrap();
        listeners.push(listener);
    }

    fn add_log_listener(&self, listener: LogListener) {
        let mut listeners = self.log_listeners.write().unwrap();
        listeners.push(listener);
    }

    fn set_commit(&self, hash: &str, message: &str) {
        let mut h = self.commit_hash.write().unwrap();
        *h = Some(hash.to_string());
        drop(h);
        let mut m = self.commit_message.write().unwrap();
        *m = Some(message.to_string());
    }

    fn wait_log(&self) {
        // В базовой реализации ничего не делаем
        // В production можно реализовать очередь логов
    }
}

// ============================================================================
// Arc-обёртка для удобного использования
// ============================================================================

/// Тип для хранения логгера в Arc
pub type TaskLoggerArc = Arc<dyn TaskLogger>;

/// Создаёт новый логгер в Arc
pub fn create_logger() -> TaskLoggerArc {
    Arc::new(BasicLogger::new())
}

// ============================================================================
// Макросы для удобного логгирования
// ============================================================================

/// Макрос для логгера с форматированием (аналог printf!)
#[macro_export]
macro_rules! logf {
    ($logger:expr, $format:expr, $($arg:tt)*) => {
        $logger.logf($format, format_args!($format, $($arg)*))
    };
}

/// Макрос для логгера с временем и форматированием
#[macro_export]
macro_rules! logf_with_time {
    ($logger:expr, $time:expr, $format:expr, $($arg:tt)*) => {
        $logger.logf_with_time($time, $format, format_args!($format, $($arg)*))
    };
}

// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_status_from_str() {
        assert_eq!(
            TaskStatus::from_str("waiting").unwrap(),
            TaskStatus::Waiting
        );
        assert_eq!(
            TaskStatus::from_str("running").unwrap(),
            TaskStatus::Running
        );
        assert_eq!(
            TaskStatus::from_str("success").unwrap(),
            TaskStatus::Success
        );
        assert_eq!(TaskStatus::from_str("error").unwrap(), TaskStatus::Error);
        assert_eq!(
            TaskStatus::from_str("stopped").unwrap(),
            TaskStatus::Stopped
        );
        assert_eq!(
            TaskStatus::from_str("starting").unwrap(),
            TaskStatus::Starting
        );
        assert_eq!(
            TaskStatus::from_str("waiting_confirmation").unwrap(),
            TaskStatus::WaitingConfirmation
        );
        assert!(TaskStatus::from_str("invalid").is_err());
    }

    #[test]
    fn test_task_status_display() {
        assert_eq!(TaskStatus::Waiting.to_string(), "waiting");
        assert_eq!(TaskStatus::Running.to_string(), "running");
        assert_eq!(TaskStatus::Success.to_string(), "success");
        assert_eq!(TaskStatus::Error.to_string(), "error");
    }

    #[test]
    fn test_task_status_is_valid() {
        assert!(TaskStatus::Waiting.is_valid());
        assert!(TaskStatus::Running.is_valid());
        assert!(TaskStatus::Success.is_valid());
        assert!(TaskStatus::Error.is_valid());
    }

    #[test]
    fn test_task_status_is_finished() {
        assert!(TaskStatus::Success.is_finished());
        assert!(TaskStatus::Error.is_finished());
        assert!(TaskStatus::Stopped.is_finished());
        assert!(TaskStatus::NotExecuted.is_finished());
        assert!(!TaskStatus::Waiting.is_finished());
        assert!(!TaskStatus::Running.is_finished());
    }

    #[test]
    fn test_task_status_is_notifiable() {
        assert!(TaskStatus::Success.is_notifiable());
        assert!(TaskStatus::Error.is_notifiable());
        assert!(TaskStatus::WaitingConfirmation.is_notifiable());
        assert!(!TaskStatus::Waiting.is_notifiable());
        assert!(!TaskStatus::Running.is_notifiable());
    }

    #[test]
    fn test_task_status_format() {
        assert!(TaskStatus::Error.format().contains("❌"));
        assert!(TaskStatus::Success.format().contains("✅"));
        assert!(TaskStatus::Stopped.format().contains("⏹️"));
        assert!(TaskStatus::WaitingConfirmation.format().contains("⚠️"));
        assert!(TaskStatus::Error.format().contains("ERROR"));
        assert!(TaskStatus::Success.format().contains("SUCCESS"));
    }

    #[test]
    fn test_unfinished_task_statuses() {
        let statuses = unfinished_task_statuses();
        assert!(statuses.contains(&TaskStatus::Waiting));
        assert!(statuses.contains(&TaskStatus::Running));
        assert!(statuses.contains(&TaskStatus::Starting));
        assert!(!statuses.contains(&TaskStatus::Success));
        assert!(!statuses.contains(&TaskStatus::Error));
    }

    #[test]
    fn test_basic_logger_creation() {
        let logger = BasicLogger::new();
        assert_eq!(logger.get_status(), TaskStatus::Waiting);
    }

    #[test]
    fn test_basic_logger_set_status() {
        let logger = BasicLogger::new();
        logger.set_status(TaskStatus::Running);
        assert_eq!(logger.get_status(), TaskStatus::Running);
    }

    #[test]
    fn test_basic_logger_status_listener() {
        let logger = BasicLogger::new();
        let notified = Arc::new(RwLock::new(false));

        let notified_clone = notified.clone();
        logger.add_status_listener(Box::new(move |_status| {
            let mut n = notified_clone.write().unwrap();
            *n = true;
        }));

        logger.set_status(TaskStatus::Running);

        assert!(*notified.read().unwrap());
    }

    #[test]
    fn test_basic_logger_log() {
        let logger = BasicLogger::new();
        // Просто проверяем, что метод вызывается без паники
        logger.log("Test message");
    }

    #[test]
    fn test_basic_logger_logf() {
        let logger = BasicLogger::new();
        logger.logf("Test {} {}", format_args!("message {}", 123));
    }

    #[test]
    fn test_create_logger_arc() {
        let logger = create_logger();
        assert_eq!(logger.get_status(), TaskStatus::Waiting);
    }
}
