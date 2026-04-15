
//! Config Logging - конфигурация логирования
//!
//! Аналог util/config.go из Go версии (часть 8: логирование)
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Тип формата логов
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    #[default]
    Text,
}

/// Тип уровня логирования
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

/// Конфигурация логирования
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Формат логов (json/text)
    pub format: LogFormat,
    /// Уровень логирования
    pub level: LogLevel,
    /// Путь к файлу логов (если пустой - в stdout)
    pub file: Option<String>,
    /// Максимальный размер файла логов в МБ
    pub max_size: u64,
    /// Максимальное количество файлов логов
    pub max_backups: u32,
    /// Максимальный возраст файлов логов в днях
    pub max_age: u32,
    /// Сжимать старые файлы логов
    pub compress: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            format: LogFormat::Text,
            level: LogLevel::Info,
            file: None,
            max_size: 100,
            max_backups: 3,
            max_age: 28,
            compress: false,
        }
    }
}

impl LoggingConfig {
    /// Создаёт новую конфигурацию логирования
    pub fn new() -> Self {
        Self::default()
    }

    /// Проверяет включено ли логирование в файл
    pub fn is_file_logging(&self) -> bool {
        self.file.is_some()
    }

    /// Получает путь к файлу логов
    pub fn get_file_path(&self) -> Option<PathBuf> {
        self.file.as_ref().map(PathBuf::from)
    }

    /// Получает уровень логирования как строку
    pub fn level_string(&self) -> &'static str {
        match self.level {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}

/// Загружает конфигурацию логирования из переменных окружения
pub fn load_logging_from_env() -> LoggingConfig {
    use std::env;
    let mut config = LoggingConfig::new();

    if let Ok(v) = env::var("VELUM_LOG_FORMAT") {
        config.format = match v.trim().to_lowercase().as_str() {
            "json" => LogFormat::Json,
            _ => LogFormat::Text,
        };
    }

    if let Ok(v) = env::var("VELUM_LOG_LEVEL") {
        config.level = match v.trim().to_lowercase().as_str() {
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Info,
        };
    }

    if let Ok(v) = env::var("VELUM_LOG_FILE") {
        let trimmed = v.trim().to_string();
        if !trimmed.is_empty() {
            config.file = Some(trimmed);
        }
    }

    if let Ok(v) = env::var("VELUM_LOG_MAX_SIZE") {
        if let Ok(val) = v.trim().parse() { config.max_size = val; }
    }
    if let Ok(v) = env::var("VELUM_LOG_MAX_BACKUPS") {
        if let Ok(val) = v.trim().parse() { config.max_backups = val; }
    }
    if let Ok(v) = env::var("VELUM_LOG_MAX_AGE") {
        if let Ok(val) = v.trim().parse() { config.max_age = val; }
    }

    if let Ok(v) = env::var("VELUM_LOG_COMPRESS") {
        config.compress = matches!(v.trim().to_lowercase().as_str(), "true" | "1" | "yes");
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.format, LogFormat::Text);
        assert_eq!(config.level, LogLevel::Info);
        assert!(!config.is_file_logging());
    }

    #[test]
    fn test_logging_config_level_string() {
        let config = LoggingConfig {
            level: LogLevel::Debug,
            ..Default::default()
        };
        assert_eq!(config.level_string(), "debug");
    }

    #[test]
    fn test_logging_config_file_path() {
        let config = LoggingConfig {
            file: Some("/var/log/velum.log".to_string()),
            ..Default::default()
        };
        assert!(config.is_file_logging());
        assert_eq!(
            config.get_file_path().unwrap(),
            PathBuf::from("/var/log/velum.log")
        );
    }

    #[test]
    fn test_load_logging_from_env() {
        unsafe {
            std::env::set_var("VELUM_LOG_FORMAT", "json");
            std::env::set_var("VELUM_LOG_LEVEL", "debug");
            std::env::set_var("VELUM_LOG_FILE", "/tmp/test.log");
        }

        let config = load_logging_from_env();
        assert_eq!(config.format, LogFormat::Json);
        assert_eq!(config.level, LogLevel::Debug);
        assert_eq!(config.file, Some("/tmp/test.log".to_string()));

        unsafe {
            std::env::remove_var("VELUM_LOG_FORMAT");
            std::env::remove_var("VELUM_LOG_LEVEL");
            std::env::remove_var("VELUM_LOG_FILE");
        }
    }

