//! Handlers module - HTTP обработчики запросов
//!
//! Разбит на подмодули для лучшей организации кода

pub mod auth;
pub mod users;
pub mod projects;
pub mod templates;
pub mod tasks;
pub mod inventory;
pub mod repository;
pub mod environment;
pub mod access_key;
pub mod totp;

// Ре-экспорт всех handlers для удобства
pub use auth::*;
pub use users::*;
pub use projects::*;
pub use templates::*;
pub use tasks::*;
pub use inventory::*;
pub use repository::*;
pub use environment::*;
pub use access_key::*;
pub use totp::*;
