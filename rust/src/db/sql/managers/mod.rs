//! Менеджеры хранилища данных
//!
//! Этот модуль содержит реализации трейтов менеджеров для SqlStore
//!
//! # Структура
//!
//! Каждый менеджер реализует свой трейт из `crate::db::store`:
//!
//! ## Основные менеджеры
//! - `ConnectionManager` - управление подключением к БД
//! - `MigrationManager` - управление миграциями схемы
//! - `OptionsManager` - управление опциями приложения
//!
//! ## Менеджеры сущностей
//! - `UserManager` - управление пользователями
//! - `ProjectStore` - управление проектами
//! - `TemplateManager` - управление шаблонами
//! - `TaskManager` - управление задачами
//! - `InventoryManager` - управление инвентарями
//! - `RepositoryManager` - управление репозиториями
//! - `EnvironmentManager` - управление окружениями
//! - `AccessKeyManager` - управление ключами доступа
//!
//! ## Дополнительные менеджеры
//! - `ScheduleManager` - управление расписаниями
//! - `SessionManager` - управление сессиями
//! - `TokenManager` - управление API токенами
//! - `EventManager` - управление событиями
//! - `HookManager` - управление хуками
//! - `RunnerManager` - управление раннерами
//! - `ViewManager` - управление представлениями
//! - `IntegrationManager` - управление интеграциями
//! - `ProjectInviteManager` - управление приглашениями
//! - `TerraformInventoryManager` - управление Terraform inventory
//! - `WebhookManager` - управление webhook

pub mod connection;
pub mod migration;
pub mod options;
pub mod user;
pub mod project;
pub mod template;
pub mod task;
pub mod inventory;
pub mod repository;
pub mod environment;
pub mod access_key;
pub mod schedule;
pub mod session;
pub mod token;
pub mod event;
pub mod hook;
pub mod runner;
pub mod view;
pub mod integration;
pub mod project_invite;
pub mod terraform;
pub mod webhook;
pub mod playbook;
pub mod playbook_run;
pub mod integration_matcher;

// Ре-экспорт трейтов для удобства
pub use connection::*;
pub use migration::*;
pub use options::*;
pub use user::*;
pub use project::*;
pub use template::*;
pub use task::*;
pub use inventory::*;
pub use repository::*;
pub use environment::*;
pub use access_key::*;
pub use schedule::*;
pub use session::*;
pub use token::*;
pub use event::*;
pub use hook::*;
pub use runner::*;
pub use view::*;
pub use integration::*;
pub use project_invite::*;
pub use terraform::*;
pub use webhook::*;
pub use playbook::*;

// ============================================================================
// Store trait implementation
// ============================================================================

use async_trait::async_trait;
use crate::db::Store;
use crate::db::sql::SqlStore;

#[async_trait]
impl Store for SqlStore {}
