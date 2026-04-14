//! Config LDAP - LDAP конфигурация
//!
//! Аналог util/config.go из Go версии (часть 5: LDAP)

use crate::config::types::LdapMappings;
use serde::{Deserialize, Serialize};

/// LDAP конфигурация
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LdapConfigFull {
    /// Включить LDAP аутентификацию
    #[serde(default)]
    pub enable: bool,

    /// LDAP сервер
    #[serde(default)]
    pub server: String,

    /// Bind DN
    #[serde(default)]
    pub bind_dn: String,

    /// Bind пароль
    #[serde(default)]
    pub bind_password: String,

    /// Search DN
    #[serde(default)]
    pub search_dn: String,

    /// Search фильтр
    #[serde(default)]
    pub search_filter: String,

    /// Требуется TLS
    #[serde(default)]
    pub need_tls: bool,

    /// LDAP маппинги
    #[serde(default)]
    pub mappings: LdapMappings,
}

impl LdapConfigFull {
    /// Создаёт новую LDAP конфигурацию
    pub fn new() -> Self {
        Self::default()
    }

    /// Проверяет включён ли LDAP
    pub fn is_enabled(&self) -> bool {
        self.enable
    }

    /// Создаёт LDAP URL
    pub fn ldap_url(&self) -> String {
        if self.need_tls {
            format!("ldaps://{}", self.server)
        } else {
            format!("ldap://{}", self.server)
        }
    }

    /// Создаёт полный search DN
    pub fn full_search_dn(&self) -> String {
        if self.search_dn.is_empty() {
            self.bind_dn.clone()
        } else {
            self.search_dn.clone()
        }
    }
}

