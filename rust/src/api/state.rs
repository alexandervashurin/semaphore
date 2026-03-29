//! Состояние приложения

use crate::db::Store;
use crate::config::Config;
use crate::services::metrics::MetricsManager;
use crate::cache::RedisCache;
use crate::kubernetes::KubernetesClusterManager;
use std::sync::Arc;
use std::collections::HashMap;
use std::sync::Mutex;
use super::websocket::WebSocketManager;
use super::store_wrapper::StoreWrapper;

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
    /// Менеджер подключений к Kubernetes (None если кластер не сконфигурирован)
    pub k8s: Option<Arc<KubernetesClusterManager>>,
}

impl AppState {
    /// Создаёт новое состояние приложения
    pub fn new(store: Arc<dyn Store + Send + Sync>, config: Config, cache: Option<Arc<RedisCache>>) -> Self {
        Self {
            store: StoreWrapper::new(store),
            config,
            ws_manager: Arc::new(WebSocketManager::new()),
            oidc_state: Arc::new(Mutex::new(HashMap::new())),
            metrics: MetricsManager::new(),
            cache,
            k8s: None,
        }
    }

    /// Создаёт состояние с Kubernetes менеджером
    pub fn with_kubernetes(mut self, k8s: Option<Arc<KubernetesClusterManager>>) -> Self {
        self.k8s = k8s;
        self
    }
}
