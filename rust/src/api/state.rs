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
use crate::pro::services::{SubscriptionService, SubscriptionServiceImpl};
use crate::services::metrics::MetricsManager;
use crate::services::runners::task_queue::TaskQueue;
use crate::services::telegram_bot::TelegramBot;
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
    /// Telegram bot для уведомлений
    pub telegram_bot: Option<Arc<TelegramBot>>,
    /// PRO / лицензирование (community edition — без ограничений по умолчанию)
    pub subscription: Arc<dyn SubscriptionService + Send + Sync>,
    /// Персистентная очередь задач (Redis или in-memory)
    pub task_queue: Option<Arc<dyn TaskQueue + Send + Sync>>,
    /// Redis URL для WebSocket Pub/Sub (если доступен)
    pub ws_redis_url: Option<String>,
}

impl AppState {
    /// Создаёт новое состояние приложения
    pub fn new(
        store: Arc<dyn Store + Send + Sync>,
        config: Config,
        cache: Option<Arc<RedisCache>>,
    ) -> Self {
        Self::with_task_queue(store, config, cache, None)
    }

    /// Создаёт состояние с персистентной очередью задач
    pub fn with_task_queue(
        store: Arc<dyn Store + Send + Sync>,
        config: Config,
        cache: Option<Arc<RedisCache>>,
        task_queue: Option<Arc<dyn TaskQueue + Send + Sync>>,
    ) -> Self {
        Self::with_ws_redis(store, config, cache, task_queue, None)
    }

    /// Создаёт состояние с WebSocket Redis Pub/Sub
    pub fn with_ws_redis(
        store: Arc<dyn Store + Send + Sync>,
        config: Config,
        cache: Option<Arc<RedisCache>>,
        task_queue: Option<Arc<dyn TaskQueue + Send + Sync>>,
        ws_redis_url: Option<String>,
    ) -> Self {
        Self::with_ws_and_task_queue(store, config, cache, task_queue, Arc::new(WebSocketManager::new()), ws_redis_url)
    }

    /// Создаёт состояние с пред-инициализированным WebSocket менеджером
    pub fn with_ws_and_task_queue(
        store: Arc<dyn Store + Send + Sync>,
        config: Config,
        cache: Option<Arc<RedisCache>>,
        task_queue: Option<Arc<dyn TaskQueue + Send + Sync>>,
        ws_manager: Arc<WebSocketManager>,
        ws_redis_url: Option<String>,
    ) -> Self {
        let telegram_bot = TelegramBot::new(&config);
        let subscription: Arc<dyn SubscriptionService + Send + Sync> =
            Arc::new(SubscriptionServiceImpl::new());

        Self {
            store: StoreWrapper::new(store),
            config,
            ws_manager,
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
            telegram_bot,
            subscription,
            task_queue,
            ws_redis_url,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oidc_state_new() {
        let state = OidcState {
            pkce_verifier: "verifier123".to_string(),
            provider: "google".to_string(),
        };
        assert_eq!(state.pkce_verifier, "verifier123");
        assert_eq!(state.provider, "google");
    }

    #[test]
    fn test_oidc_state_clone() {
        let state = OidcState {
            pkce_verifier: "abc".to_string(),
            provider: "github".to_string(),
        };
        let cloned = state.clone();
        assert_eq!(cloned.pkce_verifier, state.pkce_verifier);
        assert_eq!(cloned.provider, state.provider);
    }

    #[test]
    fn test_oidc_state_hashmap_insert() {
        let mut map: HashMap<String, OidcState> = HashMap::new();
        map.insert(
            "session1".to_string(),
            OidcState {
                pkce_verifier: "v1".to_string(),
                provider: "oidc".to_string(),
            },
        );
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("session1").unwrap().pkce_verifier, "v1");
    }

    #[test]
    fn test_oidc_state_hashmap_multiple() {
        let mut map: HashMap<String, OidcState> = HashMap::new();
        for i in 0..5 {
            map.insert(
                format!("session_{}", i),
                OidcState {
                    pkce_verifier: format!("verifier_{}", i),
                    provider: "provider".to_string(),
                },
            );
        }
        assert_eq!(map.len(), 5);
        assert_eq!(map.get("session_3").unwrap().pkce_verifier, "verifier_3");
    }

    #[test]
    fn test_oidc_state_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OidcState>();
    }

    #[test]
    fn test_oidc_state_different_providers() {
        let providers = vec!["google", "github", "keycloak", "okta"];
        for provider in providers {
            let state = OidcState {
                pkce_verifier: "test".to_string(),
                provider: provider.to_string(),
            };
            assert_eq!(state.provider, provider);
        }
    }

    #[test]
    fn test_oidc_state_pkce_verifier_formats() {
        let verifiers = vec![
            "simple",
            "with-dashes",
            "with_underscores",
            "base64urlencoded==",
        ];
        for v in verifiers {
            let state = OidcState {
                pkce_verifier: v.to_string(),
                provider: "test".to_string(),
            };
            assert_eq!(state.pkce_verifier, v);
        }
    }
}
