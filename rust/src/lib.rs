//! Semaphore UI - Современный веб-интерфейс для управления DevOps-инструментами
//!
//! Этот проект представляет собой систему автоматизации для Ansible, Terraform,
//! OpenTofu, Terragrunt, PowerShell и других инструментов.
//!
//! # Архитектура
//!
//! - **api** - HTTP API на базе Axum
//! - **db** - Слой доступа к данным (SQLite, MySQL, PostgreSQL, BoltDB)
//! - **db_lib** - Библиотека работы с БД (замена Go db_lib)
//! - **services** - Бизнес-логика
//! - **cli** - Интерфейс командной строки
//! - **models** - Модели данных
//! - **config** - Конфигурация приложения
//! - **ffi** - FFI модуль для вызова из Go (cgo)

pub mod api;
pub mod cli;
pub mod config;
pub mod db;
pub mod db_lib;
pub mod ffi;
pub mod models;
pub mod services;
pub mod utils;

mod error;
mod logging;

pub use error::{Error, Result};
pub use logging::init_logging;
