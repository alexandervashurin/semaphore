//! Config Types - структуры конфигурации
//!
//! Аналог util/config.go из Go версии (часть 1: типы)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// Типы диалектов БД
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DbDialect {
    MySQL,
    Bolt,      // Deprecated
    Postgres,
    SQLite,
}

/// Конфигурация БД
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DbConfig {
    #[serde(skip)]
    pub dialect: Option<DbDialect>,
    
    #[serde(rename = "host", default = "default_db_host")]
    pub hostname: String,
    
    #[serde(rename = "user", default)]
    pub username: String,
    
    #[serde(rename = "pass", default)]
    pub password: String,
    
    #[serde(rename = "name", default = "default_db_name")]
    pub db_name: String,
    
    #[serde(default)]
    pub options: HashMap<String, String>,
}

fn default_db_host() -> String {
    "0.0.0.0".to_string()
}

fn default_db_name() -> String {
    "semaphore".to_string()
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            dialect: None,
            hostname: default_db_host(),
            username: String::new(),
            password: String::new(),
            db_name: default_db_name(),
            options: HashMap::new(),
        }
    }
}

impl DbConfig {
    /// Проверяет присутствует ли конфигурация БД
    pub fn is_present(&self) -> bool {
        !self.hostname.is_empty() || !self.db_name.is_empty()
    }

    /// Поддержка множественных БД
    pub fn has_support_multiple_databases(&self) -> bool {
        matches!(self.dialect, Some(DbDialect::MySQL | DbDialect::Postgres))
    }

    /// Получает имя БД
    pub fn get_db_name(&self) -> &str {
        &self.db_name
    }

    /// Получает имя пользователя
    pub fn get_username(&self) -> &str {
        &self.username
    }

    /// Получает пароль
    pub fn get_password(&self) -> &str {
        &self.password
    }

    /// Получает хост
    pub fn get_hostname(&self) -> &str {
        &self.hostname
    }

    /// Получает строку подключения
    pub fn get_connection_string(&self, include_db_name: bool) -> Result<String, String> {
        match self.dialect {
            Some(DbDialect::MySQL) => {
                let mut conn = format!(
                    "{}:{}@tcp({})/",
                    self.username, self.password, self.hostname
                );
                if include_db_name {
                    conn.push_str(&self.db_name);
                }
                if !self.options.is_empty() {
                    conn.push('?');
                    let options: Vec<String> = self
                        .options
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect();
                    conn.push_str(&options.join("&"));
                }
                Ok(conn)
            }
            Some(DbDialect::Postgres) => {
                let mut conn = format!(
                    "postgres://{}:{}@{}",
                    self.username, self.password, self.hostname
                );
                if include_db_name {
                    conn.push('/');
                    conn.push_str(&self.db_name);
                }
                Ok(conn)
            }
            Some(DbDialect::SQLite) => {
                Ok(self.db_name.clone())
            }
            _ => Err("Unknown database dialect".to_string()),
        }
    }
}

/// Маппинги LDAP
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LdapMappings {
    #[serde(default = "default_ldap_dn")]
    pub dn: String,
    
    #[serde(default = "default_ldap_mail")]
    pub mail: String,
    
    #[serde(default = "default_ldap_uid")]
    pub uid: String,
    
    #[serde(default = "default_ldap_cn")]
    pub cn: String,
}

fn default_ldap_dn() -> String {
    "dn".to_string()
}

fn default_ldap_mail() -> String {
    "mail".to_string()
}

fn default_ldap_uid() -> String {
    "uid".to_string()
}

fn default_ldap_cn() -> String {
    "cn".to_string()
}

impl Default for LdapMappings {
    fn default() -> Self {
        Self {
            dn: default_ldap_dn(),
            mail: default_ldap_mail(),
            uid: default_ldap_uid(),
            cn: default_ldap_cn(),
        }
    }
}

impl LdapMappings {
    pub fn get_username_claim(&self) -> &str {
        &self.uid
    }

    pub fn get_email_claim(&self) -> &str {
        &self.mail
    }

    pub fn get_name_claim(&self) -> &str {
        &self.cn
    }
}

/// Конфигурация LDAP
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LdapConfig {
    #[serde(default)]
    pub enable: bool,
    
    #[serde(default)]
    pub server: String,
    
    #[serde(default)]
    pub bind_dn: String,
    
    #[serde(default)]
    pub bind_password: String,
    
    #[serde(default)]
    pub search_dn: String,
    
    #[serde(default)]
    pub search_filter: String,
    
    #[serde(default)]
    pub need_tls: bool,
    
    #[serde(default)]
    pub mappings: LdapMappings,
}

impl Default for LdapConfig {
    fn default() -> Self {
        Self {
            enable: false,
            server: String::new(),
            bind_dn: String::new(),
            bind_password: String::new(),
            search_dn: String::new(),
            search_filter: String::new(),
            need_tls: false,
            mappings: LdapMappings::default(),
        }
    }
}

