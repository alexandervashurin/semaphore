//! Основной трейт хранилища данных
//!
//! Агрегирует все специализированные трейты для работы с данными

use crate::models::*;
use crate::error::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

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
}

/// Хранилище проектов
#[async_trait]
pub trait ProjectStore: Send + Sync {
    async fn get_projects(&self, user_id: Option<i32>) -> Result<Vec<Project>>;
    async fn get_project(&self, project_id: i32) -> Result<Project>;
    async fn create_project(&self, project: Project) -> Result<Project>;
    async fn update_project(&self, project: Project) -> Result<()>;
    async fn delete_project(&self, project_id: i32) -> Result<()>;
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
    async fn get_api_token(&self, token_id: &str) -> Result<APIToken>;
    async fn expire_api_token(&self, user_id: i32, token_id: &str) -> Result<()>;
    async fn delete_api_token(&self, user_id: i32, token_id: &str) -> Result<()>;
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

/// Основной трейт хранилища - агрегирует все менеджеры
#[async_trait]
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
{
}
