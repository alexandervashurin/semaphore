//! Config модуль
//!
//! Конфигурация приложения

pub mod types;
pub mod loader;

pub use types::{Config, DbConfig, DbDialect, LdapConfig, LdapMappings, AuthConfig, TotpConfig, HAConfig, HARedisConfig};
pub use loader::{load_config, load_from_file, load_from_env, merge_configs};
