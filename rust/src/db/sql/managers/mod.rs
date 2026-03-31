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

pub mod access_key;
pub mod connection;
pub mod cost_estimate;
pub mod credential_type;
pub mod deployment_environment;
pub mod drift;
pub mod environment;
pub mod event;
pub mod hook;
pub mod integration;
pub mod integration_matcher;
pub mod inventory;
pub mod ldap_group;
pub mod migration;
pub mod notification;
pub mod options;
pub mod organization;
pub mod plan_approval;
pub mod playbook;
pub mod playbook_run;
pub mod project;
pub mod project_invite;
pub mod repository;
pub mod runner;
pub mod schedule;
pub mod session;
pub mod snapshot;
pub mod state_backend;
pub mod task;
pub mod task_structured_output;
pub mod template;
pub mod terraform;
pub mod token;
pub mod user;
pub mod view;
pub mod webhook;
pub mod workflow;

// Ре-экспорт трейтов для удобства
pub use access_key::*;
pub use connection::*;
pub use environment::*;
pub use event::*;
pub use hook::*;
pub use integration::*;
pub use inventory::*;
pub use migration::*;
pub use options::*;
pub use playbook::*;
pub use project::*;
pub use project_invite::*;
pub use repository::*;
pub use runner::*;
pub use schedule::*;
pub use session::*;
pub use task::*;
pub use template::*;
pub use terraform::*;
pub use token::*;
pub use user::*;
pub use view::*;
pub use webhook::*;

// ============================================================================
// Store trait implementation
// ============================================================================

use crate::db::sql::SqlStore;
use crate::db::Store;
use async_trait::async_trait;

#[async_trait]
impl Store for SqlStore {}
