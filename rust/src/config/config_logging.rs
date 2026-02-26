//! Config Logging - конфигурация логирования
//!
//! Аналог util/config.go из Go версии (часть 8: логирование)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Тип формата логов
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Text,
}

impl Default for LogFormat {
    fn default() -> Self {
        LogFormat::Text
    }
}

/// Тип уровня логирования
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

/// Конфигурация логирования
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Формат логов (json/text)
    #[serde(default)]
    pub format: LogFormat,
    
    /// Уровень логирования
    #[serde(default)]
    pub level: LogLevel,
    
    /// Путь к файлу логов (если пустой - в stdout)
    #[serde(default)]
    pub file: Option<String>,
    
    /// Максимальный размер файла логов в МБ
    #[serde(default)]
    pub max_size: u64,
    
    /// Максимальное количество файлов логов
    #[serde(default)]
    pub max_backups: u32,
    
    /// Максимальный возраст файлов логов в днях
    #[serde(default)]
    pub max_age: u32,
    
    /// Сжимать старые файлы логов
    #[serde(default)]
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
    
    if let Ok(format) = env::var("SEMAPHORE_LOG_FORMAT") {
        config.format = match format.to_lowercase().as_str() {
            "json" => LogFormat::Json,
            _ => LogFormat::Text,
        };
    }
    
    if let Ok(level) = env::var("SEMAPHORE_LOG_LEVEL") {
        config.level = match level.to_lowercase().as_str() {
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Info,
        };
    }
    
    if let Ok(file) = env::var("SEMAPHORE_LOG_FILE") {
        config.file = Some(file);
    }
    
    if let Ok(max_size) = env::var("SEMAPHORE_LOG_MAX_SIZE") {
        if let Ok(size) = max_size.parse() {
            config.max_size = size;
        }
    }
    
    if let Ok(max_backups) = env::var("SEMAPHORE_LOG_MAX_BACKUPS") {
        if let Ok(backups) = max_backups.parse() {
            config.max_backups = backups;
        }
    }
    
    if let Ok(max_age) = env::var("SEMAPHORE_LOG_MAX_AGE") {
        if let Ok(age) = max_age.parse() {
            config.max_age = age;
        }
    }
    
    if let Ok(compress) = env::var("SEMAPHORE_LOG_COMPRESS") {
        config.compress = compress.to_lowercase() == "true" || compress == "1";
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
            file: Some("/var/log/semaphore.log".to_string()),
            ..Default::default()
        };
        assert!(config.is_file_logging());
        assert_eq!(
            config.get_file_path().unwrap(),
            PathBuf::from("/var/log/semaphore.log")
        );
    }

    #[test]
    fn test_load_logging_from_env() {
        env::set_var("SEMAPHORE_LOG_FORMAT", "json");
        env::set_var("SEMAPHORE_LOG_LEVEL", "debug");
        env::set_var("SEMAPHORE_LOG_FILE", "/tmp/test.log");
        
        let config = load_logging_from_env();
        assert_eq!(config.format, LogFormat::Json);
        assert_eq!(config.level, LogLevel::Debug);
        assert_eq!(config.file, Some("/tmp/test.log".to_string()));
        
        env::remove_var("SEMAPHORE_LOG_FORMAT");
        env::remove_var("SEMAPHORE_LOG_LEVEL");
        env::remove_var("SEMAPHORE_LOG_FILE");
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
}