    #[test]
    fn test_log_format_serialization() {
        let format = LogFormat::Json;
        let serialized = serde_json::to_string(&format).unwrap();
        assert_eq!(serialized, "\"json\"");

        let deserialized: LogFormat = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, LogFormat::Json);
    }

    #[test]
    fn test_log_level_serialization() {
        let level = LogLevel::Warn;
        let serialized = serde_json::to_string(&level).unwrap();
        assert_eq!(serialized, "\"warn\"");

        let deserialized: LogLevel = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, LogLevel::Warn);
    }

    #[test]
    fn test_log_format_text_serialization() {
        let format = LogFormat::Text;
        let serialized = serde_json::to_string(&format).unwrap();
        assert_eq!(serialized, "\"text\"");

        let deserialized: LogFormat = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, LogFormat::Text);
    }

    #[test]
    fn test_log_level_debug_serialization() {
        let level = LogLevel::Debug;
        let serialized = serde_json::to_string(&level).unwrap();
        assert_eq!(serialized, "\"debug\"");
    }

    #[test]
    fn test_log_level_info_serialization() {
        let level = LogLevel::Info;
        let serialized = serde_json::to_string(&level).unwrap();
        assert_eq!(serialized, "\"info\"");
    }

    #[test]
    fn test_log_level_error_serialization() {
        let level = LogLevel::Error;
        let serialized = serde_json::to_string(&level).unwrap();
        assert_eq!(serialized, "\"error\"");
    }

    #[test]
    fn test_logging_config_new() {
        let config = LoggingConfig::new();
        assert_eq!(config.format, LogFormat::Text);
        assert_eq!(config.level, LogLevel::Info);
    }

    #[test]
    fn test_logging_config_full_serialization() {
        let config = LoggingConfig {
            format: LogFormat::Json,
            level: LogLevel::Debug,
            file: Some("/var/log/app.log".to_string()),
            max_size: 50,
            max_backups: 5,
            max_age: 14,
            compress: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"format\":\"json\""));
        assert!(json.contains("\"level\":\"debug\""));
        assert!(json.contains("\"max_size\":50"));
        assert!(json.contains("\"compress\":true"));
    }

    #[test]
    fn test_logging_config_deserialization() {
        let json = r#"{
            "format": "json",
            "level": "warn",
            "file": "/logs/app.log",
            "max_size": 200,
            "max_backups": 10,
            "max_age": 30,
            "compress": true
        }"#;

        let config: LoggingConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.format, LogFormat::Json);
        assert_eq!(config.level, LogLevel::Warn);
        assert_eq!(config.max_size, 200);
        assert_eq!(config.max_backups, 10);
        assert!(config.compress);
    }

    #[test]
    fn test_logging_config_defaults_from_serde() {
        // #[serde(default)] на структуре заставляет serde использовать LoggingConfig::default()
        let json = r#"{}"#;
        let config: LoggingConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.format, LogFormat::Text);
        assert_eq!(config.level, LogLevel::Info);
        assert_eq!(config.max_size, 100); // Исправлено: 0 -> 100 из impl Default
        assert_eq!(config.max_backups, 3); // Исправлено
        assert_eq!(config.max_age, 28); // Исправлено
        assert!(!config.compress);
    }

    #[test]
    fn test_load_logging_from_env_numeric_values() {
        unsafe {
            std::env::set_var("VELUM_LOG_MAX_SIZE", "500");
            std::env::set_var("VELUM_LOG_MAX_BACKUPS", "10");
            std::env::set_var("VELUM_LOG_MAX_AGE", "60");
        }

        let config = load_logging_from_env();
        assert_eq!(config.max_size, 500);
        assert_eq!(config.max_backups, 10);
        assert_eq!(config.max_age, 60);

        unsafe {
            std::env::remove_var("VELUM_LOG_MAX_SIZE");
            std::env::remove_var("VELUM_LOG_MAX_BACKUPS");
            std::env::remove_var("VELUM_LOG_MAX_AGE");
        }
    }

    #[test]
    fn test_load_logging_from_env_compress_true() {
        unsafe { std::env::set_var("VELUM_LOG_COMPRESS", "true") };
        let config = load_logging_from_env();
        assert!(config.compress);
        unsafe { std::env::remove_var("VELUM_LOG_COMPRESS") };
    }

    #[test]
    fn test_load_logging_from_env_compress_1() {
        unsafe { std::env::set_var("VELUM_LOG_COMPRESS", "1") };
        let config = load_logging_from_env();
        assert!(config.compress);
        unsafe { std::env::remove_var("VELUM_LOG_COMPRESS") };
    }

    #[test]
    fn test_load_logging_from_env_compress_false() {
        unsafe { std::env::set_var("VELUM_LOG_COMPRESS", "false") };
        let config = load_logging_from_env();
        assert!(!config.compress);
        unsafe { std::env::remove_var("VELUM_LOG_COMPRESS") };
    }

    #[test]
    fn test_load_logging_from_env_invalid_numeric() {
        unsafe { std::env::set_var("VELUM_LOG_MAX_SIZE", "not_a_number") };
        let config = load_logging_from_env();
        // При невалидном значении остаётся дефолтное (100)
        assert_eq!(config.max_size, 100);
        unsafe { std::env::remove_var("VELUM_LOG_MAX_SIZE") };
    }

    #[test]
    fn test_log_level_unknown_value() {
        unsafe { std::env::set_var("VELUM_LOG_LEVEL", "unknown") };
        let config = load_logging_from_env();
        assert_eq!(config.level, LogLevel::Info);
        unsafe { std::env::remove_var("VELUM_LOG_LEVEL") };
    }

    #[test]
    fn test_logging_config_clone() {
        let config = LoggingConfig {
            format: LogFormat::Json,
            level: LogLevel::Debug,
            file: Some("/test.log".to_string()),
            max_size: 100,
            max_backups: 3,
            max_age: 28,
            compress: true,
        };
        let cloned = config.clone();
        assert_eq!(cloned.format, config.format);
        assert_eq!(cloned.level, config.level);
        assert_eq!(cloned.file, config.file);
    }

    #[test]
    fn test_logging_config_get_file_path_none() {
        let config = LoggingConfig::default();
        assert!(config.get_file_path().is_none());
    }

    #[test]
    fn test_logging_config_level_string_warn() {
        let config = LoggingConfig {
            level: LogLevel::Warn,
            ..Default::default()
        };
        assert_eq!(config.level_string(), "warn");
    }

    #[test]
    fn test_logging_config_level_string_error() {
        let config = LoggingConfig {
            level: LogLevel::Error,
            ..Default::default()
        };
        assert_eq!(config.level_string(), "error");
    }

    #[test]
    fn test_load_logging_from_env_case_insensitive_level() {
        unsafe { std::env::set_var("VELUM_LOG_LEVEL", "DEBUG") };
        let config = load_logging_from_env();
        assert_eq!(config.level, LogLevel::Debug);
        unsafe { std::env::remove_var("VELUM_LOG_LEVEL") };
    }

    #[test]
    fn test_load_logging_from_env_case_insensitive_format() {
        unsafe { std::env::set_var("VELUM_LOG_FORMAT", "JSON") };
        let config = load_logging_from_env();
        assert_eq!(config.format, LogFormat::Json);
        unsafe { std::env::remove_var("VELUM_LOG_FORMAT") };
    }

    #[test]
    fn test_logging_config_unicode_file_path() {
        let config = LoggingConfig {
            file: Some("/var/log/приложение.log".to_string()),
            ..Default::default()
        };
        assert!(config.is_file_logging());
        let path = config.get_file_path().unwrap();
        assert!(path.to_string_lossy().contains("приложение"));
    }

    #[test]
    fn test_logging_config_max_values() {
        let config = LoggingConfig {
            max_size: u64::MAX,
            max_backups: u32::MAX,
            max_age: u32::MAX,
            ..Default::default()
        };
        assert_eq!(config.max_size, u64::MAX);
        assert_eq!(config.max_backups, u32::MAX);
        assert_eq!(config.max_age, u32::MAX);
    }
}
