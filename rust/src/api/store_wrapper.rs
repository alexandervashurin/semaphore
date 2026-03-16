//! Wrapper for Arc<Box<dyn Store>> to provide Store methods

use crate::db::store::*;
use crate::models::*;
use crate::models::audit_log::{AuditAction, AuditObjectType, AuditLevel, AuditLog, AuditLogFilter, AuditLogResult};
use crate::models::webhook::{Webhook, UpdateWebhook, WebhookLog};
use crate::models::playbook_run_history::{PlaybookRun, PlaybookRunCreate, PlaybookRunUpdate, PlaybookRunStatus, PlaybookRunStats, PlaybookRunFilter};
use crate::error::Result;
use crate::services::task_logger::TaskStatus;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;

/// Wrapper для Arc<Box<dyn Store>>
#[derive(Clone)]
pub struct StoreWrapper {
    inner: Arc<Box<dyn Store>>,
}

impl StoreWrapper {
    pub fn new(store: Arc<Box<dyn Store>>) -> Self {
        Self { inner: store }
    }

    /// Получает доступ к внутреннему Store
    pub fn store(&self) -> &dyn Store {
        self.inner.as_ref().as_ref()
    }
}

#[async_trait]
impl ConnectionManager for StoreWrapper {
    async fn connect(&self) -> Result<()> {
        self.inner.as_ref().as_ref().connect().await
    }

    async fn close(&self) -> Result<()> {
        self.inner.as_ref().as_ref().close().await
    }

    fn is_permanent(&self) -> bool {
        self.inner.as_ref().as_ref().is_permanent()
    }
}

#[async_trait]
impl MigrationManager for StoreWrapper {
    fn get_dialect(&self) -> &str {
        self.inner.as_ref().as_ref().get_dialect()
    }

    async fn is_initialized(&self) -> Result<bool> {
        self.inner.as_ref().as_ref().is_initialized().await
    }

    async fn apply_migration(&self, version: i64, name: String) -> Result<()> {
        self.inner.as_ref().as_ref().apply_migration(version, name).await
    }

    async fn is_migration_applied(&self, version: i64) -> Result<bool> {
        self.inner.as_ref().as_ref().is_migration_applied(version).await
    }
}

#[async_trait]
impl OptionsManager for StoreWrapper {
    async fn get_options(&self) -> Result<HashMap<String, String>> {
        self.inner.as_ref().as_ref().get_options().await
    }

    async fn get_option(&self, key: &str) -> Result<Option<String>> {
        self.inner.as_ref().as_ref().get_option(key).await
    }

    async fn set_option(&self, key: &str, value: &str) -> Result<()> {
        self.inner.as_ref().as_ref().set_option(key, value).await
    }

    async fn delete_option(&self, key: &str) -> Result<()> {
        self.inner.as_ref().as_ref().delete_option(key).await
    }
}

#[async_trait]
impl UserManager for StoreWrapper {
    async fn get_users(&self, params: RetrieveQueryParams) -> Result<Vec<User>> {
        self.inner.as_ref().as_ref().get_users(params).await
    }

    async fn get_user(&self, user_id: i32) -> Result<User> {
        self.inner.as_ref().as_ref().get_user(user_id).await
    }

    async fn get_user_by_login_or_email(&self, login: &str, email: &str) -> Result<User> {
        self.inner.as_ref().as_ref().get_user_by_login_or_email(login, email).await
    }

    async fn create_user(&self, user: User, password: &str) -> Result<User> {
        self.inner.as_ref().as_ref().create_user(user, password).await
    }

    async fn update_user(&self, user: User) -> Result<()> {
        self.inner.as_ref().as_ref().update_user(user).await
    }

    async fn delete_user(&self, user_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_user(user_id).await
    }

    async fn set_user_password(&self, user_id: i32, password: &str) -> Result<()> {
        self.inner.as_ref().as_ref().set_user_password(user_id, password).await
    }

    async fn get_all_admins(&self) -> Result<Vec<User>> {
        self.inner.as_ref().as_ref().get_all_admins().await
    }

    async fn get_user_count(&self) -> Result<usize> {
        self.inner.as_ref().as_ref().get_user_count().await
    }

