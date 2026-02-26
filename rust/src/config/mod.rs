//! Config модуль
//!
//! Конфигурация приложения

pub mod types;

pub use types::{Config, DbConfig, DbDialect, LdapConfig, LdapMappings, AuthConfig, TotpConfig, HAConfig, HARedisConfig};
