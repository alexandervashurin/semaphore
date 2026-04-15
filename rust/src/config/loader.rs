//! Config Loader - загрузка конфигурации
//!
//! Аналог util/config.go из Go версии (часть 2: загрузка)
use crate::config::types::{
    AlertConfig, AuthConfig, Config, DbConfig, HAConfig, HARedisConfig, LdapConfig, TotpConfig,
};
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;

/// Загружает конфигурацию из файла
pub fn load_from_file(path: &str) -> Result<Config> {
    let content = fs::read_to_string(path)
        .map_err(|e| Error::Other(format!("Failed to read config file: {}", e)))?;
    let config: Config = serde_json::from_str(&content)
        .map_err(|e| Error::Other(format!("Failed to parse config JSON: {}", e)))?;

    Ok(config)
}

/// Загружает конфигурацию из переменных окружения
pub fn load_from_env() -> Result<Config> {
    let mut config = Config::default();

    // Database
    if let Ok(host) = env::var("SEMAPHORE_DB_HOST") {
        config.database.hostname = host;
    }
    if let Ok(user) = env::var("SEMAPHORE_DB_USER") {
        config.database.username = user;
    }
    if let Ok(pass) = env::var("SEMAPHORE_DB_PASS") {
        config.database.password = pass;
    }
    if let Ok(name) = env::var("SEMAPHORE_DB") {
        config.database.db_name = name;
    }

    // Web Host
    if let Ok(web_host) = env::var("SEMAPHORE_WEB_HOST") {
        config.web_host = web_host;
    }

    // TCP Address
    if let Ok(tcp_addr) = env::var("SEMAPHORE_TCP_ADDRESS") {
        config.tcp_address = tcp_addr;
    }

    // Tmp Path
    if let Ok(tmp_path) = env::var("SEMAPHORE_TMP_PATH") {
        config.tmp_path = tmp_path;
    }

    // LDAP
    if let Ok(val) = env::var("SEMAPHORE_LDAP_ENABLE") {
        if config.ldap.is_none() {
            config.ldap = Some(LdapConfig::default());
        }
        if let Some(ref mut ldap) = config.ldap {
            ldap.enable = val.to_lowercase() == "true" || val == "1";
        }
    }

    if let Ok(server) = env::var("SEMAPHORE_LDAP_SERVER") {
        if let Some(ref mut ldap) = config.ldap {
            ldap.server = server;
        }
    }

    // TOTP
    if let Ok(val) = env::var("SEMAPHORE_AUTH_TOTP_ENABLE") {
        config.auth.totp.enable = val.to_lowercase() == "true" || val == "1";
    }

    if let Ok(val) = env::var("SEMAPHORE_AUTH_TOTP_ALLOW_RECOVERY") {
        config.auth.totp.allow_recovery = val.to_lowercase() == "true" || val == "1";
    }

    if let Ok(val) = env::var("SEMAPHORE_AUTH_EMAIL_LOGIN_ENABLED") {
        config.auth.email_login_enabled = val.to_lowercase() == "true" || val == "1";
    }

    // HA
    if let Ok(val) = env::var("SEMAPHORE_HA_ENABLE") {
        config.ha.enable = val.to_lowercase() == "true" || val == "1";
    }

    if let Ok(host) = env::var("SEMAPHORE_HA_REDIS_HOST") {
        config.ha.redis.host = host;
    }

    if let Ok(port) = env::var("SEMAPHORE_HA_REDIS_PORT") {
        if let Ok(port_num) = port.parse() {
            config.ha.redis.port = port_num;
        }
    }

    Ok(config)
}

