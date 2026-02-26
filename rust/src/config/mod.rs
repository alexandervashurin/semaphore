//! Config модуль
//!
//! Конфигурация приложения

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
