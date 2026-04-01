//! Logging initialization.

use crate::config::{load_logging_from_env, LogFormat};
use tracing_subscriber::{self, EnvFilter};

/// Initializes application logging.
pub fn init_logging() {
    let logging_config = load_logging_from_env();
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    match logging_config.format {
        LogFormat::Json => {
            tracing_subscriber::fmt()
                .json()
                .with_env_filter(env_filter)
                .with_target(true)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false)
                .init();
        }
        LogFormat::Text => {
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_target(true)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false)
                .init();
        }
    }

    tracing::info!("Logging initialized");
}