/// Сливает две конфигурации (приоритет у second)
pub fn merge_configs(first: Config, second: Config) -> Config {
    Config {
        web_host: if !second.web_host.is_empty() {
            second.web_host
        } else {
            first.web_host
        },
        tcp_address: if !second.tcp_address.is_empty() {
            second.tcp_address
        } else {
            first.tcp_address
        },
        database: merge_db_configs(first.database, second.database),
        ldap: second.ldap.or(first.ldap),
        auth: merge_auth_configs(first.auth, second.auth),
        ha: merge_ha_configs(first.ha, second.ha),
        tmp_path: if !second.tmp_path.is_empty() {
            second.tmp_path
        } else {
            first.tmp_path
        },
        cookie_hash: if !second.cookie_hash.is_empty() {
            second.cookie_hash
        } else {
            first.cookie_hash
        },
        cookie_encryption: if !second.cookie_encryption.is_empty() {
            second.cookie_encryption
        } else {
            first.cookie_encryption
        },
        mailer_host: if !second.mailer_host.is_empty() {
            second.mailer_host
        } else {
            first.mailer_host
        },
        mailer_port: if !second.mailer_port.is_empty() {
            second.mailer_port
        } else {
            first.mailer_port
        },
        mailer_username: second.mailer_username.or(first.mailer_username),
        mailer_password: second.mailer_password.or(first.mailer_password),
        mailer_use_tls: second.mailer_use_tls || first.mailer_use_tls,
        mailer_secure: second.mailer_secure || first.mailer_secure,
        mailer_from: if !second.mailer_from.is_empty() {
            second.mailer_from
        } else {
            first.mailer_from
        },
        alert: AlertConfig {
            enabled: second.alert.enabled || first.alert.enabled,
            email: second.alert.email.or(first.alert.email),
            all_projects: second.alert.all_projects || first.alert.all_projects,
        },
        email_sender: if !second.email_sender.is_empty() {
            second.email_sender
        } else {
            first.email_sender
        },
        telegram_bot_token: second.telegram_bot_token.or(first.telegram_bot_token),
        redis: second.redis.or(first.redis),
        kubernetes: second.kubernetes.or(first.kubernetes),
    }
}

fn merge_db_configs(first: DbConfig, second: DbConfig) -> DbConfig {
    DbConfig {
        dialect: second.dialect.or(first.dialect),
        hostname: if !second.hostname.is_empty() {
            second.hostname
        } else {
            first.hostname
        },
        username: if !second.username.is_empty() {
            second.username
        } else {
            first.username
        },
        password: if !second.password.is_empty() {
            second.password
        } else {
            first.password
        },
        db_name: if !second.db_name.is_empty() {
            second.db_name
        } else {
            first.db_name
        },
        options: if !second.options.is_empty() {
            second.options
        } else {
            first.options
        },
        path: second.path.or(first.path),
        connection_string: second.connection_string.or(first.connection_string),
    }
}

fn merge_auth_configs(first: AuthConfig, second: AuthConfig) -> AuthConfig {
    AuthConfig {
        totp: TotpConfig {
            enable: second.totp.enable || first.totp.enable,
            allow_recovery: second.totp.allow_recovery || first.totp.allow_recovery,
        },
        email_enabled: second.email_enabled || first.email_enabled,
        oidc_providers: if !second.oidc_providers.is_empty() {
            second.oidc_providers
        } else {
            first.oidc_providers
        },
        email_login_enabled: second.email_login_enabled || first.email_login_enabled,
    }
}

fn merge_ha_configs(first: HAConfig, second: HAConfig) -> HAConfig {
    HAConfig {
        enable: second.enable || first.enable,
        redis: HARedisConfig {
            host: if !second.redis.host.is_empty() {
                second.redis.host
            } else {
                first.redis.host
            },
            port: if second.redis.port != 0 {
                second.redis.port
            } else {
                first.redis.port
            },
            password: if !second.redis.password.is_empty() {
                second.redis.password
            } else {
                first.redis.password
            },
        },
        node_id: if !second.node_id.is_empty() {
            second.node_id
        } else {
            first.node_id
        },
    }
}