/// Конфигурация TOTP
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TotpConfig {
    #[serde(default)]
    pub enable: bool,
    
    #[serde(default)]
    pub allow_recovery: bool,
}

impl Default for TotpConfig {
    fn default() -> Self {
        Self {
            enable: false,
            allow_recovery: false,
        }
    }
}

/// Конфигурация аутентификации
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AuthConfig {
    #[serde(default)]
    pub totp: TotpConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            totp: TotpConfig::default(),
        }
    }
}

/// Конфигурация HA (High Availability)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct HAConfig {
    #[serde(default)]
    pub enable: bool,
    
    #[serde(default)]
    pub redis: HARedisConfig,
    
    #[serde(skip)]
    pub node_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct HARedisConfig {
    #[serde(default)]
    pub host: String,
    
    #[serde(default)]
    pub port: u16,
    
    #[serde(default)]
    pub password: String,
}

impl Default for HAConfig {
    fn default() -> Self {
        Self {
            enable: false,
            redis: HARedisConfig::default(),
            node_id: String::new(),
        }
    }
}

/// Основная структура конфигурации
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Config {
    #[serde(rename = "webHost", default)]
    pub web_host: String,
    
    #[serde(rename = "tcpAddress", default = "default_tcp_address")]
    pub tcp_address: String,
    
    #[serde(rename = "db", default)]
    #[validate(nested)]
    pub database: DbConfig,
    
    #[serde(rename = "ldap", default)]
    #[validate(nested)]
    pub ldap: Option<LdapConfig>,
    
    #[serde(rename = "auth", default)]
    #[validate(nested)]
    pub auth: AuthConfig,
    
    #[serde(rename = "ha", default)]
    #[validate(nested)]
    pub ha: HAConfig,
    
    #[serde(rename = "tmpPath", default = "default_tmp_path")]
    pub tmp_path: String,
    
    #[serde(skip)]
    pub cookie_hash: Vec<u8>,
    
    #[serde(skip)]
    pub cookie_encryption: Vec<u8>,
}

fn default_tcp_address() -> String {
    "0.0.0.0:3000".to_string()
}

fn default_tmp_path() -> String {
    "/tmp/semaphore".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            web_host: String::new(),
            tcp_address: default_tcp_address(),
            database: DbConfig::default(),
            ldap: None,
            auth: AuthConfig::default(),
            ha: HAConfig::default(),
            tmp_path: default_tmp_path(),
            cookie_hash: Vec::new(),
            cookie_encryption: Vec::new(),
        }
    }
}

impl Config {
    /// Генерирует секреты для cookie
    pub fn generate_secrets(&mut self) {
        use rand::RngCore;
        
        let mut rng = rand::thread_rng();
        
        self.cookie_hash = vec![0u8; 32];
        rng.fill_bytes(&mut self.cookie_hash);
        
        self.cookie_encryption = vec![0u8; 32];
        rng.fill_bytes(&mut self.cookie_encryption);
    }

    /// Получает директорию проекта
    pub fn get_project_tmp_dir(&self, project_id: i32) -> String {
        format!("{}/project_{}", self.tmp_path, project_id)
    }

    /// Проверяет включён ли HA режим
    pub fn ha_enabled(&self) -> bool {
        self.ha.enable
    }

    /// Инициализирует ID узла HA
    pub fn init_ha_node_id(&mut self) {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 16];
        rng.fill_bytes(&mut bytes);
        self.ha.node_id = format!("{:x}", bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_config_default() {
        let config = DbConfig::default();
        assert_eq!(config.hostname, "0.0.0.0");
        assert_eq!(config.db_name, "semaphore");
    }

    #[test]
    fn test_db_config_is_present() {
        let config = DbConfig::default();
        assert!(config.is_present());
    }

    #[test]
    fn test_ldap_mappings_default() {
        let mappings = LdapMappings::default();
        assert_eq!(mappings.dn, "dn");
        assert_eq!(mappings.mail, "mail");
        assert_eq!(mappings.uid, "uid");
        assert_eq!(mappings.cn, "cn");
    }

    #[test]
    fn test_ldap_mappings_getters() {
        let mappings = LdapMappings::default();
        assert_eq!(mappings.get_username_claim(), "uid");
        assert_eq!(mappings.get_email_claim(), "mail");
        assert_eq!(mappings.get_name_claim(), "cn");
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.tcp_address, "0.0.0.0:3000");
        assert_eq!(config.tmp_path, "/tmp/semaphore");
    }

    #[test]
    fn test_config_generate_secrets() {
        let mut config = Config::default();
        config.generate_secrets();
        assert_eq!(config.cookie_hash.len(), 32);
        assert_eq!(config.cookie_encryption.len(), 32);
    }

    #[test]
    fn test_config_get_project_tmp_dir() {
        let config = Config::default();
        let dir = config.get_project_tmp_dir(123);
        assert_eq!(dir, "/tmp/semaphore/project_123");
    }
}
