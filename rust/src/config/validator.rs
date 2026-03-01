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
            Some(DbDialect::Bolt) => {
                // Deprecated
                eprintln!("Warning: Bolt dialect is deprecated, use SQLite instead");
            }
            None => {
                return Err(ValidationError {
                    field: "db.dialect".to_string(),
                    message: "Database dialect must be specified".to_string(),
                });
            }
        }
        
        // Проверка hostname для MySQL/Postgres
        if matches!(self.dialect, Some(DbDialect::MySQL) | Some(DbDialect::Postgres)) {
            if self.hostname.is_empty() {
                return Err(ValidationError {
                    field: "db.hostname".to_string(),
                    message: "Hostname is required for MySQL/Postgres".to_string(),
                });
            }
        }
        
        // Проверка db_name для SQLite
        if matches!(self.dialect, Some(DbDialect::SQLite)) {
            if self.db_name.is_empty() {
                return Err(ValidationError {
                    field: "db.name".to_string(),
                    message: "Database name is required for SQLite".to_string(),
                });
            }
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
pub fn validate_path_exists(path: &str, create_if_not_exists: bool) -> std::result::Result<(), ValidationError> {
    use std::path::Path;
    use std::fs;
    
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
pub fn validate_config_with_warnings(config: &Config) -> (std::result::Result<(), Vec<ValidationError>>, Vec<String>) {
    let mut warnings = Vec::new();
    
    // Проверка на deprecated настройки
    if config.database.dialect == Some(DbDialect::Bolt) {
        warnings.push("Bolt database dialect is deprecated, use SQLite instead".to_string());
    }
    
    // Проверка на insecure настройки
    if config.cookie_hash.len() < 32 {
        warnings.push("Cookie hash is too short, should be at least 32 bytes".to_string());
    }
    
    if config.cookie_encryption.len() < 32 {
        warnings.push("Cookie encryption key is too short, should be at least 32 bytes".to_string());
    }
    
    let result = validate_config(config);
    (result, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{Config, DbConfig, DbDialect};

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
}
