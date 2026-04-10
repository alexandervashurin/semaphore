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

#[cfg(test)]
mod tests {
    use crate::config::{LogFormat, LogLevel, LoggingConfig};

    #[test]
    fn test_log_format_default_is_text() {
        let config = LoggingConfig::default();
        assert_eq!(config.format, LogFormat::Text);
    }

    #[test]
    fn test_log_level_default_is_info() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, LogLevel::Info);
    }

    #[test]
    fn test_logging_config_new() {
        let config = LoggingConfig::new();
        assert_eq!(config.format, LogFormat::Text);
        assert_eq!(config.level, LogLevel::Info);
    }

    #[test]
    fn test_logging_config_file_logging_flag() {
        let config = LoggingConfig {
            file: Some("/tmp/test.log".to_string()),
            ..Default::default()
        };
        assert!(config.is_file_logging());
    }

    #[test]
    fn test_logging_config_no_file_logging() {
        let config = LoggingConfig::default();
        assert!(!config.is_file_logging());
    }

    #[test]
    fn test_logging_config_level_string_debug() {
        let config = LoggingConfig {
            level: LogLevel::Debug,
            ..Default::default()
        };
        assert_eq!(config.level_string(), "debug");
    }

    #[test]
    fn test_logging_config_level_string_error() {
        let config = LoggingConfig {
            level: LogLevel::Error,
            ..Default::default()
        };
        assert_eq!(config.level_string(), "error");
    }
}