    async fn get_project_users(&self, project_id: i32, params: RetrieveQueryParams) -> Result<Vec<ProjectUser>> {
        self.inner.as_ref().as_ref().get_project_users(project_id, params).await
    }
    
    async fn get_user_totp(&self, user_id: i32) -> Result<Option<UserTotp>> {
        self.inner.as_ref().as_ref().get_user_totp(user_id).await
    }
    
    async fn set_user_totp(&self, user_id: i32, totp: &UserTotp) -> Result<()> {
        self.inner.as_ref().as_ref().set_user_totp(user_id, totp).await
    }
    
    async fn delete_user_totp(&self, user_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_user_totp(user_id).await
    }
}

#[async_trait]
impl ProjectStore for StoreWrapper {
    async fn get_projects(&self, user_id: Option<i32>) -> Result<Vec<Project>> {
        self.inner.as_ref().as_ref().get_projects(user_id).await
    }

    async fn get_project(&self, project_id: i32) -> Result<Project> {
        self.inner.as_ref().as_ref().get_project(project_id).await
    }

    async fn create_project(&self, project: Project) -> Result<Project> {
        self.inner.as_ref().as_ref().create_project(project).await
    }

    async fn update_project(&self, project: Project) -> Result<()> {
        self.inner.as_ref().as_ref().update_project(project).await
    }

    async fn delete_project(&self, project_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_project(project_id).await
    }

    async fn create_project_user(&self, project_user: crate::models::ProjectUser) -> Result<()> {
        self.inner.as_ref().as_ref().create_project_user(project_user).await
    }
}

#[async_trait]
impl TemplateManager for StoreWrapper {
    async fn get_templates(&self, project_id: i32) -> Result<Vec<Template>> {
        self.inner.as_ref().as_ref().get_templates(project_id).await
    }

    async fn get_template(&self, project_id: i32, template_id: i32) -> Result<Template> {
        self.inner.as_ref().as_ref().get_template(project_id, template_id).await
    }

    async fn create_template(&self, template: Template) -> Result<Template> {
        self.inner.as_ref().as_ref().create_template(template).await
    }

    async fn update_template(&self, template: Template) -> Result<()> {
        self.inner.as_ref().as_ref().update_template(template).await
    }

    async fn delete_template(&self, project_id: i32, template_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_template(project_id, template_id).await
    }
}

#[async_trait]
impl HookManager for StoreWrapper {
    async fn get_hooks_by_template(&self, template_id: i32) -> Result<Vec<Hook>> {
        self.inner.as_ref().as_ref().get_hooks_by_template(template_id).await
    }
}

#[async_trait]
impl InventoryManager for StoreWrapper {
    async fn get_inventories(&self, project_id: i32) -> Result<Vec<Inventory>> {
        self.inner.as_ref().as_ref().get_inventories(project_id).await
    }

    async fn get_inventory(&self, project_id: i32, inventory_id: i32) -> Result<Inventory> {
        self.inner.as_ref().as_ref().get_inventory(project_id, inventory_id).await
    }

    async fn create_inventory(&self, inventory: Inventory) -> Result<Inventory> {
        self.inner.as_ref().as_ref().create_inventory(inventory).await
    }

    async fn update_inventory(&self, inventory: Inventory) -> Result<()> {
        self.inner.as_ref().as_ref().update_inventory(inventory).await
    }

    async fn delete_inventory(&self, project_id: i32, inventory_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_inventory(project_id, inventory_id).await
    }
}

#[async_trait]
impl RepositoryManager for StoreWrapper {
    async fn get_repositories(&self, project_id: i32) -> Result<Vec<Repository>> {
        self.inner.as_ref().as_ref().get_repositories(project_id).await
    }

    async fn get_repository(&self, project_id: i32, repository_id: i32) -> Result<Repository> {
        self.inner.as_ref().as_ref().get_repository(project_id, repository_id).await
    }

    async fn create_repository(&self, repository: Repository) -> Result<Repository> {
        self.inner.as_ref().as_ref().create_repository(repository).await
    }

    async fn update_repository(&self, repository: Repository) -> Result<()> {
        self.inner.as_ref().as_ref().update_repository(repository).await
    }

    async fn delete_repository(&self, project_id: i32, repository_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_repository(project_id, repository_id).await
    }
}

