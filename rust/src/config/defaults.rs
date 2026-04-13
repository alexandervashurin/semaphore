//! Config Defaults - значения по умолчанию
//!
//! Аналог util/config.go из Go версии (часть 4: значения по умолчанию)

use crate::config::types::{
    AlertConfig, AuthConfig, Config, DbConfig, HAConfig, HARedisConfig, LdapMappings, TotpConfig,
};

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
    "velum".to_string()
}

/// Значение по умолчанию для TCP адреса
fn default_tcp_address() -> String {
    "0.0.0.0:3000".to_string()
}

/// Значение по умолчанию для временной директории
fn default_tmp_path() -> String {
    "/tmp/velum".to_string()
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
            path: None,
            connection_string: None,
        },
        ldap: None,
        auth: AuthConfig {
            totp: TotpConfig {
                enable: false,
                allow_recovery: false,
            },
            email_enabled: false,
            oidc_providers: Vec::new(),
            email_login_enabled: false,
        },
        ha: HAConfig {
            enable: false,
            redis: HARedisConfig::default(),
            node_id: String::new(),
        },
        tmp_path: default_tmp_path(),
        cookie_hash: Vec::new(),
        cookie_encryption: Vec::new(),
        mailer_host: String::new(),
        mailer_port: "25".to_string(),
        mailer_username: None,
        mailer_password: None,
        mailer_use_tls: false,
        mailer_secure: false,
        mailer_from: "noreply@localhost".to_string(),
        alert: AlertConfig {
            enabled: false,
            email: None,
            all_projects: false,
        },
        email_sender: "velum@localhost".to_string(),
        telegram_bot_token: None,
        redis: None,
        kubernetes: None,
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
        assert_eq!(default_db_name(), "velum");
    }

    #[test]
    fn test_default_tcp_address() {
        assert_eq!(default_tcp_address(), "0.0.0.0:3000");
    }

    #[test]
    fn test_default_tmp_path() {
        assert_eq!(default_tmp_path(), "/tmp/velum");
    }

    #[test]
    fn test_create_default_config() {
        let config = create_default_config();
        assert_eq!(config.tcp_address, "0.0.0.0:3000");
        assert_eq!(config.tmp_path, "/tmp/velum");
        assert_eq!(config.database.hostname, "0.0.0.0");
        assert_eq!(config.database.db_name, "velum");
    }

    #[test]
    fn test_apply_defaults() {
        let mut config = Config::default();
        config.tcp_address = String::new(); // Сбросить значение

        apply_defaults(&mut config);

        assert_eq!(config.tcp_address, "0.0.0.0:3000");
        assert_eq!(config.tmp_path, "/tmp/velum");
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
        assert_eq!(config.tmp_path, "/tmp/velum");
    }

    #[test]
    fn test_default_ldap_dn() {
        assert_eq!(default_ldap_dn(), "dn");
    }

    #[test]
    fn test_default_ldap_mail() {
        assert_eq!(default_ldap_mail(), "mail");
    }

    #[test]
    fn test_default_ldap_uid() {
        assert_eq!(default_ldap_uid(), "uid");
    }

    #[test]
    fn test_default_ldap_cn() {
        assert_eq!(default_ldap_cn(), "cn");
    }

    #[test]
    fn test_load_defaults_with_empty_config() {
        let mut config = Config::default();
        load_defaults(&mut config);

        assert_eq!(config.database.hostname, "0.0.0.0");
        assert_eq!(config.database.db_name, "velum");
    }

    #[test]
    fn test_load_defaults_preserves_existing_db_host() {
        let mut config = Config {
            database: DbConfig {
                hostname: "custom-host.example.com".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        load_defaults(&mut config);

        assert_eq!(config.database.hostname, "custom-host.example.com");
    }

    #[test]
    fn test_load_defaults_with_ldap_mappings() {
        use crate::config::types::LdapConfig;

        let mut config = Config {
            ldap: Some(LdapConfig {
                mappings: LdapMappings::default(),
                ..Default::default()
            }),
            ..Default::default()
        };

        load_defaults(&mut config);

        let ldap = config.ldap.as_ref().unwrap();
        assert_eq!(ldap.mappings.dn, "dn");
        assert_eq!(ldap.mappings.mail, "mail");
        assert_eq!(ldap.mappings.uid, "uid");
        assert_eq!(ldap.mappings.cn, "cn");
    }

    #[test]
    fn test_load_defaults_preserves_existing_ldap_mappings() {
        use crate::config::types::LdapConfig;
        use crate::config::types::LdapMappings;

        let mut config = Config {
            ldap: Some(LdapConfig {
                mappings: LdapMappings {
                    dn: "distinguishedName".to_string(),
                    mail: "email".to_string(),
                    uid: "sAMAccountName".to_string(),
                    cn: "displayName".to_string(),
                },
                ..Default::default()
            }),
            ..Default::default()
        };

        load_defaults(&mut config);

        let ldap = config.ldap.as_ref().unwrap();
        assert_eq!(ldap.mappings.dn, "distinguishedName");
        assert_eq!(ldap.mappings.mail, "email");
        assert_eq!(ldap.mappings.uid, "sAMAccountName");
        assert_eq!(ldap.mappings.cn, "displayName");
    }

    #[test]
    fn test_load_defaults_without_ldap() {
        let mut config = Config {
            ldap: None,
            ..Default::default()
        };

        load_defaults(&mut config);

        // Should not panic even without LDAP
        assert_eq!(config.database.hostname, "0.0.0.0");
        assert_eq!(config.database.db_name, "velum");
    }

    #[test]
    fn test_create_default_config_mailer() {
        let config = create_default_config();
        assert_eq!(config.mailer_port, "25");
        assert_eq!(config.mailer_from, "noreply@localhost");
        assert_eq!(config.email_sender, "velum@localhost");
    }

    #[test]
    fn test_create_default_config_ha_disabled() {
        let config = create_default_config();
        assert!(!config.ha.enable);
        assert!(config.ha.node_id.is_empty());
    }

    #[test]
    fn test_create_default_config_auth_disabled() {
        let config = create_default_config();
        assert!(!config.auth.totp.enable);
        assert!(!config.auth.totp.allow_recovery);
        assert!(!config.auth.email_enabled);
        assert!(!config.auth.email_login_enabled);
    }

    #[test]
    fn test_create_default_config_alerts() {
        let config = create_default_config();
        assert!(!config.alert.enabled);
        assert!(config.alert.email.is_none());
        assert!(!config.alert.all_projects);
    }

    #[test]
    fn test_create_default_config_empty_secrets() {
        let config = create_default_config();
        assert!(config.cookie_hash.is_empty());
        assert!(config.cookie_encryption.is_empty());
    }

    #[test]
    fn test_create_default_config_optional_fields() {
        let config = create_default_config();
        assert!(config.telegram_bot_token.is_none());
        assert!(config.redis.is_none());
        assert!(config.kubernetes.is_none());
    }

    #[test]
    fn test_apply_defaults_partial_config() {
        let mut config = Config {
            database: DbConfig {
                hostname: "my-host".to_string(),
                db_name: String::new(),
                ..Default::default()
            },
            tmp_path: String::new(),
            ..Default::default()
        };

        apply_defaults(&mut config);

        // hostname should be preserved
        assert_eq!(config.database.hostname, "my-host");
        // db_name should be filled (empty)
        assert_eq!(config.database.db_name, "velum");
        // tmp_path should be filled
        assert_eq!(config.tmp_path, "/tmp/velum");
    }

    #[test]
    fn test_apply_defaults_without_ldap() {
        let mut config = Config {
            ldap: None,
            tmp_path: String::new(),
            ..Default::default()
        };

        apply_defaults(&mut config);

        // Should not panic
        assert_eq!(config.tmp_path, "/tmp/velum");
    }

    #[test]
    fn test_load_defaults_db_name_empty() {
        let mut config = Config {
            database: DbConfig {
                db_name: String::new(),
                ..Default::default()
            },
            ..Default::default()
        };

        load_defaults(&mut config);

        assert_eq!(config.database.db_name, "velum");
    }
}
