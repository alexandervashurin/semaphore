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

    /// Путь к файлу БД (для SQLite)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Строка подключения (для PostgreSQL/MySQL)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_string: Option<String>,
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
            path: None,
            connection_string: None,
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

    #[serde(default)]
    pub oidc_providers: Vec<crate::config::config_oidc::OidcProvider>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            totp: TotpConfig::default(),
            oidc_providers: Vec::new(),
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

impl Default for HARedisConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 6379,
            password: String::new(),
        }
    }
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

    // Mailer configuration
    #[serde(rename = "mailerHost", default)]
    pub mailer_host: String,

    #[serde(rename = "mailerPort", default = "default_mailer_port")]
    pub mailer_port: String,

    #[serde(rename = "mailerUsername", default)]
    pub mailer_username: Option<String>,

    #[serde(rename = "mailerPassword", default)]
    pub mailer_password: Option<String>,

    #[serde(rename = "mailerUseTls", default)]
    pub mailer_use_tls: bool,

    #[serde(rename = "mailerSecure", default)]
    pub mailer_secure: bool,

    #[serde(rename = "mailerFrom", default = "default_mailer_from")]
    pub mailer_from: String,

    /// Конфигурация алертов
    #[serde(rename = "alert", default)]
    pub alert: AlertConfig,

    /// Отправитель email по умолчанию
    #[serde(rename = "emailSender", default = "default_email_sender")]
    pub email_sender: String,

    /// Telegram Bot Token
    #[serde(rename = "telegramBotToken", default)]
    pub telegram_bot_token: Option<String>,

    /// Redis конфигурация для кэширования
    #[serde(rename = "redis", default)]
    pub redis: Option<RedisConfig>,
}

/// Redis конфигурация
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RedisConfig {
    /// URL подключения к Redis
    #[serde(default = "default_redis_url")]
    pub url: String,
    /// Префикс для ключей
    #[serde(default = "default_redis_prefix")]
    pub prefix: String,
    /// TTL по умолчанию (секунды)
    #[serde(default = "default_redis_ttl")]
    pub default_ttl: u64,
    /// Включить кэширование
    #[serde(default = "default_redis_enabled")]
    pub enabled: bool,
}

fn default_redis_url() -> String {
    "redis://localhost:6379".to_string()
}

fn default_redis_prefix() -> String {
    "semaphore:".to_string()
}

fn default_redis_ttl() -> u64 {
    300
}

fn default_redis_enabled() -> bool {
    false
}

/// Конфигурация алертов
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlertConfig {
    /// Включены ли алерты
    #[serde(default = "default_alert_enabled")]
    pub enabled: bool,

    /// Email для алертов по умолчанию
    #[serde(default)]
    pub email: Option<String>,

    /// Включить алерты для всех проектов
    #[serde(default)]
    pub all_projects: bool,
}

fn default_alert_enabled() -> bool {
    false
}

fn default_email_sender() -> String {
    "semaphore@localhost".to_string()
}

fn default_mailer_port() -> String {
    "25".to_string()
}

fn default_mailer_from() -> String {
    "noreply@localhost".to_string()
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
            mailer_host: String::new(),
            mailer_port: default_mailer_port(),
            mailer_username: None,
            mailer_password: None,
            mailer_use_tls: false,
            mailer_secure: false,
            mailer_from: default_mailer_from(),
            alert: AlertConfig {
                enabled: false,
                email: None,
                all_projects: false,
            },
            email_sender: default_email_sender(),
            telegram_bot_token: None,
            redis: None,
        }
    }
}

impl Config {
    /// Загружает конфигурацию из переменных окружения
    pub fn from_env() -> Result<Self, crate::error::Error> {
        use std::env;
        
        let dialect_str = env::var("SEMAPHORE_DB_DIALECT")
            .unwrap_or_else(|_| "sqlite".to_string());
        
        let dialect = match dialect_str.as_str() {
            "postgres" | "postgresql" => DbDialect::Postgres,
            "mysql" => DbDialect::MySQL,
            "sqlite" => DbDialect::SQLite,
            _ => DbDialect::SQLite,
        };

        let mut config = Self::default();
        config.database.dialect = Some(dialect);
        
        // Загрузка пути к БД для SQLite
        if let Ok(db_path) = env::var("SEMAPHORE_DB_PATH") {
            config.database.path = Some(db_path);
        }
        
        // Загрузка URL для PostgreSQL/MySQL
        if let Ok(db_url) = env::var("SEMAPHORE_DB_URL") {
            config.database.connection_string = Some(db_url);
        }

        Ok(config)
    }

    /// Получает URL базы данных
    pub fn database_url(&self) -> Result<String, crate::error::Error> {
        if let Some(ref url) = self.database.connection_string {
            Ok(url.clone())
        } else if let Some(ref path) = self.database.path {
            Ok(path.clone())
        } else if self.db_dialect() == DbDialect::SQLite {
            // Значение по умолчанию: data/semaphore.db (абсолютный путь от cwd)
            let default = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join("data")
                .join("semaphore.db");
            Ok(default.to_string_lossy().to_string())
        } else {
            Err(crate::error::Error::Other("Database URL not configured. Set SEMAPHORE_DB_URL for PostgreSQL/MySQL or SEMAPHORE_DB_PATH for SQLite.".to_string()))
        }
    }

    /// Получает путь к базе данных
    pub fn db_path(&self) -> Option<String> {
        self.database.path.clone()
    }

    /// Получает диалект базы данных
    pub fn db_dialect(&self) -> DbDialect {
        self.database.dialect.clone().unwrap_or(DbDialect::SQLite)
    }

    /// Проверяет может ли пользователь создавать проекты
    pub fn non_admin_can_create_project(&self) -> bool {
        self.database.dialect.clone().unwrap_or(DbDialect::SQLite) == DbDialect::SQLite
    }

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
        self.ha.node_id = bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
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
