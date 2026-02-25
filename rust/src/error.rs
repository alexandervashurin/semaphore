//! Модуль ошибок приложения

use thiserror::Error;

/// Основной тип ошибок приложения
#[derive(Error, Debug)]
pub enum Error {
    /// Ошибка базы данных
    #[error("Ошибка базы данных: {0}")]
    Database(#[from] sqlx::Error),

    /// Ошибка валидации
    #[error("Ошибка валидации: {0}")]
    Validation(String),

    /// Объект не найден
    #[error("Объект не найден: {0}")]
    NotFound(String),

    /// Ошибка аутентификации
    #[error("Ошибка аутентификации: {0}")]
    Auth(String),

    /// Ошибка авторизации
    #[error("Доступ запрещён: {0}")]
    Forbidden(String),

    /// Ошибка конфигурации
    #[error("Ошибка конфигурации: {0}")]
    Config(String),

    /// Ошибка Git
    #[error("Ошибка Git: {0}")]
    Git(#[from] git2::Error),

    /// Ошибка парсинга JSON
    #[error("Ошибка JSON: {0}")]
    Json(#[from] serde_json::Error),

    /// Ошибка ввода-вывода
    #[error("Ошибка ввода-вывода: {0}")]
    Io(#[from] std::io::Error),

    /// Ошибка WebSocket
    #[error("Ошибка WebSocket: {0}")]
    WebSocket(String),

    /// Ошибка планировщика
    #[error("Ошибка планировщика: {0}")]
    Scheduler(String),

    /// Другая ошибка
    #[error("{0}")]
    Other(String),
}

/// Результат выполнения операции
pub type Result<T> = std::result::Result<T, Error>;
