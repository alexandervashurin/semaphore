//! Состояние приложения

use crate::db::Store;
use crate::config::Config;
use std::sync::Arc;
use super::websocket::WebSocketManager;

/// Состояние приложения, доступное всем обработчикам
pub struct AppState {
    pub store: Box<dyn Store>,
    pub config: Config,
    pub ws_manager: Arc<WebSocketManager>,
}
