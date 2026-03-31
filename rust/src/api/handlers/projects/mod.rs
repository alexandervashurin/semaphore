//! Projects API Handlers Module
//!
//! Модуль обработчиков для проектов

pub mod backup_restore;
pub mod environment;
pub mod integration;
pub mod integration_alias;
pub mod inventory;
pub mod invites;
pub mod keys;
pub mod notifications;
pub mod project;
pub mod refs;
pub mod repository;
pub mod roles;
pub mod schedules;
pub mod secret_storages;
pub mod tasks;
pub mod templates;
pub mod users;
pub mod views;

pub use backup_restore::*;
pub use environment::*;
pub use integration::*;
pub use integration_alias::*;
pub use inventory::*;
pub use invites::*;
pub use keys::*;
pub use notifications::*;
pub use project::*;
pub use refs::*;
pub use repository::*;
pub use roles::*;
pub use schedules::*;
pub use secret_storages::*;
pub use tasks::*;
pub use templates::*;
pub use users::*;
pub use views::*;
