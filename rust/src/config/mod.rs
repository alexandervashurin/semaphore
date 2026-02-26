//! Config модуль
//!
//! Конфигурация приложения

pub mod types;
pub mod loader;
pub mod validator;
pub mod defaults;

pub use types::{Config, DbConfig, DbDialect, LdapConfig, LdapMappings, AuthConfig, TotpConfig, HAConfig, HARedisConfig};
pub use loader::{load_config, load_from_file, load_from_env, merge_configs};
pub use validator::{validate_config, validate_config_with_warnings, Validate, ValidationError};
pub use defaults::{load_defaults, apply_defaults, create_default_config};
