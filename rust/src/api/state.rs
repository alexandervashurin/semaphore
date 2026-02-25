//! Состояние приложения

use crate::db::Store;
use crate::config::Config;

/// Состояние приложения, доступное всем обработчикам
pub struct AppState {
    pub store: Box<dyn Store>,
    pub config: Config,
}
