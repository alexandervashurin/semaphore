//! Config Validator - валидация конфигурации
//!
//! Аналог util/config.go из Go версии (часть 3: валидация)

use crate::config::types::{Config, DbConfig, DbDialect, LdapConfig};
use crate::error::{Error, Result};
use std::net::SocketAddr;

/// Ошибки валидации
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Трейт для валидации
pub trait Validate {
    fn validate(&self) -> std::result::Result<(), ValidationError>;
}

impl Validate for Config {
    fn validate(&self) -> std::result::Result<(), ValidationError> {
        // Валидация БД
        self.database.validate()?;

        // Валидация LDAP (если включён)
        if let Some(ref ldap) = self.ldap {
            if ldap.enable {
                ldap.validate()?;
            }
        }

        // Валидация путей
        if self.tmp_path.is_empty() {
            return Err(ValidationError {
                field: "tmp_path".to_string(),
                message: "Tmp path cannot be empty".to_string(),
            });
        }

        // Валидация TCP адреса
        if !self.tcp_address.is_empty() {
            if let Err(e) = self.tcp_address.parse::<SocketAddr>() {
                return Err(ValidationError {
                    field: "tcp_address".to_string(),
                    message: format!("Invalid TCP address: {}", e),
                });
            }
        }

        Ok(())
    }
}

impl Validate for DbConfig {
    fn validate(&self) -> std::result::Result<(), ValidationError> {
        // Проверка диалекта
        match self.dialect {
            Some(DbDialect::MySQL) | Some(DbDialect::Postgres) | Some(DbDialect::SQLite) => {
                // OK
            }
            None => {
                return Err(ValidationError {
                    field: "db.dialect".to_string(),
                    message: "Database dialect must be specified".to_string(),
                });
            }
        }

        // Проверка hostname для MySQL/Postgres
        if matches!(
            self.dialect,
            Some(DbDialect::MySQL) | Some(DbDialect::Postgres)
        ) && self.hostname.is_empty()
        {
            return Err(ValidationError {
                field: "db.hostname".to_string(),
                message: "Hostname is required for MySQL/Postgres".to_string(),
            });
        }

        // Проверка db_name для SQLite
        if matches!(self.dialect, Some(DbDialect::SQLite)) && self.db_name.is_empty() {
            return Err(ValidationError {
                field: "db.name".to_string(),
                message: "Database name is required for SQLite".to_string(),
            });
        }

        Ok(())
    }
}

impl Validate for LdapConfig {
    fn validate(&self) -> std::result::Result<(), ValidationError> {
        if self.server.is_empty() {
            return Err(ValidationError {
                field: "ldap.server".to_string(),
                message: "LDAP server cannot be empty".to_string(),
            });
        }

        if self.bind_dn.is_empty() {
            return Err(ValidationError {
                field: "ldap.bind_dn".to_string(),
                message: "LDAP bind DN cannot be empty".to_string(),
            });
        }

        if self.search_dn.is_empty() {
            return Err(ValidationError {
                field: "ldap.search_dn".to_string(),
                message: "LDAP search DN cannot be empty".to_string(),
            });
        }

        Ok(())
    }
}

/// Проверяет существование пути
pub fn validate_path_exists(
    path: &str,
    create_if_not_exists: bool,
) -> std::result::Result<(), ValidationError> {
    use std::fs;
    use std::path::Path;

    let path = Path::new(path);

    if !path.exists() {
        if create_if_not_exists {
            if let Err(e) = fs::create_dir_all(path) {
                return Err(ValidationError {
                    field: "path".to_string(),
                    message: format!("Failed to create directory: {}", e),
                });
            }
        } else {
            return Err(ValidationError {
                field: "path".to_string(),
                message: format!("Path does not exist: {}", path.display()),
            });
        }
    }

    if !path.is_dir() {
        return Err(ValidationError {
            field: "path".to_string(),
            message: format!("Path is not a directory: {}", path.display()),
        });
    }

    Ok(())
}

/// Проверяет порт
pub fn validate_port(port: u16) -> std::result::Result<(), ValidationError> {
    if port == 0 {
        return Err(ValidationError {
            field: "port".to_string(),
            message: "Port cannot be zero".to_string(),
        });
    }
    Ok(())
}

