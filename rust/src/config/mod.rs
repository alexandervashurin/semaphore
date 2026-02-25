//! Модуль конфигурации приложения

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow;

/// Основная конфигурация приложения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Хост веб-интерфейса
    pub web_host: Option<String>,

    /// Порт HTTP-сервера
    #[serde(default = "default_http_port")]
    pub http_port: u16,

    /// Путь к файлу конфигурации
    pub config_path: Option<PathBuf>,

    /// Тип базы данных
    #[serde(default = "default_db_dialect")]
    pub db_dialect: DbDialect,

    /// Путь к базе данных (для BoltDB/SQLite)
    pub db_path: Option<String>,

    /// Хост базы данных (для MySQL/PostgreSQL)
    pub db_host: Option<String>,

    /// Порт базы данных
    pub db_port: Option<u16>,

    /// Имя пользователя базы данных
    pub db_user: Option<String>,

    /// Пароль базы данных
    pub db_password: Option<String>,

    /// Имя базы данных
    pub db_name: Option<String>,

    /// Режим работы (сервер/раннер)
    #[serde(default)]
    pub mode: Mode,

    /// Токен раннера (для режима раннера)
    pub runner_token: Option<String>,

    /// URL сервера (для раннера)
    pub server_url: Option<String>,

    /// Максимальное количество параллельных задач
    #[serde(default = "default_max_parallel_tasks")]
    pub max_parallel_tasks: usize,

    /// Включить логирование в файл
    #[serde(default)]
    pub log_to_file: bool,

    /// Путь к файлу логов
    pub log_file_path: Option<PathBuf>,

    /// Уровень логирования
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

/// Тип базы данных
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DbDialect {
    #[serde(rename = "bolt")]
    Bolt,
    #[serde(rename = "sqlite")]
    SQLite,
    #[serde(rename = "mysql")]
    MySQL,
    #[serde(rename = "postgres")]
    PostgreSQL,
}

/// Режим работы приложения
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    #[serde(rename = "server")]
    Server,
    #[serde(rename = "runner")]
    Runner,
}

fn default_http_port() -> u16 {
    3000
}

fn default_db_dialect() -> DbDialect {
    DbDialect::Bolt
}

fn default_max_parallel_tasks() -> usize {
    10
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            web_host: None,
            http_port: default_http_port(),
            config_path: None,
            db_dialect: default_db_dialect(),
            db_path: None,
            db_host: None,
            db_port: None,
            db_user: None,
            db_password: None,
            db_name: None,
            mode: Mode::Server,
            runner_token: None,
            server_url: None,
            max_parallel_tasks: default_max_parallel_tasks(),
            log_to_file: false,
            log_file_path: None,
            log_level: default_log_level(),
        }
    }
}

impl Config {
    /// Загружает конфигурацию из переменных окружения
    pub fn from_env() -> Result<Self, anyhow::Error> {
        let _config = dotenvy::dotenv().ok();

        let mut cfg = Config::default();

        if let Some(val) = std::env::var("SEMAPHORE_WEB_HOST").ok() {
            cfg.web_host = Some(val);
        }

        if let Some(val) = std::env::var("SEMAPHORE_ADMIN").ok() {
            // Обработка администратора
            tracing::debug!("Администратор: {}", val);
        }

        if let Some(val) = std::env::var("SEMAPHORE_DB_DIALECT").ok() {
            cfg.db_dialect = match val.as_str() {
                "bolt" => DbDialect::Bolt,
                "sqlite" => DbDialect::SQLite,
                "mysql" => DbDialect::MySQL,
                "postgres" => DbDialect::PostgreSQL,
                _ => DbDialect::Bolt,
            };
        }

        if let Some(val) = std::env::var("SEMAPHORE_DB_PATH").ok() {
            cfg.db_path = Some(val);
        }

        if let Some(val) = std::env::var("SEMAPHORE_DB_HOST").ok() {
            cfg.db_host = Some(val);
        }

        if let Some(val) = std::env::var("SEMAPHORE_DB_PORT").ok() {
            cfg.db_port = val.parse().ok();
        }

        if let Some(val) = std::env::var("SEMAPHORE_DB_USER").ok() {
            cfg.db_user = Some(val);
        }

        if let Some(val) = std::env::var("SEMAPHORE_DB_PASS").ok() {
            cfg.db_password = Some(val);
        }

        if let Some(val) = std::env::var("SEMAPHORE_DB_NAME").ok() {
            cfg.db_name = Some(val);
        }

        Ok(cfg)
    }

    /// Получает строку подключения к базе данных
    pub fn database_url(&self) -> Result<String, String> {
        match self.db_dialect {
            DbDialect::Bolt | DbDialect::SQLite => {
                let path = self.db_path.as_ref()
                    .ok_or("Путь к базе данных не указан")?;
                Ok(format!("sqlite:{}", path))
            }
            DbDialect::MySQL => {
                let host = self.db_host.as_ref().ok_or("Хост MySQL не указан")?;
                let port = self.db_port.unwrap_or(3306);
                let user = self.db_user.as_ref().ok_or("Пользователь MySQL не указан")?;
                let password = self.db_password.as_ref().ok_or("Пароль MySQL не указан")?;
                let name = self.db_name.as_ref().ok_or("Имя базы данных не указано")?;
                Ok(format!("mysql://{}:{}@{}:{}/{}", user, password, host, port, name))
            }
            DbDialect::PostgreSQL => {
                let host = self.db_host.as_ref().ok_or("Хост PostgreSQL не указан")?;
                let port = self.db_port.unwrap_or(5432);
                let user = self.db_user.as_ref().ok_or("Пользователь PostgreSQL не указан")?;
                let password = self.db_password.as_ref().ok_or("Пароль PostgreSQL не указан")?;
                let name = self.db_name.as_ref().ok_or("Имя базы данных не указано")?;
                Ok(format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, name))
            }
        }
    }
}
