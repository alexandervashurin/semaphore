//! Модуль инициализации логирования

use tracing_subscriber::{self, EnvFilter};

/// Инициализирует систему логирования
///
/// Использует переменную окружения RUST_LOG для настройки уровня логирования.
/// По умолчанию используется уровень "info".
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    tracing::info!("Логирование инициализировано");
}