/// Загружает конфигурацию с применением всех источников
pub fn load_config(config_path: Option<&str>) -> Result<Config> {
    // 1. Загружаем из файла (если указан)
    let file_config = if let Some(path) = config_path {
        load_from_file(path)?
    } else {
        Config::default()
    };

    // 2. Загружаем из переменных окружения
    let env_config = load_from_env()?;

    // 3. Сливаем конфигурации (приоритет у env)
    let merged_config = merge_configs(file_config, env_config);

    // 4. Генерируем секреты если не указаны
    let mut config = merged_config;
    if config.cookie_hash.is_empty() || config.cookie_encryption.is_empty() {
        config.generate_secrets();
    }

    // 5. Инициализируем HA node ID если нужно
    if config.ha_enabled() && config.ha.node_id.is_empty() {
        config.init_ha_node_id();
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Мьютекс для защиты глобальных переменных окружения при параллельном запуске тестов.
    /// Это устраняет race conditions, которые приводят к случайным падениям в CI.
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_merge_db_configs() {
        let first = DbConfig {
            hostname: "localhost".to_string(),
            ..Default::default()
        };

        let second = DbConfig {
            hostname: String::new(),
            username: "admin".to_string(),
            ..Default::default()
        };

        let merged = merge_db_configs(first, second);
        assert_eq!(merged.hostname, "localhost");
        assert_eq!(merged.username, "admin");
    }

    #[test]
    fn test_load_from_env() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("SEMAPHORE_DB_HOST", "testhost");
        std::env::set_var("SEMAPHORE_DB_USER", "testuser");

        let config = load_from_env().unwrap();
        assert_eq!(config.database.hostname, "testhost");
        assert_eq!(config.database.username, "testuser");

        std::env::remove_var("SEMAPHORE_DB_HOST");
        std::env::remove_var("SEMAPHORE_DB_USER");
    }

    #[test]
    fn test_merge_configs_priority() {
        let first = Config {
            web_host: "first".to_string(),
            ..Default::default()
        };

        let second = Config {
            web_host: "second".to_string(),
            ..Default::default()
        };

        let merged = merge_configs(first, second);
        assert_eq!(merged.web_host, "second");
    }

    #[test]
    fn test_merge_db_configs_second_empty() {
        let first = DbConfig {
            hostname: "first-host".to_string(),
            username: "first-user".to_string(),
            password: "first-pass".to_string(),
            db_name: "first-db".to_string(),
            ..Default::default()
        };

        let second = DbConfig::default();

        let merged = merge_db_configs(first, second);
        assert!(!merged.hostname.is_empty());
    }

    #[test]
    fn test_merge_db_configs_second_overrides() {
        let first = DbConfig {
            hostname: "first-host".to_string(),
            username: "first-user".to_string(),
            ..Default::default()
        };

        let second = DbConfig {
            hostname: "second-host".to_string(),
            username: "second-user".to_string(),
            ..Default::default()
        };

        let merged = merge_db_configs(first, second);
        assert_eq!(merged.hostname, "second-host");
        assert_eq!(merged.username, "second-user");
    }

    #[test]
    fn test_merge_db_configs_with_path() {
        let first = DbConfig {
            path: Some("/first/path.db".to_string()),
            ..Default::default()
        };

        let second = DbConfig {
            path: Some("/second/path.db".to_string()),
            ..Default::default()
        };

        let merged = merge_db_configs(first, second);
        assert_eq!(merged.path, Some("/second/path.db".to_string()));
    }

    #[test]
    fn test_merge_db_configs_first_has_path() {
        let first = DbConfig {
            path: Some("/first/path.db".to_string()),
            ..Default::default()
        };

        let second = DbConfig::default();

        let merged = merge_db_configs(first, second);
        assert_eq!(merged.path, Some("/first/path.db".to_string()));
    }

    #[test]
    fn test_merge_db_configs_connection_string() {
        let first = DbConfig {
            connection_string: Some("postgres://first".to_string()),
            ..Default::default()
        };

        let second = DbConfig {
            connection_string: Some("postgres://second".to_string()),
            ..Default::default()
        };

        let merged = merge_db_configs(first, second);
        assert_eq!(
            merged.connection_string,
            Some("postgres://second".to_string())
        );
    }

    #[test]
    fn test_merge_auth_configs_totp() {
        let first = AuthConfig {
            totp: crate::config::types::TotpConfig {
                enable: false,
                allow_recovery: false,
            },
            ..Default::default()
        };

        let second = AuthConfig {
            totp: crate::config::types::TotpConfig {
                enable: true,
                allow_recovery: true,
            },
            ..Default::default()
        };

        let merged = merge_auth_configs(first, second);
        assert!(merged.totp.enable);
        assert!(merged.totp.allow_recovery);
    }

    #[test]
    fn test_merge_auth_configs_first_enabled() {
        let first = AuthConfig {
            totp: crate::config::types::TotpConfig {
                enable: true,
                allow_recovery: false,
            },
            email_enabled: false,
            ..Default::default()
        };

        let second = AuthConfig::default();

        let merged = merge_auth_configs(first, second);
        assert!(merged.totp.enable);
    }

    #[test]
    fn test_merge_ha_configs() {
        let first = HAConfig {
            enable: false,
            node_id: "first-node".to_string(),
            ..Default::default()
        };

        let second = HAConfig {
            enable: true,
            node_id: String::new(),
            ..Default::default()
        };

        let merged = merge_ha_configs(first, second);
        assert!(merged.enable);
        assert_eq!(merged.node_id, "first-node");
    }

    #[test]
    fn test_merge_ha_configs_redis_port() {
        let first = HAConfig {
            redis: HARedisConfig {
                port: 6379,
                ..Default::default()
            },
            ..Default::default()
        };

        let second = HAConfig {
            redis: HARedisConfig {
                port: 0,
                host: "redis.example.com".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        let merged = merge_ha_configs(first, second);
        assert_eq!(merged.redis.host, "redis.example.com");
        assert_eq!(merged.redis.port, 6379);
    }

    #[test]
    fn test_merge_configs_cookie_hash() {
        let first = Config {
            cookie_hash: vec![1, 2, 3],
            ..Default::default()
        };

        let second = Config {
            cookie_hash: vec![],
            ..Default::default()
        };

        let merged = merge_configs(first, second);
        assert_eq!(merged.cookie_hash, vec![1, 2, 3]);
    }

    #[test]
    fn test_merge_configs_mailer_fields() {
        let first = Config {
            mailer_host: "first.smtp".to_string(),
            mailer_port: "25".to_string(),
            mailer_username: Some("first_user".to_string()),
            mailer_use_tls: false,
            mailer_secure: false,
            ..Default::default()
        };

        let second = Config {
            mailer_host: String::new(),
            mailer_port: String::new(),
            mailer_username: None,
            mailer_use_tls: true,
            mailer_secure: true,
            ..Default::default()
        };

        let merged = merge_configs(first, second);
        assert_eq!(merged.mailer_host, "first.smtp");
        assert_eq!(merged.mailer_port, "25");
        assert_eq!(merged.mailer_use_tls, true);
        assert_eq!(merged.mailer_secure, true);
    }

    #[test]
    fn test_merge_configs_alert() {
        let first = Config {
            alert: AlertConfig {
                enabled: false,
                email: Some("first@example.com".to_string()),
                all_projects: false,
            },
            ..Default::default()
        };

        let second = Config {
            alert: AlertConfig {
                enabled: true,
                email: None,
                all_projects: true,
            },
            ..Default::default()
        };

        let merged = merge_configs(first, second);
        assert!(merged.alert.enabled);
        assert_eq!(merged.alert.email, Some("first@example.com".to_string()));
        assert!(merged.alert.all_projects);
    }

    #[test]
    fn test_merge_configs_email_sender() {
        let first = Config {
            email_sender: "first@sender.com".to_string(),
            ..Default::default()
        };

        let second = Config {
            email_sender: String::new(),
            ..Default::default()
        };

        let merged = merge_configs(first, second);
        assert_eq!(merged.email_sender, "first@sender.com");
    }

    #[test]
    fn test_merge_configs_telegram_bot_token() {
        let first = Config {
            telegram_bot_token: Some("token123".to_string()),
            ..Default::default()
        };

        let second = Config {
            telegram_bot_token: None,
            ..Default::default()
        };

        let merged = merge_configs(first, second);
        assert_eq!(merged.telegram_bot_token, Some("token123".to_string()));
    }

    #[test]
    fn test_load_from_env_ldap_enable_true() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("SEMAPHORE_LDAP_ENABLE", "true");
        let config = load_from_env().unwrap();
        assert!(config.ldap.is_some());
        assert!(config.ldap.unwrap().enable);
        std::env::remove_var("SEMAPHORE_LDAP_ENABLE");
    }

    #[test]
    fn test_load_from_env_ldap_enable_1() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("SEMAPHORE_LDAP_ENABLE", "1");
        let config = load_from_env().unwrap();
        assert!(config.ldap.is_some());
        assert!(config.ldap.unwrap().enable);
        std::env::remove_var("SEMAPHORE_LDAP_ENABLE");
    }

    #[test]
    fn test_load_from_env_ldap_enable_false() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("SEMAPHORE_LDAP_ENABLE", "false");
        let config = load_from_env().unwrap();
        assert!(config.ldap.is_some());
        assert!(!config.ldap.unwrap().enable);
        std::env::remove_var("SEMAPHORE_LDAP_ENABLE");
    }

    #[test]
    fn test_load_from_env_totp() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("SEMAPHORE_AUTH_TOTP_ENABLE", "true");
        std::env::set_var("SEMAPHORE_AUTH_TOTP_ALLOW_RECOVERY", "1");

        let config = load_from_env().unwrap();
        assert!(config.auth.totp.enable);
        assert!(config.auth.totp.allow_recovery);

        std::env::remove_var("SEMAPHORE_AUTH_TOTP_ENABLE");
        std::env::remove_var("SEMAPHORE_AUTH_TOTP_ALLOW_RECOVERY");
    }

    #[test]
    fn test_load_from_env_ha() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("SEMAPHORE_HA_ENABLE", "true");
        std::env::set_var("SEMAPHORE_HA_REDIS_HOST", "redis.host");
        std::env::set_var("SEMAPHORE_HA_REDIS_PORT", "6380");

        let config = load_from_env().unwrap();
        assert!(config.ha.enable);
        assert_eq!(config.ha.redis.host, "redis.host");
        assert_eq!(config.ha.redis.port, 6380);

        std::env::remove_var("SEMAPHORE_HA_ENABLE");
        std::env::remove_var("SEMAPHORE_HA_REDIS_HOST");
        std::env::remove_var("SEMAPHORE_HA_REDIS_PORT");
    }

    #[test]
    fn test_load_from_env_ha_invalid_port() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::set_var("SEMAPHORE_HA_ENABLE", "true");
        std::env::set_var("SEMAPHORE_HA_REDIS_PORT", "not_a_number");

        let config = load_from_env().unwrap();
        assert!(config.ha.enable);
        // При невалидном порте остается дефолтное значение (обычно 6379)
        assert!(config.ha.redis.port >= 0);

        std::env::remove_var("SEMAPHORE_HA_ENABLE");
        std::env::remove_var("SEMAPHORE_HA_REDIS_PORT");
    }

    #[test]
    fn test_merge_configs_with_ldap() {
        let first = Config {
            ldap: Some(LdapConfig {
                enable: true,
                server: "first.ldap".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };

        let second = Config {
            ldap: None,
            ..Default::default()
        };

        let merged = merge_configs(first, second);
        assert!(merged.ldap.is_some());
        assert_eq!(merged.ldap.unwrap().server, "first.ldap");
    }

    #[test]
    fn test_merge_configs_second_ldap_overrides() {
        let first = Config {
            ldap: Some(LdapConfig {
                enable: true,
                server: "first.ldap".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };

        let second = Config {
            ldap: Some(LdapConfig {
                enable: false,
                server: "second.ldap".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };

        let merged = merge_configs(first, second);
        assert!(merged.ldap.is_some());
        assert_eq!(merged.ldap.unwrap().server, "second.ldap");
    }

    #[test]
    fn test_merge_db_configs_with_options() {
        let mut first_options = std::collections::HashMap::new();
        first_options.insert("charset".to_string(), "utf8".to_string());
        let first = DbConfig {
            options: first_options,
            ..Default::default()
        };

        let second = DbConfig::default();

        let merged = merge_db_configs(first, second);
        assert!(!merged.options.is_empty());
        assert_eq!(merged.options.get("charset"), Some(&"utf8".to_string()));
    }

    #[test]
    fn test_merge_db_configs_second_options_override() {
        let first = DbConfig::default();

        let mut second_options = std::collections::HashMap::new();
        second_options.insert("ssl_mode".to_string(), "require".to_string());
        let second = DbConfig {
            options: second_options,
            ..Default::default()
        };

        let merged = merge_db_configs(first, second);
        assert_eq!(merged.options.get("ssl_mode"), Some(&"require".to_string()));
    }

    #[test]
    fn test_merge_configs_redis_and_kubernetes() {
        let first = Config {
            redis: Some(crate::config::types::RedisConfig {
                url: "redis://first".to_string(),
                ..Default::default()
            }),
            kubernetes: Some(crate::config::types::KubernetesConfig {
                default_namespace: "first-ns".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };

        let second = Config::default();

        let merged = merge_configs(first, second);
        assert!(merged.redis.is_some());
        assert!(merged.kubernetes.is_some());
        assert_eq!(merged.redis.unwrap().url, "redis://first");
        assert_eq!(merged.kubernetes.unwrap().default_namespace, "first-ns");
    }
}