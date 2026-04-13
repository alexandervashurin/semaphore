//! Config OIDC - OIDC конфигурация
//!
//! Аналог util/config.go из Go версии (часть 6: OIDC)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_oidc_email_claim() -> String {
    "email".to_string()
}

fn default_oidc_username_claim() -> String {
    "preferred_username".to_string()
}

fn default_oidc_name_claim() -> String {
    "name".to_string()
}

/// OIDC провайдер
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcProvider {
    /// Имя провайдера (для отображения)
    #[serde(default)]
    pub display_name: String,

    /// Client ID
    #[serde(default)]
    pub client_id: String,

    /// Client Secret
    #[serde(default)]
    pub client_secret: String,

    /// Redirect URL
    #[serde(default)]
    pub redirect_url: String,

    /// Scopes
    #[serde(default)]
    pub scopes: Vec<String>,

    /// Auto discovery URL
    #[serde(default)]
    pub auto_discovery: String,

    /// Endpoint
    #[serde(default)]
    pub endpoint: OidcEndpoint,

    /// Color для кнопки
    #[serde(default)]
    pub color: String,

    /// Icon
    #[serde(default)]
    pub icon: String,

    /// Имя claim для email в userinfo (например `email`, `mail`, `upn` для AAD)
    #[serde(default = "default_oidc_email_claim")]
    pub email_claim: String,

    /// Имя claim для логина (например `preferred_username`, `sub`)
    #[serde(default = "default_oidc_username_claim")]
    pub username_claim: String,

    /// Имя claim для отображаемого имени
    #[serde(default = "default_oidc_name_claim")]
    pub name_claim: String,
}

/// OIDC Endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcEndpoint {
    /// Issuer URL
    #[serde(default)]
    pub issuer_url: String,

    /// Auth URL
    #[serde(default)]
    pub auth_url: String,

    /// Token URL
    #[serde(default)]
    pub token_url: String,

    /// UserInfo URL
    #[serde(default)]
    pub userinfo_url: String,

    /// JWKS URL
    #[serde(default)]
    pub jwks_url: String,

    /// Algorithms
    #[serde(default)]
    pub algorithms: Vec<String>,
}

impl Default for OidcProvider {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            client_id: String::new(),
            client_secret: String::new(),
            redirect_url: String::new(),
            scopes: vec![
                "openid".to_string(),
                "profile".to_string(),
                "email".to_string(),
            ],
            auto_discovery: String::new(),
            endpoint: OidcEndpoint::default(),
            color: String::new(),
            icon: String::new(),
            email_claim: default_oidc_email_claim(),
            username_claim: default_oidc_username_claim(),
            name_claim: default_oidc_name_claim(),
        }
    }
}

impl Default for OidcEndpoint {
    fn default() -> Self {
        Self {
            issuer_url: String::new(),
            auth_url: String::new(),
            token_url: String::new(),
            userinfo_url: String::new(),
            jwks_url: String::new(),
            algorithms: vec!["RS256".to_string()],
        }
    }
}

impl OidcProvider {
    /// Создаёт новый OIDC провайдер
    pub fn new() -> Self {
        Self::default()
    }

    /// Проверяет настроен ли провайдер
    pub fn is_configured(&self) -> bool {
        !self.client_id.is_empty() && !self.client_secret.is_empty()
    }

    /// Получает scopes как строку
    pub fn scopes_string(&self) -> String {
        self.scopes.join(" ")
    }
}

