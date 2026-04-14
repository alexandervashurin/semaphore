//! Wrapper for `Arc<Box<dyn Store>>` to provide Store methods

use crate::db::store::*;
use crate::error::Result;
use crate::models::audit_log::{
    AuditAction, AuditLevel, AuditLog, AuditLogFilter, AuditLogResult, AuditObjectType,
};
use crate::models::notification::{
    NotificationPolicy, NotificationPolicyCreate, NotificationPolicyUpdate,
};
use crate::models::playbook_run_history::{
    PlaybookRun, PlaybookRunCreate, PlaybookRunFilter, PlaybookRunStats, PlaybookRunStatus,
    PlaybookRunUpdate,
};
use crate::models::webhook::{UpdateWebhook, Webhook, WebhookLog};
use crate::models::workflow::{
    Workflow, WorkflowCreate, WorkflowEdge, WorkflowEdgeCreate, WorkflowNode, WorkflowNodeCreate,
    WorkflowNodeUpdate, WorkflowRun, WorkflowUpdate,
};
use crate::models::*;
use crate::services::task_logger::TaskStatus;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;

/// Wrapper для Arc<dyn Store + Send + Sync>
#[derive(Clone)]
pub struct StoreWrapper {
    inner: Arc<dyn Store + Send + Sync>,
}

impl StoreWrapper {
    pub fn new(store: Arc<dyn Store + Send + Sync>) -> Self {
        Self { inner: store }
    }

    /// Получает доступ к внутреннему Store
    pub fn store(&self) -> &dyn Store {
        self.inner.as_ref()
    }

    /// Получает Arc на внутренний Store
    pub fn as_arc(&self) -> Arc<dyn Store + Send + Sync> {
        self.inner.clone()
    }

    /// Проверка подключения к БД
    pub async fn ping(&self) -> Result<()> {
        self.inner.as_ref().connect().await
    }
}

#[async_trait]
impl ConnectionManager for StoreWrapper {
    async fn connect(&self) -> Result<()> {
        self.inner.as_ref().connect().await
    }

    async fn close(&self) -> Result<()> {
        self.inner.as_ref().close().await
    }

    fn is_permanent(&self) -> bool {
        self.inner.as_ref().is_permanent()
    }
}

#[async_trait]
impl MigrationManager for StoreWrapper {
    fn get_dialect(&self) -> &str {
        self.inner.as_ref().get_dialect()
    }

    async fn is_initialized(&self) -> Result<bool> {
        self.inner.as_ref().is_initialized().await
    }

    async fn apply_migration(&self, version: i64, name: String) -> Result<()> {
        self.inner.as_ref().apply_migration(version, name).await
    }

    async fn is_migration_applied(&self, version: i64) -> Result<bool> {
        self.inner.as_ref().is_migration_applied(version).await
    }
}

#[async_trait]
impl OptionsManager for StoreWrapper {
    async fn get_options(&self) -> Result<HashMap<String, String>> {
        self.inner.as_ref().get_options().await
    }

    async fn get_option(&self, key: &str) -> Result<Option<String>> {
        self.inner.as_ref().get_option(key).await
    }

    async fn set_option(&self, key: &str, value: &str) -> Result<()> {
        self.inner.as_ref().set_option(key, value).await
    }

    async fn delete_option(&self, key: &str) -> Result<()> {
        self.inner.as_ref().delete_option(key).await
    }
}

#[async_trait]
impl UserManager for StoreWrapper {
    async fn get_users(&self, params: RetrieveQueryParams) -> Result<Vec<User>> {
        self.inner.as_ref().get_users(params).await
    }

    async fn get_user(&self, user_id: i32) -> Result<User> {
        self.inner.as_ref().get_user(user_id).await
    }

    async fn get_user_by_login_or_email(&self, login: &str, email: &str) -> Result<User> {
        self.inner
            .as_ref()
            .get_user_by_login_or_email(login, email)
            .await
    }

    async fn create_user(&self, user: User, password: &str) -> Result<User> {
        self.inner.as_ref().create_user(user, password).await
    }

    async fn update_user(&self, user: User) -> Result<()> {
        self.inner.as_ref().update_user(user).await
    }

    async fn delete_user(&self, user_id: i32) -> Result<()> {
        self.inner.as_ref().delete_user(user_id).await
    }

    async fn set_user_password(&self, user_id: i32, password: &str) -> Result<()> {
        self.inner
            .as_ref()
            .set_user_password(user_id, password)
            .await
    }

    async fn get_all_admins(&self) -> Result<Vec<User>> {
        self.inner.as_ref().get_all_admins().await
    }

    async fn get_user_count(&self) -> Result<usize> {
        self.inner.as_ref().get_user_count().await
    }

    async fn get_project_users(
        &self,
        project_id: i32,
        params: RetrieveQueryParams,
    ) -> Result<Vec<ProjectUser>> {
        self.inner
            .as_ref()
            .get_project_users(project_id, params)
            .await
    }

    async fn get_user_totp(&self, user_id: i32) -> Result<Option<UserTotp>> {
        self.inner.as_ref().get_user_totp(user_id).await
    }

    async fn set_user_totp(&self, user_id: i32, totp: &UserTotp) -> Result<()> {
        self.inner.as_ref().set_user_totp(user_id, totp).await
    }

    async fn delete_user_totp(&self, user_id: i32) -> Result<()> {
        self.inner.as_ref().delete_user_totp(user_id).await
    }
}

#[async_trait]
impl ProjectStore for StoreWrapper {
    async fn get_projects(&self, user_id: Option<i32>) -> Result<Vec<Project>> {
        self.inner.as_ref().get_projects(user_id).await
    }

    async fn get_project(&self, project_id: i32) -> Result<Project> {
        self.inner.as_ref().get_project(project_id).await
    }

    async fn create_project(&self, project: Project) -> Result<Project> {
        self.inner.as_ref().create_project(project).await
    }

    async fn update_project(&self, project: Project) -> Result<()> {
        self.inner.as_ref().update_project(project).await
    }

    async fn delete_project(&self, project_id: i32) -> Result<()> {
        self.inner.as_ref().delete_project(project_id).await
    }

    async fn create_project_user(&self, project_user: crate::models::ProjectUser) -> Result<()> {
        self.inner.as_ref().create_project_user(project_user).await
    }

    async fn delete_project_user(&self, project_id: i32, user_id: i32) -> Result<()> {
        self.inner
            .as_ref()
            .delete_project_user(project_id, user_id)
            .await
    }
}

#[async_trait]
impl TemplateManager for StoreWrapper {
    async fn get_templates(&self, project_id: i32) -> Result<Vec<Template>> {
        self.inner.as_ref().get_templates(project_id).await
    }

    async fn get_template(&self, project_id: i32, template_id: i32) -> Result<Template> {
        self.inner
            .as_ref()
            .get_template(project_id, template_id)
            .await
    }

    async fn create_template(&self, template: Template) -> Result<Template> {
        self.inner.as_ref().create_template(template).await
    }

    async fn update_template(&self, template: Template) -> Result<()> {
        self.inner.as_ref().update_template(template).await
    }

    async fn delete_template(&self, project_id: i32, template_id: i32) -> Result<()> {
        self.inner
            .as_ref()
            .delete_template(project_id, template_id)
            .await
    }
}

#[async_trait]
impl HookManager for StoreWrapper {
    async fn get_hooks_by_template(&self, template_id: i32) -> Result<Vec<Hook>> {
        self.inner.as_ref().get_hooks_by_template(template_id).await
    }
}

#[async_trait]
impl InventoryManager for StoreWrapper {
    async fn get_inventories(&self, project_id: i32) -> Result<Vec<Inventory>> {
        self.inner.as_ref().get_inventories(project_id).await
    }

    async fn get_inventory(&self, project_id: i32, inventory_id: i32) -> Result<Inventory> {
        self.inner
            .as_ref()
            .get_inventory(project_id, inventory_id)
            .await
    }

    async fn create_inventory(&self, inventory: Inventory) -> Result<Inventory> {
        self.inner.as_ref().create_inventory(inventory).await
    }

    async fn update_inventory(&self, inventory: Inventory) -> Result<()> {
        self.inner.as_ref().update_inventory(inventory).await
    }

    async fn delete_inventory(&self, project_id: i32, inventory_id: i32) -> Result<()> {
        self.inner
            .as_ref()
            .delete_inventory(project_id, inventory_id)
            .await
    }
}

#[async_trait]
impl RepositoryManager for StoreWrapper {
    async fn get_repositories(&self, project_id: i32) -> Result<Vec<Repository>> {
        self.inner.as_ref().get_repositories(project_id).await
    }

    async fn get_repository(&self, project_id: i32, repository_id: i32) -> Result<Repository> {
        self.inner
            .as_ref()
            .get_repository(project_id, repository_id)
            .await
    }

    async fn create_repository(&self, repository: Repository) -> Result<Repository> {
        self.inner.as_ref().create_repository(repository).await
    }

    async fn update_repository(&self, repository: Repository) -> Result<()> {
        self.inner.as_ref().update_repository(repository).await
    }

    async fn delete_repository(&self, project_id: i32, repository_id: i32) -> Result<()> {
        self.inner
            .as_ref()
            .delete_repository(project_id, repository_id)
            .await
    }
}

#[async_trait]
impl EnvironmentManager for StoreWrapper {
    async fn get_environments(&self, project_id: i32) -> Result<Vec<Environment>> {
        self.inner.as_ref().get_environments(project_id).await
    }

    async fn get_environment(&self, project_id: i32, environment_id: i32) -> Result<Environment> {
        self.inner
            .as_ref()
            .get_environment(project_id, environment_id)
            .await
    }

    async fn create_environment(&self, environment: Environment) -> Result<Environment> {
        self.inner.as_ref().create_environment(environment).await
    }

    async fn update_environment(&self, environment: Environment) -> Result<()> {
        self.inner.as_ref().update_environment(environment).await
    }

    async fn delete_environment(&self, project_id: i32, environment_id: i32) -> Result<()> {
        self.inner
            .as_ref()
            .delete_environment(project_id, environment_id)
            .await
    }
}

#[async_trait]
impl AccessKeyManager for StoreWrapper {
    async fn get_access_keys(&self, project_id: i32) -> Result<Vec<AccessKey>> {
        self.inner.as_ref().get_access_keys(project_id).await
    }

    async fn get_access_key(&self, project_id: i32, access_key_id: i32) -> Result<AccessKey> {
        self.inner
            .as_ref()
            .get_access_key(project_id, access_key_id)
            .await
    }

    async fn create_access_key(&self, access_key: AccessKey) -> Result<AccessKey> {
        self.inner.as_ref().create_access_key(access_key).await
    }

    async fn update_access_key(&self, access_key: AccessKey) -> Result<()> {
        self.inner.as_ref().update_access_key(access_key).await
    }

    async fn delete_access_key(&self, project_id: i32, access_key_id: i32) -> Result<()> {
        self.inner
            .as_ref()
            .delete_access_key(project_id, access_key_id)
            .await
    }
}

#[async_trait]
impl TaskManager for StoreWrapper {
    async fn get_tasks(
        &self,
        project_id: i32,
        template_id: Option<i32>,
    ) -> Result<Vec<TaskWithTpl>> {
        self.inner.as_ref().get_tasks(project_id, template_id).await
    }

    async fn get_task(&self, project_id: i32, task_id: i32) -> Result<Task> {
        self.inner.as_ref().get_task(project_id, task_id).await
    }

    async fn create_task(&self, task: Task) -> Result<Task> {
        self.inner.as_ref().create_task(task).await
    }

    async fn update_task(&self, task: Task) -> Result<()> {
        self.inner.as_ref().update_task(task).await
    }

    async fn delete_task(&self, project_id: i32, task_id: i32) -> Result<()> {
        self.inner.as_ref().delete_task(project_id, task_id).await
    }

    async fn get_task_outputs(&self, task_id: i32) -> Result<Vec<TaskOutput>> {
        self.inner.as_ref().get_task_outputs(task_id).await
    }

    async fn create_task_output(&self, output: TaskOutput) -> Result<TaskOutput> {
        self.inner.as_ref().create_task_output(output).await
    }

    async fn update_task_status(
        &self,
        project_id: i32,
        task_id: i32,
        status: TaskStatus,
    ) -> Result<()> {
        self.inner
            .as_ref()
            .update_task_status(project_id, task_id, status)
            .await
    }

    async fn get_global_tasks(
        &self,
        status_filter: Option<Vec<String>>,
        limit: Option<i32>,
    ) -> Result<Vec<TaskWithTpl>> {
        self.inner
            .as_ref()
            .get_global_tasks(status_filter, limit)
            .await
    }

    async fn get_running_tasks_count(&self) -> Result<usize> {
        self.inner.as_ref().get_running_tasks_count().await
    }

    async fn get_waiting_tasks_count(&self) -> Result<usize> {
        self.inner.as_ref().get_waiting_tasks_count().await
    }
}

#[async_trait]
impl ScheduleManager for StoreWrapper {
    async fn get_schedules(&self, project_id: i32) -> Result<Vec<Schedule>> {
        self.inner.as_ref().get_schedules(project_id).await
    }

    async fn get_all_schedules(&self) -> Result<Vec<Schedule>> {
        self.inner.as_ref().get_all_schedules().await
    }

    async fn get_schedule(&self, project_id: i32, schedule_id: i32) -> Result<Schedule> {
        self.inner
            .as_ref()
            .get_schedule(project_id, schedule_id)
            .await
    }

    async fn create_schedule(&self, schedule: Schedule) -> Result<Schedule> {
        self.inner.as_ref().create_schedule(schedule).await
    }

    async fn update_schedule(&self, schedule: Schedule) -> Result<()> {
        self.inner.as_ref().update_schedule(schedule).await
    }

    async fn delete_schedule(&self, project_id: i32, schedule_id: i32) -> Result<()> {
        self.inner
            .as_ref()
            .delete_schedule(project_id, schedule_id)
            .await
    }

    async fn set_schedule_active(
        &self,
        project_id: i32,
        schedule_id: i32,
        active: bool,
    ) -> Result<()> {
        self.inner
            .as_ref()
            .set_schedule_active(project_id, schedule_id, active)
            .await
    }

