//! Config Defaults - значения по умолчанию
//!
//! Аналог util/config.go из Go версии (часть 4: значения по умолчанию)

use crate::config::types::{Config, DbConfig, LdapMappings, AuthConfig, TotpConfig, HAConfig, HARedisConfig};

/// Загружает значения по умолчанию для конфигурации
pub fn load_defaults(config: &mut Config) {
    // Database defaults
    if config.database.hostname.is_empty() {
        config.database.hostname = default_db_host();
    }
    if config.database.db_name.is_empty() {
        config.database.db_name = default_db_name();
    }
    
    // TCP address default
    if config.tcp_address.is_empty() {
        config.tcp_address = default_tcp_address();
    }
    
    // Tmp path default
    if config.tmp_path.is_empty() {
        config.tmp_path = default_tmp_path();
    }
    
    // LDAP mappings defaults
    if let Some(ref mut ldap) = config.ldap {
        if ldap.mappings.dn.is_empty() {
            ldap.mappings.dn = default_ldap_dn();
        }
        if ldap.mappings.mail.is_empty() {
            ldap.mappings.mail = default_ldap_mail();
        }
        if ldap.mappings.uid.is_empty() {
            ldap.mappings.uid = default_ldap_uid();
        }
        if ldap.mappings.cn.is_empty() {
            ldap.mappings.cn = default_ldap_cn();
        }
    }
}

/// Значение по умолчанию для хоста БД
fn default_db_host() -> String {
    "0.0.0.0".to_string()
}

/// Значение по умолчанию для имени БД
fn default_db_name() -> String {
    "semaphore".to_string()
}

/// Значение по умолчанию для TCP адреса
fn default_tcp_address() -> String {
    "0.0.0.0:3000".to_string()
}

/// Значение по умолчанию для временной директории
fn default_tmp_path() -> String {
    "/tmp/semaphore".to_string()
}

/// Значение по умолчанию для LDAP DN
fn default_ldap_dn() -> String {
    "dn".to_string()
}

/// Значение по умолчанию для LDAP mail
fn default_ldap_mail() -> String {
    "mail".to_string()
}

/// Значение по умолчанию для LDAP uid
fn default_ldap_uid() -> String {
    "uid".to_string()
}

/// Значение по умолчанию для LDAP cn
fn default_ldap_cn() -> String {
    "cn".to_string()
}

/// Создаёт конфигурацию с значениями по умолчанию
pub fn create_default_config() -> Config {
    Config {
        web_host: String::new(),
        tcp_address: default_tcp_address(),
        database: DbConfig {
            dialect: None,
            hostname: default_db_host(),
            username: String::new(),
            password: String::new(),
            db_name: default_db_name(),
            options: Default::default(),
        },
        ldap: None,
        auth: AuthConfig {
            totp: TotpConfig {
                enable: false,
                allow_recovery: false,
            },
        },
        ha: HAConfig {
            enable: false,
            redis: HARedisConfig::default(),
            node_id: String::new(),
        },
        tmp_path: default_tmp_path(),
        cookie_hash: Vec::new(),
        cookie_encryption: Vec::new(),
    }
}

/// Применяет значения по умолчанию только для отсутствующих полей
pub fn apply_defaults(config: &mut Config) {
    if config.database.hostname.is_empty() {
        config.database.hostname = default_db_host();
    }
    
    if config.database.db_name.is_empty() {
        config.database.db_name = default_db_name();
    }
    
    if config.tcp_address.is_empty() {
        config.tcp_address = default_tcp_address();
    }
    
    if config.tmp_path.is_empty() {
        config.tmp_path = default_tmp_path();
    }
    
    if let Some(ref mut ldap) = config.ldap {
        if ldap.mappings.dn.is_empty() {
            ldap.mappings.dn = default_ldap_dn();
        }
        if ldap.mappings.mail.is_empty() {
            ldap.mappings.mail = default_ldap_mail();
        }
        if ldap.mappings.uid.is_empty() {
            ldap.mappings.uid = default_ldap_uid();
        }
        if ldap.mappings.cn.is_empty() {
            ldap.mappings.cn = default_ldap_cn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_db_host() {
        assert_eq!(default_db_host(), "0.0.0.0");
    }

    #[test]
    fn test_default_db_name() {
        assert_eq!(default_db_name(), "semaphore");
    }

    #[test]
    fn test_default_tcp_address() {
        assert_eq!(default_tcp_address(), "0.0.0.0:3000");
    }

    #[test]
    fn test_default_tmp_path() {
        assert_eq!(default_tmp_path(), "/tmp/semaphore");
    }

    #[test]
    fn test_create_default_config() {
        let config = create_default_config();
        assert_eq!(config.tcp_address, "0.0.0.0:3000");
        assert_eq!(config.tmp_path, "/tmp/semaphore");
        assert_eq!(config.database.hostname, "0.0.0.0");
        assert_eq!(config.database.db_name, "semaphore");
    }

    #[test]
    fn test_apply_defaults() {
        let mut config = Config::default();
        config.tcp_address = String::new(); // Сбросить значение
        
        apply_defaults(&mut config);
        
        assert_eq!(config.tcp_address, "0.0.0.0:3000");
        assert_eq!(config.tmp_path, "/tmp/semaphore");
    }

    #[test]
    fn test_apply_defaults_preserves_existing() {
        let mut config = Config {
            tcp_address: "127.0.0.1:8080".to_string(),
            ..Default::default()
        };
        
        apply_defaults(&mut config);
        
        // Существующее значение должно сохраниться
        assert_eq!(config.tcp_address, "127.0.0.1:8080");
        // Пустое значение должно заполниться
        assert_eq!(config.tmp_path, "/tmp/semaphore");
    }
}
