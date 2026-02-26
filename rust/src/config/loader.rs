//! Config Loader - загрузка конфигурации
//!
//! Аналог util/config.go из Go версии (часть 2: загрузка)

use std::fs;
use std::collections::HashMap;
use std::env;
use serde::{Deserialize, Serialize};
use crate::config::types::{Config, DbConfig, LdapConfig, AuthConfig, TotpConfig, HAConfig, HARedisConfig};
use crate::error::{Error, Result};

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
        web_host: if !second.web_host.is_empty() { second.web_host } else { first.web_host },
        tcp_address: if !second.tcp_address.is_empty() { second.tcp_address } else { first.tcp_address },
        database: merge_db_configs(first.database, second.database),
        ldap: second.ldap.or(first.ldap),
        auth: merge_auth_configs(first.auth, second.auth),
        ha: merge_ha_configs(first.ha, second.ha),
        tmp_path: if !second.tmp_path.is_empty() { second.tmp_path } else { first.tmp_path },
        cookie_hash: if !second.cookie_hash.is_empty() { second.cookie_hash } else { first.cookie_hash },
        cookie_encryption: if !second.cookie_encryption.is_empty() { second.cookie_encryption } else { first.cookie_encryption },
    }
}

fn merge_db_configs(first: DbConfig, second: DbConfig) -> DbConfig {
    DbConfig {
        dialect: second.dialect.or(first.dialect),
        hostname: if !second.hostname.is_empty() { second.hostname } else { first.hostname },
        username: if !second.username.is_empty() { second.username } else { first.username },
        password: if !second.password.is_empty() { second.password } else { first.password },
        db_name: if !second.db_name.is_empty() { second.db_name } else { first.db_name },
        options: if !second.options.is_empty() { second.options } else { first.options },
    }
}

fn merge_auth_configs(first: AuthConfig, second: AuthConfig) -> AuthConfig {
    AuthConfig {
        totp: TotpConfig {
            enable: second.totp.enable || first.totp.enable,
            allow_recovery: second.totp.allow_recovery || first.totp.allow_recovery,
        },
    }
}

fn merge_ha_configs(first: HAConfig, second: HAConfig) -> HAConfig {
    HAConfig {
        enable: second.enable || first.enable,
        redis: HARedisConfig {
            host: if !second.redis.host.is_empty() { second.redis.host } else { first.redis.host },
            port: if second.redis.port != 0 { second.redis.port } else { first.redis.port },
            password: if !second.redis.password.is_empty() { second.redis.password } else { first.redis.password },
        },
        node_id: if !second.ha.node_id.is_empty() { second.ha.node_id } else { first.ha.node_id },
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
    use std::env;

    #[test]
    fn test_merge_db_configs() {
        let first = DbConfig {
            hostname: "localhost".to_string(),
            ..Default::default()
        };
        
        let second = DbConfig {
            username: "admin".to_string(),
            ..Default::default()
        };
        
        let merged = merge_db_configs(first, second);
        assert_eq!(merged.hostname, "localhost");
        assert_eq!(merged.username, "admin");
    }

    #[test]
    fn test_load_from_env() {
        env::set_var("SEMAPHORE_DB_HOST", "testhost");
        env::set_var("SEMAPHORE_DB_USER", "testuser");
        
        let config = load_from_env().unwrap();
        assert_eq!(config.database.hostname, "testhost");
        assert_eq!(config.database.username, "testuser");
        
        env::remove_var("SEMAPHORE_DB_HOST");
        env::remove_var("SEMAPHORE_DB_USER");
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
}
