//! Handlers module - HTTP обработчики запросов
//!
//! Разбит на подмодули для лучшей организации кода

pub mod access_key;
pub mod ai;
pub mod analytics;
pub mod audit_log;
pub mod auth;
pub mod cost_estimate;
pub mod credential_type;
pub mod deployment_environment;
pub mod drift;
pub mod environment;
pub mod inventory;
pub mod kubernetes;
pub mod ldap_groups;
pub mod mailer;
pub mod metrics;
pub mod notification;
pub mod oidc;
pub mod organization;
pub mod plan_approval;
pub mod playbook;
pub mod playbook_runs;
pub mod projects;
pub mod repository;
pub mod snapshot;
pub mod state_backend;
pub mod task_structured_output;
pub mod tasks;
pub mod templates;
#[cfg(test)]
mod tests;
pub mod totp;
pub mod users;
pub mod workflow;

// Ре-экспорт всех handlers для удобства
pub use access_key::*;
pub use analytics::*;
pub use audit_log::*;
pub use auth::*;
pub use environment::*;
pub use inventory::*;
pub use kubernetes::*;
pub use mailer::*;
pub use oidc::*;
pub use playbook::*;
pub use projects::project::*;
pub use repository::*;
pub use tasks::*;
pub use templates::*;
pub use totp::*;
pub use users::*;