#[async_trait]
impl EnvironmentManager for StoreWrapper {
    async fn get_environments(&self, project_id: i32) -> Result<Vec<Environment>> {
        self.inner.as_ref().as_ref().get_environments(project_id).await
    }

    async fn get_environment(&self, project_id: i32, environment_id: i32) -> Result<Environment> {
        self.inner.as_ref().as_ref().get_environment(project_id, environment_id).await
    }

    async fn create_environment(&self, environment: Environment) -> Result<Environment> {
        self.inner.as_ref().as_ref().create_environment(environment).await
    }

    async fn update_environment(&self, environment: Environment) -> Result<()> {
        self.inner.as_ref().as_ref().update_environment(environment).await
    }

    async fn delete_environment(&self, project_id: i32, environment_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_environment(project_id, environment_id).await
    }
}

#[async_trait]
impl AccessKeyManager for StoreWrapper {
    async fn get_access_keys(&self, project_id: i32) -> Result<Vec<AccessKey>> {
        self.inner.as_ref().as_ref().get_access_keys(project_id).await
    }

    async fn get_access_key(&self, project_id: i32, access_key_id: i32) -> Result<AccessKey> {
        self.inner.as_ref().as_ref().get_access_key(project_id, access_key_id).await
    }

    async fn create_access_key(&self, access_key: AccessKey) -> Result<AccessKey> {
        self.inner.as_ref().as_ref().create_access_key(access_key).await
    }

    async fn update_access_key(&self, access_key: AccessKey) -> Result<()> {
        self.inner.as_ref().as_ref().update_access_key(access_key).await
    }

    async fn delete_access_key(&self, project_id: i32, access_key_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_access_key(project_id, access_key_id).await
    }
}

#[async_trait]
impl TaskManager for StoreWrapper {
    async fn get_tasks(&self, project_id: i32, template_id: Option<i32>) -> Result<Vec<TaskWithTpl>> {
        self.inner.as_ref().as_ref().get_tasks(project_id, template_id).await
    }

    async fn get_task(&self, project_id: i32, task_id: i32) -> Result<Task> {
        self.inner.as_ref().as_ref().get_task(project_id, task_id).await
    }

    async fn create_task(&self, task: Task) -> Result<Task> {
        self.inner.as_ref().as_ref().create_task(task).await
    }

    async fn update_task(&self, task: Task) -> Result<()> {
        self.inner.as_ref().as_ref().update_task(task).await
    }

    async fn delete_task(&self, project_id: i32, task_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_task(project_id, task_id).await
    }

    async fn get_task_outputs(&self, task_id: i32) -> Result<Vec<TaskOutput>> {
        self.inner.as_ref().as_ref().get_task_outputs(task_id).await
    }

    async fn create_task_output(&self, output: TaskOutput) -> Result<TaskOutput> {
        self.inner.as_ref().as_ref().create_task_output(output).await
    }

    async fn update_task_status(&self, project_id: i32, task_id: i32, status: TaskStatus) -> Result<()> {
        self.inner.as_ref().as_ref().update_task_status(project_id, task_id, status).await
    }
}

#[async_trait]
impl ScheduleManager for StoreWrapper {
    async fn get_schedules(&self, project_id: i32) -> Result<Vec<Schedule>> {
        self.inner.as_ref().as_ref().get_schedules(project_id).await
    }

    async fn get_schedule(&self, project_id: i32, schedule_id: i32) -> Result<Schedule> {
        self.inner.as_ref().as_ref().get_schedule(project_id, schedule_id).await
    }

    async fn create_schedule(&self, schedule: Schedule) -> Result<Schedule> {
        self.inner.as_ref().as_ref().create_schedule(schedule).await
    }

    async fn update_schedule(&self, schedule: Schedule) -> Result<()> {
        self.inner.as_ref().as_ref().update_schedule(schedule).await
    }

    async fn delete_schedule(&self, project_id: i32, schedule_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_schedule(project_id, schedule_id).await
    }

    async fn set_schedule_active(&self, project_id: i32, schedule_id: i32, active: bool) -> Result<()> {
        self.inner.as_ref().as_ref().set_schedule_active(project_id, schedule_id, active).await
    }

    async fn set_schedule_commit_hash(&self, project_id: i32, schedule_id: i32, hash: &str) -> Result<()> {
        self.inner.as_ref().as_ref().set_schedule_commit_hash(project_id, schedule_id, hash).await
    }
}

