//! Модуль общих ошибок
//!
//! Предоставляет стандартные типы ошибок и утилиты для обработки ошибок

use std::error::Error as StdError;
use std::fmt;

/// Ошибка, видимая для пользователя
///
/// Обёртка над ошибкой, которая может быть безопасно показана пользователю
#[derive(Debug)]
pub struct UserVisibleError {
    pub err: String,
}

impl UserVisibleError {
    /// Создаёт новую ошибку пользователя
    pub fn new(err: impl Into<String>) -> Self {
        Self { err: err.into() }
    }

    /// Создаёт ошибку пользователя из строки
    pub fn from_string(err: String) -> Self {
        Self { err }
    }
}

impl fmt::Display for UserVisibleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.err)
    }
}

impl StdError for UserVisibleError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

/// Ошибка неактивной подписки
#[derive(Debug)]
pub struct InvalidSubscriptionError;

impl fmt::Display for InvalidSubscriptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "has no active subscription")
    }
}

impl StdError for InvalidSubscriptionError {}

/// Получает контекст ошибки (файл, функция, строка)
///
/// В Rust реализуем через макросы или вручную через std::panic::Location
pub fn get_error_context() -> String {
    // В Rust лучше использовать panic::Location или backtrace
    // Для простоты возвращаем заглушку
    "unknown".to_string()
}

/// Создаёт ошибку пользователя из строки
pub fn new_user_error(message: impl Into<String>) -> UserVisibleError {
    UserVisibleError::new(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_visible_error_display() {
        let err = UserVisibleError::new("test error");
        assert_eq!(err.to_string(), "test error");
    }

    #[test]
    fn test_user_visible_error_from_string() {
        let err = UserVisibleError::from_string("test error".to_string());
        assert_eq!(err.to_string(), "test error");
    }

    #[test]
    fn test_invalid_subscription_error() {
        let err = InvalidSubscriptionError;
        assert_eq!(
            err.to_string(),
            "has no active subscription"
        );
    }

    #[test]
    fn test_new_user_error() {
        let err = new_user_error("test message");
        assert_eq!(err.to_string(), "test message");
    }
}
