//! Velum — Система автоматизации DevOps-задач с открытым исходным кодом
//!
//! # Описание
//!
//! Этот проект представляет собой систему автоматизации для Ansible, Terraform,
//! OpenTofu, Terragrunt, PowerShell, Bash и других инструментов.
//! Включает полный Kubernetes UI для управления кластерами.
//!
//! # Архитектура
//!
//! - **api** — HTTP API на базе Axum (REST + GraphQL + WebSocket + MCP)
//! - **db** — Слой доступа к данным (SQLite, MySQL, PostgreSQL)
//! - **db_lib** — Библиотека работы с БД (замена Go db_lib)
//! - **services** — Бизнес-логика (task runner, scheduler, notifications)
//! - **cli** — Интерфейс командной строки
//! - **models** — Модели данных
//! - **config** — Конфигурация приложения (HA, LDAP, OIDC)
//! - **ffi** — FFI модуль для вызова из Go (cgo)
//! - **plugins** — Система плагинов
//! - **kubernetes** — Kubernetes client (kube 0.98 + k8s-openapi 0.24)
//! - **grpc** — gRPC сервисы
//! - **cache** — Кэширование (Redis + in-memory)
//! - **validators** — Валидаторы (cron, YAML, etc.)
//!
//! # Пример использования
//!
//! ## Запуск сервера
//!
//! ```no_run
//! use velum::config::Config;
//! use velum::init_logging;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Инициализация логирования
//!     init_logging();
//!
//!     // Загрузка конфигурации из переменных окружения
//!     let config = Config::from_env();
//!
//!     // Запуск сервера
//!     velum::run_server(config).await.unwrap();
//! }
//! ```
//!
//! ## Использование Kubernetes клиента
//!
//! ```no_run
//! use velum::kubernetes::service::KubernetesClusterService;
//!
//! async fn list_pods() {
//!     let service = KubernetesClusterService::new("default".to_string());
//!     let pods = service.list_pods("kube-system").await.unwrap();
//!     for pod in pods {
//!         println!("Pod: {}", pod.name);
//!     }
//! }
//! ```
//!
//! # Переменные окружения
//!
//! ## Обязательные
//!
//! - `VELUM_DB_DIALECT` — тип БД (postgres, mysql, sqlite)
//! - `VELUM_DB_URL` — URL подключения к БД
//! - `VELUM_WEB_PATH` — путь к веб-интерфейсу
//! - `VELUM_JWT_SECRET` — секрет для JWT
//! - `VELUM_ADMIN` — имя администратора
//! - `VELUM_ADMIN_PASSWORD` — пароль администратора
//!
//! ## Опциональные
//!
//! - `VELUM_LDAP_*` — настройки LDAP
//! - `VELUM_OIDC_*` — настройки OIDC
//! - `RUST_LOG` — уровень логирования (info, debug, error)
//! - `VELUM_HA_REDIS_HOST` — Redis host для HA режима
//! - `VELUM_HA_REDIS_PORT` — Redis port для HA режима
//!
//! # Ресурсы
//!
//! - GitHub: <https://github.com/tnl-o/velum>
//! - Документация: <https://github.com/tnl-o/velum/tree/main/docs>

#![allow(unused_imports, unused_variables, dead_code, unused_mut)]

pub mod api;
pub mod cache;
pub mod cli;
pub mod config;
pub mod db;
pub mod db_lib;
pub mod ffi;
pub mod grpc;
pub mod kubernetes;
pub mod models;
pub mod plugins;
pub mod pro;
pub mod services;
pub mod utils;
pub mod validators;

mod error;
mod logging;

pub use error::{Error, Result};
pub use logging::init_logging;
