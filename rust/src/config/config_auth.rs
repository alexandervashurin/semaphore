//! Configuration Types
//!
//! Типы конфигурации приложения

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
}
