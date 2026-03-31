//! Config модуль
//!
//! Конфигурация приложения

pub mod config_auth;
pub mod config_dirs;
pub mod config_ha;
pub mod config_helpers;
pub mod config_ldap;
pub mod config_logging;
pub mod config_oidc;
pub mod config_sysproc;
pub mod defaults;
pub mod loader;
pub mod types;
pub mod validator;

pub use config_dirs::{
    clear_dir, clear_project_tmp_dir, create_project_tmp_dir, create_unique_tmp_dir,
    ensure_dir_exists, get_or_create_project_tmp_dir, get_project_tmp_dir, is_safe_path,
};
pub use config_ha::{load_ha_from_env, HAConfigFull, HARedisConfigFull};
pub use config_helpers::{
    check_update, find_velum, generate_recovery_code, get_ansible_version, get_public_alias_url,
    get_public_host, lookup_default_apps, verify_recovery_code,
};
pub use config_ldap::{load_ldap_from_env, LdapConfigFull};
pub use config_logging::{load_logging_from_env, LogFormat, LogLevel, LoggingConfig};
pub use config_oidc::{load_oidc_from_env, OidcEndpoint, OidcProvider};
pub use defaults::{apply_defaults, create_default_config, load_defaults};
pub use loader::{load_config, load_from_env, load_from_file, merge_configs};
pub use types::{
    AuthConfig, Config, DbConfig, DbDialect, HAConfig, HARedisConfig, LdapConfig, LdapMappings,
    RedisConfig, TotpConfig,
};
pub use validator::{validate_config, validate_config_with_warnings, Validate, ValidationError};

/// Проверяет, включены ли email уведомления
pub fn email_alert_enabled() -> bool {
    // В полной реализации нужно загружать конфиг и проверять alert.enabled
    // Пока используем переменную окружения
    std::env::var("VELUM_ALERT_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

/// Получает отправителя email
pub fn get_email_sender() -> String {
    std::env::var("VELUM_EMAIL_SENDER")
        .or_else(|_| std::env::var("VELUM_MAILER_FROM"))
        .unwrap_or_else(|_| String::from("velum@localhost"))
}

/// Собирает SmtpConfig из переменных окружения
pub fn get_smtp_config() -> crate::utils::mailer::SmtpConfig {
    crate::utils::mailer::SmtpConfig {
        host: std::env::var("VELUM_MAILER_HOST").unwrap_or_else(|_| String::from("localhost")),
        port: std::env::var("VELUM_MAILER_PORT").unwrap_or_else(|_| String::from("25")),
        username: std::env::var("VELUM_MAILER_USERNAME").ok(),
        password: std::env::var("VELUM_MAILER_PASSWORD").ok(),
        use_tls: std::env::var("VELUM_MAILER_USE_TLS")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false),
        secure: std::env::var("VELUM_MAILER_SECURE")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false),
        from: get_email_sender(),
    }
}
