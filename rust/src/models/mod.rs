//! Модели данных приложения
//!
//! Этот модуль содержит основные структуры данных, используемые в приложении.
//! Модели переведены из Go-версии Semaphore с сохранением совместимости.

pub mod user;
pub mod project;
pub mod task;
pub mod template;
pub mod template_vault;
pub mod totp_verification;
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
pub mod project_invite;
pub mod object_referrers;
pub mod option;
pub mod secret_storage;
pub mod hook;
pub mod terraform_inventory;

#[cfg(test)]
mod tests;

// Ре-экспорт основных типов
pub use user::{User, UserTotp, UserEmailOtp, UserWithProjectRole};
pub use project::Project;
pub use task::{Task, TaskWithTpl, TaskOutput, TaskStage, TaskStageType, TaskStageWithResult, TaskStageResult, AnsibleTaskParams, TerraformTaskParams, DefaultTaskParams};
pub use template::{Template, TemplateWithPerms, TemplateRolePerm, TemplateType, TemplateApp, TemplateFilter};
pub use template_vault::TemplateVault;
pub use totp_verification::TotpVerification;
pub use inventory::{Inventory, InventoryType};
pub use repository::Repository;
pub use environment::{Environment, EnvironmentSecret, EnvironmentSecretType};
pub use access_key::{AccessKey, AccessKeyOwner, AccessKeyType, SshKeyData, LoginPasswordData};
pub use integration::{Integration, IntegrationExtractValue, IntegrationMatcher, IntegrationAlias};
pub use schedule::{Schedule, ScheduleWithTpl};
pub use session::{Session, SessionVerificationMethod};
pub use token::APIToken;
pub use event::{Event, EventType};
pub use runner::{Runner, RunnerTag};
pub use view::View;
pub use role::Role;
pub use project_invite::{ProjectInvite, ProjectInviteWithUser, RetrieveQueryParams};
pub use object_referrers::ObjectReferrers;
pub use crate::services::access_key_installation_service::GetAccessKeyOptions;
pub use option::{OptionItem, Option};
pub use secret_storage::{SecretStorage, SecretStorageType};
pub use hook::{Hook, HookType};
pub use terraform_inventory::{TerraformInventoryAlias, TerraformInventoryState, Alias};
