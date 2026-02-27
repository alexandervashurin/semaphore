//! OIDC Provider Types
//!
//! Типы для OIDC провайдеров

use serde::{Deserialize, Serialize};

/// OIDC Endpoint
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OidcEndpoint {
    pub auth_url: String,
    pub token_url: String,
    pub userinfo_url: String,
    pub jwks_url: String,
}

/// OIDC Провайдер
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcProvider {
    /// Client ID
    pub client_id: String,

    /// Путь к файлу с Client ID
    #[serde(default)]
    pub client_id_file: String,

    /// Client Secret
    #[serde(default)]
    pub client_secret: String,

    /// Путь к файлу с Client Secret
    #[serde(default)]
    pub client_secret_file: String,

    /// Redirect URL
    #[serde(default)]
    pub redirect_url: String,

    /// OAuth scopes
    #[serde(default)]
    pub scopes: Vec<String>,

    /// Отображаемое имя
    #[serde(default)]
    pub display_name: String,

    /// Цвет
    #[serde(default)]
    pub color: String,

    /// Иконка
    #[serde(default)]
    pub icon: String,

    /// URL автообнаружения провайдера
    #[serde(default, rename = "provider_url")]
    pub auto_discovery: String,

    /// Endpoint
    #[serde(default)]
    pub endpoint: OidcEndpoint,

    /// Claim для имени пользователя
    #[serde(default = "default_username_claim")]
    pub username_claim: String,

    /// Claim для имени
    #[serde(default = "default_name_claim")]
    pub name_claim: String,

    /// Claim для email
    #[serde(default = "default_email_claim")]
    pub email_claim: String,

    /// Порядок отображения
    #[serde(default)]
    pub order: i32,

    /// Использовать state для return path
    #[serde(default = "default_true")]
    pub return_via_state: bool,
}

fn default_username_claim() -> String {
    "preferred_username".to_string()
}

fn default_name_claim() -> String {
    "preferred_username".to_string()
}

fn default_email_claim() -> String {
    "email".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for OidcProvider {
    fn default() -> Self {
        Self {
            client_id: String::new(),
            client_id_file: String::new(),
            client_secret: String::new(),
            client_secret_file: String::new(),
            redirect_url: String::new(),
            scopes: Vec::new(),
            display_name: String::new(),
            color: String::new(),
            icon: String::new(),
            auto_discovery: String::new(),
            endpoint: OidcEndpoint::default(),
            username_claim: default_username_claim(),
            name_claim: default_name_claim(),
            email_claim: default_email_claim(),
            order: 0,
            return_via_state: true,
        }
    }
}

impl OidcProvider {
    /// Получает claim имени пользователя
    pub fn get_username_claim(&self) -> &str {
        &self.username_claim
    }

    /// Получает claim email
    pub fn get_email_claim(&self) -> &str {
        &self.email_claim
    }

    /// Получает claim имени
    pub fn get_name_claim(&self) -> &str {
        &self.name_claim
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oidc_provider_default_claims() {
        let provider = OidcProvider::default();
        assert_eq!(provider.get_username_claim(), "preferred_username");
        assert_eq!(provider.get_name_claim(), "preferred_username");
        assert_eq!(provider.get_email_claim(), "email");
    }

    #[test]
    fn test_oidc_provider_custom_claims() {
        let provider = OidcProvider {
            username_claim: "sub".to_string(),
            name_claim: "name".to_string(),
            email_claim: "mail".to_string(),
            ..Default::default()
        };
        assert_eq!(provider.get_username_claim(), "sub");
        assert_eq!(provider.get_name_claim(), "name");
        assert_eq!(provider.get_email_claim(), "mail");
    }

    #[test]
    fn test_oidc_provider_serialization() {
        let provider = OidcProvider::default();
        let json = serde_json::to_string(&provider).unwrap();
        assert!(json.contains("client_id"));
        assert!(json.contains("preferred_username"));
    }
}