/// Валидирует конфигурацию и возвращает ошибки
pub fn validate_config(config: &Config) -> std::result::Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    if let Err(e) = config.validate() {
        errors.push(e);
    }

    if let Err(e) = validate_path_exists(&config.tmp_path, true) {
        errors.push(e);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Валидирует и выводит предупреждения
pub fn validate_config_with_warnings(
    config: &Config,
) -> (std::result::Result<(), Vec<ValidationError>>, Vec<String>) {
    let mut warnings = Vec::new();

    // Проверка на insecure настройки
    if config.cookie_hash.len() < 32 {
        warnings.push("Cookie hash is too short, should be at least 32 bytes".to_string());
    }

    if config.cookie_encryption.len() < 32 {
        warnings
            .push("Cookie encryption key is too short, should be at least 32 bytes".to_string());
    }

    let result = validate_config(config);
    (result, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{Config, DbConfig, DbDialect, LdapConfig, LdapMappings};

    #[test]
    fn test_validate_db_config_mysql() {
        let config = DbConfig {
            dialect: Some(DbDialect::MySQL),
            hostname: "localhost".to_string(),
            db_name: "semaphore".to_string(),
            ..Default::default()
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_db_config_missing_dialect() {
        let config = DbConfig {
            dialect: None,
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("dialect"));
    }

    #[test]
    fn test_validate_config_empty_tmp_path() {
        let config = Config {
            tmp_path: String::new(),
            database: DbConfig {
                dialect: Some(DbDialect::SQLite),
                db_name: "test.db".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("tmp_path"));
    }

    #[test]
    fn test_validate_port() {
        assert!(validate_port(3000).is_ok());
        assert!(validate_port(0).is_err());
    }

    #[test]
    fn test_validate_path_exists() {
        use std::env;
        let temp_dir = env::temp_dir().to_string_lossy().to_string();
        assert!(validate_path_exists(&temp_dir, false).is_ok());
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError {
            field: "test_field".to_string(),
            message: "test error message".to_string(),
        };
        assert_eq!(format!("{}", error), "test_field: test error message");
    }

    #[test]
    fn test_validation_error_display_unicode() {
        let error = ValidationError {
            field: "поле".to_string(),
            message: "ошибка валидации".to_string(),
        };
        assert_eq!(format!("{}", error), "поле: ошибка валидации");
    }

    #[test]
    fn test_validate_db_config_postgres() {
        let config = DbConfig {
            dialect: Some(DbDialect::Postgres),
            hostname: "pg.example.com".to_string(),
            db_name: "appdb".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_db_config_sqlite() {
        let config = DbConfig {
            dialect: Some(DbDialect::SQLite),
            db_name: "/path/to/db.sqlite".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_db_config_mysql_missing_hostname() {
        let config = DbConfig {
            dialect: Some(DbDialect::MySQL),
            hostname: String::new(),
            db_name: "testdb".to_string(),
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("hostname"));
    }

    #[test]
    fn test_validate_db_config_postgres_missing_hostname() {
        let config = DbConfig {
            dialect: Some(DbDialect::Postgres),
            hostname: String::new(),
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("hostname"));
    }

    #[test]
    fn test_validate_db_config_sqlite_missing_db_name() {
        let config = DbConfig {
            dialect: Some(DbDialect::SQLite),
            db_name: String::new(),
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name"));
    }

    #[test]
    fn test_validate_ldap_config_valid() {
        let config = LdapConfig {
            enable: true,
            server: "ldap.example.com".to_string(),
            bind_dn: "cn=admin,dc=example,dc=com".to_string(),
            search_dn: "ou=users,dc=example,dc=com".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_ldap_config_empty_server() {
        let config = LdapConfig {
            enable: true,
            server: String::new(),
            bind_dn: "cn=admin".to_string(),
            search_dn: "ou=users".to_string(),
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("server"));
    }

    #[test]
    fn test_validate_ldap_config_empty_bind_dn() {
        let config = LdapConfig {
            enable: true,
            server: "ldap.example.com".to_string(),
            bind_dn: String::new(),
            search_dn: "ou=users".to_string(),
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("bind_dn"));
    }

    #[test]
    fn test_validate_ldap_config_empty_search_dn() {
        let config = LdapConfig {
            enable: true,
            server: "ldap.example.com".to_string(),
            bind_dn: "cn=admin".to_string(),
            search_dn: String::new(),
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("search_dn"));
    }

    #[test]
    fn test_validate_config_valid_tcp_address() {
        let config = Config {
            tmp_path: "/tmp/test".to_string(),
            tcp_address: "0.0.0.0:3000".to_string(),
            database: DbConfig {
                dialect: Some(DbDialect::SQLite),
                db_name: "test.db".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_config_invalid_tcp_address() {
        let config = Config {
            tmp_path: "/tmp/test".to_string(),
            tcp_address: "not_an_address".to_string(),
            database: DbConfig {
                dialect: Some(DbDialect::SQLite),
                db_name: "test.db".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("tcp_address"));
    }

    #[test]
    fn test_validate_config_empty_tcp_address() {
        let config = Config {
            tmp_path: "/tmp/test".to_string(),
            tcp_address: String::new(),
            database: DbConfig {
                dialect: Some(DbDialect::SQLite),
                db_name: "test.db".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        // Empty tcp_address is allowed (skipped validation)
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_config_with_ldap_disabled() {
        let config = Config {
            tmp_path: "/tmp/test".to_string(),
            ldap: Some(LdapConfig {
                enable: false,
                server: String::new(),
                ..Default::default()
            }),
            database: DbConfig {
                dialect: Some(DbDialect::SQLite),
                db_name: "test.db".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        // LDAP is disabled, so validation should pass even with empty server
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_config_with_ldap_enabled_valid() {
        let config = Config {
            tmp_path: "/tmp/test".to_string(),
            ldap: Some(LdapConfig {
                enable: true,
                server: "ldap.example.com".to_string(),
                bind_dn: "cn=admin".to_string(),
                search_dn: "ou=users".to_string(),
                ..Default::default()
            }),
            database: DbConfig {
                dialect: Some(DbDialect::SQLite),
                db_name: "test.db".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_config_with_ldap_enabled_invalid() {
        let config = Config {
            tmp_path: "/tmp/test".to_string(),
            ldap: Some(LdapConfig {
                enable: true,
                server: String::new(),
                bind_dn: "cn=admin".to_string(),
                search_dn: "ou=users".to_string(),
                ..Default::default()
            }),
            database: DbConfig {
                dialect: Some(DbDialect::SQLite),
                db_name: "test.db".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_exists_create() {
        use std::env;
        let new_dir = env::temp_dir().join("test_validate_create_dir");
        let _ = std::fs::remove_dir_all(&new_dir);
        assert!(validate_path_exists(&new_dir.to_string_lossy(), true).is_ok());
        assert!(new_dir.exists());
        let _ = std::fs::remove_dir_all(&new_dir);
    }

    #[test]
    fn test_validate_path_not_exists_no_create() {
        let result = validate_path_exists("/nonexistent/path/that/does/not/exist", false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_validate_port_edge_cases() {
        assert!(validate_port(1).is_ok());
        assert!(validate_port(80).is_ok());
        assert!(validate_port(443).is_ok());
        assert!(validate_port(8080).is_ok());
        assert!(validate_port(65535).is_ok());
        assert!(validate_port(0).is_err());
    }

    #[test]
    fn test_validate_config_with_warnings_short_cookie_hash() {
        let config = Config {
            tmp_path: "/tmp/test".to_string(),
            cookie_hash: vec![1, 2, 3],
            cookie_encryption: vec![0u8; 32],
            database: DbConfig {
                dialect: Some(DbDialect::SQLite),
                db_name: "test.db".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let (result, warnings) = validate_config_with_warnings(&config);
        assert!(result.is_ok());
        assert!(warnings.iter().any(|w| w.contains("Cookie hash")));
    }

    #[test]
    fn test_validate_config_with_warnings_short_encryption() {
        let config = Config {
            tmp_path: "/tmp/test".to_string(),
            cookie_hash: vec![0u8; 32],
            cookie_encryption: vec![1, 2, 3],
            database: DbConfig {
                dialect: Some(DbDialect::SQLite),
                db_name: "test.db".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let (result, warnings) = validate_config_with_warnings(&config);
        assert!(result.is_ok());
        assert!(warnings.iter().any(|w| w.contains("encryption")));
    }

    #[test]
    fn test_validate_config_with_warnings_both_short() {
        let config = Config {
            tmp_path: "/tmp/test".to_string(),
            cookie_hash: vec![1],
            cookie_encryption: vec![2],
            database: DbConfig {
                dialect: Some(DbDialect::SQLite),
                db_name: "test.db".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let (result, warnings) = validate_config_with_warnings(&config);
        assert!(result.is_ok());
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn test_validate_config_no_warnings() {
        let config = Config {
            tmp_path: "/tmp/test".to_string(),
            cookie_hash: vec![0u8; 64],
            cookie_encryption: vec![0u8; 64],
            database: DbConfig {
                dialect: Some(DbDialect::SQLite),
                db_name: "test.db".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let (result, warnings) = validate_config_with_warnings(&config);
        assert!(result.is_ok());
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_validate_config_db_name_edge_cases() {
        let config = DbConfig {
            dialect: Some(DbDialect::SQLite),
            db_name: "数据库".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_ldap_unicode_values() {
        let config = LdapConfig {
            enable: true,
            server: "ldap.example.com".to_string(),
            bind_dn: "cn=админ,dc=пример,dc=ru".to_string(),
            search_dn: "ou=пользователи,dc=пример,dc=ru".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
}
