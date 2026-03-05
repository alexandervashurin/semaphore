//! Модуль ошибок приложения

use thiserror::Error;
use axum::http::StatusCode;

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

    /// Неавторизован
    #[error("Неавторизован: {0}")]
    Unauthorized(String),

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

    /// Функция не реализована
    #[error("Не реализовано: {0}")]
    NotImplemented(String),

    /// Ошибка reqwest
    #[error("Ошибка HTTP: {0}")]
    Http(#[from] reqwest::Error),

    /// Ошибка SystemTime
    #[error("Ошибка времени: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),

    /// Другая ошибка
    #[error("{0}")]
    Other(String),
}

/// Результат выполнения операции
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Преобразует ошибку в HTTP статус-код
    pub fn to_status_code(&self) -> StatusCode {
        match self {
            Error::Validation(_) => StatusCode::BAD_REQUEST,
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::Auth(_) | Error::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Error::Forbidden(_) => StatusCode::FORBIDDEN,
            Error::Config(_) => StatusCode::SERVICE_UNAVAILABLE,
            Error::Database(_) | Error::Io(_) | Error::SystemTime(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Json(_) => StatusCode::BAD_REQUEST,
            Error::Git(_) | Error::Http(_) => StatusCode::BAD_GATEWAY,
            Error::WebSocket(_) | Error::Scheduler(_) | Error::NotImplemented(_) | Error::Other(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    /// Возвращает код ошибки для API
    pub fn error_code(&self) -> &'static str {
        match self {
            Error::Validation(_) => "VALIDATION_ERROR",
            Error::NotFound(_) => "NOT_FOUND",
            Error::Auth(_) | Error::Unauthorized(_) => "UNAUTHORIZED",
            Error::Forbidden(_) => "FORBIDDEN",
            Error::Config(_) => "CONFIG_ERROR",
            Error::Database(_) => "DATABASE_ERROR",
            Error::Json(_) => "INVALID_JSON",
            Error::Other(_) => "INTERNAL_ERROR",
            _ => "INTERNAL_ERROR",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_to_status_code() {
        assert_eq!(Error::NotFound("x".to_string()).to_status_code(), StatusCode::NOT_FOUND);
        assert_eq!(Error::Unauthorized("x".to_string()).to_status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(Error::Validation("x".to_string()).to_status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_code() {
        assert_eq!(Error::NotFound("x".to_string()).error_code(), "NOT_FOUND");
        assert_eq!(Error::Validation("x".to_string()).error_code(), "VALIDATION_ERROR");
    }
}
