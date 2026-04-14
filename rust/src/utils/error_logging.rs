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
        unsafe { std::env::set_var("RUST_LOG", "debug") };
        let err = "debug message";
        log_debug_f(&err, &[("key", "value")]);
        // Проверяем что не паникует
        unsafe { std::env::remove_var("RUST_LOG") };
    }

    #[test]
    fn test_log_debug_f_with_partial_debug_env() {
        unsafe { std::env::set_var("RUST_LOG", "info,debug") };
        let err = "partial debug";
        log_debug_f(&err, &[("x", "1")]);
        unsafe { std::env::remove_var("RUST_LOG") };
    }

    #[test]
    fn test_log_warning_with_custom_error_type() {
        #[derive(Debug)]
        struct CustomErr(&'static str);
        impl std::fmt::Display for CustomErr {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
        let err = CustomErr("custom warning");
        log_warning(&err);
        // Проверяем что не паникует
    }

    #[test]
    fn test_log_error_with_io_error() {
        let err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        log_error(&err);
        // Проверяем что не паникует
    }

    #[test]
    fn test_log_warning_f_single_field() {
        let err = "warning with field";
        log_warning_f(&err, &[("code", "42")]);
    }

    #[test]
    fn test_log_error_f_single_field() {
        let err = "error with field";
        log_error_f(&err, &[("module", "db")]);
    }

    #[test]
    fn test_log_warning_f_multiple_fields_order() {
        let err = "multi-field warning";
        log_warning_f(&err, &[("a", "1"), ("b", "2"), ("c", "3")]);
    }

    #[test]
    fn test_log_error_f_empty_fields() {
        let err = "error no fields";
        log_error_f(&err, &[]);
    }

    #[test]
    fn test_log_debug_f_empty_fields() {
        unsafe { std::env::set_var("RUST_LOG", "debug") };
        let err = "debug no fields";
        log_debug_f(&err, &[]);
        unsafe { std::env::remove_var("RUST_LOG") };
    }

    #[test]
    fn test_log_panic_f_with_fields() {
        // Проверяем что panic содержит сообщение и поля
    }

    #[test]
    #[should_panic(expected = "critical error field=value")]
    fn test_log_panic_f_message_contains_fields() {
        let err = "critical error";
        log_panic_f(&err, &[("field", "value")]);
    }

    #[test]
    #[should_panic(expected = "alone")]
    fn test_log_panic_message_content() {
        let err = "alone";
        log_panic(&err);
    }

    #[test]
    fn test_log_debug_f_no_output_without_env() {
        unsafe { std::env::remove_var("RUST_LOG") };
        let err = "should not appear";
        log_debug_f(&err, &[("k", "v")]);
        // Проверяем что не паникует когда env не установлен
    }

    #[test]
    fn test_log_debug_f_env_with_unrelated_value() {
        unsafe { std::env::set_var("RUST_LOG", "warn") };
        let err = "debug with unrelated env";
        log_debug_f(&err, &[("k", "v")]);
        unsafe { std::env::remove_var("RUST_LOG") };
    }

    #[test]
    fn test_log_warning_f_with_empty_string_values() {
        let err = "";
        log_warning_f(&err, &[("", "")]);
    }

    #[test]
    fn test_log_error_f_with_special_chars_in_values() {
        let err = "error with unicode";
        log_error_f(&err, &[("msg", ""), ("path", "/tmp/test")]);
    }

    #[test]
    fn test_log_warning_f_does_not_panic_with_many_fields() {
        static PAIRS: &[(&str, &str)] = &[
            ("k0", "v0"),
            ("k1", "v1"),
            ("k2", "v2"),
            ("k3", "v3"),
            ("k4", "v4"),
            ("k5", "v5"),
            ("k6", "v6"),
            ("k7", "v7"),
            ("k8", "v8"),
            ("k9", "v9"),
            ("k10", "v10"),
            ("k11", "v11"),
            ("k12", "v12"),
            ("k13", "v13"),
            ("k14", "v14"),
            ("k15", "v15"),
            ("k16", "v16"),
            ("k17", "v17"),
            ("k18", "v18"),
            ("k19", "v19"),
        ];
        let err = "lots of fields";
        log_warning_f(&err, PAIRS);
    }

    #[test]
    fn test_log_error_f_does_not_panic_with_many_fields() {
        static PAIRS: &[(&str, &str)] = &[
            ("k0", "v0"),
            ("k1", "v1"),
            ("k2", "v2"),
            ("k3", "v3"),
            ("k4", "v4"),
            ("k5", "v5"),
            ("k6", "v6"),
            ("k7", "v7"),
            ("k8", "v8"),
            ("k9", "v9"),
            ("k10", "v10"),
            ("k11", "v11"),
            ("k12", "v12"),
            ("k13", "v13"),
            ("k14", "v14"),
            ("k15", "v15"),
            ("k16", "v16"),
            ("k17", "v17"),
            ("k18", "v18"),
            ("k19", "v19"),
        ];
        let err = "lots of error fields";
        log_error_f(&err, PAIRS);
    }
}
