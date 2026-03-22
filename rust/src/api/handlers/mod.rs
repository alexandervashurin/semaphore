//! Handlers module - HTTP обработчики запросов
//!
//! Разбит на подмодули для лучшей организации кода

pub mod auth;
pub mod oidc;
pub mod metrics;
pub mod analytics;
#[cfg(test)]
mod tests;
pub mod users;
pub mod projects;
pub mod templates;
pub mod tasks;
pub mod inventory;
pub mod repository;
pub mod environment;
pub mod access_key;
pub mod totp;
pub mod mailer;
pub mod audit_log;
pub mod playbook;
pub mod playbook_runs;
pub mod workflow;
pub mod notification;
pub mod ai;
pub mod credential_type;
pub mod drift;
pub mod ldap_groups;
pub mod snapshot;
pub mod cost_estimate;

// Ре-экспорт всех handlers для удобства
pub use auth::*;
pub use oidc::*;
pub use users::*;
pub use projects::project::*;
pub use templates::*;
pub use tasks::*;
pub use inventory::*;
pub use repository::*;
pub use environment::*;
pub use access_key::*;
pub use totp::*;
pub use mailer::*;
pub use audit_log::*;
pub use playbook::*;
pub use analytics::*;
