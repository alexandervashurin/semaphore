//! Модели данных приложения
//!
//! Этот модуль содержит основные структуры данных, используемые в приложении.
//! Модели переведены из Go-версии Semaphore с сохранением совместимости.

pub mod user;
pub mod project;
pub mod task;
pub mod template;
pub mod inventory;
pub mod repository;
pub mod environment;
pub mod access_key;
pub mod integration;
pub mod schedule;
pub mod session;
pub mod token;
pub mod event;
pub mod runner;
pub mod view;
pub mod role;

#[cfg(test)]
mod tests;

// Ре-экспорт основных типов
pub use user::{User, UserTotp, UserEmailOtp, UserWithProjectRole};
pub use project::Project;
pub use task::{Task, TaskWithTpl, TaskOutput, TaskStage, TaskStageType};
pub use template::Template;
pub use inventory::Inventory;
pub use repository::Repository;
pub use environment::Environment;
pub use access_key::AccessKey;
pub use integration::{Integration, IntegrationExtractValue, IntegrationMatcher, IntegrationAlias};
pub use schedule::{Schedule, ScheduleWithTpl};
pub use session::Session;
pub use token::APIToken;
pub use event::Event;
pub use runner::{Runner, RunnerTag};
pub use view::View;
pub use role::Role;
