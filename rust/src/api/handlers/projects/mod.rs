//! Projects API Handlers Module
//!
//! Модуль обработчиков для проектов

pub mod keys;
pub mod schedules;
pub mod users;
pub mod templates;
pub mod tasks;
pub mod inventory;
pub mod repository;
pub mod environment;
pub mod integration;
pub mod views;
pub mod integration_alias;
pub mod secret_storages;
pub mod project;
pub mod backup_restore;
pub mod refs;

pub use keys::*;
pub use schedules::*;
pub use users::*;
pub use templates::*;
pub use tasks::*;
pub use inventory::*;
pub use repository::*;
pub use environment::*;
pub use integration::*;
pub use views::*;
pub use integration_alias::*;
pub use secret_storages::*;
pub use project::*;
pub use backup_restore::*;
pub use refs::*;
