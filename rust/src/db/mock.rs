//! Mock-реализация Store для тестов

use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::*;
use crate::services::task_logger::TaskStatus;
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::RwLock;

/// Mock-хранилище для тестов
pub struct MockStore {
    users: RwLock<HashMap<i32, User>>,
    projects: RwLock<HashMap<i32, Project>>,
    tasks: RwLock<HashMap<i32, Task>>,
    templates: RwLock<HashMap<i32, Template>>,
}

impl Default for MockStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MockStore {
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
            projects: RwLock::new(HashMap::new()),
            tasks: RwLock::new(HashMap::new()),
            templates: RwLock::new(HashMap::new()),
        }
    }
}

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
        Ok(self.users.read().unwrap().values().cloned().collect())
    }
    async fn get_user(&self, id: i32) -> Result<User> {
        self.users
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("User {} not found", id)))
    }
    async fn get_user_by_login_or_email(&self, login: &str, email: &str) -> Result<User> {
        self.users
            .read()
            .unwrap()
            .values()
            .find(|u| u.username == login || u.email == email)
            .cloned()
            .ok_or_else(|| Error::NotFound("User not found".to_string()))
    }
    async fn create_user(&self, user: User, _password: &str) -> Result<User> {
        self.users.write().unwrap().insert(user.id, user.clone());
        Ok(user)
    }
    async fn update_user(&self, user: User) -> Result<()> {
        self.users.write().unwrap().insert(user.id, user.clone());
        Ok(())
    }
    async fn delete_user(&self, id: i32) -> Result<()> {
        self.users.write().unwrap().remove(&id);
        Ok(())
    }
    async fn set_user_password(&self, _user_id: i32, _password: &str) -> Result<()> {
        Ok(())
    }
    async fn get_all_admins(&self) -> Result<Vec<User>> {
        Ok(self
            .users
            .read()
            .unwrap()
            .values()
            .filter(|u| u.admin)
            .cloned()
            .collect())
    }
    async fn get_user_count(&self) -> Result<usize> {
        Ok(self.users.read().unwrap().len())
    }
    async fn get_project_users(&self, _project_id: i32, _params: RetrieveQueryParams) -> Result<Vec<ProjectUser>> {
        Ok(vec![])
    }
    
    async fn get_user_totp(&self, user_id: i32) -> Result<Option<UserTotp>> {
        // Mock implementation - возвращаем None
        Ok(self.users.read().unwrap().get(&user_id).and_then(|u| u.totp.clone()))
    }
    
    async fn set_user_totp(&self, user_id: i32, totp: &UserTotp) -> Result<()> {
        // Mock implementation - обновляем пользователя
        if let Some(user) = self.users.write().unwrap().get_mut(&user_id) {
            user.totp = Some(totp.clone());
        }
        Ok(())
    }
    
    async fn delete_user_totp(&self, user_id: i32) -> Result<()> {
        // Mock implementation - удаляем TOTP
        if let Some(user) = self.users.write().unwrap().get_mut(&user_id) {
            user.totp = None;
        }
        Ok(())
    }
}

#[async_trait]
impl HookManager for MockStore {
    async fn get_hooks_by_template(&self, _template_id: i32) -> Result<Vec<Hook>> {
        // Mock - возвращаем пустой список
        Ok(Vec::new())
    }
}