    async fn set_schedule_commit_hash(
        &self,
        project_id: i32,
        schedule_id: i32,
        hash: &str,
    ) -> Result<()> {
        self.inner
            .as_ref()
            .set_schedule_commit_hash(project_id, schedule_id, hash)
            .await
    }
}

#[async_trait]
impl SessionManager for StoreWrapper {
    async fn get_session(&self, user_id: i32, session_id: i32) -> Result<Session> {
        self.inner.as_ref().get_session(user_id, session_id).await
    }

    async fn create_session(&self, session: Session) -> Result<Session> {
        self.inner.as_ref().create_session(session).await
    }

    async fn expire_session(&self, user_id: i32, session_id: i32) -> Result<()> {
        self.inner
            .as_ref()
            .expire_session(user_id, session_id)
            .await
    }

    async fn verify_session(&self, user_id: i32, session_id: i32) -> Result<()> {
        self.inner
            .as_ref()
            .verify_session(user_id, session_id)
            .await
    }

    async fn touch_session(&self, user_id: i32, session_id: i32) -> Result<()> {
        self.inner.as_ref().touch_session(user_id, session_id).await
    }
}

#[async_trait]
impl TokenManager for StoreWrapper {
    async fn get_api_tokens(&self, user_id: i32) -> Result<Vec<APIToken>> {
        self.inner.as_ref().get_api_tokens(user_id).await
    }

    async fn create_api_token(&self, token: APIToken) -> Result<APIToken> {
        self.inner.as_ref().create_api_token(token).await
    }

    async fn get_api_token(&self, token_id: i32) -> Result<APIToken> {
        self.inner.as_ref().get_api_token(token_id).await
    }

    async fn expire_api_token(&self, user_id: i32, token_id: i32) -> Result<()> {
        self.inner
            .as_ref()
            .expire_api_token(user_id, token_id)
            .await
    }

    async fn delete_api_token(&self, user_id: i32, token_id: i32) -> Result<()> {
        self.inner
            .as_ref()
            .delete_api_token(user_id, token_id)
            .await
    }
}

#[async_trait]
impl EventManager for StoreWrapper {
    async fn get_events(&self, project_id: Option<i32>, limit: usize) -> Result<Vec<Event>> {
        self.inner.as_ref().get_events(project_id, limit).await
    }

    async fn create_event(&self, event: Event) -> Result<Event> {
        self.inner.as_ref().create_event(event).await
    }
}

#[async_trait]
impl RunnerManager for StoreWrapper {
    async fn get_runners(&self, project_id: Option<i32>) -> Result<Vec<Runner>> {
        self.inner.as_ref().get_runners(project_id).await
    }

    async fn get_runner(&self, runner_id: i32) -> Result<Runner> {
        self.inner.as_ref().get_runner(runner_id).await
    }

    async fn create_runner(&self, runner: Runner) -> Result<Runner> {
        self.inner.as_ref().create_runner(runner).await
    }

    async fn update_runner(&self, runner: Runner) -> Result<()> {
        self.inner.as_ref().update_runner(runner).await
    }

    async fn delete_runner(&self, runner_id: i32) -> Result<()> {
        self.inner.as_ref().delete_runner(runner_id).await
    }

    async fn get_runners_count(&self) -> Result<usize> {
        self.inner.as_ref().get_runners_count().await
    }

    async fn get_active_runners_count(&self) -> Result<usize> {
        self.inner.as_ref().get_active_runners_count().await
    }

    async fn find_runner_by_token(&self, token: &str) -> Result<Runner> {
        self.inner.as_ref().find_runner_by_token(token).await
    }

    async fn touch_runner(&self, runner_id: i32) -> Result<()> {
        self.inner.as_ref().touch_runner(runner_id).await
    }
}

#[async_trait]
impl ViewManager for StoreWrapper {
    async fn get_views(&self, project_id: i32) -> Result<Vec<View>> {
        self.inner.as_ref().get_views(project_id).await
    }

    async fn get_view(&self, project_id: i32, view_id: i32) -> Result<View> {
        self.inner.as_ref().get_view(project_id, view_id).await
    }

    async fn create_view(&self, view: View) -> Result<View> {
        self.inner.as_ref().create_view(view).await
    }

    async fn update_view(&self, view: View) -> Result<()> {
        self.inner.as_ref().update_view(view).await
    }

    async fn delete_view(&self, project_id: i32, view_id: i32) -> Result<()> {
        self.inner.as_ref().delete_view(project_id, view_id).await
    }

    async fn set_view_positions(&self, project_id: i32, positions: Vec<(i32, i32)>) -> Result<()> {
        self.inner
            .as_ref()
            .set_view_positions(project_id, positions)
            .await
    }
}

#[async_trait]
impl IntegrationManager for StoreWrapper {
    async fn get_integrations(&self, project_id: i32) -> Result<Vec<Integration>> {
        self.inner.as_ref().get_integrations(project_id).await
    }

    async fn get_integration(&self, project_id: i32, integration_id: i32) -> Result<Integration> {
        self.inner
            .as_ref()
            .get_integration(project_id, integration_id)
            .await
    }

    async fn create_integration(&self, integration: Integration) -> Result<Integration> {
        self.inner.as_ref().create_integration(integration).await
    }

    async fn update_integration(&self, integration: Integration) -> Result<()> {
        self.inner.as_ref().update_integration(integration).await
    }

    async fn delete_integration(&self, project_id: i32, integration_id: i32) -> Result<()> {
        self.inner
            .as_ref()
            .delete_integration(project_id, integration_id)
            .await
    }
}

#[async_trait]
impl ProjectInviteManager for StoreWrapper {
    async fn get_project_invites(
        &self,
        project_id: i32,
        params: RetrieveQueryParams,
    ) -> Result<Vec<ProjectInviteWithUser>> {
        self.inner
            .as_ref()
            .get_project_invites(project_id, params)
            .await
    }

    async fn create_project_invite(&self, invite: ProjectInvite) -> Result<ProjectInvite> {
        self.inner.as_ref().create_project_invite(invite).await
    }

    async fn get_project_invite(&self, project_id: i32, invite_id: i32) -> Result<ProjectInvite> {
        self.inner
            .as_ref()
            .get_project_invite(project_id, invite_id)
            .await
    }

    async fn get_project_invite_by_token(&self, token: &str) -> Result<ProjectInvite> {
        self.inner.as_ref().get_project_invite_by_token(token).await
    }

    async fn update_project_invite(&self, invite: ProjectInvite) -> Result<()> {
        self.inner.as_ref().update_project_invite(invite).await
    }

    async fn delete_project_invite(&self, project_id: i32, invite_id: i32) -> Result<()> {
        self.inner
            .as_ref()
            .delete_project_invite(project_id, invite_id)
            .await
    }
}

#[async_trait]
impl TerraformInventoryManager for StoreWrapper {
    async fn create_terraform_inventory_alias(
        &self,
        alias: TerraformInventoryAlias,
    ) -> Result<TerraformInventoryAlias> {
        self.inner
            .as_ref()
            .create_terraform_inventory_alias(alias)
            .await
    }

    async fn update_terraform_inventory_alias(&self, alias: TerraformInventoryAlias) -> Result<()> {
        self.inner
            .as_ref()
            .update_terraform_inventory_alias(alias)
            .await
    }

    async fn get_terraform_inventory_alias_by_alias(
        &self,
        alias: &str,
    ) -> Result<TerraformInventoryAlias> {
        self.inner
            .as_ref()
            .get_terraform_inventory_alias_by_alias(alias)
            .await
    }

    async fn get_terraform_inventory_alias(
        &self,
        project_id: i32,
        inventory_id: i32,
        alias_id: &str,
    ) -> Result<TerraformInventoryAlias> {
        self.inner
            .as_ref()
            .get_terraform_inventory_alias(project_id, inventory_id, alias_id)
            .await
    }

    async fn get_terraform_inventory_aliases(
        &self,
        project_id: i32,
        inventory_id: i32,
    ) -> Result<Vec<TerraformInventoryAlias>> {
        self.inner
            .as_ref()
            .get_terraform_inventory_aliases(project_id, inventory_id)
            .await
    }

    async fn delete_terraform_inventory_alias(
        &self,
        project_id: i32,
        inventory_id: i32,
        alias_id: &str,
    ) -> Result<()> {
        self.inner
            .as_ref()
            .delete_terraform_inventory_alias(project_id, inventory_id, alias_id)
            .await
    }

    async fn get_terraform_inventory_states(
        &self,
        project_id: i32,
        inventory_id: i32,
        params: RetrieveQueryParams,
    ) -> Result<Vec<TerraformInventoryState>> {
        self.inner
            .as_ref()
            .get_terraform_inventory_states(project_id, inventory_id, params)
            .await
    }

    async fn create_terraform_inventory_state(
        &self,
        state: TerraformInventoryState,
    ) -> Result<TerraformInventoryState> {
        self.inner
            .as_ref()
            .create_terraform_inventory_state(state)
            .await
    }

    async fn delete_terraform_inventory_state(
        &self,
        project_id: i32,
        inventory_id: i32,
        state_id: i32,
    ) -> Result<()> {
        self.inner
            .as_ref()
            .delete_terraform_inventory_state(project_id, inventory_id, state_id)
            .await
    }

    async fn get_terraform_inventory_state(
        &self,
        project_id: i32,
        inventory_id: i32,
        state_id: i32,
    ) -> Result<TerraformInventoryState> {
        self.inner
            .as_ref()
            .get_terraform_inventory_state(project_id, inventory_id, state_id)
            .await
    }

    async fn get_terraform_state_count(&self) -> Result<i32> {
        self.inner.as_ref().get_terraform_state_count().await
    }
}

#[async_trait]
impl SecretStorageManager for StoreWrapper {
    async fn get_secret_storages(&self, project_id: i32) -> Result<Vec<SecretStorage>> {
        self.inner.as_ref().get_secret_storages(project_id).await
    }

    async fn get_secret_storage(&self, project_id: i32, storage_id: i32) -> Result<SecretStorage> {
        self.inner
            .as_ref()
            .get_secret_storage(project_id, storage_id)
            .await
    }

    async fn create_secret_storage(&self, storage: SecretStorage) -> Result<SecretStorage> {
        self.inner.as_ref().create_secret_storage(storage).await
    }

    async fn update_secret_storage(&self, storage: SecretStorage) -> Result<()> {
        self.inner.as_ref().update_secret_storage(storage).await
    }

    async fn delete_secret_storage(&self, project_id: i32, storage_id: i32) -> Result<()> {
        self.inner
            .as_ref()
            .delete_secret_storage(project_id, storage_id)
            .await
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
        self.inner
            .as_ref()
            .create_audit_log(
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
            )
            .await
    }

    async fn get_audit_log(&self, id: i64) -> Result<AuditLog> {
        self.inner.as_ref().get_audit_log(id).await
    }

    async fn search_audit_logs(&self, filter: &AuditLogFilter) -> Result<AuditLogResult> {
        self.inner.as_ref().search_audit_logs(filter).await
    }

