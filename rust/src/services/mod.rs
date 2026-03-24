//! Сервисы приложения

pub mod access_key_installation_service;
pub mod access_key_installer;
pub mod alert;
pub mod auto_backup;
pub mod backup;
pub mod cache_service;
pub mod exporter;
pub mod exporter_main;
pub mod executor;
pub mod git_repository;
pub mod key_encryption;
pub mod local_job;
pub mod metrics;
pub mod playbook_run_service;
pub mod playbook_run_status_service;
pub mod playbook_sync_service;
pub mod restore;
pub mod scheduler;
pub mod ssh_agent;
pub mod ssh_auth_service;
pub mod task_execution;
pub mod task_logger;
pub mod task_pool;
pub mod task_pool_queue;
pub mod task_pool_runner;
pub mod task_pool_status;
pub mod task_pool_types;
pub mod task_runner;
pub mod telegram_bot;
pub mod totp;
pub mod webhook;
pub mod workflow_executor;

pub use access_key_installation_service::{
    AccessKeyEncryptionService, AccessKeyInstallationServiceTrait,
    AccessKeyInstallationServiceImpl, AccessKeyServiceTrait,
    AccessKeyServiceImpl, GetAccessKeyOptions, SimpleEncryptionService,
};
pub use cache_service::{CacheService, CacheServiceConfig, CacheKeys, SessionData};
pub use local_job::LocalJob;
pub use alert::AlertService;
pub use backup::{BackupFormat, BackupDB, BackupEntity, BackupSluggedEntity};
pub use restore::{RestoreDB, RestoreEntry, generate_random_slug};
pub use exporter::{ExporterChain, TypeKeyMapper, ValueMap, ProgressBar, init_project_exporters, new_key_mapper};
pub use task_pool_types::{TaskPool, RunningTask};
pub use task_pool_status::TaskStatusMessage;
pub use task_runner::TaskRunner;
pub use webhook::{WebhookService, WebhookConfig, WebhookEvent, WebhookResult, WebhookType, WebhookMetadata};
pub use metrics::{MetricsManager, TaskCounters, ProjectTaskCounters, TemplateTaskCounters, UserTaskCounters};
pub use workflow_executor::{WorkflowExecutor, run_workflow as execute_workflow};

#[cfg(test)]
mod webhook_tests;

#[cfg(test)]
mod metrics_tests;
