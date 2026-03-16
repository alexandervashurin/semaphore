//! Основной трейт хранилища данных
//!
//! Агрегирует все специализированные трейты для работы с данными

use crate::models::*;
use crate::models::audit_log::{AuditAction, AuditObjectType, AuditLevel, AuditLog, AuditLogResult};
use crate::models::playbook::{Playbook, PlaybookCreate, PlaybookUpdate};
use crate::models::playbook_run_history::{PlaybookRun, PlaybookRunCreate, PlaybookRunUpdate, PlaybookRunStatus, PlaybookRunStats, PlaybookRunFilter};
use crate::models::Hook;
use crate::error::Result;
use crate::services::task_logger::TaskStatus;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;

/// Параметры для выборки объектов
#[derive(Debug, Clone, Default)]
pub struct RetrieveQueryParams {
    pub offset: usize,
    pub count: Option<usize>,
    pub sort_by: Option<String>,
    pub sort_inverted: bool,
    pub filter: Option<String>,
}

/// Фильтр задач
#[derive(Debug, Clone, Default)]
pub struct TaskFilter {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub user_id: Option<i32>,
    pub status: Vec<crate::services::task_logger::TaskStatus>,
}

/// Менеджер подключений к базе данных
#[async_trait]
pub trait ConnectionManager: Send + Sync {
    /// Подключается к базе данных
    async fn connect(&self) -> Result<()>;

    /// Закрывает подключение
    async fn close(&self) -> Result<()>;

    /// Проверяет, является ли подключение постоянным
    fn is_permanent(&self) -> bool;
}

/// Менеджер миграций базы данных
#[async_trait]
pub trait MigrationManager: Send + Sync {
    /// Получает диалект базы данных
    fn get_dialect(&self) -> &str;

    /// Проверяет, инициализирована ли база данных
    async fn is_initialized(&self) -> Result<bool>;

    /// Применяет миграцию
    async fn apply_migration(&self, version: i64, name: String) -> Result<()>;

    /// Проверяет, применена ли миграция
    async fn is_migration_applied(&self, version: i64) -> Result<bool>;
}

/// Менеджер опций системы
#[async_trait]
pub trait OptionsManager: Send + Sync {
    async fn get_options(&self) -> Result<HashMap<String, String>>;
    async fn get_option(&self, key: &str) -> Result<Option<String>>;
    async fn set_option(&self, key: &str, value: &str) -> Result<()>;
    async fn delete_option(&self, key: &str) -> Result<()>;
}

/// Менеджер пользователей
#[async_trait]
pub trait UserManager: Send + Sync {
    async fn get_users(&self, params: RetrieveQueryParams) -> Result<Vec<User>>;
    async fn get_user(&self, user_id: i32) -> Result<User>;
    async fn get_user_by_login_or_email(&self, login: &str, email: &str) -> Result<User>;
    async fn create_user(&self, user: User, password: &str) -> Result<User>;
    async fn update_user(&self, user: User) -> Result<()>;
    async fn delete_user(&self, user_id: i32) -> Result<()>;
    async fn set_user_password(&self, user_id: i32, password: &str) -> Result<()>;
    async fn get_all_admins(&self) -> Result<Vec<User>>;
    async fn get_user_count(&self) -> Result<usize>;
    async fn get_project_users(&self, project_id: i32, params: RetrieveQueryParams) -> Result<Vec<ProjectUser>>;
    
    /// TOTP методы
    async fn get_user_totp(&self, user_id: i32) -> Result<Option<UserTotp>>;
    async fn set_user_totp(&self, user_id: i32, totp: &UserTotp) -> Result<()>;
    async fn delete_user_totp(&self, user_id: i32) -> Result<()>;
}

/// Хранилище проектов
#[async_trait]
pub trait ProjectStore: Send + Sync {
    async fn get_projects(&self, user_id: Option<i32>) -> Result<Vec<Project>>;
    async fn get_project(&self, project_id: i32) -> Result<Project>;
    async fn create_project(&self, project: Project) -> Result<Project>;
    async fn update_project(&self, project: Project) -> Result<()>;
    async fn delete_project(&self, project_id: i32) -> Result<()>;
    /// Добавляет пользователя в проект (связь project__user)
    async fn create_project_user(&self, project_user: crate::models::ProjectUser) -> Result<()>;
}