    async fn get_audit_logs_by_project(
        &self,
        project_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>> {
        self.inner
            .as_ref()
            .get_audit_logs_by_project(project_id, limit, offset)
            .await
    }

    async fn get_audit_logs_by_user(
        &self,
        user_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>> {
        self.inner
            .as_ref()
            .get_audit_logs_by_user(user_id, limit, offset)
            .await
    }

    async fn get_audit_logs_by_action(
        &self,
        action: &AuditAction,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>> {
        self.inner
            .as_ref()
            .get_audit_logs_by_action(action, limit, offset)
            .await
    }

    async fn delete_audit_logs_before(&self, before: DateTime<Utc>) -> Result<u64> {
        self.inner.as_ref().delete_audit_logs_before(before).await
    }

    async fn clear_audit_log(&self) -> Result<u64> {
        self.inner.as_ref().clear_audit_log().await
    }
}

#[async_trait]
impl crate::db::store::WebhookManager for StoreWrapper {
    async fn get_webhook(&self, webhook_id: i64) -> Result<Webhook> {
        self.inner.as_ref().get_webhook(webhook_id).await
    }

    async fn get_webhooks_by_project(&self, project_id: i64) -> Result<Vec<Webhook>> {
        self.inner
            .as_ref()
            .get_webhooks_by_project(project_id)
            .await
    }

    async fn create_webhook(&self, webhook: Webhook) -> Result<Webhook> {
        self.inner.as_ref().create_webhook(webhook).await
    }

    async fn update_webhook(&self, webhook_id: i64, webhook: UpdateWebhook) -> Result<Webhook> {
        self.inner
            .as_ref()
            .update_webhook(webhook_id, webhook)
            .await
    }

    async fn delete_webhook(&self, webhook_id: i64) -> Result<()> {
        self.inner.as_ref().delete_webhook(webhook_id).await
    }

    async fn get_webhook_logs(&self, webhook_id: i64) -> Result<Vec<WebhookLog>> {
        self.inner.as_ref().get_webhook_logs(webhook_id).await
    }

    async fn create_webhook_log(&self, log: WebhookLog) -> Result<WebhookLog> {
        self.inner.as_ref().create_webhook_log(log).await
    }
}

#[async_trait]
impl crate::db::store::PlaybookManager for StoreWrapper {
    async fn get_playbooks(&self, project_id: i32) -> Result<Vec<crate::models::Playbook>> {
        self.inner.as_ref().get_playbooks(project_id).await
    }

    async fn get_playbook(&self, id: i32, project_id: i32) -> Result<crate::models::Playbook> {
        self.inner.as_ref().get_playbook(id, project_id).await
    }

    async fn create_playbook(
        &self,
        project_id: i32,
        playbook: crate::models::PlaybookCreate,
    ) -> Result<crate::models::Playbook> {
        self.inner
            .as_ref()
            .create_playbook(project_id, playbook)
            .await
    }

    async fn update_playbook(
        &self,
        id: i32,
        project_id: i32,
        playbook: crate::models::PlaybookUpdate,
    ) -> Result<crate::models::Playbook> {
        self.inner
            .as_ref()
            .update_playbook(id, project_id, playbook)
            .await
    }

    async fn delete_playbook(&self, id: i32, project_id: i32) -> Result<()> {
        self.inner.as_ref().delete_playbook(id, project_id).await
    }
}

#[async_trait]
impl crate::db::store::PlaybookRunManager for StoreWrapper {
    async fn get_playbook_runs(&self, filter: PlaybookRunFilter) -> Result<Vec<PlaybookRun>> {
        self.inner.as_ref().get_playbook_runs(filter).await
    }

    async fn get_playbook_run(&self, id: i32, project_id: i32) -> Result<PlaybookRun> {
        self.inner.as_ref().get_playbook_run(id, project_id).await
    }

    async fn get_playbook_run_by_task_id(&self, task_id: i32) -> Result<Option<PlaybookRun>> {
        self.inner
            .as_ref()
            .get_playbook_run_by_task_id(task_id)
            .await
    }

    async fn create_playbook_run(&self, run: PlaybookRunCreate) -> Result<PlaybookRun> {
        self.inner.as_ref().create_playbook_run(run).await
    }

    async fn update_playbook_run(
        &self,
        id: i32,
        project_id: i32,
        update: PlaybookRunUpdate,
    ) -> Result<PlaybookRun> {
        self.inner
            .as_ref()
            .update_playbook_run(id, project_id, update)
            .await
    }

    async fn update_playbook_run_status(&self, id: i32, status: PlaybookRunStatus) -> Result<()> {
        self.inner
            .as_ref()
            .update_playbook_run_status(id, status)
            .await
    }

    async fn delete_playbook_run(&self, id: i32, project_id: i32) -> Result<()> {
        self.inner
            .as_ref()
            .delete_playbook_run(id, project_id)
            .await
    }

    async fn get_playbook_run_stats(&self, playbook_id: i32) -> Result<PlaybookRunStats> {
        self.inner
            .as_ref()
            .get_playbook_run_stats(playbook_id)
            .await
    }
}

#[async_trait]
impl IntegrationMatcherManager for StoreWrapper {
    async fn get_integration_matchers(
        &self,
        project_id: i32,
        integration_id: i32,
    ) -> Result<Vec<IntegrationMatcher>> {
        self.inner
            .as_ref()
            .get_integration_matchers(project_id, integration_id)
            .await
    }
    async fn create_integration_matcher(
        &self,
        matcher: IntegrationMatcher,
    ) -> Result<IntegrationMatcher> {
        self.inner
            .as_ref()
            .create_integration_matcher(matcher)
            .await
    }
    async fn update_integration_matcher(&self, matcher: IntegrationMatcher) -> Result<()> {
        self.inner
            .as_ref()
            .update_integration_matcher(matcher)
            .await
    }
    async fn delete_integration_matcher(
        &self,
        project_id: i32,
        integration_id: i32,
        matcher_id: i32,
    ) -> Result<()> {
        self.inner
            .as_ref()
            .delete_integration_matcher(project_id, integration_id, matcher_id)
            .await
    }
}

#[async_trait]
impl IntegrationExtractValueManager for StoreWrapper {
    async fn get_integration_extract_values(
        &self,
        project_id: i32,
        integration_id: i32,
    ) -> Result<Vec<IntegrationExtractValue>> {
        self.inner
            .as_ref()
            .get_integration_extract_values(project_id, integration_id)
            .await
    }
    async fn create_integration_extract_value(
        &self,
        value: IntegrationExtractValue,
    ) -> Result<IntegrationExtractValue> {
        self.inner
            .as_ref()
            .create_integration_extract_value(value)
            .await
    }
    async fn update_integration_extract_value(&self, value: IntegrationExtractValue) -> Result<()> {
        self.inner
            .as_ref()
            .update_integration_extract_value(value)
            .await
    }
    async fn delete_integration_extract_value(
        &self,
        project_id: i32,
        integration_id: i32,
        value_id: i32,
    ) -> Result<()> {
        self.inner
            .as_ref()
            .delete_integration_extract_value(project_id, integration_id, value_id)
            .await
    }
}

#[async_trait]
impl ProjectRoleManager for StoreWrapper {
    async fn get_project_roles(&self, project_id: i32) -> Result<Vec<crate::models::Role>> {
        self.inner.as_ref().get_project_roles(project_id).await
    }
    async fn create_project_role(&self, role: crate::models::Role) -> Result<crate::models::Role> {
        self.inner.as_ref().create_project_role(role).await
    }
    async fn update_project_role(&self, role: crate::models::Role) -> Result<()> {
        self.inner.as_ref().update_project_role(role).await
    }
    async fn delete_project_role(&self, project_id: i32, role_id: i32) -> Result<()> {
        self.inner
            .as_ref()
            .delete_project_role(project_id, role_id)
            .await
    }
}

#[async_trait]
impl crate::db::store::WorkflowManager for StoreWrapper {
    async fn get_workflows(&self, project_id: i32) -> crate::error::Result<Vec<Workflow>> {
        self.inner.as_ref().get_workflows(project_id).await
    }
    async fn get_workflow(&self, id: i32, project_id: i32) -> crate::error::Result<Workflow> {
        self.inner.as_ref().get_workflow(id, project_id).await
    }
    async fn create_workflow(
        &self,
        project_id: i32,
        payload: WorkflowCreate,
    ) -> crate::error::Result<Workflow> {
        self.inner
            .as_ref()
            .create_workflow(project_id, payload)
            .await
    }
    async fn update_workflow(
        &self,
        id: i32,
        project_id: i32,
        payload: WorkflowUpdate,
    ) -> crate::error::Result<Workflow> {
        self.inner
            .as_ref()
            .update_workflow(id, project_id, payload)
            .await
    }
    async fn delete_workflow(&self, id: i32, project_id: i32) -> crate::error::Result<()> {
        self.inner.as_ref().delete_workflow(id, project_id).await
    }
    async fn get_workflow_nodes(
        &self,
        workflow_id: i32,
    ) -> crate::error::Result<Vec<WorkflowNode>> {
        self.inner.as_ref().get_workflow_nodes(workflow_id).await
    }
    async fn create_workflow_node(
        &self,
        workflow_id: i32,
        payload: WorkflowNodeCreate,
    ) -> crate::error::Result<WorkflowNode> {
        self.inner
            .as_ref()
            .create_workflow_node(workflow_id, payload)
            .await
    }
    async fn update_workflow_node(
        &self,
        id: i32,
        workflow_id: i32,
        payload: WorkflowNodeUpdate,
    ) -> crate::error::Result<WorkflowNode> {
        self.inner
            .as_ref()
            .update_workflow_node(id, workflow_id, payload)
            .await
    }
    async fn delete_workflow_node(&self, id: i32, workflow_id: i32) -> crate::error::Result<()> {
        self.inner
            .as_ref()
            .delete_workflow_node(id, workflow_id)
            .await
    }
    async fn get_workflow_edges(
        &self,
        workflow_id: i32,
    ) -> crate::error::Result<Vec<WorkflowEdge>> {
        self.inner.as_ref().get_workflow_edges(workflow_id).await
    }
    async fn create_workflow_edge(
        &self,
        workflow_id: i32,
        payload: WorkflowEdgeCreate,
    ) -> crate::error::Result<WorkflowEdge> {
        self.inner
            .as_ref()
            .create_workflow_edge(workflow_id, payload)
            .await
    }
    async fn delete_workflow_edge(&self, id: i32, workflow_id: i32) -> crate::error::Result<()> {
        self.inner
            .as_ref()
            .delete_workflow_edge(id, workflow_id)
            .await
    }
    async fn get_workflow_runs(
        &self,
        workflow_id: i32,
        project_id: i32,
    ) -> crate::error::Result<Vec<WorkflowRun>> {
        self.inner
            .as_ref()
            .get_workflow_runs(workflow_id, project_id)
            .await
    }
    async fn create_workflow_run(
        &self,
        workflow_id: i32,
        project_id: i32,
    ) -> crate::error::Result<WorkflowRun> {
        self.inner
            .as_ref()
            .create_workflow_run(workflow_id, project_id)
            .await
    }
    async fn update_workflow_run_status(
        &self,
        id: i32,
        status: &str,
        message: Option<String>,
    ) -> crate::error::Result<()> {
        self.inner
            .as_ref()
            .update_workflow_run_status(id, status, message)
            .await
    }
}

#[async_trait]
impl crate::db::store::NotificationPolicyManager for StoreWrapper {
    async fn get_notification_policies(
        &self,
        project_id: i32,
    ) -> crate::error::Result<Vec<NotificationPolicy>> {
        self.inner
            .as_ref()
            .get_notification_policies(project_id)
            .await
    }
    async fn get_notification_policy(
        &self,
        id: i32,
        project_id: i32,
    ) -> crate::error::Result<NotificationPolicy> {
        self.inner
            .as_ref()
            .get_notification_policy(id, project_id)
            .await
    }
    async fn create_notification_policy(
        &self,
        project_id: i32,
        payload: NotificationPolicyCreate,
    ) -> crate::error::Result<NotificationPolicy> {
        self.inner
            .as_ref()
            .create_notification_policy(project_id, payload)
            .await
    }
    async fn update_notification_policy(
        &self,
        id: i32,
        project_id: i32,
        payload: NotificationPolicyUpdate,
    ) -> crate::error::Result<NotificationPolicy> {
        self.inner
            .as_ref()
            .update_notification_policy(id, project_id, payload)
            .await
    }
    async fn delete_notification_policy(
        &self,
        id: i32,
        project_id: i32,
    ) -> crate::error::Result<()> {
        self.inner
            .as_ref()
            .delete_notification_policy(id, project_id)
            .await
    }
    async fn get_matching_policies(
        &self,
        project_id: i32,
        trigger: &str,
        template_id: Option<i32>,
    ) -> crate::error::Result<Vec<NotificationPolicy>> {
        self.inner
            .as_ref()
            .get_matching_policies(project_id, trigger, template_id)
            .await
    }
}

#[async_trait]
impl crate::db::store::CredentialTypeManager for StoreWrapper {
    async fn get_credential_types(
        &self,
    ) -> crate::error::Result<Vec<crate::models::credential_type::CredentialType>> {
        self.inner.as_ref().get_credential_types().await
    }
    async fn get_credential_type(
        &self,
        id: i32,
    ) -> crate::error::Result<crate::models::credential_type::CredentialType> {
        self.inner.as_ref().get_credential_type(id).await
    }
    async fn create_credential_type(
        &self,
        payload: crate::models::credential_type::CredentialTypeCreate,
    ) -> crate::error::Result<crate::models::credential_type::CredentialType> {
        self.inner.as_ref().create_credential_type(payload).await
    }
    async fn update_credential_type(
        &self,
        id: i32,
        payload: crate::models::credential_type::CredentialTypeUpdate,
    ) -> crate::error::Result<crate::models::credential_type::CredentialType> {
        self.inner
            .as_ref()
            .update_credential_type(id, payload)
            .await
    }
    async fn delete_credential_type(&self, id: i32) -> crate::error::Result<()> {
        self.inner.as_ref().delete_credential_type(id).await
    }
    async fn get_credential_instances(
        &self,
        project_id: i32,
    ) -> crate::error::Result<Vec<crate::models::credential_type::CredentialInstance>> {
        self.inner
            .as_ref()
            .get_credential_instances(project_id)
            .await
    }
    async fn get_credential_instance(
        &self,
        id: i32,
        project_id: i32,
    ) -> crate::error::Result<crate::models::credential_type::CredentialInstance> {
        self.inner
            .as_ref()
            .get_credential_instance(id, project_id)
            .await
    }
    async fn create_credential_instance(
        &self,
        project_id: i32,
        payload: crate::models::credential_type::CredentialInstanceCreate,
    ) -> crate::error::Result<crate::models::credential_type::CredentialInstance> {
        self.inner
            .as_ref()
            .create_credential_instance(project_id, payload)
            .await
    }
    async fn delete_credential_instance(
        &self,
        id: i32,
        project_id: i32,
    ) -> crate::error::Result<()> {
        self.inner
            .as_ref()
            .delete_credential_instance(id, project_id)
            .await
    }
}

#[async_trait]
impl crate::db::store::DriftManager for StoreWrapper {
    async fn get_drift_configs(
        &self,
        project_id: i32,
    ) -> crate::error::Result<Vec<crate::models::drift::DriftConfig>> {
        self.inner.as_ref().get_drift_configs(project_id).await
    }
    async fn get_drift_config(
        &self,
        id: i32,
        project_id: i32,
    ) -> crate::error::Result<crate::models::drift::DriftConfig> {
        self.inner.as_ref().get_drift_config(id, project_id).await
    }
    async fn create_drift_config(
        &self,
        project_id: i32,
        payload: crate::models::drift::DriftConfigCreate,
    ) -> crate::error::Result<crate::models::drift::DriftConfig> {
        self.inner
            .as_ref()
            .create_drift_config(project_id, payload)
            .await
    }
    async fn update_drift_config_enabled(
        &self,
        id: i32,
        project_id: i32,
        enabled: bool,
    ) -> crate::error::Result<()> {
        self.inner
            .as_ref()
            .update_drift_config_enabled(id, project_id, enabled)
            .await
    }
    async fn delete_drift_config(&self, id: i32, project_id: i32) -> crate::error::Result<()> {
        self.inner
            .as_ref()
            .delete_drift_config(id, project_id)
            .await
    }
    async fn get_drift_results(
        &self,
        drift_config_id: i32,
        limit: i64,
    ) -> crate::error::Result<Vec<crate::models::drift::DriftResult>> {
        self.inner
            .as_ref()
            .get_drift_results(drift_config_id, limit)
            .await
    }
    async fn create_drift_result(
        &self,
        project_id: i32,
        drift_config_id: i32,
        template_id: i32,
        status: &str,
        summary: Option<String>,
        task_id: Option<i32>,
    ) -> crate::error::Result<crate::models::drift::DriftResult> {
        self.inner
            .as_ref()
            .create_drift_result(
                project_id,
                drift_config_id,
                template_id,
                status,
                summary,
                task_id,
            )
            .await
    }
    async fn get_latest_drift_results(
        &self,
        project_id: i32,
    ) -> crate::error::Result<Vec<crate::models::drift::DriftResult>> {
        self.inner
            .as_ref()
            .get_latest_drift_results(project_id)
            .await
    }
}

#[async_trait]
impl crate::db::store::LdapGroupMappingManager for StoreWrapper {
    async fn get_ldap_group_mappings(
        &self,
    ) -> crate::error::Result<Vec<crate::models::ldap_group::LdapGroupMapping>> {
        self.inner.as_ref().get_ldap_group_mappings().await
    }
    async fn create_ldap_group_mapping(
        &self,
        payload: crate::models::ldap_group::LdapGroupMappingCreate,
    ) -> crate::error::Result<crate::models::ldap_group::LdapGroupMapping> {
        self.inner.as_ref().create_ldap_group_mapping(payload).await
    }
    async fn delete_ldap_group_mapping(&self, id: i32) -> crate::error::Result<()> {
        self.inner.as_ref().delete_ldap_group_mapping(id).await
    }
    async fn get_mappings_for_groups(
        &self,
        group_dns: &[String],
    ) -> crate::error::Result<Vec<crate::models::ldap_group::LdapGroupMapping>> {
        self.inner.as_ref().get_mappings_for_groups(group_dns).await
    }
}

#[async_trait]
impl crate::db::store::SnapshotManager for StoreWrapper {
    async fn get_snapshots(
        &self,
        project_id: i32,
        template_id: Option<i32>,
        limit: i64,
    ) -> crate::error::Result<Vec<crate::models::snapshot::TaskSnapshot>> {
        self.inner
            .as_ref()
            .get_snapshots(project_id, template_id, limit)
            .await
    }
    async fn get_snapshot(
        &self,
        id: i32,
        project_id: i32,
    ) -> crate::error::Result<crate::models::snapshot::TaskSnapshot> {
        self.inner.as_ref().get_snapshot(id, project_id).await
    }
    async fn create_snapshot(
        &self,
        project_id: i32,
        payload: crate::models::snapshot::TaskSnapshotCreate,
    ) -> crate::error::Result<crate::models::snapshot::TaskSnapshot> {
        self.inner
            .as_ref()
            .create_snapshot(project_id, payload)
            .await
    }
    async fn delete_snapshot(&self, id: i32, project_id: i32) -> crate::error::Result<()> {
        self.inner.as_ref().delete_snapshot(id, project_id).await
    }
}

#[async_trait]
impl crate::db::store::CostEstimateManager for StoreWrapper {
    async fn get_cost_estimates(
        &self,
        project_id: i32,
        limit: i64,
    ) -> crate::error::Result<Vec<crate::models::cost_estimate::CostEstimate>> {
        self.inner
            .as_ref()
            .get_cost_estimates(project_id, limit)
            .await
    }
    async fn get_cost_estimate_for_task(
        &self,
        project_id: i32,
        task_id: i32,
    ) -> crate::error::Result<Option<crate::models::cost_estimate::CostEstimate>> {
        self.inner
            .as_ref()
            .get_cost_estimate_for_task(project_id, task_id)
            .await
    }
    async fn create_cost_estimate(
        &self,
        payload: crate::models::cost_estimate::CostEstimateCreate,
    ) -> crate::error::Result<crate::models::cost_estimate::CostEstimate> {
        self.inner.as_ref().create_cost_estimate(payload).await
    }
    async fn get_cost_summaries(
        &self,
        project_id: i32,
    ) -> crate::error::Result<Vec<crate::models::cost_estimate::CostSummary>> {
        self.inner.as_ref().get_cost_summaries(project_id).await
    }
}

#[async_trait]
impl crate::db::store::TerraformStateManager for StoreWrapper {
    async fn get_terraform_state(
        &self,
        project_id: i32,
        workspace: &str,
    ) -> crate::error::Result<Option<crate::models::TerraformState>> {
        self.inner
            .as_ref()
            .get_terraform_state(project_id, workspace)
            .await
    }
    async fn list_terraform_states(
        &self,
        project_id: i32,
        workspace: &str,
    ) -> crate::error::Result<Vec<crate::models::TerraformStateSummary>> {
        self.inner
            .as_ref()
            .list_terraform_states(project_id, workspace)
            .await
    }
    async fn get_terraform_state_by_serial(
        &self,
        project_id: i32,
        workspace: &str,
        serial: i32,
    ) -> crate::error::Result<Option<crate::models::TerraformState>> {
        self.inner
            .as_ref()
            .get_terraform_state_by_serial(project_id, workspace, serial)
            .await
    }
    async fn create_terraform_state(
        &self,
        state: crate::models::TerraformState,
    ) -> crate::error::Result<crate::models::TerraformState> {
        self.inner.as_ref().create_terraform_state(state).await
    }
    async fn delete_terraform_state(
        &self,
        project_id: i32,
        workspace: &str,
    ) -> crate::error::Result<()> {
        self.inner
            .as_ref()
            .delete_terraform_state(project_id, workspace)
            .await
    }
    async fn delete_all_terraform_states(
        &self,
        project_id: i32,
        workspace: &str,
    ) -> crate::error::Result<()> {
        self.inner
            .as_ref()
            .delete_all_terraform_states(project_id, workspace)
            .await
    }
    async fn lock_terraform_state(
        &self,
        project_id: i32,
        workspace: &str,
        lock: crate::models::TerraformStateLock,
    ) -> crate::error::Result<crate::models::TerraformStateLock> {
        self.inner
            .as_ref()
            .lock_terraform_state(project_id, workspace, lock)
            .await
    }
    async fn unlock_terraform_state(
        &self,
        project_id: i32,
        workspace: &str,
        lock_id: &str,
    ) -> crate::error::Result<()> {
        self.inner
            .as_ref()
            .unlock_terraform_state(project_id, workspace, lock_id)
            .await
    }
    async fn get_terraform_lock(
        &self,
        project_id: i32,
        workspace: &str,
    ) -> crate::error::Result<Option<crate::models::TerraformStateLock>> {
        self.inner
            .as_ref()
            .get_terraform_lock(project_id, workspace)
            .await
    }
    async fn list_terraform_workspaces(
        &self,
        project_id: i32,
    ) -> crate::error::Result<Vec<String>> {
        self.inner
            .as_ref()
            .list_terraform_workspaces(project_id)
            .await
    }
    async fn purge_expired_terraform_locks(&self) -> crate::error::Result<u64> {
        self.inner.as_ref().purge_expired_terraform_locks().await
    }
}

#[async_trait]
impl crate::db::store::PlanApprovalManager for StoreWrapper {
    async fn create_plan(
        &self,
        plan: crate::models::TerraformPlan,
    ) -> crate::error::Result<crate::models::TerraformPlan> {
        self.inner.as_ref().create_plan(plan).await
    }
    async fn get_plan_by_task(
        &self,
        project_id: i32,
        task_id: i32,
    ) -> crate::error::Result<Option<crate::models::TerraformPlan>> {
        self.inner
            .as_ref()
            .get_plan_by_task(project_id, task_id)
            .await
    }
    async fn list_pending_plans(
        &self,
        project_id: i32,
    ) -> crate::error::Result<Vec<crate::models::TerraformPlan>> {
        self.inner.as_ref().list_pending_plans(project_id).await
    }
    async fn approve_plan(
        &self,
        id: i64,
        reviewed_by: i32,
        comment: Option<String>,
    ) -> crate::error::Result<()> {
        self.inner
            .as_ref()
            .approve_plan(id, reviewed_by, comment)
            .await
    }
    async fn reject_plan(
        &self,
        id: i64,
        reviewed_by: i32,
        comment: Option<String>,
    ) -> crate::error::Result<()> {
        self.inner
            .as_ref()
            .reject_plan(id, reviewed_by, comment)
            .await
    }
    async fn update_plan_output(
        &self,
        task_id: i32,
        output: String,
        json: Option<String>,
        added: i32,
        changed: i32,
        removed: i32,
    ) -> crate::error::Result<()> {
        self.inner
            .as_ref()
            .update_plan_output(task_id, output, json, added, changed, removed)
            .await
    }
}

#[async_trait]
impl crate::db::store::OrganizationManager for StoreWrapper {
    async fn get_organizations(&self) -> crate::error::Result<Vec<crate::models::Organization>> {
        self.inner.as_ref().get_organizations().await
    }
    async fn get_organization(&self, id: i32) -> crate::error::Result<crate::models::Organization> {
        self.inner.as_ref().get_organization(id).await
    }
    async fn get_organization_by_slug(
        &self,
        slug: &str,
    ) -> crate::error::Result<crate::models::Organization> {
        self.inner.as_ref().get_organization_by_slug(slug).await
    }
    async fn create_organization(
        &self,
        payload: crate::models::OrganizationCreate,
    ) -> crate::error::Result<crate::models::Organization> {
        self.inner.as_ref().create_organization(payload).await
    }
    async fn update_organization(
        &self,
        id: i32,
        payload: crate::models::OrganizationUpdate,
    ) -> crate::error::Result<crate::models::Organization> {
        self.inner.as_ref().update_organization(id, payload).await
    }
    async fn delete_organization(&self, id: i32) -> crate::error::Result<()> {
        self.inner.as_ref().delete_organization(id).await
    }
    async fn get_organization_users(
        &self,
        org_id: i32,
    ) -> crate::error::Result<Vec<crate::models::OrganizationUser>> {
        self.inner.as_ref().get_organization_users(org_id).await
    }
    async fn add_user_to_organization(
        &self,
        payload: crate::models::OrganizationUserCreate,
    ) -> crate::error::Result<crate::models::OrganizationUser> {
        self.inner.as_ref().add_user_to_organization(payload).await
    }
    async fn remove_user_from_organization(
        &self,
        org_id: i32,
        user_id: i32,
    ) -> crate::error::Result<()> {
        self.inner
            .as_ref()
            .remove_user_from_organization(org_id, user_id)
            .await
    }
    async fn update_user_organization_role(
        &self,
        org_id: i32,
        user_id: i32,
        role: &str,
    ) -> crate::error::Result<()> {
        self.inner
            .as_ref()
            .update_user_organization_role(org_id, user_id, role)
            .await
    }
    async fn get_user_organizations(
        &self,
        user_id: i32,
    ) -> crate::error::Result<Vec<crate::models::Organization>> {
        self.inner.as_ref().get_user_organizations(user_id).await
    }
    async fn check_organization_quota(
        &self,
        org_id: i32,
        quota_type: &str,
    ) -> crate::error::Result<bool> {
        self.inner
            .as_ref()
            .check_organization_quota(org_id, quota_type)
            .await
    }
}

#[async_trait]
impl crate::db::store::DeploymentEnvironmentManager for StoreWrapper {
    async fn get_deployment_environments(
        &self,
        project_id: i32,
    ) -> Result<Vec<crate::models::DeploymentEnvironment>> {
        self.inner
            .as_ref()
            .get_deployment_environments(project_id)
            .await
    }
    async fn get_deployment_environment(
        &self,
        id: i32,
        project_id: i32,
    ) -> Result<crate::models::DeploymentEnvironment> {
        self.inner
            .as_ref()
            .get_deployment_environment(id, project_id)
            .await
    }
    async fn create_deployment_environment(
        &self,
        project_id: i32,
        payload: crate::models::DeploymentEnvironmentCreate,
    ) -> Result<crate::models::DeploymentEnvironment> {
        self.inner
            .as_ref()
            .create_deployment_environment(project_id, payload)
            .await
    }
    async fn update_deployment_environment(
        &self,
        id: i32,
        project_id: i32,
        payload: crate::models::DeploymentEnvironmentUpdate,
    ) -> Result<crate::models::DeploymentEnvironment> {
        self.inner
            .as_ref()
            .update_deployment_environment(id, project_id, payload)
            .await
    }
    async fn delete_deployment_environment(&self, id: i32, project_id: i32) -> Result<()> {
        self.inner
            .as_ref()
            .delete_deployment_environment(id, project_id)
            .await
    }
    async fn get_deployment_history(
        &self,
        env_id: i32,
        project_id: i32,
    ) -> Result<Vec<crate::models::DeploymentRecord>> {
        self.inner
            .as_ref()
            .get_deployment_history(env_id, project_id)
            .await
    }
    async fn record_deployment(
        &self,
        env_id: i32,
        task_id: i32,
        project_id: i32,
        version: Option<String>,
        deployed_by: Option<i32>,
        status: &str,
    ) -> Result<()> {
        self.inner
            .as_ref()
            .record_deployment(env_id, task_id, project_id, version, deployed_by, status)
            .await
    }
}

#[async_trait]
impl crate::db::store::StructuredOutputManager for StoreWrapper {
    async fn get_task_structured_outputs(
        &self,
        task_id: i32,
        project_id: i32,
    ) -> Result<Vec<crate::models::TaskStructuredOutput>> {
        self.inner
            .as_ref()
            .get_task_structured_outputs(task_id, project_id)
            .await
    }
    async fn get_task_outputs_map(
        &self,
        task_id: i32,
        project_id: i32,
    ) -> Result<crate::models::TaskOutputsMap> {
        self.inner
            .as_ref()
            .get_task_outputs_map(task_id, project_id)
            .await
    }
    async fn create_task_structured_output(
        &self,
        task_id: i32,
        project_id: i32,
        payload: crate::models::TaskStructuredOutputCreate,
    ) -> Result<crate::models::TaskStructuredOutput> {
        self.inner
            .as_ref()
            .create_task_structured_output(task_id, project_id, payload)
            .await
    }
    async fn create_task_structured_outputs_batch(
        &self,
        task_id: i32,
        project_id: i32,
        outputs: Vec<crate::models::TaskStructuredOutputCreate>,
    ) -> Result<()> {
        self.inner
            .as_ref()
            .create_task_structured_outputs_batch(task_id, project_id, outputs)
            .await
    }
    async fn delete_task_structured_outputs(&self, task_id: i32, project_id: i32) -> Result<()> {
        self.inner
            .as_ref()
            .delete_task_structured_outputs(task_id, project_id)
            .await
    }
    async fn get_template_last_outputs(
        &self,
        template_id: i32,
        project_id: i32,
    ) -> Result<crate::models::TaskOutputsMap> {
        self.inner
            .as_ref()
            .get_template_last_outputs(template_id, project_id)
            .await
    }
}

impl Store for StoreWrapper {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // ============================================
    // Mock Store для тестирования
    // ============================================

    struct MockStore;

    #[async_trait]
    impl ConnectionManager for MockStore {
        async fn connect(&self) -> Result<()> {
            Ok(())
        }
        async fn close(&self) -> Result<()> {
            Ok(())
        }
        fn is_permanent(&self) -> bool {
            true
        }
    }

    #[async_trait]
    impl MigrationManager for MockStore {
        fn get_dialect(&self) -> &str {
            "mock"
        }
        async fn is_initialized(&self) -> Result<bool> {
            Ok(true)
        }
        async fn apply_migration(&self, _version: i64, _name: String) -> Result<()> {
            Ok(())
        }
        async fn is_migration_applied(&self, _version: i64) -> Result<bool> {
            Ok(true)
        }
    }

    #[async_trait]
    impl OptionsManager for MockStore {
        async fn get_options(&self) -> Result<HashMap<String, String>> {
            Ok(HashMap::new())
        }
        async fn get_option(&self, _key: &str) -> Result<Option<String>> {
            Ok(None)
        }
        async fn set_option(&self, _key: &str, _value: &str) -> Result<()> {
            Ok(())
        }
        async fn delete_option(&self, _key: &str) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl UserManager for MockStore {
        async fn get_users(&self, _params: RetrieveQueryParams) -> Result<Vec<User>> {
            Ok(vec![])
        }
        async fn get_user(&self, _user_id: i32) -> Result<User> {
            Err(crate::error::Error::NotFound("User not found".into()))
        }
        async fn get_user_by_login_or_email(&self, _login: &str, _email: &str) -> Result<User> {
            Err(crate::error::Error::NotFound("User not found".into()))
        }
        async fn create_user(&self, _user: User, _password: &str) -> Result<User> {
            Err(crate::error::Error::Validation("Cannot create user".into()))
        }
        async fn update_user(&self, _user: User) -> Result<()> {
            Ok(())
        }
        async fn delete_user(&self, _user_id: i32) -> Result<()> {
            Ok(())
        }
        async fn set_user_password(&self, _user_id: i32, _password: &str) -> Result<()> {
            Ok(())
        }
        async fn get_all_admins(&self) -> Result<Vec<User>> {
            Ok(vec![])
        }
        async fn get_user_count(&self) -> Result<usize> {
            Ok(0)
        }
        async fn get_project_users(
            &self,
            _project_id: i32,
            _params: RetrieveQueryParams,
        ) -> Result<Vec<ProjectUser>> {
            Ok(vec![])
        }
        async fn get_user_totp(&self, _user_id: i32) -> Result<Option<UserTotp>> {
            Ok(None)
        }
        async fn set_user_totp(&self, _user_id: i32, _totp: &UserTotp) -> Result<()> {
            Ok(())
        }
        async fn delete_user_totp(&self, _user_id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl ProjectStore for MockStore {
        async fn get_projects(&self, _user_id: Option<i32>) -> Result<Vec<Project>> {
            Ok(vec![])
        }
        async fn get_project(&self, _project_id: i32) -> Result<Project> {
            Err(crate::error::Error::NotFound("Project not found".into()))
        }
        async fn create_project(&self, _project: Project) -> Result<Project> {
            Err(crate::error::Error::Validation(
                "Cannot create project".into(),
            ))
        }
        async fn update_project(&self, _project: Project) -> Result<()> {
            Ok(())
        }
        async fn delete_project(&self, _project_id: i32) -> Result<()> {
            Ok(())
        }
        async fn create_project_user(&self, _project_user: ProjectUser) -> Result<()> {
            Ok(())
        }
        async fn delete_project_user(&self, _project_id: i32, _user_id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl TemplateManager for MockStore {
        async fn get_templates(&self, _project_id: i32) -> Result<Vec<Template>> {
            Ok(vec![])
        }
        async fn get_template(&self, _project_id: i32, _template_id: i32) -> Result<Template> {
            Err(crate::error::Error::NotFound("Template not found".into()))
        }
        async fn create_template(&self, _template: Template) -> Result<Template> {
            Err(crate::error::Error::Validation(
                "Cannot create template".into(),
            ))
        }
        async fn update_template(&self, _template: Template) -> Result<()> {
            Ok(())
        }
        async fn delete_template(&self, _project_id: i32, _template_id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl HookManager for MockStore {
        async fn get_hooks_by_template(&self, _template_id: i32) -> Result<Vec<Hook>> {
            Ok(vec![])
        }
    }

    #[async_trait]
    impl InventoryManager for MockStore {
        async fn get_inventories(&self, _project_id: i32) -> Result<Vec<Inventory>> {
            Ok(vec![])
        }
        async fn get_inventory(&self, _project_id: i32, _inventory_id: i32) -> Result<Inventory> {
            Err(crate::error::Error::NotFound("Inventory not found".into()))
        }
        async fn create_inventory(&self, _inventory: Inventory) -> Result<Inventory> {
            Err(crate::error::Error::Validation(
                "Cannot create inventory".into(),
            ))
        }
        async fn update_inventory(&self, _inventory: Inventory) -> Result<()> {
            Ok(())
        }
        async fn delete_inventory(&self, _project_id: i32, _inventory_id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl RepositoryManager for MockStore {
        async fn get_repositories(&self, _project_id: i32) -> Result<Vec<Repository>> {
            Ok(vec![])
        }
        async fn get_repository(
            &self,
            _project_id: i32,
            _repository_id: i32,
        ) -> Result<Repository> {
            Err(crate::error::Error::NotFound("Repository not found".into()))
        }
        async fn create_repository(&self, _repository: Repository) -> Result<Repository> {
            Err(crate::error::Error::Validation(
                "Cannot create repository".into(),
            ))
        }
        async fn update_repository(&self, _repository: Repository) -> Result<()> {
            Ok(())
        }
        async fn delete_repository(&self, _project_id: i32, _repository_id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl EnvironmentManager for MockStore {
        async fn get_environments(&self, _project_id: i32) -> Result<Vec<Environment>> {
            Ok(vec![])
        }
        async fn get_environment(
            &self,
            _project_id: i32,
            _environment_id: i32,
        ) -> Result<Environment> {
            Err(crate::error::Error::NotFound(
                "Environment not found".into(),
            ))
        }
        async fn create_environment(&self, _environment: Environment) -> Result<Environment> {
            Err(crate::error::Error::Validation(
                "Cannot create environment".into(),
            ))
        }
        async fn update_environment(&self, _environment: Environment) -> Result<()> {
            Ok(())
        }
        async fn delete_environment(&self, _project_id: i32, _environment_id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl AccessKeyManager for MockStore {
        async fn get_access_keys(&self, _project_id: i32) -> Result<Vec<AccessKey>> {
            Ok(vec![])
        }
        async fn get_access_key(&self, _project_id: i32, _access_key_id: i32) -> Result<AccessKey> {
            Err(crate::error::Error::NotFound("AccessKey not found".into()))
        }
        async fn create_access_key(&self, _access_key: AccessKey) -> Result<AccessKey> {
            Err(crate::error::Error::Validation(
                "Cannot create access key".into(),
            ))
        }
        async fn update_access_key(&self, _access_key: AccessKey) -> Result<()> {
            Ok(())
        }
        async fn delete_access_key(&self, _project_id: i32, _access_key_id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl TaskManager for MockStore {
        async fn get_tasks(
            &self,
            _project_id: i32,
            _template_id: Option<i32>,
        ) -> Result<Vec<TaskWithTpl>> {
            Ok(vec![])
        }
        async fn get_task(&self, _project_id: i32, _task_id: i32) -> Result<Task> {
            Err(crate::error::Error::NotFound("Task not found".into()))
        }
        async fn create_task(&self, _task: Task) -> Result<Task> {
            Err(crate::error::Error::Validation("Cannot create task".into()))
        }
        async fn update_task(&self, _task: Task) -> Result<()> {
            Ok(())
        }
        async fn delete_task(&self, _project_id: i32, _task_id: i32) -> Result<()> {
            Ok(())
        }
        async fn get_task_outputs(&self, _task_id: i32) -> Result<Vec<TaskOutput>> {
            Ok(vec![])
        }
        async fn create_task_output(&self, _output: TaskOutput) -> Result<TaskOutput> {
            Err(crate::error::Error::Validation(
                "Cannot create output".into(),
            ))
        }
        async fn update_task_status(
            &self,
            _project_id: i32,
            _task_id: i32,
            _status: TaskStatus,
        ) -> Result<()> {
            Ok(())
        }
        async fn get_global_tasks(
            &self,
            _status_filter: Option<Vec<String>>,
            _limit: Option<i32>,
        ) -> Result<Vec<TaskWithTpl>> {
            Ok(vec![])
        }
        async fn get_running_tasks_count(&self) -> Result<usize> {
            Ok(0)
        }
        async fn get_waiting_tasks_count(&self) -> Result<usize> {
            Ok(0)
        }
    }

    #[async_trait]
    impl ScheduleManager for MockStore {
        async fn get_schedules(&self, _project_id: i32) -> Result<Vec<Schedule>> {
            Ok(vec![])
        }
        async fn get_all_schedules(&self) -> Result<Vec<Schedule>> {
            Ok(vec![])
        }
        async fn get_schedule(&self, _project_id: i32, _schedule_id: i32) -> Result<Schedule> {
            Err(crate::error::Error::NotFound("Schedule not found".into()))
        }
        async fn create_schedule(&self, _schedule: Schedule) -> Result<Schedule> {
            Err(crate::error::Error::Validation(
                "Cannot create schedule".into(),
            ))
        }
        async fn update_schedule(&self, _schedule: Schedule) -> Result<()> {
            Ok(())
        }
        async fn delete_schedule(&self, _project_id: i32, _schedule_id: i32) -> Result<()> {
            Ok(())
        }
        async fn set_schedule_active(
            &self,
            _project_id: i32,
            _schedule_id: i32,
            _active: bool,
        ) -> Result<()> {
            Ok(())
        }
        async fn set_schedule_commit_hash(
            &self,
            _project_id: i32,
            _schedule_id: i32,
            _hash: &str,
        ) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl SessionManager for MockStore {
        async fn get_session(&self, _user_id: i32, _session_id: i32) -> Result<Session> {
            Err(crate::error::Error::NotFound("Session not found".into()))
        }
        async fn create_session(&self, _session: Session) -> Result<Session> {
            Err(crate::error::Error::Validation(
                "Cannot create session".into(),
            ))
        }
        async fn expire_session(&self, _user_id: i32, _session_id: i32) -> Result<()> {
            Ok(())
        }
        async fn verify_session(&self, _user_id: i32, _session_id: i32) -> Result<()> {
            Ok(())
        }
        async fn touch_session(&self, _user_id: i32, _session_id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl TokenManager for MockStore {
        async fn get_api_tokens(&self, _user_id: i32) -> Result<Vec<APIToken>> {
            Ok(vec![])
        }
        async fn create_api_token(&self, _token: APIToken) -> Result<APIToken> {
            Err(crate::error::Error::Validation(
                "Cannot create token".into(),
            ))
        }
        async fn get_api_token(&self, _token_id: i32) -> Result<APIToken> {
            Err(crate::error::Error::NotFound("Token not found".into()))
        }
        async fn expire_api_token(&self, _user_id: i32, _token_id: i32) -> Result<()> {
            Ok(())
        }
        async fn delete_api_token(&self, _user_id: i32, _token_id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl EventManager for MockStore {
        async fn get_events(&self, _project_id: Option<i32>, _limit: usize) -> Result<Vec<Event>> {
            Ok(vec![])
        }
        async fn create_event(&self, _event: Event) -> Result<Event> {
            Err(crate::error::Error::Validation(
                "Cannot create event".into(),
            ))
        }
    }

    #[async_trait]
    impl RunnerManager for MockStore {
        async fn get_runners(&self, _project_id: Option<i32>) -> Result<Vec<Runner>> {
            Ok(vec![])
        }
        async fn get_runner(&self, _runner_id: i32) -> Result<Runner> {
            Err(crate::error::Error::NotFound("Runner not found".into()))
        }
        async fn create_runner(&self, _runner: Runner) -> Result<Runner> {
            Err(crate::error::Error::Validation(
                "Cannot create runner".into(),
            ))
        }
        async fn update_runner(&self, _runner: Runner) -> Result<()> {
            Ok(())
        }
        async fn delete_runner(&self, _runner_id: i32) -> Result<()> {
            Ok(())
        }
        async fn get_runners_count(&self) -> Result<usize> {
            Ok(0)
        }
        async fn get_active_runners_count(&self) -> Result<usize> {
            Ok(0)
        }
        async fn find_runner_by_token(&self, _token: &str) -> Result<Runner> {
            Err(crate::error::Error::NotFound("Runner not found".into()))
        }
        async fn touch_runner(&self, _runner_id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl ViewManager for MockStore {
        async fn get_views(&self, _project_id: i32) -> Result<Vec<View>> {
            Ok(vec![])
        }
        async fn get_view(&self, _project_id: i32, _view_id: i32) -> Result<View> {
            Err(crate::error::Error::NotFound("View not found".into()))
        }
        async fn create_view(&self, _view: View) -> Result<View> {
            Err(crate::error::Error::Validation("Cannot create view".into()))
        }
        async fn update_view(&self, _view: View) -> Result<()> {
            Ok(())
        }
        async fn delete_view(&self, _project_id: i32, _view_id: i32) -> Result<()> {
            Ok(())
        }
        async fn set_view_positions(
            &self,
            _project_id: i32,
            _positions: Vec<(i32, i32)>,
        ) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl IntegrationManager for MockStore {
        async fn get_integrations(&self, _project_id: i32) -> Result<Vec<Integration>> {
            Ok(vec![])
        }
        async fn get_integration(
            &self,
            _project_id: i32,
            _integration_id: i32,
        ) -> Result<Integration> {
            Err(crate::error::Error::NotFound(
                "Integration not found".into(),
            ))
        }
        async fn create_integration(&self, _integration: Integration) -> Result<Integration> {
            Err(crate::error::Error::Validation(
                "Cannot create integration".into(),
            ))
        }
        async fn update_integration(&self, _integration: Integration) -> Result<()> {
            Ok(())
        }
        async fn delete_integration(&self, _project_id: i32, _integration_id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl ProjectInviteManager for MockStore {
        async fn get_project_invites(
            &self,
            _project_id: i32,
            _params: RetrieveQueryParams,
        ) -> Result<Vec<ProjectInviteWithUser>> {
            Ok(vec![])
        }
        async fn create_project_invite(&self, _invite: ProjectInvite) -> Result<ProjectInvite> {
            Err(crate::error::Error::Validation(
                "Cannot create invite".into(),
            ))
        }
        async fn get_project_invite(
            &self,
            _project_id: i32,
            _invite_id: i32,
        ) -> Result<ProjectInvite> {
            Err(crate::error::Error::NotFound("Invite not found".into()))
        }
        async fn get_project_invite_by_token(&self, _token: &str) -> Result<ProjectInvite> {
            Err(crate::error::Error::NotFound("Invite not found".into()))
        }
        async fn update_project_invite(&self, _invite: ProjectInvite) -> Result<()> {
            Ok(())
        }
        async fn delete_project_invite(&self, _project_id: i32, _invite_id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl TerraformInventoryManager for MockStore {
        async fn create_terraform_inventory_alias(
            &self,
            _alias: TerraformInventoryAlias,
        ) -> Result<TerraformInventoryAlias> {
            Err(crate::error::Error::Validation(
                "Cannot create alias".into(),
            ))
        }
        async fn update_terraform_inventory_alias(
            &self,
            _alias: TerraformInventoryAlias,
        ) -> Result<()> {
            Ok(())
        }
        async fn get_terraform_inventory_alias_by_alias(
            &self,
            _alias: &str,
        ) -> Result<TerraformInventoryAlias> {
            Err(crate::error::Error::NotFound("Alias not found".into()))
        }
        async fn get_terraform_inventory_alias(
            &self,
            _project_id: i32,
            _inventory_id: i32,
            _alias_id: &str,
        ) -> Result<TerraformInventoryAlias> {
            Err(crate::error::Error::NotFound("Alias not found".into()))
        }
        async fn get_terraform_inventory_aliases(
            &self,
            _project_id: i32,
            _inventory_id: i32,
        ) -> Result<Vec<TerraformInventoryAlias>> {
            Ok(vec![])
        }
        async fn delete_terraform_inventory_alias(
            &self,
            _project_id: i32,
            _inventory_id: i32,
            _alias_id: &str,
        ) -> Result<()> {
            Ok(())
        }
        async fn get_terraform_inventory_states(
            &self,
            _project_id: i32,
            _inventory_id: i32,
            _params: RetrieveQueryParams,
        ) -> Result<Vec<TerraformInventoryState>> {
            Ok(vec![])
        }
        async fn create_terraform_inventory_state(
            &self,
            _state: TerraformInventoryState,
        ) -> Result<TerraformInventoryState> {
            Err(crate::error::Error::Validation(
                "Cannot create state".into(),
            ))
        }
        async fn delete_terraform_inventory_state(
            &self,
            _project_id: i32,
            _inventory_id: i32,
            _state_id: i32,
        ) -> Result<()> {
            Ok(())
        }
        async fn get_terraform_inventory_state(
            &self,
            _project_id: i32,
            _inventory_id: i32,
            _state_id: i32,
        ) -> Result<TerraformInventoryState> {
            Err(crate::error::Error::NotFound("State not found".into()))
        }
        async fn get_terraform_state_count(&self) -> Result<i32> {
            Ok(0)
        }
    }

    #[async_trait]
    impl SecretStorageManager for MockStore {
        async fn get_secret_storages(&self, _project_id: i32) -> Result<Vec<SecretStorage>> {
            Ok(vec![])
        }
        async fn get_secret_storage(
            &self,
            _project_id: i32,
            _storage_id: i32,
        ) -> Result<SecretStorage> {
            Err(crate::error::Error::NotFound(
                "Secret storage not found".into(),
            ))
        }
        async fn create_secret_storage(&self, _storage: SecretStorage) -> Result<SecretStorage> {
            Err(crate::error::Error::Validation(
                "Cannot create storage".into(),
            ))
        }
        async fn update_secret_storage(&self, _storage: SecretStorage) -> Result<()> {
            Ok(())
        }
        async fn delete_secret_storage(&self, _project_id: i32, _storage_id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl AuditLogManager for MockStore {
        async fn create_audit_log(
            &self,
            _project_id: Option<i64>,
            _user_id: Option<i64>,
            _username: Option<String>,
            _action: &AuditAction,
            _object_type: &AuditObjectType,
            _object_id: Option<i64>,
            _object_name: Option<String>,
            _description: String,
            _level: &AuditLevel,
            _ip_address: Option<String>,
            _user_agent: Option<String>,
            _details: Option<serde_json::Value>,
        ) -> Result<AuditLog> {
            Err(crate::error::Error::Validation(
                "Cannot create audit log".into(),
            ))
        }
        async fn get_audit_log(&self, _id: i64) -> Result<AuditLog> {
            Err(crate::error::Error::NotFound("Audit log not found".into()))
        }
        async fn search_audit_logs(&self, _filter: &AuditLogFilter) -> Result<AuditLogResult> {
            Ok(AuditLogResult {
                total: 0,
                records: vec![],
                limit: 0,
                offset: 0,
            })
        }
        async fn get_audit_logs_by_project(
            &self,
            _project_id: i64,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<AuditLog>> {
            Ok(vec![])
        }
        async fn get_audit_logs_by_user(
            &self,
            _user_id: i64,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<AuditLog>> {
            Ok(vec![])
        }
        async fn get_audit_logs_by_action(
            &self,
            _action: &AuditAction,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<AuditLog>> {
            Ok(vec![])
        }
        async fn delete_audit_logs_before(&self, _before: DateTime<Utc>) -> Result<u64> {
            Ok(0)
        }
        async fn clear_audit_log(&self) -> Result<u64> {
            Ok(0)
        }
    }

    #[async_trait]
    impl crate::db::store::WebhookManager for MockStore {
        async fn get_webhook(&self, _webhook_id: i64) -> Result<Webhook> {
            Err(crate::error::Error::NotFound("Webhook not found".into()))
        }
        async fn get_webhooks_by_project(&self, _project_id: i64) -> Result<Vec<Webhook>> {
            Ok(vec![])
        }
        async fn create_webhook(&self, _webhook: Webhook) -> Result<Webhook> {
            Err(crate::error::Error::Validation(
                "Cannot create webhook".into(),
            ))
        }
        async fn update_webhook(
            &self,
            _webhook_id: i64,
            _webhook: UpdateWebhook,
        ) -> Result<Webhook> {
            Err(crate::error::Error::Validation(
                "Cannot update webhook".into(),
            ))
        }
        async fn delete_webhook(&self, _webhook_id: i64) -> Result<()> {
            Ok(())
        }
        async fn get_webhook_logs(&self, _webhook_id: i64) -> Result<Vec<WebhookLog>> {
            Ok(vec![])
        }
        async fn create_webhook_log(&self, _log: WebhookLog) -> Result<WebhookLog> {
            Err(crate::error::Error::Validation("Cannot create log".into()))
        }
    }

    #[async_trait]
    impl crate::db::store::PlaybookManager for MockStore {
        async fn get_playbooks(&self, _project_id: i32) -> Result<Vec<crate::models::Playbook>> {
            Ok(vec![])
        }
        async fn get_playbook(
            &self,
            _id: i32,
            _project_id: i32,
        ) -> Result<crate::models::Playbook> {
            Err(crate::error::Error::NotFound("Playbook not found".into()))
        }
        async fn create_playbook(
            &self,
            _project_id: i32,
            _playbook: crate::models::PlaybookCreate,
        ) -> Result<crate::models::Playbook> {
            Err(crate::error::Error::Validation(
                "Cannot create playbook".into(),
            ))
        }
        async fn update_playbook(
            &self,
            _id: i32,
            _project_id: i32,
            _playbook: crate::models::PlaybookUpdate,
        ) -> Result<crate::models::Playbook> {
            Err(crate::error::Error::Validation(
                "Cannot update playbook".into(),
            ))
        }
        async fn delete_playbook(&self, _id: i32, _project_id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl crate::db::store::PlaybookRunManager for MockStore {
        async fn get_playbook_runs(&self, _filter: PlaybookRunFilter) -> Result<Vec<PlaybookRun>> {
            Ok(vec![])
        }
        async fn get_playbook_run(&self, _id: i32, _project_id: i32) -> Result<PlaybookRun> {
            Err(crate::error::Error::NotFound(
                "Playbook run not found".into(),
            ))
        }
        async fn get_playbook_run_by_task_id(&self, _task_id: i32) -> Result<Option<PlaybookRun>> {
            Ok(None)
        }
        async fn create_playbook_run(&self, _run: PlaybookRunCreate) -> Result<PlaybookRun> {
            Err(crate::error::Error::Validation("Cannot create run".into()))
        }
        async fn update_playbook_run(
            &self,
            _id: i32,
            _project_id: i32,
            _update: PlaybookRunUpdate,
        ) -> Result<PlaybookRun> {
            Err(crate::error::Error::Validation("Cannot update run".into()))
        }
        async fn update_playbook_run_status(
            &self,
            _id: i32,
            _status: PlaybookRunStatus,
        ) -> Result<()> {
            Ok(())
        }
        async fn delete_playbook_run(&self, _id: i32, _project_id: i32) -> Result<()> {
            Ok(())
        }
        async fn get_playbook_run_stats(&self, _playbook_id: i32) -> Result<PlaybookRunStats> {
            Ok(PlaybookRunStats {
                total_runs: 0,
                success_runs: 0,
                failed_runs: 0,
                avg_duration_seconds: None,
                last_run: None,
            })
        }
    }

    #[async_trait]
    impl IntegrationMatcherManager for MockStore {
        async fn get_integration_matchers(
            &self,
            _project_id: i32,
            _integration_id: i32,
        ) -> Result<Vec<IntegrationMatcher>> {
            Ok(vec![])
        }
        async fn create_integration_matcher(
            &self,
            _matcher: IntegrationMatcher,
        ) -> Result<IntegrationMatcher> {
            Err(crate::error::Error::Validation(
                "Cannot create matcher".into(),
            ))
        }
        async fn update_integration_matcher(&self, _matcher: IntegrationMatcher) -> Result<()> {
            Ok(())
        }
        async fn delete_integration_matcher(
            &self,
            _project_id: i32,
            _integration_id: i32,
            _matcher_id: i32,
        ) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl IntegrationExtractValueManager for MockStore {
        async fn get_integration_extract_values(
            &self,
            _project_id: i32,
            _integration_id: i32,
        ) -> Result<Vec<IntegrationExtractValue>> {
            Ok(vec![])
        }
        async fn create_integration_extract_value(
            &self,
            _value: IntegrationExtractValue,
        ) -> Result<IntegrationExtractValue> {
            Err(crate::error::Error::Validation(
                "Cannot create value".into(),
            ))
        }
        async fn update_integration_extract_value(
            &self,
            _value: IntegrationExtractValue,
        ) -> Result<()> {
            Ok(())
        }
        async fn delete_integration_extract_value(
            &self,
            _project_id: i32,
            _integration_id: i32,
            _value_id: i32,
        ) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl ProjectRoleManager for MockStore {
        async fn get_project_roles(&self, _project_id: i32) -> Result<Vec<crate::models::Role>> {
            Ok(vec![])
        }
        async fn create_project_role(
            &self,
            _role: crate::models::Role,
        ) -> Result<crate::models::Role> {
            Err(crate::error::Error::Validation("Cannot create role".into()))
        }
        async fn update_project_role(&self, _role: crate::models::Role) -> Result<()> {
            Ok(())
        }
        async fn delete_project_role(&self, _project_id: i32, _role_id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl crate::db::store::WorkflowManager for MockStore {
        async fn get_workflows(&self, _project_id: i32) -> Result<Vec<Workflow>> {
            Ok(vec![])
        }
        async fn get_workflow(&self, _id: i32, _project_id: i32) -> Result<Workflow> {
            Err(crate::error::Error::NotFound("Workflow not found".into()))
        }
        async fn create_workflow(
            &self,
            _project_id: i32,
            _payload: WorkflowCreate,
        ) -> Result<Workflow> {
            Err(crate::error::Error::Validation(
                "Cannot create workflow".into(),
            ))
        }
        async fn update_workflow(
            &self,
            _id: i32,
            _project_id: i32,
            _payload: WorkflowUpdate,
        ) -> Result<Workflow> {
            Err(crate::error::Error::Validation(
                "Cannot update workflow".into(),
            ))
        }
        async fn delete_workflow(&self, _id: i32, _project_id: i32) -> Result<()> {
            Ok(())
        }
        async fn get_workflow_nodes(&self, _workflow_id: i32) -> Result<Vec<WorkflowNode>> {
            Ok(vec![])
        }
        async fn create_workflow_node(
            &self,
            _workflow_id: i32,
            _payload: WorkflowNodeCreate,
        ) -> Result<WorkflowNode> {
            Err(crate::error::Error::Validation("Cannot create node".into()))
        }
        async fn update_workflow_node(
            &self,
            _id: i32,
            _workflow_id: i32,
            _payload: WorkflowNodeUpdate,
        ) -> Result<WorkflowNode> {
            Err(crate::error::Error::Validation("Cannot update node".into()))
        }
        async fn delete_workflow_node(&self, _id: i32, _workflow_id: i32) -> Result<()> {
            Ok(())
        }
        async fn get_workflow_edges(&self, _workflow_id: i32) -> Result<Vec<WorkflowEdge>> {
            Ok(vec![])
        }
        async fn create_workflow_edge(
            &self,
            _workflow_id: i32,
            _payload: WorkflowEdgeCreate,
        ) -> Result<WorkflowEdge> {
            Err(crate::error::Error::Validation("Cannot create edge".into()))
        }
        async fn delete_workflow_edge(&self, _id: i32, _workflow_id: i32) -> Result<()> {
            Ok(())
        }
        async fn get_workflow_runs(
            &self,
            _workflow_id: i32,
            _project_id: i32,
        ) -> Result<Vec<WorkflowRun>> {
            Ok(vec![])
        }
        async fn create_workflow_run(
            &self,
            _workflow_id: i32,
            _project_id: i32,
        ) -> Result<WorkflowRun> {
            Err(crate::error::Error::Validation("Cannot create run".into()))
        }
        async fn update_workflow_run_status(
            &self,
            _id: i32,
            _status: &str,
            _message: Option<String>,
        ) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl crate::db::store::NotificationPolicyManager for MockStore {
        async fn get_notification_policies(
            &self,
            _project_id: i32,
        ) -> Result<Vec<NotificationPolicy>> {
            Ok(vec![])
        }
        async fn get_notification_policy(
            &self,
            _id: i32,
            _project_id: i32,
        ) -> Result<NotificationPolicy> {
            Err(crate::error::Error::NotFound("Policy not found".into()))
        }
        async fn create_notification_policy(
            &self,
            _project_id: i32,
            _payload: NotificationPolicyCreate,
        ) -> Result<NotificationPolicy> {
            Err(crate::error::Error::Validation(
                "Cannot create policy".into(),
            ))
        }
        async fn update_notification_policy(
            &self,
            _id: i32,
            _project_id: i32,
            _payload: NotificationPolicyUpdate,
        ) -> Result<NotificationPolicy> {
            Err(crate::error::Error::Validation(
                "Cannot update policy".into(),
            ))
        }
        async fn delete_notification_policy(&self, _id: i32, _project_id: i32) -> Result<()> {
            Ok(())
        }
        async fn get_matching_policies(
            &self,
            _project_id: i32,
            _trigger: &str,
            _template_id: Option<i32>,
        ) -> Result<Vec<NotificationPolicy>> {
            Ok(vec![])
        }
    }

    #[async_trait]
    impl crate::db::store::CredentialTypeManager for MockStore {
        async fn get_credential_types(
            &self,
        ) -> Result<Vec<crate::models::credential_type::CredentialType>> {
            Ok(vec![])
        }
        async fn get_credential_type(
            &self,
            _id: i32,
        ) -> Result<crate::models::credential_type::CredentialType> {
            Err(crate::error::Error::NotFound(
                "Credential type not found".into(),
            ))
        }
        async fn create_credential_type(
            &self,
            _payload: crate::models::credential_type::CredentialTypeCreate,
        ) -> Result<crate::models::credential_type::CredentialType> {
            Err(crate::error::Error::Validation("Cannot create type".into()))
        }
        async fn update_credential_type(
            &self,
            _id: i32,
            _payload: crate::models::credential_type::CredentialTypeUpdate,
        ) -> Result<crate::models::credential_type::CredentialType> {
            Err(crate::error::Error::Validation("Cannot update type".into()))
        }
        async fn delete_credential_type(&self, _id: i32) -> Result<()> {
            Ok(())
        }
        async fn get_credential_instances(
            &self,
            _project_id: i32,
        ) -> Result<Vec<crate::models::credential_type::CredentialInstance>> {
            Ok(vec![])
        }
        async fn get_credential_instance(
            &self,
            _id: i32,
            _project_id: i32,
        ) -> Result<crate::models::credential_type::CredentialInstance> {
            Err(crate::error::Error::NotFound(
                "Credential instance not found".into(),
            ))
        }
        async fn create_credential_instance(
            &self,
            _project_id: i32,
            _payload: crate::models::credential_type::CredentialInstanceCreate,
        ) -> Result<crate::models::credential_type::CredentialInstance> {
            Err(crate::error::Error::Validation(
                "Cannot create instance".into(),
            ))
        }
        async fn delete_credential_instance(&self, _id: i32, _project_id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl crate::db::store::DriftManager for MockStore {
        async fn get_drift_configs(
            &self,
            _project_id: i32,
        ) -> Result<Vec<crate::models::drift::DriftConfig>> {
            Ok(vec![])
        }
        async fn get_drift_config(
            &self,
            _id: i32,
            _project_id: i32,
        ) -> Result<crate::models::drift::DriftConfig> {
            Err(crate::error::Error::NotFound(
                "Drift config not found".into(),
            ))
        }
        async fn create_drift_config(
            &self,
            _project_id: i32,
            _payload: crate::models::drift::DriftConfigCreate,
        ) -> Result<crate::models::drift::DriftConfig> {
            Err(crate::error::Error::Validation(
                "Cannot create config".into(),
            ))
        }
        async fn update_drift_config_enabled(
            &self,
            _id: i32,
            _project_id: i32,
            _enabled: bool,
        ) -> Result<()> {
            Ok(())
        }
        async fn delete_drift_config(&self, _id: i32, _project_id: i32) -> Result<()> {
            Ok(())
        }
        async fn get_drift_results(
            &self,
            _drift_config_id: i32,
            _limit: i64,
        ) -> Result<Vec<crate::models::drift::DriftResult>> {
            Ok(vec![])
        }
        async fn create_drift_result(
            &self,
            _project_id: i32,
            _drift_config_id: i32,
            _template_id: i32,
            _status: &str,
            _summary: Option<String>,
            _task_id: Option<i32>,
        ) -> Result<crate::models::drift::DriftResult> {
            Err(crate::error::Error::Validation(
                "Cannot create drift result".into(),
            ))
        }
        async fn get_latest_drift_results(
            &self,
            _project_id: i32,
        ) -> Result<Vec<crate::models::drift::DriftResult>> {
            Ok(vec![])
        }
    }

    #[async_trait]
    impl crate::db::store::LdapGroupMappingManager for MockStore {
        async fn get_ldap_group_mappings(
            &self,
        ) -> Result<Vec<crate::models::ldap_group::LdapGroupMapping>> {
            Ok(vec![])
        }
        async fn create_ldap_group_mapping(
            &self,
            _payload: crate::models::ldap_group::LdapGroupMappingCreate,
        ) -> Result<crate::models::ldap_group::LdapGroupMapping> {
            Err(crate::error::Error::Validation(
                "Cannot create mapping".into(),
            ))
        }
        async fn delete_ldap_group_mapping(&self, _id: i32) -> Result<()> {
            Ok(())
        }
        async fn get_mappings_for_groups(
            &self,
            _group_dns: &[String],
        ) -> Result<Vec<crate::models::ldap_group::LdapGroupMapping>> {
            Ok(vec![])
        }
    }

    #[async_trait]
    impl crate::db::store::SnapshotManager for MockStore {
        async fn get_snapshots(
            &self,
            _project_id: i32,
            _template_id: Option<i32>,
            _limit: i64,
        ) -> Result<Vec<crate::models::snapshot::TaskSnapshot>> {
            Ok(vec![])
        }
        async fn get_snapshot(
            &self,
            _id: i32,
            _project_id: i32,
        ) -> Result<crate::models::snapshot::TaskSnapshot> {
            Err(crate::error::Error::NotFound("Snapshot not found".into()))
        }
        async fn create_snapshot(
            &self,
            _project_id: i32,
            _payload: crate::models::snapshot::TaskSnapshotCreate,
        ) -> Result<crate::models::snapshot::TaskSnapshot> {
            Err(crate::error::Error::Validation(
                "Cannot create snapshot".into(),
            ))
        }
        async fn delete_snapshot(&self, _id: i32, _project_id: i32) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl crate::db::store::CostEstimateManager for MockStore {
        async fn get_cost_estimates(
            &self,
            _project_id: i32,
            _limit: i64,
        ) -> Result<Vec<crate::models::cost_estimate::CostEstimate>> {
            Ok(vec![])
        }
        async fn get_cost_estimate_for_task(
            &self,
            _project_id: i32,
            _task_id: i32,
        ) -> Result<Option<crate::models::cost_estimate::CostEstimate>> {
            Ok(None)
        }
        async fn create_cost_estimate(
            &self,
            _payload: crate::models::cost_estimate::CostEstimateCreate,
        ) -> Result<crate::models::cost_estimate::CostEstimate> {
            Err(crate::error::Error::Validation(
                "Cannot create estimate".into(),
            ))
        }
        async fn get_cost_summaries(
            &self,
            _project_id: i32,
        ) -> Result<Vec<crate::models::cost_estimate::CostSummary>> {
            Ok(vec![])
        }
    }

    #[async_trait]
    impl crate::db::store::TerraformStateManager for MockStore {
        async fn get_terraform_state(
            &self,
            _project_id: i32,
            _workspace: &str,
        ) -> Result<Option<crate::models::TerraformState>> {
            Ok(None)
        }
        async fn list_terraform_states(
            &self,
            _project_id: i32,
            _workspace: &str,
        ) -> Result<Vec<crate::models::TerraformStateSummary>> {
            Ok(vec![])
        }
        async fn get_terraform_state_by_serial(
            &self,
            _project_id: i32,
            _workspace: &str,
            _serial: i32,
        ) -> Result<Option<crate::models::TerraformState>> {
            Ok(None)
        }
        async fn create_terraform_state(
            &self,
            _state: crate::models::TerraformState,
        ) -> Result<crate::models::TerraformState> {
            Err(crate::error::Error::Validation(
                "Cannot create state".into(),
            ))
        }
        async fn delete_terraform_state(&self, _project_id: i32, _workspace: &str) -> Result<()> {
            Ok(())
        }
        async fn delete_all_terraform_states(
            &self,
            _project_id: i32,
            _workspace: &str,
        ) -> Result<()> {
            Ok(())
        }
        async fn lock_terraform_state(
            &self,
            _project_id: i32,
            _workspace: &str,
            _lock: crate::models::TerraformStateLock,
        ) -> Result<crate::models::TerraformStateLock> {
            Err(crate::error::Error::Validation("Cannot lock state".into()))
        }
        async fn unlock_terraform_state(
            &self,
            _project_id: i32,
            _workspace: &str,
            _lock_id: &str,
        ) -> Result<()> {
            Ok(())
        }
        async fn get_terraform_lock(
            &self,
            _project_id: i32,
            _workspace: &str,
        ) -> Result<Option<crate::models::TerraformStateLock>> {
            Ok(None)
        }
        async fn list_terraform_workspaces(&self, _project_id: i32) -> Result<Vec<String>> {
            Ok(vec![])
        }
        async fn purge_expired_terraform_locks(&self) -> Result<u64> {
            Ok(0)
        }
    }

    #[async_trait]
    impl crate::db::store::PlanApprovalManager for MockStore {
        async fn create_plan(
            &self,
            _plan: crate::models::TerraformPlan,
        ) -> Result<crate::models::TerraformPlan> {
            Err(crate::error::Error::Validation("Cannot create plan".into()))
        }
        async fn get_plan_by_task(
            &self,
            _project_id: i32,
            _task_id: i32,
        ) -> Result<Option<crate::models::TerraformPlan>> {
            Ok(None)
        }
        async fn list_pending_plans(
            &self,
            _project_id: i32,
        ) -> Result<Vec<crate::models::TerraformPlan>> {
            Ok(vec![])
        }
        async fn approve_plan(
            &self,
            _id: i64,
            _reviewed_by: i32,
            _comment: Option<String>,
        ) -> Result<()> {
            Ok(())
        }
        async fn reject_plan(
            &self,
            _id: i64,
            _reviewed_by: i32,
            _comment: Option<String>,
        ) -> Result<()> {
            Ok(())
        }
        async fn update_plan_output(
            &self,
            _task_id: i32,
            _output: String,
            _json: Option<String>,
            _added: i32,
            _changed: i32,
            _removed: i32,
        ) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl crate::db::store::OrganizationManager for MockStore {
        async fn get_organizations(&self) -> Result<Vec<crate::models::Organization>> {
            Ok(vec![])
        }
        async fn get_organization(&self, _id: i32) -> Result<crate::models::Organization> {
            Err(crate::error::Error::NotFound(
                "Organization not found".into(),
            ))
        }
        async fn get_organization_by_slug(
            &self,
            _slug: &str,
        ) -> Result<crate::models::Organization> {
            Err(crate::error::Error::NotFound(
                "Organization not found".into(),
            ))
        }
        async fn create_organization(
            &self,
            _payload: crate::models::OrganizationCreate,
        ) -> Result<crate::models::Organization> {
            Err(crate::error::Error::Validation(
                "Cannot create organization".into(),
            ))
        }
        async fn update_organization(
            &self,
            _id: i32,
            _payload: crate::models::OrganizationUpdate,
        ) -> Result<crate::models::Organization> {
            Err(crate::error::Error::Validation(
                "Cannot update organization".into(),
            ))
        }
        async fn delete_organization(&self, _id: i32) -> Result<()> {
            Ok(())
        }
        async fn get_organization_users(
            &self,
            _org_id: i32,
        ) -> Result<Vec<crate::models::OrganizationUser>> {
            Ok(vec![])
        }
        async fn add_user_to_organization(
            &self,
            _payload: crate::models::OrganizationUserCreate,
        ) -> Result<crate::models::OrganizationUser> {
            Err(crate::error::Error::Validation("Cannot add user".into()))
        }
        async fn remove_user_from_organization(&self, _org_id: i32, _user_id: i32) -> Result<()> {
            Ok(())
        }
        async fn update_user_organization_role(
            &self,
            _org_id: i32,
            _user_id: i32,
            _role: &str,
        ) -> Result<()> {
            Ok(())
        }
        async fn get_user_organizations(
            &self,
            _user_id: i32,
        ) -> Result<Vec<crate::models::Organization>> {
            Ok(vec![])
        }
        async fn check_organization_quota(&self, _org_id: i32, _quota_type: &str) -> Result<bool> {
            Ok(true)
        }
    }

    #[async_trait]
    impl crate::db::store::DeploymentEnvironmentManager for MockStore {
        async fn get_deployment_environments(
            &self,
            _project_id: i32,
        ) -> Result<Vec<crate::models::DeploymentEnvironment>> {
            Ok(vec![])
        }
        async fn get_deployment_environment(
            &self,
            _id: i32,
            _project_id: i32,
        ) -> Result<crate::models::DeploymentEnvironment> {
            Err(crate::error::Error::NotFound(
                "Deployment environment not found".into(),
            ))
        }
        async fn create_deployment_environment(
            &self,
            _project_id: i32,
            _payload: crate::models::DeploymentEnvironmentCreate,
        ) -> Result<crate::models::DeploymentEnvironment> {
            Err(crate::error::Error::Validation(
                "Cannot create environment".into(),
            ))
        }
        async fn update_deployment_environment(
            &self,
            _id: i32,
            _project_id: i32,
            _payload: crate::models::DeploymentEnvironmentUpdate,
        ) -> Result<crate::models::DeploymentEnvironment> {
            Err(crate::error::Error::Validation(
                "Cannot update environment".into(),
            ))
        }
        async fn delete_deployment_environment(&self, _id: i32, _project_id: i32) -> Result<()> {
            Ok(())
        }
        async fn get_deployment_history(
            &self,
            _env_id: i32,
            _project_id: i32,
        ) -> Result<Vec<crate::models::DeploymentRecord>> {
            Ok(vec![])
        }
        async fn record_deployment(
            &self,
            _env_id: i32,
            _task_id: i32,
            _project_id: i32,
            _version: Option<String>,
            _deployed_by: Option<i32>,
            _status: &str,
        ) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl crate::db::store::StructuredOutputManager for MockStore {
        async fn get_task_structured_outputs(
            &self,
            _task_id: i32,
            _project_id: i32,
        ) -> Result<Vec<crate::models::TaskStructuredOutput>> {
            Ok(vec![])
        }
        async fn get_task_outputs_map(
            &self,
            _task_id: i32,
            _project_id: i32,
        ) -> Result<crate::models::TaskOutputsMap> {
            Ok(crate::models::TaskOutputsMap {
                task_id: 0,
                outputs: HashMap::new(),
            })
        }
        async fn create_task_structured_output(
            &self,
            _task_id: i32,
            _project_id: i32,
            _payload: crate::models::TaskStructuredOutputCreate,
        ) -> Result<crate::models::TaskStructuredOutput> {
            Err(crate::error::Error::Validation(
                "Cannot create output".into(),
            ))
        }
        async fn create_task_structured_outputs_batch(
            &self,
            _task_id: i32,
            _project_id: i32,
            _outputs: Vec<crate::models::TaskStructuredOutputCreate>,
        ) -> Result<()> {
            Ok(())
        }
        async fn delete_task_structured_outputs(
            &self,
            _task_id: i32,
            _project_id: i32,
        ) -> Result<()> {
            Ok(())
        }
        async fn get_template_last_outputs(
            &self,
            _template_id: i32,
            _project_id: i32,
        ) -> Result<crate::models::TaskOutputsMap> {
            Ok(crate::models::TaskOutputsMap {
                task_id: 0,
                outputs: HashMap::new(),
            })
        }
    }

    impl Store for MockStore {}

    // ============================================
    // Helper для создания StoreWrapper с MockStore
    // ============================================

    fn create_wrapper() -> StoreWrapper {
        let mock = Arc::new(MockStore);
        StoreWrapper::new(mock)
    }

    // ============================================
    // StoreWrapper структура и конструктор
    // ============================================

    #[test]
    fn test_store_wrapper_new() {
        let mock = Arc::new(MockStore);
        let _wrapper = StoreWrapper::new(mock);
        // Проверяем что wrapper создан и store() возвращает валидную ссылку
    }

    #[test]
    fn test_store_wrapper_clone() {
        let wrapper = create_wrapper();
        let cloned = wrapper.clone();
        // Оба указывают на один и тот же MockStore
        assert!(Arc::ptr_eq(&wrapper.inner, &cloned.inner));
    }

    #[test]
    fn test_store_wrapper_store_returns_reference() {
        let wrapper = create_wrapper();
        let store_ref = wrapper.store();
        // Проверка что ссылка валидна (не паникует)
        let dialect = store_ref.get_dialect();
        assert_eq!(dialect, "mock");
    }

    #[test]
    fn test_store_wrapper_as_arc() {
        let wrapper = create_wrapper();
        let arc = wrapper.as_arc();
        assert!(arc.is_permanent());
    }

    // ============================================
    // ConnectionManager тесты
    // ============================================

    #[tokio::test]
    async fn test_connection_manager_connect() {
        let wrapper = create_wrapper();
        let result = wrapper.connect().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connection_manager_close() {
        let wrapper = create_wrapper();
        let result = wrapper.close().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_connection_manager_is_permanent() {
        let wrapper = create_wrapper();
        assert!(wrapper.is_permanent());
    }

    // ============================================
    // MigrationManager тесты
    // ============================================

    #[test]
    fn test_migration_manager_get_dialect() {
        let wrapper = create_wrapper();
        assert_eq!(wrapper.get_dialect(), "mock");
    }

    #[tokio::test]
    async fn test_migration_manager_is_initialized() {
        let wrapper = create_wrapper();
        let result = wrapper.is_initialized().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_migration_manager_apply_migration() {
        let wrapper = create_wrapper();
        let result = wrapper
            .apply_migration(1, "test_migration".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_migration_manager_is_migration_applied() {
        let wrapper = create_wrapper();
        let result = wrapper.is_migration_applied(1).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    // ============================================
    // OptionsManager тесты
    // ============================================

    #[tokio::test]
    async fn test_options_manager_get_options() {
        let wrapper = create_wrapper();
        let result = wrapper.get_options().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_options_manager_get_option_returns_none() {
        let wrapper = create_wrapper();
        let result = wrapper.get_option("nonexistent").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_options_manager_set_and_delete_option() {
        let wrapper = create_wrapper();
        let set_result = wrapper.set_option("key", "value").await;
        assert!(set_result.is_ok());

        let delete_result = wrapper.delete_option("key").await;
        assert!(delete_result.is_ok());
    }

    // ============================================
    // UserManager тесты
    // ============================================

    #[tokio::test]
    async fn test_user_manager_get_users_empty() {
        let wrapper = create_wrapper();
        let params = RetrieveQueryParams::default();
        let result = wrapper.get_users(params).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_user_manager_get_user_not_found() {
        let wrapper = create_wrapper();
        let result = wrapper.get_user(999).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::Error::NotFound(msg) => assert!(msg.contains("User not found")),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_user_manager_get_user_count() {
        let wrapper = create_wrapper();
        let result = wrapper.get_user_count().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_user_manager_get_all_admins_empty() {
        let wrapper = create_wrapper();
        let result = wrapper.get_all_admins().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // ============================================
    // ProjectStore тесты
    // ============================================

    #[tokio::test]
    async fn test_project_store_get_projects_empty() {
        let wrapper = create_wrapper();
        let result = wrapper.get_projects(None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_project_store_get_projects_with_user_id() {
        let wrapper = create_wrapper();
        let result = wrapper.get_projects(Some(1)).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_project_store_get_project_not_found() {
        let wrapper = create_wrapper();
        let result = wrapper.get_project(1).await;
        assert!(result.is_err());
    }

    // ============================================
    // TemplateManager тесты
    // ============================================

    #[tokio::test]
    async fn test_template_manager_get_templates_empty() {
        let wrapper = create_wrapper();
        let result = wrapper.get_templates(1).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // ============================================
    // TaskManager тесты
    // ============================================

    #[tokio::test]
    async fn test_task_manager_get_tasks_empty() {
        let wrapper = create_wrapper();
        let result = wrapper.get_tasks(1, None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_task_manager_get_running_tasks_count() {
        let wrapper = create_wrapper();
        let result = wrapper.get_running_tasks_count().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_task_manager_get_waiting_tasks_count() {
        let wrapper = create_wrapper();
        let result = wrapper.get_waiting_tasks_count().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_task_manager_get_global_tasks() {
        let wrapper = create_wrapper();
        let result = wrapper.get_global_tasks(None, None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // ============================================
    // ScheduleManager тесты
    // ============================================

    #[tokio::test]
    async fn test_schedule_manager_get_all_schedules_empty() {
        let wrapper = create_wrapper();
        let result = wrapper.get_all_schedules().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // ============================================
    // RunnerManager тесты
    // ============================================

    #[tokio::test]
    async fn test_runner_manager_get_runners_count() {
        let wrapper = create_wrapper();
        let result = wrapper.get_runners_count().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_runner_manager_get_active_runners_count() {
        let wrapper = create_wrapper();
        let result = wrapper.get_active_runners_count().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_runner_manager_get_runners_empty() {
        let wrapper = create_wrapper();
        let result = wrapper.get_runners(None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // ============================================
    // EventManager тесты
    // ============================================

    #[tokio::test]
    async fn test_event_manager_get_events_empty() {
        let wrapper = create_wrapper();
        let result = wrapper.get_events(None, 10).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_event_manager_get_events_with_project() {
        let wrapper = create_wrapper();
        let result = wrapper.get_events(Some(1), 5).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // ============================================
    // AuditLogManager тесты
    // ============================================

    #[tokio::test]
    async fn test_audit_log_manager_search_returns_empty_result() {
        let wrapper = create_wrapper();
        let filter = AuditLogFilter::default();
        let result = wrapper.search_audit_logs(&filter).await;
        assert!(result.is_ok());
        let logs_result = result.unwrap();
        assert!(logs_result.records.is_empty());
        assert_eq!(logs_result.total, 0);
    }

    #[tokio::test]
    async fn test_audit_log_manager_delete_before() {
        let wrapper = create_wrapper();
        let before = Utc::now();
        let result = wrapper.delete_audit_logs_before(before).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_audit_log_manager_clear() {
        let wrapper = create_wrapper();
        let result = wrapper.clear_audit_log().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    // ============================================
    // PlaybookRunManager тесты
    // ============================================

    #[tokio::test]
    async fn test_playbook_run_manager_get_run_by_task_returns_none() {
        let wrapper = create_wrapper();
        let result = wrapper.get_playbook_run_by_task_id(1).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_playbook_run_manager_stats() {
        let wrapper = create_wrapper();
        let result = wrapper.get_playbook_run_stats(1).await;
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.total_runs, 0);
        assert_eq!(stats.success_runs, 0);
        assert_eq!(stats.failed_runs, 0);
    }

    // ============================================
    // Ping тест
    // ============================================

    #[tokio::test]
    async fn test_ping() {
        let wrapper = create_wrapper();
        let result = wrapper.ping().await;
        assert!(result.is_ok());
    }

    // ============================================
    // RetrieveQueryParams тесты
    // ============================================

    #[test]
    fn test_retrieve_query_params_default() {
        let params = RetrieveQueryParams::default();
        assert_eq!(params.offset, 0);
        assert!(params.count.is_none());
        assert!(params.sort_by.is_none());
        assert!(!params.sort_inverted);
        assert!(params.filter.is_none());
    }

    #[test]
    fn test_retrieve_query_params_custom() {
        let params = RetrieveQueryParams {
            offset: 10,
            count: Some(20),
            sort_by: Some("name".to_string()),
            sort_inverted: true,
            filter: Some("test".to_string()),
        };
        assert_eq!(params.offset, 10);
        assert_eq!(params.count, Some(20));
        assert_eq!(params.sort_by, Some("name".to_string()));
        assert!(params.sort_inverted);
        assert_eq!(params.filter, Some("test".to_string()));
    }

    // ============================================
    // Store trait impl тест (маркерный)
    // ============================================

    #[test]
    fn test_store_wrapper_implements_store_trait() {
        // Компиляционная проверка что StoreWrapper реализует Store
        fn assert_store<S: Store>() {}
        assert_store::<StoreWrapper>();
    }

    // ============================================
    // Send + Sync тесты
    // ============================================

    #[test]
    fn test_store_wrapper_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<StoreWrapper>();
    }

    #[test]
    fn test_store_wrapper_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<StoreWrapper>();
    }

    // ============================================
    // Arc внутренности тест
    // ============================================

    #[test]
    fn test_store_wrapper_arc_clone() {
        let wrapper = create_wrapper();
        let arc = wrapper.as_arc();
        // Убеждаемся что Arc клонируется корректно
        let arc2 = arc.clone();
        assert!(Arc::ptr_eq(&arc, &arc2));
    }

    // ============================================
    // AuditAction и AuditObjectType тесты
    // ============================================

    #[test]
    fn test_audit_action_variants() {
        let actions = vec![
            AuditAction::Login,
            AuditAction::Logout,
            AuditAction::UserCreated,
            AuditAction::ProjectCreated,
            AuditAction::TaskCreated,
            AuditAction::TemplateCreated,
            AuditAction::InventoryCreated,
            AuditAction::Other,
        ];
        for action in actions {
            let s = format!("{:?}", action);
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn test_audit_object_type_variants() {
        let types = vec![
            AuditObjectType::User,
            AuditObjectType::Project,
            AuditObjectType::Task,
            AuditObjectType::Template,
            AuditObjectType::Inventory,
            AuditObjectType::Repository,
            AuditObjectType::Environment,
            AuditObjectType::AccessKey,
            AuditObjectType::Integration,
            AuditObjectType::Schedule,
            AuditObjectType::Runner,
            AuditObjectType::View,
            AuditObjectType::Secret,
            AuditObjectType::System,
            AuditObjectType::Other,
        ];
        assert_eq!(types.len(), 15);
    }

    #[test]
    fn test_audit_level_variants() {
        let levels = vec![
            AuditLevel::Info,
            AuditLevel::Warning,
            AuditLevel::Error,
            AuditLevel::Critical,
        ];
        assert_eq!(levels.len(), 4);
    }

    // ============================================
    // PlaybookRunStatus тесты
    // ============================================

    #[test]
    fn test_playbook_run_status_variants() {
        let statuses = vec![
            PlaybookRunStatus::Waiting,
            PlaybookRunStatus::Running,
            PlaybookRunStatus::Success,
            PlaybookRunStatus::Failed,
            PlaybookRunStatus::Cancelled,
        ];
        assert_eq!(statuses.len(), 5);
    }

    // ============================================
    // Тесты для StoreWrapper как Arc<dyn Store> конверсии
    // ============================================

    #[tokio::test]
    async fn test_store_wrapper_delegates_to_inner_connect() {
        let wrapper = create_wrapper();
        // Метод connect делегируется на inner через as_ref()
        let result = wrapper.connect().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_store_wrapper_delegates_to_inner_dialect() {
        let wrapper = create_wrapper();
        let dialect = wrapper.get_dialect();
        assert_eq!(dialect, "mock");
    }

    // ============================================
    // Тесты для OrganizationManager
    // ============================================

    #[tokio::test]
    async fn test_organization_manager_get_organizations_empty() {
        let wrapper = create_wrapper();
        let result = wrapper.get_organizations().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_organization_manager_quota_check() {
        let wrapper = create_wrapper();
        let result = wrapper.check_organization_quota(1, "projects").await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    // ============================================
    // Тесты для StructuredOutputManager
    // ============================================

    #[tokio::test]
    async fn test_structured_output_manager_empty_map() {
        let wrapper = create_wrapper();
        let result = wrapper.get_task_outputs_map(1, 1).await;
        assert!(result.is_ok());
        let map = result.unwrap();
        assert!(map.outputs.is_empty());
    }

    // ============================================
    // Тесты для CostEstimateManager
    // ============================================

    #[tokio::test]
    async fn test_cost_estimate_manager_returns_none() {
        let wrapper = create_wrapper();
        let result = wrapper.get_cost_estimate_for_task(1, 1).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_cost_estimate_manager_summaries_empty() {
        let wrapper = create_wrapper();
        let result = wrapper.get_cost_summaries(1).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
