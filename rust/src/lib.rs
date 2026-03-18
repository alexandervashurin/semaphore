//! Velum UI - Современный веб-интерфейс для управления DevOps-инструментами
#![allow(unused_imports, unused_variables, dead_code, unused_mut)]
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
//! - **plugins** - Система плагинов

pub mod api;
pub mod cli;
pub mod config;
pub mod db;
pub mod db_lib;
pub mod ffi;
pub mod models;
pub mod pro;
pub mod services;
pub mod utils;
pub mod plugins;
pub mod cache;
pub mod grpc;
pub mod kubernetes;
pub mod validators;

mod error;
mod logging;

pub use error::{Error, Result};
pub use logging::init_logging;
