//! Состояние приложения

use crate::db::Store;
use crate::config::Config;
use std::sync::Arc;
use super::websocket::WebSocketManager;
use super::store_wrapper::StoreWrapper;

/// Состояние приложения, доступное всем обработчикам
pub struct AppState {
    pub store: StoreWrapper,
    pub config: Config,
    pub ws_manager: Arc<WebSocketManager>,
}

impl AppState {
    /// Создаёт новое состояние приложения
    pub fn new(store: Box<dyn Store>, config: Config) -> Self {
        Self {
            store: StoreWrapper::new(Arc::new(store)),
            config,
            ws_manager: Arc::new(WebSocketManager::new()),
        }
    }
}
