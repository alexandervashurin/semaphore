//! Error Logging Utilities
//!
//! Утилиты для логирования ошибок

use tracing::{debug, error, warn};

/// Логирует warning с произвольным полем если есть ошибка
pub fn log_warning<E: std::fmt::Display>(err: &E) {
    log_warning_f(err, &[])
}

/// Логирует debug с дополнительными полями если есть ошибка
pub fn log_debug_f<E: std::fmt::Display>(err: &E, fields: &[(&str, &str)]) {
    if std::env::var("RUST_LOG")
        .unwrap_or_default()
        .contains("debug")
    {
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

    #[test]
    #[should_panic]
    fn test_log_panic_f() {
        let err = "test error";
        log_panic_f(&err, &[("field", "value")]);
    }

    #[test]
    fn test_log_warning_f_empty_fields() {
        let err = "simple warning";
        log_warning_f(&err, &[]);
        // Проверяем что не паникует с пустым массивом полей
    }

    #[test]
    fn test_log_error_f_multiple_fields() {
        let err = "complex error";
        log_error_f(
            &err,
            &[("file", "test.rs"), ("line", "42"), ("module", "utils")],
        );
        // Проверяем что не паникует с несколькими полями
    }

    #[test]
    fn test_log_debug_f_without_debug_env() {
        // RUST_LOG не содержит "debug", поэтому log_debug_f не должна логировать
        let err = "debug message";
        log_debug_f(&err, &[("key", "value")]);
        // Проверяем что не паникует
    }

    #[test]
    fn test_log_debug_f_with_debug_env() {
        // Установим RUST_LOG=debug чтобы покрыть ветку с логированием
        std::env::set_var("RUST_LOG", "debug");
        let err = "debug message";
        log_debug_f(&err, &[("key", "value")]);
        // Проверяем что не паникует
        std::env::remove_var("RUST_LOG");
    }

    #[test]
    fn test_log_debug_f_with_partial_debug_env() {
        std::env::set_var("RUST_LOG", "info,debug");
        let err = "partial debug";
        log_debug_f(&err, &[("x", "1")]);
        std::env::remove_var("RUST_LOG");
    }
}