/// Менеджер шаблонов
#[async_trait]
pub trait TemplateManager: Send + Sync {
    async fn get_templates(&self, project_id: i32) -> Result<Vec<Template>>;
    async fn get_template(&self, project_id: i32, template_id: i32) -> Result<Template>;
    async fn create_template(&self, template: Template) -> Result<Template>;
    async fn update_template(&self, template: Template) -> Result<()>;
    async fn delete_template(&self, project_id: i32, template_id: i32) -> Result<()>;
}

/// Менеджер хуков
#[async_trait]
pub trait HookManager: Send + Sync {
    async fn get_hooks_by_template(&self, template_id: i32) -> Result<Vec<Hook>>;
}

/// Менеджер инвентарей
#[async_trait]
pub trait InventoryManager: Send + Sync {
    async fn get_inventories(&self, project_id: i32) -> Result<Vec<Inventory>>;
    async fn get_inventory(&self, project_id: i32, inventory_id: i32) -> Result<Inventory>;
    async fn create_inventory(&self, inventory: Inventory) -> Result<Inventory>;
    async fn update_inventory(&self, inventory: Inventory) -> Result<()>;
    async fn delete_inventory(&self, project_id: i32, inventory_id: i32) -> Result<()>;
}

/// Менеджер репозиториев
#[async_trait]
pub trait RepositoryManager: Send + Sync {
    async fn get_repositories(&self, project_id: i32) -> Result<Vec<Repository>>;
    async fn get_repository(&self, project_id: i32, repository_id: i32) -> Result<Repository>;
    async fn create_repository(&self, repository: Repository) -> Result<Repository>;
    async fn update_repository(&self, repository: Repository) -> Result<()>;
    async fn delete_repository(&self, project_id: i32, repository_id: i32) -> Result<()>;
}

/// Менеджер окружений
#[async_trait]
pub trait EnvironmentManager: Send + Sync {
    async fn get_environments(&self, project_id: i32) -> Result<Vec<Environment>>;
    async fn get_environment(&self, project_id: i32, environment_id: i32) -> Result<Environment>;
    async fn create_environment(&self, environment: Environment) -> Result<Environment>;
    async fn update_environment(&self, environment: Environment) -> Result<()>;
    async fn delete_environment(&self, project_id: i32, environment_id: i32) -> Result<()>;
}

/// Менеджер ключей доступа
#[async_trait]
pub trait AccessKeyManager: Send + Sync {
    async fn get_access_keys(&self, project_id: i32) -> Result<Vec<AccessKey>>;
    async fn get_access_key(&self, project_id: i32, key_id: i32) -> Result<AccessKey>;
    async fn create_access_key(&self, key: AccessKey) -> Result<AccessKey>;
    async fn update_access_key(&self, key: AccessKey) -> Result<()>;
    async fn delete_access_key(&self, project_id: i32, key_id: i32) -> Result<()>;
}

/// Менеджер задач
#[async_trait]
pub trait TaskManager: Send + Sync {
    async fn get_tasks(&self, project_id: i32, template_id: Option<i32>) -> Result<Vec<TaskWithTpl>>;
    async fn get_task(&self, project_id: i32, task_id: i32) -> Result<Task>;
    async fn create_task(&self, task: Task) -> Result<Task>;
    async fn update_task(&self, task: Task) -> Result<()>;
    async fn delete_task(&self, project_id: i32, task_id: i32) -> Result<()>;
    async fn get_task_outputs(&self, task_id: i32) -> Result<Vec<TaskOutput>>;
    async fn create_task_output(&self, output: TaskOutput) -> Result<TaskOutput>;
    async fn update_task_status(&self, project_id: i32, task_id: i32, status: TaskStatus) -> Result<()>;
}

/// Менеджер расписаний
#[async_trait]
pub trait ScheduleManager: Send + Sync {
    async fn get_schedules(&self, project_id: i32) -> Result<Vec<Schedule>>;
    async fn get_schedule(&self, project_id: i32, schedule_id: i32) -> Result<Schedule>;
    async fn create_schedule(&self, schedule: Schedule) -> Result<Schedule>;
    async fn update_schedule(&self, schedule: Schedule) -> Result<()>;
    async fn delete_schedule(&self, project_id: i32, schedule_id: i32) -> Result<()>;
    async fn set_schedule_active(&self, project_id: i32, schedule_id: i32, active: bool) -> Result<()>;
    async fn set_schedule_commit_hash(&self, project_id: i32, schedule_id: i32, hash: &str) -> Result<()>;
}

