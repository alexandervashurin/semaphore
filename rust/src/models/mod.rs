//! Модели данных приложения
//!
//! Этот модуль содержит основные структуры данных, используемые в приложении.
//! Модели переведены из Go-версии Velum с сохранением совместимости.

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
pub mod alias;
pub mod ansible;
pub mod backup_entity;
pub mod export_entity_type;
pub mod migration;
pub mod project_stats;
pub mod project_user;
pub mod task_params;
pub mod audit_log;
pub mod webhook;
pub mod analytics;
pub mod playbook;
pub mod playbook_run;
pub mod playbook_run_history;
pub mod workflow;
pub mod notification;
pub mod credential_type;
pub mod drift;
pub mod ldap_group;
pub mod snapshot;
pub mod cost_estimate;

#[cfg(test)]
mod tests;

// Ре-экспорт основных типов
pub use user::{User, UserTotp, UserEmailOtp, UserWithProjectRole, ProjectUserRole};
pub use project::Project;
pub use task::{Task, TaskWithTpl, TaskOutput, TaskStage, TaskStageType, TaskStageWithResult, TaskStageResult, AnsibleTaskParams, TerraformTaskParams, DefaultTaskParams};
pub use template::{Template, TemplateWithPerms, TemplateRolePerm, TemplateType, TemplateApp, TemplateFilter, SurveyVar, TemplateVaultRef};
pub use template_vault::TemplateVault;
pub use playbook::{Playbook, PlaybookCreate, PlaybookUpdate};
pub use playbook_run::{PlaybookRunRequest, PlaybookRunResult, AnsiblePlaybookParams};
pub use playbook_run_history::{PlaybookRun, PlaybookRunCreate, PlaybookRunUpdate, PlaybookRunStatus, PlaybookRunStats, PlaybookRunFilter};
pub use workflow::{Workflow, WorkflowCreate, WorkflowUpdate, WorkflowNode, WorkflowNodeCreate, WorkflowNodeUpdate, WorkflowEdge, WorkflowEdgeCreate, WorkflowRun as WorkflowRunModel, WorkflowFull, EdgeCondition};
pub use notification::{NotificationPolicy, NotificationPolicyCreate, NotificationPolicyUpdate, NotificationChannelType};
pub use totp_verification::TotpVerification;
pub use inventory::{Inventory, InventoryType};
pub use repository::{Repository, RepositoryType};
pub use environment::{Environment, EnvironmentSecret, EnvironmentSecretType, EnvironmentSecretValue};
pub use access_key::{AccessKey, AccessKeyOwner, AccessKeyType, SshKeyData, LoginPasswordData};
pub use integration::{Integration, IntegrationExtractValue, IntegrationMatcher, IntegrationAlias};
pub use schedule::{Schedule, ScheduleWithTpl};
pub use session::{Session, SessionVerificationMethod};
pub use token::APIToken;
pub use event::{Event, EventType};
pub use runner::Runner;
pub use view::View;
pub use role::Role;
pub use project_invite::{ProjectInvite, ProjectInviteWithUser};
pub use object_referrers::ObjectReferrers;
pub use crate::services::access_key_installation_service::GetAccessKeyOptions;
pub use option::OptionItem;
pub use secret_storage::{SecretStorage, SecretStorageType};
pub use hook::{Hook, HookType};
pub use terraform_inventory::{TerraformInventoryAlias, TerraformInventoryState, Alias as TerraformAlias};
pub use alias::Alias;
pub use ansible::{AnsiblePlaybook, AnsibleGalaxyRequirements, GalaxyRequirement};
pub use backup_entity::BackupEntity;
pub use export_entity_type::ExportEntityType;
pub use migration::Migration;
pub use project_stats::ProjectStats;
pub use project_user::ProjectUser;
pub use task_params::{AnsibleTaskParams as AnsibleTaskParamsStruct, TerraformTaskParams as TerraformTaskParamsStruct, DefaultTaskParams as DefaultTaskParamsStruct};
pub use audit_log::{AuditLog, AuditAction, AuditObjectType, AuditLevel, AuditDetails, AuditLogFilter, AuditLogResult};
pub use webhook::{Webhook, WebhookType, CreateWebhook, UpdateWebhook, TestWebhook, WebhookLog};
pub use analytics::{TaskStats, UserActivity, PerformanceMetrics, ResourceUsage, ChartData, TimeSeries, SystemStatus, TopItem, TopSlowTask, TopUser, AnalyticsQueryParams, ProjectAnalytics, RunnerMetrics, SystemMetrics};
pub use credential_type::{CredentialType, CredentialTypeCreate, CredentialTypeUpdate, CredentialInstance, CredentialInstanceCreate, CredentialField, CredentialInjector};
pub use ldap_group::{LdapGroupMapping, LdapGroupMappingCreate};
pub use snapshot::{TaskSnapshot, TaskSnapshotCreate, RollbackRequest};
pub use cost_estimate::{CostEstimate, CostEstimateCreate, CostSummary};

// Ре-экспорт RetrieveQueryParams из db::store
pub use crate::db::store::RetrieveQueryParams;