#[async_trait]
impl SessionManager for StoreWrapper {
    async fn get_session(&self, user_id: i32, session_id: i32) -> Result<Session> {
        self.inner.as_ref().as_ref().get_session(user_id, session_id).await
    }

    async fn create_session(&self, session: Session) -> Result<Session> {
        self.inner.as_ref().as_ref().create_session(session).await
    }

    async fn expire_session(&self, user_id: i32, session_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().expire_session(user_id, session_id).await
    }

    async fn verify_session(&self, user_id: i32, session_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().verify_session(user_id, session_id).await
    }

    async fn touch_session(&self, user_id: i32, session_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().touch_session(user_id, session_id).await
    }
}

#[async_trait]
impl TokenManager for StoreWrapper {
    async fn get_api_tokens(&self, user_id: i32) -> Result<Vec<APIToken>> {
        self.inner.as_ref().as_ref().get_api_tokens(user_id).await
    }

    async fn create_api_token(&self, token: APIToken) -> Result<APIToken> {
        self.inner.as_ref().as_ref().create_api_token(token).await
    }

    async fn get_api_token(&self, token_id: i32) -> Result<APIToken> {
        self.inner.as_ref().as_ref().get_api_token(token_id).await
    }

    async fn expire_api_token(&self, user_id: i32, token_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().expire_api_token(user_id, token_id).await
    }

    async fn delete_api_token(&self, user_id: i32, token_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_api_token(user_id, token_id).await
    }
}

#[async_trait]
impl EventManager for StoreWrapper {
    async fn get_events(&self, project_id: Option<i32>, limit: usize) -> Result<Vec<Event>> {
        self.inner.as_ref().as_ref().get_events(project_id, limit).await
    }

    async fn create_event(&self, event: Event) -> Result<Event> {
        self.inner.as_ref().as_ref().create_event(event).await
    }
}

#[async_trait]
impl RunnerManager for StoreWrapper {
    async fn get_runners(&self, project_id: Option<i32>) -> Result<Vec<Runner>> {
        self.inner.as_ref().as_ref().get_runners(project_id).await
    }

    async fn get_runner(&self, runner_id: i32) -> Result<Runner> {
        self.inner.as_ref().as_ref().get_runner(runner_id).await
    }

    async fn create_runner(&self, runner: Runner) -> Result<Runner> {
        self.inner.as_ref().as_ref().create_runner(runner).await
    }

    async fn update_runner(&self, runner: Runner) -> Result<()> {
        self.inner.as_ref().as_ref().update_runner(runner).await
    }

    async fn delete_runner(&self, runner_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_runner(runner_id).await
    }
}

#[async_trait]
impl ViewManager for StoreWrapper {
    async fn get_views(&self, project_id: i32) -> Result<Vec<View>> {
        self.inner.as_ref().as_ref().get_views(project_id).await
    }

    async fn get_view(&self, project_id: i32, view_id: i32) -> Result<View> {
        self.inner.as_ref().as_ref().get_view(project_id, view_id).await
    }

    async fn create_view(&self, view: View) -> Result<View> {
        self.inner.as_ref().as_ref().create_view(view).await
    }

    async fn update_view(&self, view: View) -> Result<()> {
        self.inner.as_ref().as_ref().update_view(view).await
    }

    async fn delete_view(&self, project_id: i32, view_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_view(project_id, view_id).await
    }
    
    async fn set_view_positions(&self, project_id: i32, positions: Vec<(i32, i32)>) -> Result<()> {
        self.inner.as_ref().as_ref().set_view_positions(project_id, positions).await
    }
}

#[async_trait]
impl IntegrationManager for StoreWrapper {
    async fn get_integrations(&self, project_id: i32) -> Result<Vec<Integration>> {
        self.inner.as_ref().as_ref().get_integrations(project_id).await
    }

    async fn get_integration(&self, project_id: i32, integration_id: i32) -> Result<Integration> {
        self.inner.as_ref().as_ref().get_integration(project_id, integration_id).await
    }

    async fn create_integration(&self, integration: Integration) -> Result<Integration> {
        self.inner.as_ref().as_ref().create_integration(integration).await
    }

    async fn update_integration(&self, integration: Integration) -> Result<()> {
        self.inner.as_ref().as_ref().update_integration(integration).await
    }

