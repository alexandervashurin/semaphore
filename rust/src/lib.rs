//! Semaphore UI - Современный веб-интерфейс для управления DevOps-инструментами
//!
//! Этот проект представляет собой систему автоматизации для Ansible, Terraform,
//! OpenTofu, Terragrunt, PowerShell и других инструментов.
//!
//! # Архитектура
//!
//! - **api** - HTTP API на базе Axum
//! - **db** - Слой доступа к данным (SQLite, MySQL, PostgreSQL, BoltDB)
//! - **services** - Бизнес-логика
//! - **cli** - Интерфейс командной строки
//! - **models** - Модели данных
//! - **config** - Конфигурация приложения

pub mod api;
pub mod cli;
pub mod config;
pub mod db;
pub mod models;
pub mod services;
pub mod utils;

mod error;
mod logging;

pub use error::{Error, Result};
pub use logging::init_logging;
