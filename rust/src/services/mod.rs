//! Сервисы приложения

pub mod access_key_installation_service;
pub mod access_key_installer;
pub mod alert;
pub mod auto_backup;
pub mod backup;
pub mod cache_service;
pub mod executor;
pub mod exporter;
pub mod exporter_main;
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
    AccessKeyEncryptionService, AccessKeyInstallationServiceImpl,
    AccessKeyInstallationServiceTrait, AccessKeyServiceImpl, AccessKeyServiceTrait,
    GetAccessKeyOptions, SimpleEncryptionService,
};
pub use alert::AlertService;
pub use backup::{BackupDB, BackupEntity, BackupFormat, BackupSluggedEntity};
pub use cache_service::{CacheKeys, CacheService, CacheServiceConfig, SessionData};
pub use exporter::{
    init_project_exporters, new_key_mapper, ExporterChain, ProgressBar, TypeKeyMapper, ValueMap,
};
pub use local_job::LocalJob;
pub use metrics::{
    MetricsManager, ProjectTaskCounters, TaskCounters, TemplateTaskCounters, UserTaskCounters,
};
pub use restore::{generate_random_slug, RestoreDB, RestoreEntry};
pub use task_pool_status::TaskStatusMessage;
pub use task_pool_types::{RunningTask, TaskPool};
pub use task_runner::TaskRunner;
pub use webhook::{
    WebhookConfig, WebhookEvent, WebhookMetadata, WebhookResult, WebhookService, WebhookType,
};
pub use workflow_executor::{run_workflow as execute_workflow, WorkflowExecutor};

#[cfg(test)]
mod webhook_tests;

#[cfg(test)]
mod metrics_tests;
