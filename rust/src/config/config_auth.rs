//! Configuration Types
//!
//! Типы конфигурации приложения

use crate::config::config_oidc::OidcProvider;
use serde::{Deserialize, Serialize};

/// Конфигурация reCAPTCHA
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecaptchaConfig {
    #[serde(default)]
    pub enabled: String,

    #[serde(default)]
    pub site_key: String,
}

/// Конфигурация Email аутентификации
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmailAuthConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub allow_login_as_external_user: bool,

    #[serde(default)]
    pub allow_create_external_users: bool,

    #[serde(default)]
    pub allowed_domains: Vec<String>,

    #[serde(default)]
    pub disable_for_oidc: bool,
}

/// Конфигурация аутентификации
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default)]
    pub totp: Option<super::types::TotpConfig>,

    #[serde(default)]
    pub email: Option<EmailAuthConfig>,

    #[serde(default)]
    pub oidc_providers: Vec<OidcProvider>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recaptcha_config_default() {
        let config = RecaptchaConfig::default();
        assert!(config.enabled.is_empty());
        assert!(config.site_key.is_empty());
    }

    #[test]
    fn test_email_auth_config_default() {
        let config = EmailAuthConfig::default();
        assert!(!config.enabled);
        assert!(!config.allow_login_as_external_user);
        assert!(config.allowed_domains.is_empty());
    }

    #[test]
    fn test_email_auth_config_serialization() {
        let config = EmailAuthConfig {
            enabled: true,
            allowed_domains: vec!["example.com".to_string()],
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("true"));
        assert!(json.contains("example.com"));
    }

    #[test]
    fn test_auth_config_default() {
        let config = AuthConfig::default();
        assert!(config.totp.is_none());
        assert!(config.email.is_none());
    }

    #[test]
    fn test_auth_config_with_email() {
        let config = AuthConfig {
            email: Some(EmailAuthConfig {
                enabled: true,
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(config.email.is_some());
        assert!(config.email.as_ref().unwrap().enabled);
    }

    #[test]
    fn test_recaptcha_config_serialization() {
        let config = RecaptchaConfig {
            enabled: "true".to_string(),
            site_key: "6LeIxAcTAAAAAJcZVRqyHh71UMIEGNQ_MXjiZKhI".to_string(),
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"enabled\":\"true\""));
        assert!(json.contains("\"site_key\":\"6LeIxAcTAAAAAJcZVRqyHh71UMIEGNQ_MXjiZKhI\""));
    }

    #[test]
    fn test_recaptcha_config_deserialization() {
        let json = r#"{"enabled": "true", "site_key": "test_key_123"}"#;
        let config: RecaptchaConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.enabled, "true");
        assert_eq!(config.site_key, "test_key_123");
    }

    #[test]
    fn test_email_auth_config_full_serialization() {
        let config = EmailAuthConfig {
            enabled: true,
            allow_login_as_external_user: true,
            allow_create_external_users: true,
            allowed_domains: vec!["example.com".to_string(), "test.org".to_string()],
            disable_for_oidc: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"allow_login_as_external_user\":true"));
        assert!(json.contains("\"allow_create_external_users\":true"));
        assert!(json.contains("\"disable_for_oidc\":true"));
        assert!(json.contains("example.com"));
        assert!(json.contains("test.org"));
    }

    #[test]
    fn test_email_auth_config_deserialization() {
        let json = r#"{
            "enabled": true,
            "allow_login_as_external_user": false,
            "allow_create_external_users": true,
            "allowed_domains": ["domain1.com", "domain2.org"],
            "disable_for_oidc": false
        }"#;
        let config: EmailAuthConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert!(!config.allow_login_as_external_user);
        assert!(config.allow_create_external_users);
        assert_eq!(config.allowed_domains.len(), 2);
        assert!(!config.disable_for_oidc);
    }

    #[test]
    fn test_email_auth_config_empty_domains() {
        let config = EmailAuthConfig {
            allowed_domains: vec![],
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"allowed_domains\":[]"));
    }

    #[test]
    fn test_auth_config_with_oidc_providers() {
        use crate::config::config_oidc::OidcProvider;
        let config = AuthConfig {
            oidc_providers: vec![OidcProvider {
                display_name: "Google".to_string(),
                client_id: "google_id".to_string(),
                client_secret: "google_secret".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert_eq!(config.oidc_providers.len(), 1);
        assert_eq!(config.oidc_providers[0].display_name, "Google");
    }

    #[test]
    fn test_auth_config_clone() {
        let config = AuthConfig {
            email: Some(EmailAuthConfig {
                enabled: true,
                ..Default::default()
            }),
            ..Default::default()
        };
        let cloned = config.clone();
        assert!(cloned.email.is_some());
        assert!(cloned.email.unwrap().enabled);
    }

    #[test]
    fn test_email_auth_config_unicode_domains() {
        let config = EmailAuthConfig {
            allowed_domains: vec!["пример.рф".to_string(), "münchen.de".to_string()],
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("пример.рф"));
        assert!(json.contains("münchen"));

        let deserialized: EmailAuthConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.allowed_domains[0], "пример.рф");
    }

    #[test]
    fn test_recaptcha_config_clone() {
        let config = RecaptchaConfig {
            enabled: "1".to_string(),
            site_key: "site_key".to_string(),
        };
        let cloned = config.clone();
        assert_eq!(cloned.enabled, "1");
        assert_eq!(cloned.site_key, "site_key");
    }

    #[test]
    fn test_email_auth_config_single_domain() {
        let config = EmailAuthConfig {
            enabled: true,
            allowed_domains: vec!["single.com".to_string()],
            ..Default::default()
        };
        assert_eq!(config.allowed_domains.len(), 1);
        assert_eq!(config.allowed_domains[0], "single.com");
    }

    #[test]
    fn test_email_auth_config_disable_for_oidc() {
        let config = EmailAuthConfig {
            disable_for_oidc: true,
            ..Default::default()
        };
        assert!(config.disable_for_oidc);
    }

    #[test]
    fn test_auth_config_serialization_roundtrip() {
        let config = AuthConfig {
            email: Some(EmailAuthConfig {
                enabled: true,
                allowed_domains: vec!["test.com".to_string()],
                ..Default::default()
            }),
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AuthConfig = serde_json::from_str(&json).unwrap();

        assert!(deserialized.email.is_some());
        assert!(deserialized.email.unwrap().enabled);
    }

    #[test]
    fn test_email_auth_config_all_false() {
        let config = EmailAuthConfig {
            enabled: false,
            allow_login_as_external_user: false,
            allow_create_external_users: false,
            disable_for_oidc: false,
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"enabled\":false"));
        assert!(json.contains("\"allow_login_as_external_user\":false"));
    }
}