/// Менеджер сессий
#[async_trait]
pub trait SessionManager: Send + Sync {
    async fn get_session(&self, user_id: i32, session_id: i32) -> Result<Session>;
    async fn create_session(&self, session: Session) -> Result<Session>;
    async fn expire_session(&self, user_id: i32, session_id: i32) -> Result<()>;
    async fn verify_session(&self, user_id: i32, session_id: i32) -> Result<()>;
    async fn touch_session(&self, user_id: i32, session_id: i32) -> Result<()>;
}

/// Менеджер токенов
#[async_trait]
pub trait TokenManager: Send + Sync {
    async fn get_api_tokens(&self, user_id: i32) -> Result<Vec<APIToken>>;
    async fn create_api_token(&self, token: APIToken) -> Result<APIToken>;
    async fn get_api_token(&self, token_id: i32) -> Result<APIToken>;
    async fn expire_api_token(&self, user_id: i32, token_id: i32) -> Result<()>;
    async fn delete_api_token(&self, user_id: i32, token_id: i32) -> Result<()>;
}

/// Менеджер событий
#[async_trait]
pub trait EventManager: Send + Sync {
    async fn get_events(&self, project_id: Option<i32>, limit: usize) -> Result<Vec<Event>>;
    async fn create_event(&self, event: Event) -> Result<Event>;
}

/// Менеджер раннеров
#[async_trait]
pub trait RunnerManager: Send + Sync {
    async fn get_runners(&self, project_id: Option<i32>) -> Result<Vec<Runner>>;
    async fn get_runner(&self, runner_id: i32) -> Result<Runner>;
    async fn create_runner(&self, runner: Runner) -> Result<Runner>;
    async fn update_runner(&self, runner: Runner) -> Result<()>;
    async fn delete_runner(&self, runner_id: i32) -> Result<()>;
}

/// Менеджер представлений
#[async_trait]
pub trait ViewManager: Send + Sync {
    async fn get_views(&self, project_id: i32) -> Result<Vec<View>>;
    async fn get_view(&self, project_id: i32, view_id: i32) -> Result<View>;
    async fn create_view(&self, view: View) -> Result<View>;
    async fn update_view(&self, view: View) -> Result<()>;
    async fn delete_view(&self, project_id: i32, view_id: i32) -> Result<()>;
    async fn set_view_positions(&self, project_id: i32, positions: Vec<(i32, i32)>) -> Result<()>;
}

/// Менеджер интеграций
#[async_trait]
pub trait IntegrationManager: Send + Sync {
    async fn get_integrations(&self, project_id: i32) -> Result<Vec<Integration>>;
    async fn get_integration(&self, project_id: i32, integration_id: i32) -> Result<Integration>;
    async fn create_integration(&self, integration: Integration) -> Result<Integration>;
    async fn update_integration(&self, integration: Integration) -> Result<()>;
    async fn delete_integration(&self, project_id: i32, integration_id: i32) -> Result<()>;
}

/// Менеджер приглашений проекта
#[async_trait]
pub trait ProjectInviteManager: Send + Sync {
    async fn get_project_invites(&self, project_id: i32, params: RetrieveQueryParams) -> Result<Vec<ProjectInviteWithUser>>;
    async fn create_project_invite(&self, invite: ProjectInvite) -> Result<ProjectInvite>;
    async fn get_project_invite(&self, project_id: i32, invite_id: i32) -> Result<ProjectInvite>;
    async fn get_project_invite_by_token(&self, token: &str) -> Result<ProjectInvite>;
    async fn update_project_invite(&self, invite: ProjectInvite) -> Result<()>;
    async fn delete_project_invite(&self, project_id: i32, invite_id: i32) -> Result<()>;
}

