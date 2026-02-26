//! Config LDAP - LDAP конфигурация
//!
//! Аналог util/config.go из Go версии (часть 5: LDAP)

use serde::{Deserialize, Serialize};
use crate::config::types::LdapMappings;

/// LDAP конфигурация
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for LdapConfigFull {
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
        env::set_var("SEMAPHORE_LDAP_ENABLE", "true");
        env::set_var("SEMAPHORE_LDAP_SERVER", "test.server");
        
        let config = load_ldap_from_env();
        assert!(config.enable);
        assert_eq!(config.server, "test.server");
        
        env::remove_var("SEMAPHORE_LDAP_ENABLE");
        env::remove_var("SEMAPHORE_LDAP_SERVER");
    }
}
