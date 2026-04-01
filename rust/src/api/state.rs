//! Состояние приложения

use super::middleware::rate_limiter::{RateLimitConfig, RateLimiter};
use super::store_wrapper::StoreWrapper;
use super::token_blacklist::TokenBlacklist;
use super::websocket::WebSocketManager;
use crate::api::handlers::kubernetes::client::{KubeClient, KubeConfig};
use crate::cache::RedisCache;
use crate::config::Config;
use crate::db::Store;
use crate::error::{Error, Result};
use crate::services::metrics::MetricsManager;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

/// OIDC state для хранения PKCE verifier между redirect и callback
#[derive(Clone)]
pub struct OidcState {
    pub pkce_verifier: String,
    pub provider: String,
}

/// Состояние приложения, доступное всем обработчикам
pub struct AppState {
    pub store: StoreWrapper,
    pub config: Config,
    pub ws_manager: Arc<WebSocketManager>,
    pub oidc_state: Arc<Mutex<HashMap<String, OidcState>>>,
    pub metrics: MetricsManager,
    pub cache: Option<Arc<RedisCache>>,
    /// Rate limiter для API запросов (100 req/min per IP)
    pub rate_limiter_api: Arc<RateLimiter>,
    /// Rate limiter для auth эндпоинтов (5 req/min per IP)
    pub rate_limiter_auth: Arc<RateLimiter>,
    /// JWT blacklist — отозванные токены до истечения их TTL
    pub token_blacklist: TokenBlacklist,
    /// Зашифрованные kubeconfig'и (name → AES-256-GCM encrypted base64)
    pub kubeconfigs: Arc<DashMap<String, String>>,
}

impl AppState {
    /// Создаёт новое состояние приложения
    pub fn new(
        store: Arc<dyn Store + Send + Sync>,
        config: Config,
        cache: Option<Arc<RedisCache>>,
    ) -> Self {
        Self {
            store: StoreWrapper::new(store),
            config,
            ws_manager: Arc::new(WebSocketManager::new()),
            oidc_state: Arc::new(Mutex::new(HashMap::new())),
            metrics: MetricsManager::new(),
            cache,
            rate_limiter_api: Arc::new(RateLimiter::new(RateLimitConfig {
                max_requests: 100,
                period_secs: 60,
                burst_size: Some(20),
            })),
            rate_limiter_auth: Arc::new(RateLimiter::new(RateLimitConfig {
                max_requests: 5,
                period_secs: 60,
                burst_size: None,
            })),
            token_blacklist: TokenBlacklist::new(),
            kubeconfigs: Arc::new(DashMap::new()),
        }
    }

    /// Создаёт Kubernetes клиент из конфигурации
    pub fn kubernetes_client(&self) -> Result<Arc<KubeClient>> {
        let kubeconfig_path = self
            .config
            .kubernetes
            .as_ref()
            .and_then(|k| k.kubeconfig_path.clone());
        let context = self
            .config
            .kubernetes
            .as_ref()
            .and_then(|k| k.context.clone());
        let default_namespace = self
            .config
            .kubernetes
            .as_ref()
            .map(|k| k.default_namespace.clone())
            .unwrap_or_else(|| "default".to_string());
        let timeout_secs = self
            .config
            .kubernetes
            .as_ref()
            .map(|k| k.request_timeout_secs)
            .unwrap_or(30);
        let list_default_limit = self
            .config
            .kubernetes
            .as_ref()
            .map(|k| k.default_list_limit)
            .unwrap_or(200);

        let kube_config = KubeConfig {
            kubeconfig_path,
            context,
            default_namespace,
            timeout_secs,
            list_default_limit,
        };

        // Используем blocking-обёртку для async создания клиента
        // В реальном приложении лучше кэшировать клиент при старте
        let client = futures::executor::block_on(KubeClient::new(kube_config))?;
        Ok(Arc::new(client))
    }
}