/// Менеджер Terraform Inventory (PRO)
#[async_trait]
pub trait TerraformInventoryManager: Send + Sync {
    async fn create_terraform_inventory_alias(&self, alias: TerraformInventoryAlias) -> Result<TerraformInventoryAlias>;
    async fn update_terraform_inventory_alias(&self, alias: TerraformInventoryAlias) -> Result<()>;
    async fn get_terraform_inventory_alias_by_alias(&self, alias: &str) -> Result<TerraformInventoryAlias>;
    async fn get_terraform_inventory_alias(&self, project_id: i32, inventory_id: i32, alias_id: &str) -> Result<TerraformInventoryAlias>;
    async fn get_terraform_inventory_aliases(&self, project_id: i32, inventory_id: i32) -> Result<Vec<TerraformInventoryAlias>>;
    async fn delete_terraform_inventory_alias(&self, project_id: i32, inventory_id: i32, alias_id: &str) -> Result<()>;
    async fn get_terraform_inventory_states(&self, project_id: i32, inventory_id: i32, params: RetrieveQueryParams) -> Result<Vec<TerraformInventoryState>>;
    async fn create_terraform_inventory_state(&self, state: TerraformInventoryState) -> Result<TerraformInventoryState>;
    async fn delete_terraform_inventory_state(&self, project_id: i32, inventory_id: i32, state_id: i32) -> Result<()>;
    async fn get_terraform_inventory_state(&self, project_id: i32, inventory_id: i32, state_id: i32) -> Result<TerraformInventoryState>;
    async fn get_terraform_state_count(&self) -> Result<i32>;
}

/// Менеджер хранилищ секретов
#[async_trait]
pub trait SecretStorageManager: Send + Sync {
    async fn get_secret_storages(&self, project_id: i32) -> Result<Vec<SecretStorage>>;
    async fn get_secret_storage(&self, project_id: i32, storage_id: i32) -> Result<SecretStorage>;
    async fn create_secret_storage(&self, storage: SecretStorage) -> Result<SecretStorage>;
    async fn update_secret_storage(&self, storage: SecretStorage) -> Result<()>;
    async fn delete_secret_storage(&self, project_id: i32, storage_id: i32) -> Result<()>;
}

/// Менеджер audit log
#[async_trait]
pub trait AuditLogManager: Send + Sync {
    /// Создаёт новую запись audit log
    #[allow(clippy::too_many_arguments)]
    async fn create_audit_log(
        &self,
        project_id: Option<i64>,
        user_id: Option<i64>,
        username: Option<String>,
        action: &AuditAction,
        object_type: &AuditObjectType,
        object_id: Option<i64>,
        object_name: Option<String>,
        description: String,
        level: &AuditLevel,
        ip_address: Option<String>,
        user_agent: Option<String>,
        details: Option<serde_json::Value>,
    ) -> Result<AuditLog>;

    /// Получает запись audit log по ID
    async fn get_audit_log(&self, id: i64) -> Result<AuditLog>;

    /// Поиск записей audit log с фильтрацией и пагинацией
    async fn search_audit_logs(&self, filter: &AuditLogFilter) -> Result<AuditLogResult>;

    /// Получает записи audit log по project_id с пагинацией
    async fn get_audit_logs_by_project(&self, project_id: i64, limit: i64, offset: i64) -> Result<Vec<AuditLog>>;

    /// Получает записи audit log по user_id с пагинацией
    async fn get_audit_logs_by_user(&self, user_id: i64, limit: i64, offset: i64) -> Result<Vec<AuditLog>>;

    /// Получает записи audit log по action с пагинацией
    async fn get_audit_logs_by_action(&self, action: &AuditAction, limit: i64, offset: i64) -> Result<Vec<AuditLog>>;

    /// Удаляет старые записи audit log (до указанной даты)
    async fn delete_audit_logs_before(&self, before: DateTime<Utc>) -> Result<u64>;

    /// Очищает весь audit log
    async fn clear_audit_log(&self) -> Result<u64>;
}

/// Менеджер Playbook
#[async_trait]
pub trait PlaybookManager: Send + Sync {
    async fn get_playbooks(&self, project_id: i32) -> Result<Vec<Playbook>>;
    async fn get_playbook(&self, id: i32, project_id: i32) -> Result<Playbook>;
    async fn create_playbook(&self, project_id: i32, playbook: PlaybookCreate) -> Result<Playbook>;
    async fn update_playbook(&self, id: i32, project_id: i32, playbook: PlaybookUpdate) -> Result<Playbook>;
    async fn delete_playbook(&self, id: i32, project_id: i32) -> Result<()>;
}