    async fn delete_integration(&self, project_id: i32, integration_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_integration(project_id, integration_id).await
    }
}

#[async_trait]
impl ProjectInviteManager for StoreWrapper {
    async fn get_project_invites(&self, project_id: i32, params: RetrieveQueryParams) -> Result<Vec<ProjectInviteWithUser>> {
        self.inner.as_ref().as_ref().get_project_invites(project_id, params).await
    }

    async fn create_project_invite(&self, invite: ProjectInvite) -> Result<ProjectInvite> {
        self.inner.as_ref().as_ref().create_project_invite(invite).await
    }

    async fn get_project_invite(&self, project_id: i32, invite_id: i32) -> Result<ProjectInvite> {
        self.inner.as_ref().as_ref().get_project_invite(project_id, invite_id).await
    }

    async fn get_project_invite_by_token(&self, token: &str) -> Result<ProjectInvite> {
        self.inner.as_ref().as_ref().get_project_invite_by_token(token).await
    }

    async fn update_project_invite(&self, invite: ProjectInvite) -> Result<()> {
        self.inner.as_ref().as_ref().update_project_invite(invite).await
    }

    async fn delete_project_invite(&self, project_id: i32, invite_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_project_invite(project_id, invite_id).await
    }
}

#[async_trait]
impl TerraformInventoryManager for StoreWrapper {
    async fn create_terraform_inventory_alias(&self, alias: TerraformInventoryAlias) -> Result<TerraformInventoryAlias> {
        self.inner.as_ref().as_ref().create_terraform_inventory_alias(alias).await
    }

    async fn update_terraform_inventory_alias(&self, alias: TerraformInventoryAlias) -> Result<()> {
        self.inner.as_ref().as_ref().update_terraform_inventory_alias(alias).await
    }

    async fn get_terraform_inventory_alias_by_alias(&self, alias: &str) -> Result<TerraformInventoryAlias> {
        self.inner.as_ref().as_ref().get_terraform_inventory_alias_by_alias(alias).await
    }

    async fn get_terraform_inventory_alias(&self, project_id: i32, inventory_id: i32, alias_id: &str) -> Result<TerraformInventoryAlias> {
        self.inner.as_ref().as_ref().get_terraform_inventory_alias(project_id, inventory_id, alias_id).await
    }

    async fn get_terraform_inventory_aliases(&self, project_id: i32, inventory_id: i32) -> Result<Vec<TerraformInventoryAlias>> {
        self.inner.as_ref().as_ref().get_terraform_inventory_aliases(project_id, inventory_id).await
    }

    async fn delete_terraform_inventory_alias(&self, project_id: i32, inventory_id: i32, alias_id: &str) -> Result<()> {
        self.inner.as_ref().as_ref().delete_terraform_inventory_alias(project_id, inventory_id, alias_id).await
    }

    async fn get_terraform_inventory_states(&self, project_id: i32, inventory_id: i32, params: RetrieveQueryParams) -> Result<Vec<TerraformInventoryState>> {
        self.inner.as_ref().as_ref().get_terraform_inventory_states(project_id, inventory_id, params).await
    }

    async fn create_terraform_inventory_state(&self, state: TerraformInventoryState) -> Result<TerraformInventoryState> {
        self.inner.as_ref().as_ref().create_terraform_inventory_state(state).await
    }

    async fn delete_terraform_inventory_state(&self, project_id: i32, inventory_id: i32, state_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_terraform_inventory_state(project_id, inventory_id, state_id).await
    }

    async fn get_terraform_inventory_state(&self, project_id: i32, inventory_id: i32, state_id: i32) -> Result<TerraformInventoryState> {
        self.inner.as_ref().as_ref().get_terraform_inventory_state(project_id, inventory_id, state_id).await
    }

    async fn get_terraform_state_count(&self) -> Result<i32> {
        self.inner.as_ref().as_ref().get_terraform_state_count().await
    }
}

#[async_trait]
impl SecretStorageManager for StoreWrapper {
    async fn get_secret_storages(&self, project_id: i32) -> Result<Vec<SecretStorage>> {
        self.inner.as_ref().as_ref().get_secret_storages(project_id).await
    }

