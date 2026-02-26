//! Config OIDC - OIDC конфигурация
//!
//! Аналог util/config.go из Go версии (часть 6: OIDC)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
            scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
            auto_discovery: String::new(),
            endpoint: OidcEndpoint::default(),
            color: String::new(),
            icon: String::new(),
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
            
            let prefix = format!("SEMAPHORE_OIDC_{}_{}", 
                provider_name.to_uppercase(), 
                "{}");
            
            if let Ok(display_name) = env::var(format!("SEMAPHORE_OIDC_{}_DISPLAY_NAME", provider_name.to_uppercase())) {
                provider.display_name = display_name;
            }
            
            if let Ok(client_id) = env::var(format!("SEMAPHORE_OIDC_{}_CLIENT_ID", provider_name.to_uppercase())) {
                provider.client_id = client_id;
            }
            
            if let Ok(client_secret) = env::var(format!("SEMAPHORE_OIDC_{}_CLIENT_SECRET", provider_name.to_uppercase())) {
                provider.client_secret = client_secret;
            }
            
            if let Ok(redirect_url) = env::var(format!("SEMAPHORE_OIDC_{}_REDIRECT_URL", provider_name.to_uppercase())) {
                provider.redirect_url = redirect_url;
            }
            
            if let Ok(scopes) = env::var(format!("SEMAPHORE_OIDC_{}_SCOPES", provider_name.to_uppercase())) {
                provider.scopes = scopes.split(' ').map(|s| s.to_string()).collect();
            }
            
            if let Ok(auto_discovery) = env::var(format!("SEMAPHORE_OIDC_{}_AUTO_DISCOVERY", provider_name.to_uppercase())) {
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
}