#[async_trait]
impl ProjectStore for MockStore {
    async fn get_projects(&self, _user_id: Option<i32>) -> Result<Vec<Project>> {
        Ok(self.projects.read().unwrap().values().cloned().collect())
    }
    async fn get_project(&self, project_id: i32) -> Result<Project> {
        self.projects
            .read()
            .unwrap()
            .get(&project_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Project {} not found", project_id)))
    }
    async fn create_project(&self, mut project: Project) -> Result<Project> {
        if project.id == 0 {
            project.id = (self.projects.read().unwrap().len() as i32) + 1;
        }
        self.projects.write().unwrap().insert(project.id, project.clone());
        Ok(project)
    }
    async fn update_project(&self, project: Project) -> Result<()> {
        self.projects.write().unwrap().insert(project.id, project.clone());
        Ok(())
    }
    async fn delete_project(&self, project_id: i32) -> Result<()> {
        self.projects.write().unwrap().remove(&project_id);
        Ok(())
    }
    async fn create_project_user(&self, _project_user: crate::models::ProjectUser) -> Result<()> {
        Ok(())
    }
    async fn delete_project_user(&self, _project_id: i32, _user_id: i32) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl TemplateManager for MockStore {
    async fn get_templates(&self, _project_id: i32) -> Result<Vec<Template>> {
        Ok(self.templates.read().unwrap().values().cloned().collect())
    }
    async fn get_template(&self, project_id: i32, template_id: i32) -> Result<Template> {
        self.templates
            .read()
            .unwrap()
            .get(&template_id)
            .filter(|t| t.project_id == project_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Template {} not found", template_id)))
    }
    async fn create_template(&self, mut template: Template) -> Result<Template> {
        if template.id == 0 {
            template.id = (self.templates.read().unwrap().len() as i32) + 1;
        }
        self.templates.write().unwrap().insert(template.id, template.clone());
        Ok(template)
    }
    async fn update_template(&self, template: Template) -> Result<()> {
        self.templates.write().unwrap().insert(template.id, template.clone());
        Ok(())
    }
    async fn delete_template(&self, _project_id: i32, template_id: i32) -> Result<()> {
        self.templates.write().unwrap().remove(&template_id);
        Ok(())
    }
}

#[async_trait]
impl InventoryManager for MockStore {
    async fn get_inventories(&self, _project_id: i32) -> Result<Vec<Inventory>> {
        Ok(vec![])
    }
    async fn get_inventory(&self, _project_id: i32, inventory_id: i32) -> Result<Inventory> {
        Err(Error::NotFound(format!("Inventory {} not found", inventory_id)))
    }
    async fn create_inventory(&self, inventory: Inventory) -> Result<Inventory> {
        Ok(inventory)
    }
    async fn update_inventory(&self, inventory: Inventory) -> Result<()> {
        let _ = inventory;
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
    async fn get_repository(&self, _project_id: i32, repository_id: i32) -> Result<Repository> {
        Err(Error::NotFound(format!("Repository {} not found", repository_id)))
    }
    async fn create_repository(&self, repository: Repository) -> Result<Repository> {
        Ok(repository)
    }
    async fn update_repository(&self, repository: Repository) -> Result<()> {
        let _ = repository;
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
    async fn get_environment(&self, _project_id: i32, environment_id: i32) -> Result<Environment> {
        Err(Error::NotFound(format!("Environment {} not found", environment_id)))
    }
    async fn create_environment(&self, environment: Environment) -> Result<Environment> {
        Ok(environment)
    }
    async fn update_environment(&self, environment: Environment) -> Result<()> {
        let _ = environment;
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
    async fn get_access_key(&self, _project_id: i32, key_id: i32) -> Result<AccessKey> {
        Err(Error::NotFound(format!("AccessKey {} not found", key_id)))
    }
    async fn create_access_key(&self, key: AccessKey) -> Result<AccessKey> {
        Ok(key)
    }
    async fn update_access_key(&self, key: AccessKey) -> Result<()> {
        let _ = key;
        Ok(())
    }
    async fn delete_access_key(&self, _project_id: i32, _key_id: i32) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl TaskManager for MockStore {
    async fn get_tasks(&self, _project_id: i32, template_id: Option<i32>) -> Result<Vec<TaskWithTpl>> {
        let tasks: Vec<Task> = self.tasks.read().unwrap().values().cloned().collect();
        Ok(tasks
            .into_iter()
            .filter(|t| template_id.map_or(true, |tid| t.template_id == tid))
            .map(|t| TaskWithTpl {
                task: t,
                tpl_playbook: None,
                tpl_type: None,
                tpl_app: None,
                user_name: None,
                build_task: None,
            })
            .collect())
    }
    async fn get_task(&self, _project_id: i32, task_id: i32) -> Result<Task> {
        self.tasks
            .read()
            .unwrap()
            .get(&task_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Task {} not found", task_id)))
    }
    async fn create_task(&self, mut task: Task) -> Result<Task> {
        if task.id == 0 {
            task.id = (self.tasks.read().unwrap().len() as i32) + 1;
        }
        self.tasks.write().unwrap().insert(task.id, task.clone());
        Ok(task)
    }
    async fn update_task(&self, task: Task) -> Result<()> {
        self.tasks.write().unwrap().insert(task.id, task.clone());
        Ok(())
    }
    async fn delete_task(&self, _project_id: i32, task_id: i32) -> Result<()> {
        self.tasks.write().unwrap().remove(&task_id);
        Ok(())
    }
    async fn get_task_outputs(&self, _task_id: i32) -> Result<Vec<TaskOutput>> {
        Ok(vec![])
    }
    async fn create_task_output(&self, output: TaskOutput) -> Result<TaskOutput> {
        Ok(output)
    }
    async fn update_task_status(&self, _project_id: i32, _task_id: i32, _status: TaskStatus) -> Result<()> {
        Ok(())
    }
    async fn get_global_tasks(&self, _status_filter: Option<Vec<String>>, _limit: Option<i32>) -> Result<Vec<TaskWithTpl>> {
        let tasks: Vec<Task> = self.tasks.read().unwrap().values().cloned().collect();
        Ok(tasks.into_iter().map(|t| TaskWithTpl {
            task: t,
            tpl_playbook: None,
            tpl_type: None,
            tpl_app: None,
            user_name: None,
            build_task: None,
        }).collect())
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
    async fn get_schedule(&self, _project_id: i32, schedule_id: i32) -> Result<Schedule> {
        Err(Error::NotFound(format!("Schedule {} not found", schedule_id)))
    }
    async fn create_schedule(&self, schedule: Schedule) -> Result<Schedule> {
        Ok(schedule)
    }
    async fn update_schedule(&self, schedule: Schedule) -> Result<()> {
        let _ = schedule;
        Ok(())
    }
    async fn delete_schedule(&self, _project_id: i32, _schedule_id: i32) -> Result<()> {
        Ok(())
    }
    async fn set_schedule_active(&self, _project_id: i32, _schedule_id: i32, _active: bool) -> Result<()> {
        Ok(())
    }
    async fn set_schedule_commit_hash(&self, _project_id: i32, _schedule_id: i32, _hash: &str) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl SessionManager for MockStore {
    async fn get_session(&self, _user_id: i32, session_id: i32) -> Result<Session> {
        Err(Error::NotFound(format!("Session {} not found", session_id)))
    }
    async fn create_session(&self, session: Session) -> Result<Session> {
        Ok(session)
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
    async fn create_api_token(&self, token: APIToken) -> Result<APIToken> {
        Ok(token)
    }
    async fn get_api_token(&self, token_id: i32) -> Result<APIToken> {
        Err(Error::NotFound(format!("Token {} not found", token_id)))
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
    async fn create_event(&self, event: Event) -> Result<Event> {
        Ok(event)
    }
}

#[async_trait]
impl RunnerManager for MockStore {
    async fn get_runners(&self, _project_id: Option<i32>) -> Result<Vec<Runner>> {
        Ok(vec![])
    }
    async fn get_runner(&self, runner_id: i32) -> Result<Runner> {
        Err(Error::NotFound(format!("Runner {} not found", runner_id)))
    }
    async fn create_runner(&self, runner: Runner) -> Result<Runner> {
        Ok(runner)
    }
    async fn update_runner(&self, runner: Runner) -> Result<()> {
        let _ = runner;
        Ok(())
    }
    async fn delete_runner(&self, _runner_id: i32) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl ViewManager for MockStore {
    async fn get_views(&self, _project_id: i32) -> Result<Vec<View>> {
        Ok(vec![])
    }
    async fn get_view(&self, _project_id: i32, view_id: i32) -> Result<View> {
        Err(Error::NotFound(format!("View {} not found", view_id)))
    }
    async fn create_view(&self, view: View) -> Result<View> {
        Ok(view)
    }
    async fn update_view(&self, view: View) -> Result<()> {
        let _ = view;
        Ok(())
    }
    async fn delete_view(&self, _project_id: i32, _view_id: i32) -> Result<()> {
        Ok(())
    }
    
    async fn set_view_positions(&self, _project_id: i32, _positions: Vec<(i32, i32)>) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl IntegrationManager for MockStore {
    async fn get_integrations(&self, _project_id: i32) -> Result<Vec<Integration>> {
        Ok(vec![])
    }
    async fn get_integration(&self, _project_id: i32, integration_id: i32) -> Result<Integration> {
        Err(Error::NotFound(format!("Integration {} not found", integration_id)))
    }
    async fn create_integration(&self, integration: Integration) -> Result<Integration> {
        Ok(integration)
    }
    async fn update_integration(&self, integration: Integration) -> Result<()> {
        let _ = integration;
        Ok(())
    }
    async fn delete_integration(&self, _project_id: i32, _integration_id: i32) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl ProjectInviteManager for MockStore {
    async fn get_project_invites(&self, _project_id: i32, _params: RetrieveQueryParams) -> Result<Vec<ProjectInviteWithUser>> {
        Ok(vec![])
    }
    async fn create_project_invite(&self, invite: ProjectInvite) -> Result<ProjectInvite> {
        Ok(invite)
    }
    async fn get_project_invite(&self, _project_id: i32, invite_id: i32) -> Result<ProjectInvite> {
        Err(Error::NotFound(format!("ProjectInvite {} not found", invite_id)))
    }
    async fn get_project_invite_by_token(&self, _token: &str) -> Result<ProjectInvite> {
        Err(Error::NotFound("ProjectInvite not found".to_string()))
    }
    async fn update_project_invite(&self, invite: ProjectInvite) -> Result<()> {
        let _ = invite;
        Ok(())
    }
    async fn delete_project_invite(&self, _project_id: i32, _invite_id: i32) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl TerraformInventoryManager for MockStore {
    async fn create_terraform_inventory_alias(&self, alias: TerraformInventoryAlias) -> Result<TerraformInventoryAlias> {
        Ok(alias)
    }
    async fn update_terraform_inventory_alias(&self, alias: TerraformInventoryAlias) -> Result<()> {
        let _ = alias;
        Ok(())
    }
    async fn get_terraform_inventory_alias_by_alias(&self, alias: &str) -> Result<TerraformInventoryAlias> {
        Err(Error::NotFound(format!("Alias {} not found", alias)))
    }
    async fn get_terraform_inventory_alias(
        &self,
        _project_id: i32,
        _inventory_id: i32,
        alias_id: &str,
    ) -> Result<TerraformInventoryAlias> {
        Err(Error::NotFound(format!("Alias {} not found", alias_id)))
    }
    async fn get_terraform_inventory_aliases(&self, _project_id: i32, _inventory_id: i32) -> Result<Vec<TerraformInventoryAlias>> {
        Ok(vec![])
    }
    async fn delete_terraform_inventory_alias(&self, _project_id: i32, _inventory_id: i32, _alias_id: &str) -> Result<()> {
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
    async fn create_terraform_inventory_state(&self, state: TerraformInventoryState) -> Result<TerraformInventoryState> {
        Ok(state)
    }
    async fn delete_terraform_inventory_state(&self, _project_id: i32, _inventory_id: i32, _state_id: i32) -> Result<()> {
        Ok(())
    }
    async fn get_terraform_inventory_state(
        &self,
        _project_id: i32,
        _inventory_id: i32,
        state_id: i32,
    ) -> Result<TerraformInventoryState> {
        Err(Error::NotFound(format!("State {} not found", state_id)))
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
    async fn get_secret_storage(&self, _project_id: i32, storage_id: i32) -> Result<SecretStorage> {
        Err(Error::NotFound(format!("SecretStorage {} not found", storage_id)))
    }
    async fn create_secret_storage(&self, storage: SecretStorage) -> Result<SecretStorage> {
        Ok(storage)
    }
    async fn update_secret_storage(&self, storage: SecretStorage) -> Result<()> {
        let _ = storage;
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
        Err(Error::NotFound("AuditLog not found".to_string()))
    }

    async fn get_audit_log(&self, _id: i64) -> Result<AuditLog> {
        Err(Error::NotFound("AuditLog not found".to_string()))
    }

    async fn search_audit_logs(&self, _filter: &AuditLogFilter) -> Result<AuditLogResult> {
        Ok(AuditLogResult {
            records: vec![],
            total: 0,
            limit: 0,
            offset: 0,
        })
    }

    async fn get_audit_logs_by_project(&self, _project_id: i64, _limit: i64, _offset: i64) -> Result<Vec<AuditLog>> {
        Ok(vec![])
    }

    async fn get_audit_logs_by_user(&self, _user_id: i64, _limit: i64, _offset: i64) -> Result<Vec<AuditLog>> {
        Ok(vec![])
    }

    async fn get_audit_logs_by_action(&self, _action: &AuditAction, _limit: i64, _offset: i64) -> Result<Vec<AuditLog>> {
        Ok(vec![])
    }

    async fn delete_audit_logs_before(&self, _before: chrono::DateTime<chrono::Utc>) -> Result<u64> {
        Ok(0)
    }
    async fn clear_audit_log(&self) -> Result<u64> {
        Ok(0)
    }
}

#[async_trait]
impl IntegrationMatcherManager for MockStore {
    async fn get_integration_matchers(&self, _project_id: i32, _integration_id: i32) -> Result<Vec<IntegrationMatcher>> {
        Ok(vec![])
    }
    async fn create_integration_matcher(&self, matcher: IntegrationMatcher) -> Result<IntegrationMatcher> {
        Ok(matcher)
    }
    async fn update_integration_matcher(&self, _matcher: IntegrationMatcher) -> Result<()> {
        Ok(())
    }
    async fn delete_integration_matcher(&self, _project_id: i32, _integration_id: i32, _matcher_id: i32) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl IntegrationExtractValueManager for MockStore {
    async fn get_integration_extract_values(&self, _project_id: i32, _integration_id: i32) -> Result<Vec<IntegrationExtractValue>> {
        Ok(vec![])
    }
    async fn create_integration_extract_value(&self, value: IntegrationExtractValue) -> Result<IntegrationExtractValue> {
        Ok(value)
    }
    async fn update_integration_extract_value(&self, _value: IntegrationExtractValue) -> Result<()> {
        Ok(())
    }
    async fn delete_integration_extract_value(&self, _project_id: i32, _integration_id: i32, _value_id: i32) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl ProjectRoleManager for MockStore {
    async fn get_project_roles(&self, _project_id: i32) -> Result<Vec<crate::models::Role>> {
        Ok(vec![])
    }
    async fn create_project_role(&self, role: crate::models::Role) -> Result<crate::models::Role> {
        Ok(role)
    }
    async fn update_project_role(&self, _role: crate::models::Role) -> Result<()> {
        Ok(())
    }
    async fn delete_project_role(&self, _project_id: i32, _role_id: i32) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl Store for MockStore {}

#[async_trait]
impl crate::db::store::DriftManager for MockStore {
    async fn get_drift_configs(&self, _project_id: i32) -> Result<Vec<crate::models::drift::DriftConfig>> { Ok(Vec::new()) }
    async fn get_drift_config(&self, _id: i32, _project_id: i32) -> Result<crate::models::drift::DriftConfig> { Err(Error::NotFound("DriftConfig not found".to_string())) }
    async fn create_drift_config(&self, _project_id: i32, _payload: crate::models::drift::DriftConfigCreate) -> Result<crate::models::drift::DriftConfig> { Err(Error::Other("not implemented".to_string())) }
    async fn update_drift_config_enabled(&self, _id: i32, _project_id: i32, _enabled: bool) -> Result<()> { Ok(()) }
    async fn delete_drift_config(&self, _id: i32, _project_id: i32) -> Result<()> { Ok(()) }
    async fn get_drift_results(&self, _drift_config_id: i32, _limit: i64) -> Result<Vec<crate::models::drift::DriftResult>> { Ok(Vec::new()) }
    async fn create_drift_result(&self, _project_id: i32, _drift_config_id: i32, _template_id: i32, _status: &str, _summary: Option<String>, _task_id: Option<i32>) -> Result<crate::models::drift::DriftResult> { Err(Error::Other("not implemented".to_string())) }
    async fn get_latest_drift_results(&self, _project_id: i32) -> Result<Vec<crate::models::drift::DriftResult>> { Ok(Vec::new()) }
}

#[async_trait]
impl crate::db::store::LdapGroupMappingManager for MockStore {
    async fn get_ldap_group_mappings(&self) -> Result<Vec<crate::models::ldap_group::LdapGroupMapping>> { Ok(Vec::new()) }
    async fn create_ldap_group_mapping(&self, _payload: crate::models::ldap_group::LdapGroupMappingCreate) -> Result<crate::models::ldap_group::LdapGroupMapping> { Err(Error::Other("not implemented".to_string())) }
    async fn delete_ldap_group_mapping(&self, _id: i32) -> Result<()> { Ok(()) }
    async fn get_mappings_for_groups(&self, _group_dns: &[String]) -> Result<Vec<crate::models::ldap_group::LdapGroupMapping>> { Ok(Vec::new()) }
}

#[async_trait]
impl WebhookManager for MockStore {
    async fn get_webhook(&self, _webhook_id: i64) -> Result<crate::models::webhook::Webhook> {
        Err(Error::NotFound("Webhook not found".to_string()))
    }

    async fn get_webhooks_by_project(&self, _project_id: i64) -> Result<Vec<crate::models::webhook::Webhook>> {
        Ok(Vec::new())
    }

    async fn create_webhook(&self, _webhook: crate::models::webhook::Webhook) -> Result<crate::models::webhook::Webhook> {
        Err(Error::Database(sqlx::Error::Protocol("Not implemented in mock".to_string())))
    }

    async fn update_webhook(&self, _webhook_id: i64, _webhook: crate::models::webhook::UpdateWebhook) -> Result<crate::models::webhook::Webhook> {
        Err(Error::Database(sqlx::Error::Protocol("Not implemented in mock".to_string())))
    }

    async fn delete_webhook(&self, _webhook_id: i64) -> Result<()> {
        Ok(())
    }

    async fn get_webhook_logs(&self, _webhook_id: i64) -> Result<Vec<crate::models::webhook::WebhookLog>> {
        Ok(Vec::new())
    }

    async fn create_webhook_log(&self, _log: crate::models::webhook::WebhookLog) -> Result<crate::models::webhook::WebhookLog> {
        Err(Error::Database(sqlx::Error::Protocol("Not implemented in mock".to_string())))
    }
}

#[async_trait]
impl PlaybookManager for MockStore {
    async fn get_playbooks(&self, _project_id: i32) -> Result<Vec<crate::models::Playbook>> {
        Ok(Vec::new())
    }

    async fn get_playbook(&self, _id: i32, _project_id: i32) -> Result<crate::models::Playbook> {
        Err(Error::NotFound("Playbook not found".to_string()))
    }

    async fn create_playbook(&self, _project_id: i32, _playbook: crate::models::PlaybookCreate) -> Result<crate::models::Playbook> {
        Err(Error::Database(sqlx::Error::Protocol("Not implemented in mock".to_string())))
    }

    async fn update_playbook(&self, _id: i32, _project_id: i32, _playbook: crate::models::PlaybookUpdate) -> Result<crate::models::Playbook> {
        Err(Error::Database(sqlx::Error::Protocol("Not implemented in mock".to_string())))
    }

    async fn delete_playbook(&self, _id: i32, _project_id: i32) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl PlaybookRunManager for MockStore {
    async fn get_playbook_runs(&self, _filter: PlaybookRunFilter) -> Result<Vec<PlaybookRun>> {
        Ok(Vec::new())
    }

    async fn get_playbook_run(&self, _id: i32, _project_id: i32) -> Result<PlaybookRun> {
        Err(Error::NotFound("PlaybookRun not found".to_string()))
    }

    async fn get_playbook_run_by_task_id(&self, _task_id: i32) -> Result<Option<PlaybookRun>> {
        Ok(None)
    }

    async fn create_playbook_run(&self, _run: PlaybookRunCreate) -> Result<PlaybookRun> {
        Err(Error::Database(sqlx::Error::Protocol("Not implemented in mock".to_string())))
    }

    async fn update_playbook_run(&self, _id: i32, _project_id: i32, _update: PlaybookRunUpdate) -> Result<PlaybookRun> {
        Err(Error::Database(sqlx::Error::Protocol("Not implemented in mock".to_string())))
    }

    async fn update_playbook_run_status(&self, _id: i32, _status: PlaybookRunStatus) -> Result<()> {
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
impl crate::db::store::WorkflowManager for MockStore {
    async fn get_workflows(&self, _project_id: i32) -> Result<Vec<crate::models::workflow::Workflow>> {
        Ok(Vec::new())
    }
    async fn get_workflow(&self, _id: i32, _project_id: i32) -> Result<crate::models::workflow::Workflow> {
        Err(Error::NotFound("Workflow not found".to_string()))
    }
    async fn create_workflow(&self, _project_id: i32, _payload: crate::models::workflow::WorkflowCreate) -> Result<crate::models::workflow::Workflow> {
        Err(Error::Other("not implemented".to_string()))
    }
    async fn update_workflow(&self, _id: i32, _project_id: i32, _payload: crate::models::workflow::WorkflowUpdate) -> Result<crate::models::workflow::Workflow> {
        Err(Error::Other("not implemented".to_string()))
    }
    async fn delete_workflow(&self, _id: i32, _project_id: i32) -> Result<()> {
        Ok(())
    }
    async fn get_workflow_nodes(&self, _workflow_id: i32) -> Result<Vec<crate::models::workflow::WorkflowNode>> {
        Ok(Vec::new())
    }
    async fn create_workflow_node(&self, _workflow_id: i32, _payload: crate::models::workflow::WorkflowNodeCreate) -> Result<crate::models::workflow::WorkflowNode> {
        Err(Error::Other("not implemented".to_string()))
    }
    async fn update_workflow_node(&self, _id: i32, _workflow_id: i32, _payload: crate::models::workflow::WorkflowNodeUpdate) -> Result<crate::models::workflow::WorkflowNode> {
        Err(Error::Other("not implemented".to_string()))
    }
    async fn delete_workflow_node(&self, _id: i32, _workflow_id: i32) -> Result<()> {
        Ok(())
    }
    async fn get_workflow_edges(&self, _workflow_id: i32) -> Result<Vec<crate::models::workflow::WorkflowEdge>> {
        Ok(Vec::new())
    }
    async fn create_workflow_edge(&self, _workflow_id: i32, _payload: crate::models::workflow::WorkflowEdgeCreate) -> Result<crate::models::workflow::WorkflowEdge> {
        Err(Error::Other("not implemented".to_string()))
    }
    async fn delete_workflow_edge(&self, _id: i32, _workflow_id: i32) -> Result<()> {
        Ok(())
    }
    async fn get_workflow_runs(&self, _workflow_id: i32, _project_id: i32) -> Result<Vec<crate::models::workflow::WorkflowRun>> {
        Ok(Vec::new())
    }
    async fn create_workflow_run(&self, _workflow_id: i32, _project_id: i32) -> Result<crate::models::workflow::WorkflowRun> {
        Err(Error::Other("not implemented".to_string()))
    }
    async fn update_workflow_run_status(&self, _id: i32, _status: &str, _message: Option<String>) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl crate::db::store::NotificationPolicyManager for MockStore {
    async fn get_notification_policies(&self, _project_id: i32) -> Result<Vec<crate::models::notification::NotificationPolicy>> {
        Ok(Vec::new())
    }
    async fn get_notification_policy(&self, _id: i32, _project_id: i32) -> Result<crate::models::notification::NotificationPolicy> {
        Err(Error::NotFound("NotificationPolicy not found".to_string()))
    }
    async fn create_notification_policy(&self, _project_id: i32, _payload: crate::models::notification::NotificationPolicyCreate) -> Result<crate::models::notification::NotificationPolicy> {
        Err(Error::Other("not implemented".to_string()))
    }
    async fn update_notification_policy(&self, _id: i32, _project_id: i32, _payload: crate::models::notification::NotificationPolicyUpdate) -> Result<crate::models::notification::NotificationPolicy> {
        Err(Error::Other("not implemented".to_string()))
    }
    async fn delete_notification_policy(&self, _id: i32, _project_id: i32) -> Result<()> {
        Ok(())
    }
    async fn get_matching_policies(&self, _project_id: i32, _trigger: &str, _template_id: Option<i32>) -> Result<Vec<crate::models::notification::NotificationPolicy>> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl crate::db::store::CredentialTypeManager for MockStore {
    async fn get_credential_types(&self) -> Result<Vec<crate::models::credential_type::CredentialType>> {
        Ok(Vec::new())
    }
    async fn get_credential_type(&self, _id: i32) -> Result<crate::models::credential_type::CredentialType> {
        Err(Error::NotFound("CredentialType not found".to_string()))
    }
    async fn create_credential_type(&self, _payload: crate::models::credential_type::CredentialTypeCreate) -> Result<crate::models::credential_type::CredentialType> {
        Err(Error::Other("not implemented".to_string()))
    }
    async fn update_credential_type(&self, _id: i32, _payload: crate::models::credential_type::CredentialTypeUpdate) -> Result<crate::models::credential_type::CredentialType> {
        Err(Error::Other("not implemented".to_string()))
    }
    async fn delete_credential_type(&self, _id: i32) -> Result<()> {
        Ok(())
    }
    async fn get_credential_instances(&self, _project_id: i32) -> Result<Vec<crate::models::credential_type::CredentialInstance>> {
        Ok(Vec::new())
    }
    async fn get_credential_instance(&self, _id: i32, _project_id: i32) -> Result<crate::models::credential_type::CredentialInstance> {
        Err(Error::NotFound("CredentialInstance not found".to_string()))
    }
    async fn create_credential_instance(&self, _project_id: i32, _payload: crate::models::credential_type::CredentialInstanceCreate) -> Result<crate::models::credential_type::CredentialInstance> {
        Err(Error::Other("not implemented".to_string()))
    }
    async fn delete_credential_instance(&self, _id: i32, _project_id: i32) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl crate::db::store::SnapshotManager for MockStore {
    async fn get_snapshots(&self, _project_id: i32, _template_id: Option<i32>, _limit: i64) -> Result<Vec<crate::models::snapshot::TaskSnapshot>> { Ok(Vec::new()) }
    async fn get_snapshot(&self, _id: i32, _project_id: i32) -> Result<crate::models::snapshot::TaskSnapshot> { Err(Error::NotFound("Snapshot not found".to_string())) }
    async fn create_snapshot(&self, _project_id: i32, _payload: crate::models::snapshot::TaskSnapshotCreate) -> Result<crate::models::snapshot::TaskSnapshot> { Err(Error::Other("not implemented".to_string())) }
    async fn delete_snapshot(&self, _id: i32, _project_id: i32) -> Result<()> { Ok(()) }
}

#[async_trait]
impl crate::db::store::CostEstimateManager for MockStore {
    async fn get_cost_estimates(&self, _project_id: i32, _limit: i64) -> Result<Vec<crate::models::cost_estimate::CostEstimate>> { Ok(Vec::new()) }
    async fn get_cost_estimate_for_task(&self, _project_id: i32, _task_id: i32) -> Result<Option<crate::models::cost_estimate::CostEstimate>> { Ok(None) }
    async fn create_cost_estimate(&self, _payload: crate::models::cost_estimate::CostEstimateCreate) -> Result<crate::models::cost_estimate::CostEstimate> { Err(crate::error::Error::Other("not implemented".to_string())) }
    async fn get_cost_summaries(&self, _project_id: i32) -> Result<Vec<crate::models::cost_estimate::CostSummary>> { Ok(Vec::new()) }
}