    async fn get_secret_storage(&self, project_id: i32, storage_id: i32) -> Result<SecretStorage> {
        self.inner.as_ref().as_ref().get_secret_storage(project_id, storage_id).await
    }

    async fn create_secret_storage(&self, storage: SecretStorage) -> Result<SecretStorage> {
        self.inner.as_ref().as_ref().create_secret_storage(storage).await
    }

    async fn update_secret_storage(&self, storage: SecretStorage) -> Result<()> {
        self.inner.as_ref().as_ref().update_secret_storage(storage).await
    }

    async fn delete_secret_storage(&self, project_id: i32, storage_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_secret_storage(project_id, storage_id).await
    }
}

#[async_trait]
impl AuditLogManager for StoreWrapper {
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
    ) -> Result<AuditLog> {
        self.inner.as_ref().as_ref().create_audit_log(
            project_id,
            user_id,
            username,
            action,
            object_type,
            object_id,
            object_name,
            description,
            level,
            ip_address,
            user_agent,
            details,
        ).await
    }

    async fn get_audit_log(&self, id: i64) -> Result<AuditLog> {
        self.inner.as_ref().as_ref().get_audit_log(id).await
    }

    async fn search_audit_logs(&self, filter: &AuditLogFilter) -> Result<AuditLogResult> {
        self.inner.as_ref().as_ref().search_audit_logs(filter).await
    }

    async fn get_audit_logs_by_project(&self, project_id: i64, limit: i64, offset: i64) -> Result<Vec<AuditLog>> {
        self.inner.as_ref().as_ref().get_audit_logs_by_project(project_id, limit, offset).await
    }

    async fn get_audit_logs_by_user(&self, user_id: i64, limit: i64, offset: i64) -> Result<Vec<AuditLog>> {
        self.inner.as_ref().as_ref().get_audit_logs_by_user(user_id, limit, offset).await
    }

    async fn get_audit_logs_by_action(&self, action: &AuditAction, limit: i64, offset: i64) -> Result<Vec<AuditLog>> {
        self.inner.as_ref().as_ref().get_audit_logs_by_action(action, limit, offset).await
    }

    async fn delete_audit_logs_before(&self, before: DateTime<Utc>) -> Result<u64> {
        self.inner.as_ref().as_ref().delete_audit_logs_before(before).await
    }

    async fn clear_audit_log(&self) -> Result<u64> {
        self.inner.as_ref().as_ref().clear_audit_log().await
    }
}

#[async_trait]
impl crate::db::store::WebhookManager for StoreWrapper {
    async fn get_webhook(&self, webhook_id: i64) -> Result<Webhook> {
        self.inner.as_ref().as_ref().get_webhook(webhook_id).await
    }

    async fn get_webhooks_by_project(&self, project_id: i64) -> Result<Vec<Webhook>> {
        self.inner.as_ref().as_ref().get_webhooks_by_project(project_id).await
    }

    async fn create_webhook(&self, webhook: Webhook) -> Result<Webhook> {
        self.inner.as_ref().as_ref().create_webhook(webhook).await
    }

    async fn update_webhook(&self, webhook_id: i64, webhook: UpdateWebhook) -> Result<Webhook> {
        self.inner.as_ref().as_ref().update_webhook(webhook_id, webhook).await
    }

    async fn delete_webhook(&self, webhook_id: i64) -> Result<()> {
        self.inner.as_ref().as_ref().delete_webhook(webhook_id).await
    }

    async fn get_webhook_logs(&self, webhook_id: i64) -> Result<Vec<WebhookLog>> {
        self.inner.as_ref().as_ref().get_webhook_logs(webhook_id).await
    }

    async fn create_webhook_log(&self, log: WebhookLog) -> Result<WebhookLog> {
        self.inner.as_ref().as_ref().create_webhook_log(log).await
    }
}


#[async_trait]
impl crate::db::store::PlaybookManager for StoreWrapper {
    async fn get_playbooks(&self, project_id: i32) -> Result<Vec<crate::models::Playbook>> {
        self.inner.as_ref().as_ref().get_playbooks(project_id).await
    }

    async fn get_playbook(&self, id: i32, project_id: i32) -> Result<crate::models::Playbook> {
        self.inner.as_ref().as_ref().get_playbook(id, project_id).await
    }