/// Загружает LDAP конфигурацию из переменных окружения
pub fn load_ldap_from_env() -> LdapConfigFull {
    use std::env;

    let mut config = LdapConfigFull::new();

    if let Ok(val) = env::var("SEMAPHORE_LDAP_ENABLE") {
        config.enable = val.to_lowercase() == "true" || val == "1";
    }

    if let Ok(server) = env::var("SEMAPHORE_LDAP_SERVER") {
        config.server = server;
    }

    if let Ok(bind_dn) = env::var("SEMAPHORE_LDAP_BIND_DN") {
        config.bind_dn = bind_dn;
    }

    if let Ok(bind_password) = env::var("SEMAPHORE_LDAP_BIND_PASSWORD") {
        config.bind_password = bind_password;
    }

    if let Ok(search_dn) = env::var("SEMAPHORE_LDAP_SEARCH_DN") {
        config.search_dn = search_dn;
    }

    if let Ok(search_filter) = env::var("SEMAPHORE_LDAP_SEARCH_FILTER") {
        config.search_filter = search_filter;
    }

    if let Ok(val) = env::var("SEMAPHORE_LDAP_NEEDTLS") {
        config.need_tls = val.to_lowercase() == "true" || val == "1";
    }

    // LDAP mappings
    if let Ok(dn) = env::var("SEMAPHORE_LDAP_MAPPING_DN") {
        config.mappings.dn = dn;
    }

    if let Ok(mail) = env::var("SEMAPHORE_LDAP_MAPPING_MAIL") {
        config.mappings.mail = mail;
    }

    if let Ok(uid) = env::var("SEMAPHORE_LDAP_MAPPING_UID") {
        config.mappings.uid = uid;
    }

    if let Ok(cn) = env::var("SEMAPHORE_LDAP_MAPPING_CN") {
        config.mappings.cn = cn;
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_ldap_config_default() {
        let config = LdapConfigFull::default();
        assert!(!config.enable);
        assert!(!config.need_tls);
        assert_eq!(config.server, "");
    }

    #[test]
    fn test_ldap_config_new() {
        let config = LdapConfigFull::new();
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_ldap_url_without_tls() {
        let config = LdapConfigFull {
            server: "ldap.example.com".to_string(),
            need_tls: false,
            ..Default::default()
        };

        assert_eq!(config.ldap_url(), "ldap://ldap.example.com");
    }

    #[test]
    fn test_ldap_url_with_tls() {
        let config = LdapConfigFull {
            server: "ldap.example.com".to_string(),
            need_tls: true,
            ..Default::default()
        };

        assert_eq!(config.ldap_url(), "ldaps://ldap.example.com");
    }

    #[test]
    fn test_full_search_dn_with_search_dn() {
        let config = LdapConfigFull {
            bind_dn: "cn=admin,dc=example,dc=com".to_string(),
            search_dn: "ou=users,dc=example,dc=com".to_string(),
            ..Default::default()
        };

        assert_eq!(config.full_search_dn(), "ou=users,dc=example,dc=com");
    }

    #[test]
    fn test_full_search_dn_without_search_dn() {
        let config = LdapConfigFull {
            bind_dn: "cn=admin,dc=example,dc=com".to_string(),
            search_dn: String::new(),
            ..Default::default()
        };

        assert_eq!(config.full_search_dn(), "cn=admin,dc=example,dc=com");
    }

    #[test]
    fn test_load_ldap_from_env() {
        unsafe {
            std::env::set_var("SEMAPHORE_LDAP_ENABLE", "true");
            std::env::set_var("SEMAPHORE_LDAP_SERVER", "test.server");
        }

        let config = load_ldap_from_env();
        assert!(config.enable);
        assert_eq!(config.server, "test.server");

        unsafe {
            std::env::remove_var("SEMAPHORE_LDAP_ENABLE");
            std::env::remove_var("SEMAPHORE_LDAP_SERVER");
        }
    }

    #[test]
    fn test_ldap_config_serialization() {
        let config = LdapConfigFull {
            enable: true,
            server: "ldap.example.com".to_string(),
            bind_dn: "cn=admin,dc=example,dc=com".to_string(),
            bind_password: "secret".to_string(),
            search_dn: "ou=users,dc=example,dc=com".to_string(),
            search_filter: "(uid={login})".to_string(),
            need_tls: true,
            mappings: LdapMappings::default(),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"enable\":true"));
        assert!(json.contains("\"server\":\"ldap.example.com\""));
        assert!(json.contains("\"need_tls\":true"));
    }

    #[test]
    fn test_ldap_config_deserialization() {
        let json = r#"{
            "enable": true,
            "server": "ldap.test.com",
            "bind_dn": "cn=bind",
            "bind_password": "pass",
            "search_dn": "ou=people",
            "search_filter": "(cn={login})",
            "need_tls": false,
            "mappings": {"dn": "dn", "mail": "mail", "uid": "uid", "cn": "cn"}
        }"#;

        let config: LdapConfigFull = serde_json::from_str(json).unwrap();
        assert!(config.enable);
        assert_eq!(config.server, "ldap.test.com");
        assert_eq!(config.search_filter, "(cn={login})");
    }

    #[test]
    fn test_ldap_config_clone() {
        let config = LdapConfigFull {
            enable: true,
            server: "ldap.clone.test".to_string(),
            ..Default::default()
        };
        let cloned = config.clone();
        assert_eq!(cloned.server, "ldap.clone.test");
        assert!(cloned.enable);
    }

    #[test]
    fn test_ldap_config_empty_server_url() {
        let config = LdapConfigFull {
            server: String::new(),
            need_tls: false,
            ..Default::default()
        };
        assert_eq!(config.ldap_url(), "ldap://");
    }

    #[test]
    fn test_ldap_config_empty_server_url_tls() {
        let config = LdapConfigFull {
            server: String::new(),
            need_tls: true,
            ..Default::default()
        };
        assert_eq!(config.ldap_url(), "ldaps://");
    }

    #[test]
    fn test_load_ldap_from_env_all_fields() {
        unsafe {
            std::env::set_var("SEMAPHORE_LDAP_ENABLE", "1");
            std::env::set_var("SEMAPHORE_LDAP_SERVER", "full.ldap.server");
            std::env::set_var("SEMAPHORE_LDAP_BIND_DN", "cn=bind,dc=test");
            std::env::set_var("SEMAPHORE_LDAP_BIND_PASSWORD", "bindpass");
            std::env::set_var("SEMAPHORE_LDAP_SEARCH_DN", "ou=search,dc=test");
            std::env::set_var(
                "SEMAPHORE_LDAP_SEARCH_FILTER",
                "(&(uid={login})(active=TRUE))",
            );
            std::env::set_var("SEMAPHORE_LDAP_NEEDTLS", "true");
        }

        let config = load_ldap_from_env();
        assert!(config.enable);
        assert_eq!(config.server, "full.ldap.server");
        assert_eq!(config.bind_dn, "cn=bind,dc=test");
        assert_eq!(config.bind_password, "bindpass");
        assert_eq!(config.search_dn, "ou=search,dc=test");
        assert_eq!(config.search_filter, "(&(uid={login})(active=TRUE))");
        assert!(config.need_tls);

        unsafe {
            std::env::remove_var("SEMAPHORE_LDAP_ENABLE");
            std::env::remove_var("SEMAPHORE_LDAP_SERVER");
            std::env::remove_var("SEMAPHORE_LDAP_BIND_DN");
            std::env::remove_var("SEMAPHORE_LDAP_BIND_PASSWORD");
            std::env::remove_var("SEMAPHORE_LDAP_SEARCH_DN");
            std::env::remove_var("SEMAPHORE_LDAP_SEARCH_FILTER");
            std::env::remove_var("SEMAPHORE_LDAP_NEEDTLS");
        }
    }

    #[test]
    fn test_load_ldap_from_env_mappings() {
        unsafe {
            std::env::set_var("SEMAPHORE_LDAP_MAPPING_DN", "distinguishedName");
            std::env::set_var("SEMAPHORE_LDAP_MAPPING_MAIL", "email");
            std::env::set_var("SEMAPHORE_LDAP_MAPPING_UID", "sAMAccountName");
            std::env::set_var("SEMAPHORE_LDAP_MAPPING_CN", "displayName");
        }

        let config = load_ldap_from_env();
        assert_eq!(config.mappings.dn, "distinguishedName");
        assert_eq!(config.mappings.mail, "email");
        assert_eq!(config.mappings.uid, "sAMAccountName");
        assert_eq!(config.mappings.cn, "displayName");

        unsafe {
            std::env::remove_var("SEMAPHORE_LDAP_MAPPING_DN");
            std::env::remove_var("SEMAPHORE_LDAP_MAPPING_MAIL");
            std::env::remove_var("SEMAPHORE_LDAP_MAPPING_UID");
            std::env::remove_var("SEMAPHORE_LDAP_MAPPING_CN");
        }
    }

    #[test]
    fn test_ldap_config_unicode_values() {
        let config = LdapConfigFull {
            enable: true,
            server: "ldap.пример.ru".to_string(),
            bind_dn: "cn=администратор,dc=пример,dc=ru".to_string(),
            bind_password: "пароль".to_string(),
            search_dn: "ou=пользователи,dc=пример,dc=ru".to_string(),
            search_filter: "(uid={логин})".to_string(),
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("администратор"));
        assert!(json.contains("пользователи"));

        let deserialized: LdapConfigFull = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.bind_dn, "cn=администратор,dc=пример,dc=ru");
    }

    #[test]
    fn test_ldap_config_is_enabled_true() {
        let config = LdapConfigFull {
            enable: true,
            ..Default::default()
        };
        assert!(config.is_enabled());
    }

    #[test]
    fn test_ldap_config_is_enabled_false() {
        let config = LdapConfigFull {
            enable: false,
            ..Default::default()
        };
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_load_ldap_from_env_enable_false() {
        unsafe { std::env::set_var("SEMAPHORE_LDAP_ENABLE", "false") };
        let config = load_ldap_from_env();
        assert!(!config.enable);
        unsafe { std::env::remove_var("SEMAPHORE_LDAP_ENABLE") };
    }

    #[test]
    fn test_load_ldap_from_env_enable_invalid() {
        unsafe { std::env::set_var("SEMAPHORE_LDAP_ENABLE", "invalid") };
        let config = load_ldap_from_env();
        assert!(!config.enable);
        unsafe { std::env::remove_var("SEMAPHORE_LDAP_ENABLE") };
    }

    #[test]
    fn test_ldap_config_search_filter_special_chars() {
        let config = LdapConfigFull {
            search_filter: "(&(objectClass=inetOrgPerson)(uid={login})(!(userAccountControl:1.2.840.113556.1.4.803:=2)))".to_string(),
            ..Default::default()
        };
        assert!(config.search_filter.contains("objectClass"));
        assert!(config.search_filter.contains("userAccountControl"));
    }

    #[test]
    fn test_ldap_config_serialization_defaults() {
        let config = LdapConfigFull::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"enable\":false"));
        assert!(json.contains("\"need_tls\":false"));
    }
}
