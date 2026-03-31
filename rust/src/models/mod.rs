//! Модели данных приложения
//!
//! Этот модуль содержит основные структуры данных, используемые в приложении.
//! Модели переведены из Go-версии Velum с сохранением совместимости.

pub mod access_key;
pub mod alias;
pub mod analytics;
pub mod ansible;
pub mod audit_log;
pub mod backup_entity;
pub mod cost_estimate;
pub mod credential_type;
pub mod drift;
pub mod environment;
pub mod event;
pub mod export_entity_type;
pub mod hook;
pub mod integration;
pub mod inventory;
pub mod ldap_group;
pub mod migration;
pub mod notification;
pub mod object_referrers;
pub mod option;
pub mod organization;
pub mod playbook;
pub mod playbook_run;
pub mod playbook_run_history;
pub mod project;
pub mod project_invite;
pub mod project_stats;
pub mod project_user;
pub mod repository;
pub mod role;
pub mod runner;
pub mod schedule;
pub mod secret_storage;
pub mod session;
pub mod snapshot;
pub mod task;
pub mod task_params;
pub mod template;
pub mod template_vault;
pub mod terraform_inventory;
pub mod token;
pub mod totp_verification;
pub mod user;
pub mod view;
pub mod webhook;
pub mod workflow;

#[cfg(test)]
mod tests;

// Ре-экспорт основных типов
pub use access_key::{AccessKey, AccessKeyOwner, AccessKeyType, LoginPasswordData, SshKeyData};
pub use analytics::{
    AnalyticsQueryParams, ChartData, PerformanceMetrics, ProjectAnalytics, ResourceUsage,
    RunnerMetrics, SystemMetrics, SystemStatus, TaskStats, TimeSeries, TopItem, TopSlowTask,
    TopUser, UserActivity,
};
pub use ansible::{AnsibleGalaxyRequirements, AnsiblePlaybook, GalaxyRequirement};
pub use audit_log::{
    AuditAction, AuditDetails, AuditLevel, AuditLog, AuditLogFilter, AuditLogResult,
    AuditObjectType,
};
pub use backup_entity::BackupEntity;
pub use drift::{DriftConfig, DriftConfigCreate};
pub use environment::{
    Environment, EnvironmentSecret, EnvironmentSecretType, EnvironmentSecretValue,
};
pub use event::{Event, EventType};
pub use export_entity_type::ExportEntityType;
pub use hook::{Hook, HookType};
pub use integration::{Integration, IntegrationAlias, IntegrationExtractValue, IntegrationMatcher};
pub use inventory::{Inventory, InventoryType};
pub use ldap_group::{LdapGroupMapping, LdapGroupMappingCreate};
pub use migration::Migration;
pub use notification::{
    NotificationChannelType, NotificationPolicy, NotificationPolicyCreate, NotificationPolicyUpdate,
};
pub use object_referrers::ObjectReferrers;
pub use playbook::{Playbook, PlaybookCreate, PlaybookUpdate};
pub use playbook_run::{AnsiblePlaybookParams, PlaybookRunRequest, PlaybookRunResult};
pub use playbook_run_history::{
    PlaybookRun, PlaybookRunCreate, PlaybookRunFilter, PlaybookRunStats, PlaybookRunStatus,
    PlaybookRunUpdate,
};
pub use project::Project;
pub use project_invite::{ProjectInvite, ProjectInviteWithUser};
pub use project_stats::ProjectStats;
pub use project_user::ProjectUser;
pub use repository::{Repository, RepositoryType};
pub use role::Role;
pub use runner::Runner;
pub use schedule::{Schedule, ScheduleWithTpl};
pub use secret_storage::{SecretStorage, SecretStorageType};
pub use session::{Session, SessionVerificationMethod};
pub use task::{
    AnsibleTaskParams, DefaultTaskParams, Task, TaskOutput, TaskStage, TaskStageResult,
    TaskStageType, TaskStageWithResult, TaskWithTpl, TerraformTaskParams,
};
pub use task_params::{
    AnsibleTaskParams as AnsibleTaskParamsStruct, DefaultTaskParams as DefaultTaskParamsStruct,
    TerraformTaskParams as TerraformTaskParamsStruct,
};
pub use template::{
    SurveyVar, Template, TemplateApp, TemplateFilter, TemplateRolePerm, TemplateType,
    TemplateVaultRef, TemplateWithPerms,
};
pub use template_vault::TemplateVault;
pub use terraform_inventory::{Alias, TerraformInventoryAlias, TerraformInventoryState};
pub use token::APIToken;
pub use totp_verification::TotpVerification;
pub use user::{ProjectUserRole, User, UserEmailOtp, UserTotp, UserWithProjectRole};
pub use view::View;
pub use webhook::{CreateWebhook, TestWebhook, UpdateWebhook, Webhook, WebhookLog, WebhookType};
pub use workflow::{
    EdgeCondition, Workflow, WorkflowCreate, WorkflowEdge, WorkflowEdgeCreate, WorkflowFull,
    WorkflowNode, WorkflowNodeCreate, WorkflowNodeUpdate, WorkflowRun as WorkflowRunModel,
    WorkflowUpdate,
};

// Organization (Multi-Tenancy)
pub use organization::{
    Organization, OrganizationCreate, OrganizationUpdate, OrganizationUser, OrganizationUserCreate,
};

// Cost Estimate
pub use cost_estimate::{CostEstimate, CostEstimateCreate, CostSummary};

// Option
pub use option::OptionItem;

// Terraform State и Plan Approval
pub mod terraform_state;
pub use terraform_state::{
    LockInfo, StateDiff, StateDiffResource, TerraformState, TerraformStateLock,
    TerraformStateSummary,
};
pub mod plan_approval;
pub use plan_approval::{PlanReviewPayload, PlanStatus, TerraformPlan};

pub mod deployment_environment;
pub use deployment_environment::{
    DeploymentEnvironment, DeploymentEnvironmentCreate, DeploymentEnvironmentUpdate,
    DeploymentRecord,
};

pub mod task_structured_output;
pub use task_structured_output::{
    TaskOutputsMap, TaskStructuredOutput, TaskStructuredOutputBatch, TaskStructuredOutputCreate,
};

// Ре-экспорт RetrieveQueryParams из db::store
pub use crate::db::store::RetrieveQueryParams;