    async fn create_playbook(&self, project_id: i32, playbook: crate::models::PlaybookCreate) -> Result<crate::models::Playbook> {
        self.inner.as_ref().as_ref().create_playbook(project_id, playbook).await
    }

    async fn update_playbook(&self, id: i32, project_id: i32, playbook: crate::models::PlaybookUpdate) -> Result<crate::models::Playbook> {
        self.inner.as_ref().as_ref().update_playbook(id, project_id, playbook).await
    }

    async fn delete_playbook(&self, id: i32, project_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_playbook(id, project_id).await
    }
}

#[async_trait]
impl crate::db::store::PlaybookRunManager for StoreWrapper {
    async fn get_playbook_runs(&self, filter: PlaybookRunFilter) -> Result<Vec<PlaybookRun>> {
        self.inner.as_ref().as_ref().get_playbook_runs(filter).await
    }

    async fn get_playbook_run(&self, id: i32, project_id: i32) -> Result<PlaybookRun> {
        self.inner.as_ref().as_ref().get_playbook_run(id, project_id).await
    }

    async fn create_playbook_run(&self, run: PlaybookRunCreate) -> Result<PlaybookRun> {
        self.inner.as_ref().as_ref().create_playbook_run(run).await
    }

    async fn update_playbook_run(&self, id: i32, project_id: i32, update: PlaybookRunUpdate) -> Result<PlaybookRun> {
        self.inner.as_ref().as_ref().update_playbook_run(id, project_id, update).await
    }

    async fn delete_playbook_run(&self, id: i32, project_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_playbook_run(id, project_id).await
    }

    async fn get_playbook_run_stats(&self, playbook_id: i32) -> Result<PlaybookRunStats> {
        self.inner.as_ref().as_ref().get_playbook_run_stats(playbook_id).await
    }
}

#[async_trait]
impl IntegrationMatcherManager for StoreWrapper {
    async fn get_integration_matchers(&self, project_id: i32, integration_id: i32) -> Result<Vec<IntegrationMatcher>> {
        self.inner.as_ref().as_ref().get_integration_matchers(project_id, integration_id).await
    }
    async fn create_integration_matcher(&self, matcher: IntegrationMatcher) -> Result<IntegrationMatcher> {
        self.inner.as_ref().as_ref().create_integration_matcher(matcher).await
    }
    async fn update_integration_matcher(&self, matcher: IntegrationMatcher) -> Result<()> {
        self.inner.as_ref().as_ref().update_integration_matcher(matcher).await
    }
    async fn delete_integration_matcher(&self, project_id: i32, integration_id: i32, matcher_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_integration_matcher(project_id, integration_id, matcher_id).await
    }
}

#[async_trait]
impl IntegrationExtractValueManager for StoreWrapper {
    async fn get_integration_extract_values(&self, project_id: i32, integration_id: i32) -> Result<Vec<IntegrationExtractValue>> {
        self.inner.as_ref().as_ref().get_integration_extract_values(project_id, integration_id).await
    }
    async fn create_integration_extract_value(&self, value: IntegrationExtractValue) -> Result<IntegrationExtractValue> {
        self.inner.as_ref().as_ref().create_integration_extract_value(value).await
    }
    async fn update_integration_extract_value(&self, value: IntegrationExtractValue) -> Result<()> {
        self.inner.as_ref().as_ref().update_integration_extract_value(value).await
    }
    async fn delete_integration_extract_value(&self, project_id: i32, integration_id: i32, value_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_integration_extract_value(project_id, integration_id, value_id).await
    }
}

#[async_trait]
impl ProjectRoleManager for StoreWrapper {
    async fn get_project_roles(&self, project_id: i32) -> Result<Vec<crate::models::Role>> {
        self.inner.as_ref().as_ref().get_project_roles(project_id).await
    }
    async fn create_project_role(&self, role: crate::models::Role) -> Result<crate::models::Role> {
        self.inner.as_ref().as_ref().create_project_role(role).await
    }
    async fn update_project_role(&self, role: crate::models::Role) -> Result<()> {
        self.inner.as_ref().as_ref().update_project_role(role).await
    }
    async fn delete_project_role(&self, project_id: i32, role_id: i32) -> Result<()> {
        self.inner.as_ref().as_ref().delete_project_role(project_id, role_id).await
    }
}

#[async_trait]
impl Store for StoreWrapper {}