/// Менеджер истории запусков Playbook
#[async_trait]
pub trait PlaybookRunManager: Send + Sync {
    async fn get_playbook_runs(&self, filter: PlaybookRunFilter) -> Result<Vec<PlaybookRun>>;
    async fn get_playbook_run(&self, id: i32, project_id: i32) -> Result<PlaybookRun>;
    async fn create_playbook_run(&self, run: PlaybookRunCreate) -> Result<PlaybookRun>;
    async fn update_playbook_run(&self, id: i32, project_id: i32, update: PlaybookRunUpdate) -> Result<PlaybookRun>;
    async fn delete_playbook_run(&self, id: i32, project_id: i32) -> Result<()>;
    async fn get_playbook_run_stats(&self, playbook_id: i32) -> Result<PlaybookRunStats>;
}

/// Менеджер webhook
#[async_trait]
pub trait WebhookManager: Send + Sync {
    /// Получает webhook по ID
    async fn get_webhook(&self, webhook_id: i64) -> Result<crate::models::webhook::Webhook>;

    /// Получает webhook проекта
    async fn get_webhooks_by_project(&self, project_id: i64) -> Result<Vec<crate::models::webhook::Webhook>>;

    /// Создаёт webhook
    async fn create_webhook(&self, webhook: crate::models::webhook::Webhook) -> Result<crate::models::webhook::Webhook>;

    /// Обновляет webhook
    async fn update_webhook(&self, webhook_id: i64, webhook: crate::models::webhook::UpdateWebhook) -> Result<crate::models::webhook::Webhook>;

    /// Удаляет webhook
    async fn delete_webhook(&self, webhook_id: i64) -> Result<()>;

    /// Получает логи webhook
    async fn get_webhook_logs(&self, webhook_id: i64) -> Result<Vec<crate::models::webhook::WebhookLog>>;

    /// Создаёт лог webhook
    async fn create_webhook_log(&self, log: crate::models::webhook::WebhookLog) -> Result<crate::models::webhook::WebhookLog>;
}

/// Менеджер матчеров интеграции
#[async_trait]
pub trait IntegrationMatcherManager: Send + Sync {
    async fn get_integration_matchers(&self, project_id: i32, integration_id: i32) -> Result<Vec<IntegrationMatcher>>;
    async fn create_integration_matcher(&self, matcher: IntegrationMatcher) -> Result<IntegrationMatcher>;
    async fn update_integration_matcher(&self, matcher: IntegrationMatcher) -> Result<()>;
    async fn delete_integration_matcher(&self, project_id: i32, integration_id: i32, matcher_id: i32) -> Result<()>;
}

/// Менеджер extract values интеграции
#[async_trait]
pub trait IntegrationExtractValueManager: Send + Sync {
    async fn get_integration_extract_values(&self, project_id: i32, integration_id: i32) -> Result<Vec<IntegrationExtractValue>>;
    async fn create_integration_extract_value(&self, value: IntegrationExtractValue) -> Result<IntegrationExtractValue>;
    async fn update_integration_extract_value(&self, value: IntegrationExtractValue) -> Result<()>;
    async fn delete_integration_extract_value(&self, project_id: i32, integration_id: i32, value_id: i32) -> Result<()>;
}

/// Менеджер ролей проекта
#[async_trait]
pub trait ProjectRoleManager: Send + Sync {
    async fn get_project_roles(&self, project_id: i32) -> Result<Vec<crate::models::Role>>;
    async fn create_project_role(&self, role: crate::models::Role) -> Result<crate::models::Role>;
    async fn update_project_role(&self, role: crate::models::Role) -> Result<()>;
    async fn delete_project_role(&self, project_id: i32, role_id: i32) -> Result<()>;
}

/// Менеджер Playbook
pub trait Store:
    ConnectionManager
    + MigrationManager
    + OptionsManager
    + UserManager
    + ProjectStore
    + TemplateManager
    + InventoryManager
    + RepositoryManager
    + EnvironmentManager
    + AccessKeyManager
    + TaskManager
    + ScheduleManager
    + SessionManager
    + TokenManager
    + EventManager
    + RunnerManager
    + ViewManager
    + IntegrationManager
    + ProjectInviteManager
    + TerraformInventoryManager
    + SecretStorageManager
    + HookManager
    + AuditLogManager
    + WebhookManager
    + PlaybookManager
    + PlaybookRunManager
    + IntegrationMatcherManager
    + IntegrationExtractValueManager
    + ProjectRoleManager
{
}
