//! Config модуль
//!
//! Конфигурация приложения

pub mod config_dirs;
pub mod config_helpers;
pub mod defaults;
pub mod config_ldap;
pub mod config_oidc;
pub mod config_ha;
pub mod config_logging;
pub mod loader;
pub mod types;
pub mod validator;

pub use types::{Config, DbConfig, DbDialect, LdapConfig, LdapMappings, AuthConfig, TotpConfig, HAConfig, HARedisConfig};
pub use loader::{load_config, load_from_file, load_from_env, merge_configs};
pub use validator::{validate_config, validate_config_with_warnings, Validate, ValidationError};
pub use defaults::{load_defaults, apply_defaults, create_default_config};
pub use config_ldap::{LdapConfigFull, load_ldap_from_env};
pub use config_oidc::{OidcProvider, OidcEndpoint, load_oidc_from_env};
pub use config_ha::{HAConfigFull, HARedisConfigFull, load_ha_from_env};
pub use config_logging::{LoggingConfig, LogFormat, LogLevel, load_logging_from_env};
pub use config_dirs::{clear_dir, ensure_dir_exists, get_project_tmp_dir, clear_project_tmp_dir, create_project_tmp_dir, get_or_create_project_tmp_dir, is_safe_path, create_unique_tmp_dir};
pub use config_helpers::{find_semaphore, get_ansible_version, check_update, lookup_default_apps, get_public_host, generate_recovery_code, verify_recovery_code, get_public_alias_url};

/// Проверяет, включены ли email уведомления
/// TODO: Реализовать через Config.alert
pub fn email_alert_enabled() -> bool {
    false // Заглушка до реализации конфигурации алертов
}

/// Получает отправителя email
/// TODO: Реализовать через Config.email_sender
pub fn get_email_sender() -> String {
    String::from("semaphore@localhost") // Заглушка
}
