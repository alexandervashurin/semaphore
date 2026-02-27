//! Error Logging Utilities
//!
//! Утилиты для логирования ошибок

use tracing::{error, warn, debug};

/// Логирует warning с произвольным полем если есть ошибка
pub fn log_warning<E: std::fmt::Display>(err: &E) {
    log_warning_f(err, &[])
}

/// Логирует debug с дополнительными полями если есть ошибка
pub fn log_debug_f<E: std::fmt::Display>(err: &E, fields: &[(&str, &str)]) {
    if std::env::var("RUST_LOG").unwrap_or_default().contains("debug") {
        let mut msg = format!("{}", err);
        for (key, value) in fields {
            msg.push_str(&format!(" {}={}", key, value));
        }
        debug!("{}", msg);
    }
}

/// Логирует warning с дополнительными полями если есть ошибка
pub fn log_warning_f<E: std::fmt::Display>(err: &E, fields: &[(&str, &str)]) {
    let mut msg = format!("{}", err);
    for (key, value) in fields {
        msg.push_str(&format!(" {}={}", key, value));
    }
    warn!("{}", msg);
}

/// Логирует error с произвольным полем если есть ошибка
pub fn log_error<E: std::fmt::Display>(err: &E) {
    log_error_f(err, &[])
}

/// Логирует error с дополнительными полями если есть ошибка
pub fn log_error_f<E: std::fmt::Display>(err: &E, fields: &[(&str, &str)]) {
    let mut msg = format!("{}", err);
    for (key, value) in fields {
        msg.push_str(&format!(" {}={}", key, value));
    }
    error!("{}", msg);
}

/// Логирует и паникует если есть ошибка
pub fn log_panic<E: std::fmt::Display>(err: &E) {
    log_panic_f(err, &[])
}

/// Логирует и паникует с дополнительными полями если есть ошибка
pub fn log_panic_f<E: std::fmt::Display>(err: &E, fields: &[(&str, &str)]) {
    let mut msg = format!("{}", err);
    for (key, value) in fields {
        msg.push_str(&format!(" {}={}", key, value));
    }
    panic!("{}", msg);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_warning() {
        let err = "test error";
        log_warning(&err);
        // Визуальная проверка в логах
    }

    #[test]
    fn test_log_warning_f() {
        let err = "test error";
        log_warning_f(&err, &[("field", "value")]);
        // Визуальная проверка в логах
    }

    #[test]
    fn test_log_error() {
        let err = "test error";
        log_error(&err);
        // Визуальная проверка в логах
    }

    #[test]
    fn test_log_error_f() {
        let err = "test error";
        log_error_f(&err, &[("field", "value")]);
        // Визуальная проверка в логах
    }

    #[test]
    #[should_panic]
    fn test_log_panic() {
        let err = "test error";
        log_panic(&err);
    }
}
