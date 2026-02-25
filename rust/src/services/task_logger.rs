//! Модуль логирования задач

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Статус задачи
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Задача ожидает выполнения
    Waiting,
    /// Задача запущена
    Running,
    /// Задача выполнена успешно
    Success,
    /// Задача выполнена с ошибкой
    Error,
    /// Задача остановлена пользователем
    Stopped,
    /// Задача не выполнена (отменена)
    NotExecuted,
}

impl FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "waiting" => Ok(TaskStatus::Waiting),
            "running" => Ok(TaskStatus::Running),
            "success" => Ok(TaskStatus::Success),
            "error" => Ok(TaskStatus::Error),
            "stopped" => Ok(TaskStatus::Stopped),
            "not_executed" => Ok(TaskStatus::NotExecuted),
            _ => Err(format!("Неизвестный статус задачи: {}", s)),
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Waiting => write!(f, "waiting"),
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Success => write!(f, "success"),
            TaskStatus::Error => write!(f, "error"),
            TaskStatus::Stopped => write!(f, "stopped"),
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
    /// Проверяет, завершена ли задача
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            TaskStatus::Success | TaskStatus::Error | TaskStatus::Stopped | TaskStatus::NotExecuted
        )
    }
}

/// Трейт для логгера задач
pub trait TaskLogger: Send + Sync {
    /// Логирует сообщение
    fn log(&self, msg: &str);

    /// Устанавливает статус задачи
    fn set_status(&self, status: TaskStatus);

    /// Получает текущий статус задачи
    fn get_status(&self) -> TaskStatus;

    /// Добавляет слушателя статуса
    fn add_status_listener(&self, _listener: Box<dyn Fn(TaskStatus) + Send>) {}

    /// Добавляет слушателя логов
    fn add_log_listener(&self, _listener: Box<dyn Fn(std::time::Instant, String) + Send>) {}

    /// Ждёт завершения обработки всех логов
    fn wait_log(&self) {}
}

/// Простая реализация логгера для тестов
pub struct SimpleLogger {
    status: std::sync::RwLock<TaskStatus>,
}

impl SimpleLogger {
    pub fn new() -> Self {
        Self {
            status: std::sync::RwLock::new(TaskStatus::Waiting),
        }
    }
}

impl Default for SimpleLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskLogger for SimpleLogger {
    fn log(&self, msg: &str) {
        println!("[LOG] {}", msg);
    }

    fn set_status(&self, status: TaskStatus) {
        let mut s = self.status.write().unwrap();
        *s = status;
    }

    fn get_status(&self) -> TaskStatus {
        *self.status.read().unwrap()
    }
}