/// Загружает OIDC конфигурацию из переменных окружения
pub fn load_oidc_from_env() -> HashMap<String, OidcProvider> {
    use std::env;

    let mut providers = HashMap::new();

    // Пример: SEMAPHORE_OIDC_PROVIDERS=google,github
    if let Ok(providers_list) = env::var("SEMAPHORE_OIDC_PROVIDERS") {
        for provider_name in providers_list.split(',') {
            let provider_name = provider_name.trim();
            if provider_name.is_empty() {
                continue;
            }

            let mut provider = OidcProvider::new();

            let prefix = format!("SEMAPHORE_OIDC_{}_{}", provider_name.to_uppercase(), "{}");

            if let Ok(display_name) = env::var(format!(
                "SEMAPHORE_OIDC_{}_DISPLAY_NAME",
                provider_name.to_uppercase()
            )) {
                provider.display_name = display_name;
            }

            if let Ok(client_id) = env::var(format!(
                "SEMAPHORE_OIDC_{}_CLIENT_ID",
                provider_name.to_uppercase()
            )) {
                provider.client_id = client_id;
            }

            if let Ok(client_secret) = env::var(format!(
                "SEMAPHORE_OIDC_{}_CLIENT_SECRET",
                provider_name.to_uppercase()
            )) {
                provider.client_secret = client_secret;
            }

            if let Ok(redirect_url) = env::var(format!(
                "SEMAPHORE_OIDC_{}_REDIRECT_URL",
                provider_name.to_uppercase()
            )) {
                provider.redirect_url = redirect_url;
            }

            if let Ok(scopes) = env::var(format!(
                "SEMAPHORE_OIDC_{}_SCOPES",
                provider_name.to_uppercase()
            )) {
                provider.scopes = scopes.split(' ').map(|s| s.to_string()).collect();
            }

            if let Ok(auto_discovery) = env::var(format!(
                "SEMAPHORE_OIDC_{}_AUTO_DISCOVERY",
                provider_name.to_uppercase()
            )) {
                provider.auto_discovery = auto_discovery;
            }

            providers.insert(provider_name.to_string(), provider);
        }
    }

    providers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oidc_provider_default() {
        let provider = OidcProvider::default();
        assert!(!provider.is_configured());
        assert_eq!(provider.scopes.len(), 3);
        assert!(provider.scopes.contains(&"openid".to_string()));
    }

    #[test]
    fn test_oidc_provider_is_configured() {
        let provider = OidcProvider {
            client_id: "test_id".to_string(),
            client_secret: "test_secret".to_string(),
            ..Default::default()
        };

        assert!(provider.is_configured());
    }

    #[test]
    fn test_oidc_provider_scopes_string() {
        let provider = OidcProvider {
            scopes: vec!["openid".to_string(), "profile".to_string()],
            ..Default::default()
        };

        assert_eq!(provider.scopes_string(), "openid profile");
    }

    #[test]
    fn test_oidc_endpoint_default() {
        let endpoint = OidcEndpoint::default();
        assert_eq!(endpoint.algorithms.len(), 1);
        assert_eq!(endpoint.algorithms[0], "RS256");
    }

    #[test]
    fn test_oidc_provider_new() {
        let provider = OidcProvider::new();
        assert!(!provider.is_configured());
    }

    #[test]
    fn test_oidc_provider_serialization() {
        let provider = OidcProvider {
            display_name: "Google".to_string(),
            client_id: "google_client_id".to_string(),
            client_secret: "google_secret".to_string(),
            redirect_url: "https://example.com/callback".to_string(),
            scopes: vec!["openid".to_string(), "email".to_string()],
            auto_discovery: "https://accounts.google.com".to_string(),
            endpoint: OidcEndpoint::default(),
            color: "#4285F4".to_string(),
            icon: "google-icon.svg".to_string(),
            email_claim: "email".to_string(),
            username_claim: "email".to_string(),
            name_claim: "name".to_string(),
        };

        let json = serde_json::to_string(&provider).unwrap();
        assert!(json.contains("\"display_name\":\"Google\""));
        assert!(json.contains("\"client_id\":\"google_client_id\""));
        assert!(json.contains("\"color\":\"#4285F4\""));
    }

    #[test]
    fn test_oidc_provider_deserialization() {
        let provider = OidcProvider {
            display_name: "GitHub".to_string(),
            client_id: "gh_client".to_string(),
            client_secret: "gh_secret".to_string(),
            redirect_url: "https://example.com/gh/callback".to_string(),
            scopes: vec!["openid".to_string(), "profile".to_string()],
            auto_discovery: "https://github.com/.well-known".to_string(),
            endpoint: OidcEndpoint::default(),
            color: "#24292e".to_string(),
            icon: "github.svg".to_string(),
            email_claim: "email".to_string(),
            username_claim: "login".to_string(),
            name_claim: "name".to_string(),
        };

        assert_eq!(provider.display_name, "GitHub");
        assert_eq!(provider.client_id, "gh_client");
        assert_eq!(provider.scopes.len(), 2);
        assert_eq!(provider.color, "#24292e");
    }

    #[test]
    fn test_oidc_provider_default_claims() {
        let provider = OidcProvider::default();
        assert_eq!(provider.email_claim, "email");
        assert_eq!(provider.username_claim, "preferred_username");
        assert_eq!(provider.name_claim, "name");
    }

    #[test]
    fn test_oidc_provider_empty_scopes_string() {
        let provider = OidcProvider {
            scopes: vec![],
            ..Default::default()
        };
        assert_eq!(provider.scopes_string(), "");
    }

    #[test]
    fn test_oidc_provider_single_scope_string() {
        let provider = OidcProvider {
            scopes: vec!["openid".to_string()],
            ..Default::default()
        };
        assert_eq!(provider.scopes_string(), "openid");
    }

    #[test]
    fn test_oidc_provider_is_configured_only_client_id() {
        let provider = OidcProvider {
            client_id: "test_id".to_string(),
            ..Default::default()
        };
        assert!(!provider.is_configured());
    }

    #[test]
    fn test_oidc_provider_is_configured_only_client_secret() {
        let provider = OidcProvider {
            client_secret: "test_secret".to_string(),
            ..Default::default()
        };
        assert!(!provider.is_configured());
    }

    #[test]
    fn test_oidc_endpoint_serialization() {
        let endpoint = OidcEndpoint {
            issuer_url: "https://issuer.example.com".to_string(),
            auth_url: "https://auth.example.com".to_string(),
            token_url: "https://token.example.com".to_string(),
            userinfo_url: "https://userinfo.example.com".to_string(),
            jwks_url: "https://jwks.example.com".to_string(),
            algorithms: vec!["RS256".to_string(), "ES256".to_string()],
        };

        let json = serde_json::to_string(&endpoint).unwrap();
        assert!(json.contains("\"issuer_url\":\"https://issuer.example.com\""));
        assert!(json.contains("\"algorithms\":[\"RS256\",\"ES256\"]"));
    }

    #[test]
    fn test_oidc_endpoint_deserialization() {
        let json = r#"{
            "issuer_url": "https://issuer.test.com",
            "auth_url": "https://auth.test.com",
            "token_url": "https://token.test.com",
            "userinfo_url": "https://userinfo.test.com",
            "jwks_url": "https://jwks.test.com",
            "algorithms": ["RS256"]
        }"#;

        let endpoint: OidcEndpoint = serde_json::from_str(json).unwrap();
        assert_eq!(endpoint.issuer_url, "https://issuer.test.com");
        assert_eq!(endpoint.algorithms.len(), 1);
    }

    #[test]
    fn test_oidc_provider_clone() {
        let provider = OidcProvider {
            display_name: "Clone Test".to_string(),
            client_id: "clone_id".to_string(),
            client_secret: "clone_secret".to_string(),
            ..Default::default()
        };
        let cloned = provider.clone();
        assert_eq!(cloned.display_name, "Clone Test");
        assert_eq!(cloned.client_id, "clone_id");
    }

    #[test]
    fn test_oidc_provider_unicode_display_name() {
        let provider = OidcProvider {
            display_name: "Провайдер".to_string(),
            client_id: "id".to_string(),
            client_secret: "secret".to_string(),
            ..Default::default()
        };
        let json = serde_json::to_string(&provider).unwrap();
        assert!(json.contains("Провайдер"));

        let deserialized: OidcProvider = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.display_name, "Провайдер");
    }

    #[test]
    fn test_oidc_provider_custom_claims() {
        let provider = OidcProvider {
            email_claim: "mail".to_string(),
            username_claim: "sub".to_string(),
            name_claim: "display_name".to_string(),
            ..Default::default()
        };
        assert_eq!(provider.email_claim, "mail");
        assert_eq!(provider.username_claim, "sub");
        assert_eq!(provider.name_claim, "display_name");
    }

    #[test]
    fn test_oidc_provider_serialization_defaults() {
        let provider = OidcProvider::default();
        let json = serde_json::to_string(&provider).unwrap();
        assert!(json.contains("\"scopes\":[\"openid\",\"profile\",\"email\"]"));
        assert!(json.contains("\"email_claim\":\"email\""));
    }

    #[test]
    fn test_oidc_endpoint_empty_values() {
        let endpoint = OidcEndpoint::default();
        assert!(endpoint.issuer_url.is_empty());
        assert!(endpoint.auth_url.is_empty());
        assert!(endpoint.token_url.is_empty());
        assert!(endpoint.userinfo_url.is_empty());
        assert!(endpoint.jwks_url.is_empty());
    }

    #[test]
    fn test_oidc_hashmap_load_from_env_empty() {
        let providers = load_oidc_from_env();
        assert!(providers.is_empty());
    }

    #[test]
    fn test_oidc_endpoint_clone() {
        let endpoint = OidcEndpoint {
            issuer_url: "https://issuer.test".to_string(),
            algorithms: vec!["RS256".to_string()],
            ..Default::default()
        };
        let cloned = endpoint.clone();
        assert_eq!(cloned.issuer_url, "https://issuer.test");
        assert_eq!(cloned.algorithms.len(), 1);
    }
}
