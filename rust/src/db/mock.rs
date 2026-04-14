//! Mock-реализация Store для тестов

use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::organization::*;
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
    inventories: RwLock<HashMap<i32, Inventory>>,
    repositories: RwLock<HashMap<i32, Repository>>,
    environments: RwLock<HashMap<i32, Environment>>,
    playbook_runs: RwLock<HashMap<i32, PlaybookRun>>,
    organizations: RwLock<HashMap<i32, Organization>>,
    organization_users: RwLock<HashMap<i32, OrganizationUser>>,
    terraform_plans: RwLock<HashMap<(i32, i32), TerraformPlan>>,
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
            inventories: RwLock::new(HashMap::new()),
            repositories: RwLock::new(HashMap::new()),
            environments: RwLock::new(HashMap::new()),
            playbook_runs: RwLock::new(HashMap::new()),
            organizations: RwLock::new(HashMap::new()),
            organization_users: RwLock::new(HashMap::new()),
            terraform_plans: RwLock::new(HashMap::new()),
        }
    }

    /// Seed helper для тестов: добавить template
    pub fn seed_template(&self, template: Template) {
        self.templates
            .write()
            .unwrap()
            .insert(template.id, template);
    }

    /// Seed helper для тестов: добавить inventory
    pub fn seed_inventory(&self, inventory: Inventory) {
        self.inventories
            .write()
            .unwrap()
            .insert(inventory.id, inventory);
    }

    /// Seed helper для тестов: добавить repository
    pub fn seed_repository(&self, repository: Repository) {
        self.repositories
            .write()
            .unwrap()
            .insert(repository.id, repository);
    }

    /// Seed helper для тестов: добавить environment
    pub fn seed_environment(&self, environment: Environment) {
        self.environments
            .write()
            .unwrap()
            .insert(environment.id, environment);
    }

    /// Seed helper для тестов: добавить playbook_run
    pub fn seed_playbook_run(&self, run: PlaybookRun) {
        self.playbook_runs.write().unwrap().insert(run.id, run);
    }

    /// Тестовый хелпер: положить план Terraform по (project_id, task_id)
    pub fn seed_terraform_plan(&self, plan: TerraformPlan) {
        self.terraform_plans
            .write()
            .unwrap()
            .insert((plan.project_id, plan.task_id), plan);
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
    async fn get_project_users(
        &self,
        _project_id: i32,
        _params: RetrieveQueryParams,
    ) -> Result<Vec<ProjectUser>> {
        Ok(vec![])
    }

    async fn get_user_totp(&self, user_id: i32) -> Result<Option<UserTotp>> {
        // Mock implementation - возвращаем None
        Ok(self
            .users
            .read()
            .unwrap()
            .get(&user_id)
            .and_then(|u| u.totp.clone()))
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
        self.projects
            .write()
            .unwrap()
            .insert(project.id, project.clone());
        Ok(project)
    }
    async fn update_project(&self, project: Project) -> Result<()> {
        self.projects
            .write()
            .unwrap()
            .insert(project.id, project.clone());
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
        self.templates
            .write()
            .unwrap()
            .insert(template.id, template.clone());
        Ok(template)
    }
    async fn update_template(&self, template: Template) -> Result<()> {
        self.templates
            .write()
            .unwrap()
            .insert(template.id, template.clone());
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
        Ok(self.inventories.read().unwrap().values().cloned().collect())
    }
    async fn get_inventory(&self, _project_id: i32, inventory_id: i32) -> Result<Inventory> {
        self.inventories
            .read()
            .unwrap()
            .get(&inventory_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Inventory {} not found", inventory_id)))
    }
    async fn create_inventory(&self, mut inventory: Inventory) -> Result<Inventory> {
        if inventory.id == 0 {
            inventory.id = (self.inventories.read().unwrap().len() as i32) + 1;
        }
        self.inventories
            .write()
            .unwrap()
            .insert(inventory.id, inventory.clone());
        Ok(inventory)
    }
    async fn update_inventory(&self, inventory: Inventory) -> Result<()> {
        self.inventories
            .write()
            .unwrap()
            .insert(inventory.id, inventory.clone());
        Ok(())
    }
    async fn delete_inventory(&self, _project_id: i32, inventory_id: i32) -> Result<()> {
        self.inventories.write().unwrap().remove(&inventory_id);
        Ok(())
    }
}

#[async_trait]
impl RepositoryManager for MockStore {
    async fn get_repositories(&self, _project_id: i32) -> Result<Vec<Repository>> {
        Ok(self
            .repositories
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect())
    }
    async fn get_repository(&self, _project_id: i32, repository_id: i32) -> Result<Repository> {
        self.repositories
            .read()
            .unwrap()
            .get(&repository_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Repository {} not found", repository_id)))
    }
    async fn create_repository(&self, mut repository: Repository) -> Result<Repository> {
        if repository.id == 0 {
            repository.id = (self.repositories.read().unwrap().len() as i32) + 1;
        }
        self.repositories
            .write()
            .unwrap()
            .insert(repository.id, repository.clone());
        Ok(repository)
    }
    async fn update_repository(&self, repository: Repository) -> Result<()> {
        self.repositories
            .write()
            .unwrap()
            .insert(repository.id, repository.clone());
        Ok(())
    }
    async fn delete_repository(&self, _project_id: i32, repository_id: i32) -> Result<()> {
        self.repositories.write().unwrap().remove(&repository_id);
        Ok(())
    }
}

#[async_trait]
impl EnvironmentManager for MockStore {
    async fn get_environments(&self, _project_id: i32) -> Result<Vec<Environment>> {
        Ok(self
            .environments
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect())
    }
    async fn get_environment(&self, _project_id: i32, environment_id: i32) -> Result<Environment> {
        self.environments
            .read()
            .unwrap()
            .get(&environment_id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Environment {} not found", environment_id)))
    }
    async fn create_environment(&self, mut environment: Environment) -> Result<Environment> {
        if environment.id == 0 {
            environment.id = (self.environments.read().unwrap().len() as i32) + 1;
        }
        self.environments
            .write()
            .unwrap()
            .insert(environment.id, environment.clone());
        Ok(environment)
    }
    async fn update_environment(&self, environment: Environment) -> Result<()> {
        self.environments
            .write()
            .unwrap()
            .insert(environment.id, environment.clone());
        Ok(())
    }
    async fn delete_environment(&self, _project_id: i32, environment_id: i32) -> Result<()> {
        self.environments.write().unwrap().remove(&environment_id);
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
    async fn get_tasks(
        &self,
        _project_id: i32,
        template_id: Option<i32>,
    ) -> Result<Vec<TaskWithTpl>> {
        let tasks: Vec<Task> = self.tasks.read().unwrap().values().cloned().collect();
        Ok(tasks
            .into_iter()
            .filter(|t| template_id.is_none_or(|tid| t.template_id == tid))
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
    async fn update_task_status(
        &self,
        _project_id: i32,
        task_id: i32,
        status: TaskStatus,
    ) -> Result<()> {
        let mut tasks = self.tasks.write().unwrap();
        if let Some(t) = tasks.get_mut(&task_id) {
            t.status = status;
        }
        Ok(())
    }
    async fn get_global_tasks(
        &self,
        _status_filter: Option<Vec<String>>,
        _limit: Option<i32>,
    ) -> Result<Vec<TaskWithTpl>> {
        let tasks: Vec<Task> = self.tasks.read().unwrap().values().cloned().collect();
        Ok(tasks
            .into_iter()
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
    async fn get_schedule(&self, _project_id: i32, schedule_id: i32) -> Result<Schedule> {
        Err(Error::NotFound(format!(
            "Schedule {} not found",
            schedule_id
        )))
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

    async fn get_runners_count(&self) -> Result<usize> {
        Ok(0)
    }

    async fn get_active_runners_count(&self) -> Result<usize> {
        Ok(0)
    }

    async fn find_runner_by_token(&self, _token: &str) -> Result<Runner> {
        Err(Error::NotFound("Runner not found".to_string()))
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
    async fn get_integration(&self, _project_id: i32, integration_id: i32) -> Result<Integration> {
        Err(Error::NotFound(format!(
            "Integration {} not found",
            integration_id
        )))
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
    async fn get_project_invites(
        &self,
        _project_id: i32,
        _params: RetrieveQueryParams,
    ) -> Result<Vec<ProjectInviteWithUser>> {
        Ok(vec![])
    }
    async fn create_project_invite(&self, invite: ProjectInvite) -> Result<ProjectInvite> {
        Ok(invite)
    }
    async fn get_project_invite(&self, _project_id: i32, invite_id: i32) -> Result<ProjectInvite> {
        Err(Error::NotFound(format!(
            "ProjectInvite {} not found",
            invite_id
        )))
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
    async fn create_terraform_inventory_alias(
        &self,
        alias: TerraformInventoryAlias,
    ) -> Result<TerraformInventoryAlias> {
        Ok(alias)
    }
    async fn update_terraform_inventory_alias(&self, alias: TerraformInventoryAlias) -> Result<()> {
        let _ = alias;
        Ok(())
    }
    async fn get_terraform_inventory_alias_by_alias(
        &self,
        alias: &str,
    ) -> Result<TerraformInventoryAlias> {
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
        state: TerraformInventoryState,
    ) -> Result<TerraformInventoryState> {
        Ok(state)
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
        Err(Error::NotFound(format!(
            "SecretStorage {} not found",
            storage_id
        )))
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

    async fn delete_audit_logs_before(
        &self,
        _before: chrono::DateTime<chrono::Utc>,
    ) -> Result<u64> {
        Ok(0)
    }
    async fn clear_audit_log(&self) -> Result<u64> {
        Ok(0)
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
        matcher: IntegrationMatcher,
    ) -> Result<IntegrationMatcher> {
        Ok(matcher)
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
        value: IntegrationExtractValue,
    ) -> Result<IntegrationExtractValue> {
        Ok(value)
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
impl OrganizationManager for MockStore {
    async fn get_organizations(&self) -> Result<Vec<Organization>> {
        Ok(self
            .organizations
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect())
    }
    async fn get_organization(&self, id: i32) -> Result<Organization> {
        self.organizations
            .read()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Organization {id} not found")))
    }
    async fn get_organization_by_slug(&self, slug: &str) -> Result<Organization> {
        self.organizations
            .read()
            .unwrap()
            .values()
            .find(|o| o.slug == slug)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Organization {slug} not found")))
    }
    async fn create_organization(&self, payload: OrganizationCreate) -> Result<Organization> {
        use chrono::Utc;
        let slug = payload.slug.clone().unwrap_or_else(|| {
            payload
                .name
                .to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '-' })
                .collect()
        });
        let id = (self.organizations.read().unwrap().len() as i32) + 1;
        let org = Organization {
            id,
            name: payload.name.clone(),
            slug: slug.clone(),
            description: payload.description.clone(),
            settings: payload.settings.clone(),
            quota_max_projects: payload.quota_max_projects,
            quota_max_users: payload.quota_max_users,
            quota_max_tasks_per_month: payload.quota_max_tasks_per_month,
            active: true,
            created: Utc::now(),
            updated: None,
        };
        self.organizations.write().unwrap().insert(id, org.clone());
        Ok(org)
    }
    async fn update_organization(
        &self,
        id: i32,
        payload: OrganizationUpdate,
    ) -> Result<Organization> {
        let mut orgs = self.organizations.write().unwrap();
        if let Some(org) = orgs.get_mut(&id) {
            if let Some(name) = &payload.name {
                org.name = name.clone();
            }
            if let Some(desc) = &payload.description {
                org.description = Some(desc.clone());
            }
            if let Some(active) = payload.active {
                org.active = active;
            }
            org.updated = Some(Utc::now());
            Ok(org.clone())
        } else {
            Err(Error::NotFound(format!("Organization {id} not found")))
        }
    }
    async fn delete_organization(&self, id: i32) -> Result<()> {
        self.organizations.write().unwrap().remove(&id);
        Ok(())
    }
    async fn get_organization_users(&self, org_id: i32) -> Result<Vec<OrganizationUser>> {
        Ok(self
            .organization_users
            .read()
            .unwrap()
            .values()
            .filter(|&u| u.org_id == org_id)
            .cloned()
            .collect())
    }
    async fn add_user_to_organization(
        &self,
        payload: OrganizationUserCreate,
    ) -> Result<OrganizationUser> {
        use chrono::Utc;
        let id = (self.organization_users.read().unwrap().len() as i32) + 1;
        let ou = OrganizationUser {
            id,
            org_id: payload.org_id,
            user_id: payload.user_id,
            role: payload.role.clone(),
            created: Utc::now(),
        };
        self.organization_users
            .write()
            .unwrap()
            .insert(id, ou.clone());
        Ok(ou)
    }
    async fn remove_user_from_organization(&self, org_id: i32, user_id: i32) -> Result<()> {
        let mut users = self.organization_users.write().unwrap();
        let to_remove: Vec<i32> = users
            .iter()
            .filter(|(_, u)| u.org_id == org_id && u.user_id == user_id)
            .map(|(k, _)| *k)
            .collect();
        for k in to_remove {
            users.remove(&k);
        }
        Ok(())
    }
    async fn update_user_organization_role(
        &self,
        org_id: i32,
        user_id: i32,
        role: &str,
    ) -> Result<()> {
        let mut users = self.organization_users.write().unwrap();
        if let Some(ou) = users
            .values_mut()
            .find(|u| u.org_id == org_id && u.user_id == user_id)
        {
            ou.role = role.to_string();
        }
        Ok(())
    }
    async fn get_user_organizations(&self, user_id: i32) -> Result<Vec<Organization>> {
        let org_ids: Vec<i32> = self
            .organization_users
            .read()
            .unwrap()
            .values()
            .filter(|u| u.user_id == user_id)
            .map(|u| u.org_id)
            .collect();
        Ok(self
            .organizations
            .read()
            .unwrap()
            .values()
            .filter(|&o| org_ids.contains(&o.id))
            .cloned()
            .collect())
    }
    async fn check_organization_quota(&self, org_id: i32, quota_type: &str) -> Result<bool> {
        let opt_org = self.organizations.read().unwrap().get(&org_id).cloned();
        let org = match opt_org {
            Some(o) => o,
            None => return Err(Error::NotFound(format!("Organization {org_id} not found"))),
        };
        match quota_type {
            "projects" => {
                if let Some(max) = org.quota_max_projects {
                    let count = self.organizations.read().unwrap().len() as i64;
                    Ok(count < (max as i64))
                } else {
                    Ok(true)
                }
            }
            "users" => {
                if let Some(max) = org.quota_max_users {
                    let count = self
                        .organization_users
                        .read()
                        .unwrap()
                        .values()
                        .filter(|u| u.org_id == org_id)
                        .count() as i64;
                    Ok(count < (max as i64))
                } else {
                    Ok(true)
                }
            }
            _ => Ok(true),
        }
    }
}

#[async_trait]
impl Store for MockStore {}

#[async_trait]
impl crate::db::store::DriftManager for MockStore {
    async fn get_drift_configs(
        &self,
        _project_id: i32,
    ) -> Result<Vec<crate::models::drift::DriftConfig>> {
        Ok(Vec::new())
    }
    async fn get_drift_config(
        &self,
        _id: i32,
        _project_id: i32,
    ) -> Result<crate::models::drift::DriftConfig> {
        Err(Error::NotFound("DriftConfig not found".to_string()))
    }
    async fn create_drift_config(
        &self,
        _project_id: i32,
        _payload: crate::models::drift::DriftConfigCreate,
    ) -> Result<crate::models::drift::DriftConfig> {
        Err(Error::Other("not implemented".to_string()))
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
        Ok(Vec::new())
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
        Err(Error::Other("not implemented".to_string()))
    }
    async fn get_latest_drift_results(
        &self,
        _project_id: i32,
    ) -> Result<Vec<crate::models::drift::DriftResult>> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl crate::db::store::LdapGroupMappingManager for MockStore {
    async fn get_ldap_group_mappings(
        &self,
    ) -> Result<Vec<crate::models::ldap_group::LdapGroupMapping>> {
        Ok(Vec::new())
    }
    async fn create_ldap_group_mapping(
        &self,
        _payload: crate::models::ldap_group::LdapGroupMappingCreate,
    ) -> Result<crate::models::ldap_group::LdapGroupMapping> {
        Err(Error::Other("not implemented".to_string()))
    }
    async fn delete_ldap_group_mapping(&self, _id: i32) -> Result<()> {
        Ok(())
    }
    async fn get_mappings_for_groups(
        &self,
        _group_dns: &[String],
    ) -> Result<Vec<crate::models::ldap_group::LdapGroupMapping>> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl WebhookManager for MockStore {
    async fn get_webhook(&self, _webhook_id: i64) -> Result<crate::models::webhook::Webhook> {
        Err(Error::NotFound("Webhook not found".to_string()))
    }

    async fn get_webhooks_by_project(
        &self,
        _project_id: i64,
    ) -> Result<Vec<crate::models::webhook::Webhook>> {
        Ok(Vec::new())
    }

    async fn create_webhook(
        &self,
        _webhook: crate::models::webhook::Webhook,
    ) -> Result<crate::models::webhook::Webhook> {
        Err(Error::Database(sqlx::Error::Protocol(
            "Not implemented in mock".to_string(),
        )))
    }

    async fn update_webhook(
        &self,
        _webhook_id: i64,
        _webhook: crate::models::webhook::UpdateWebhook,
    ) -> Result<crate::models::webhook::Webhook> {
        Err(Error::Database(sqlx::Error::Protocol(
            "Not implemented in mock".to_string(),
        )))
    }

    async fn delete_webhook(&self, _webhook_id: i64) -> Result<()> {
        Ok(())
    }

    async fn get_webhook_logs(
        &self,
        _webhook_id: i64,
    ) -> Result<Vec<crate::models::webhook::WebhookLog>> {
        Ok(Vec::new())
    }

    async fn create_webhook_log(
        &self,
        _log: crate::models::webhook::WebhookLog,
    ) -> Result<crate::models::webhook::WebhookLog> {
        Err(Error::Database(sqlx::Error::Protocol(
            "Not implemented in mock".to_string(),
        )))
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

    async fn create_playbook(
        &self,
        _project_id: i32,
        _playbook: crate::models::PlaybookCreate,
    ) -> Result<crate::models::Playbook> {
        Err(Error::Database(sqlx::Error::Protocol(
            "Not implemented in mock".to_string(),
        )))
    }

    async fn update_playbook(
        &self,
        _id: i32,
        _project_id: i32,
        _playbook: crate::models::PlaybookUpdate,
    ) -> Result<crate::models::Playbook> {
        Err(Error::Database(sqlx::Error::Protocol(
            "Not implemented in mock".to_string(),
        )))
    }

    async fn delete_playbook(&self, _id: i32, _project_id: i32) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl PlaybookRunManager for MockStore {
    async fn get_playbook_runs(&self, filter: PlaybookRunFilter) -> Result<Vec<PlaybookRun>> {
        let runs: Vec<PlaybookRun> = self
            .playbook_runs
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect();
        let filtered: Vec<PlaybookRun> = runs
            .into_iter()
            .filter(|r| {
                if let Some(pid) = filter.project_id {
                    if r.project_id != pid {
                        return false;
                    }
                }
                if let Some(pb_id) = filter.playbook_id {
                    if r.playbook_id != pb_id {
                        return false;
                    }
                }
                if let Some(st) = &filter.status {
                    if r.status != *st {
                        return false;
                    }
                }
                true
            })
            .collect();
        let limit = filter.limit.unwrap_or(100);
        let offset = filter.offset.unwrap_or(0);
        Ok(filtered.into_iter().skip(offset).take(limit).collect())
    }

    async fn get_playbook_run(&self, id: i32, project_id: i32) -> Result<PlaybookRun> {
        self.playbook_runs
            .read()
            .unwrap()
            .get(&id)
            .filter(|r| r.project_id == project_id)
            .cloned()
            .ok_or_else(|| Error::NotFound("PlaybookRun not found".to_string()))
    }

    async fn get_playbook_run_by_task_id(&self, task_id: i32) -> Result<Option<PlaybookRun>> {
        Ok(self
            .playbook_runs
            .read()
            .unwrap()
            .values()
            .find(|&r| r.task_id == Some(task_id))
            .cloned())
    }

    async fn create_playbook_run(&self, run_create: PlaybookRunCreate) -> Result<PlaybookRun> {
        use chrono::Utc;
        let id = (self.playbook_runs.read().unwrap().len() as i32) + 1;
        let run = PlaybookRun {
            id,
            project_id: run_create.project_id,
            playbook_id: run_create.playbook_id,
            task_id: run_create.task_id,
            template_id: run_create.template_id,
            inventory_id: run_create.inventory_id,
            environment_id: run_create.environment_id,
            extra_vars: run_create.extra_vars,
            limit_hosts: run_create.limit_hosts,
            tags: run_create.tags,
            skip_tags: run_create.skip_tags,
            user_id: run_create.user_id,
            status: PlaybookRunStatus::Waiting,
            output: None,
            error_message: None,
            created: Utc::now(),
            updated: Utc::now(),
            start_time: None,
            end_time: None,
            duration_seconds: None,
            hosts_total: Some(0),
            hosts_changed: Some(0),
            hosts_unreachable: Some(0),
            hosts_failed: Some(0),
        };
        self.playbook_runs.write().unwrap().insert(id, run.clone());
        Ok(run)
    }

    async fn update_playbook_run(
        &self,
        id: i32,
        _project_id: i32,
        update: PlaybookRunUpdate,
    ) -> Result<PlaybookRun> {
        let mut runs = self.playbook_runs.write().unwrap();
        if let Some(run) = runs.get_mut(&id) {
            if let Some(status) = update.status {
                run.status = status;
            }
            if let Some(output) = update.output {
                run.output = Some(output);
            }
            if let Some(msg) = update.error_message {
                run.error_message = Some(msg);
            }
            if let Some(v) = update.hosts_total {
                run.hosts_total = Some(v);
            }
            if let Some(v) = update.hosts_changed {
                run.hosts_changed = Some(v);
            }
            if let Some(v) = update.hosts_unreachable {
                run.hosts_unreachable = Some(v);
            }
            if let Some(v) = update.hosts_failed {
                run.hosts_failed = Some(v);
            }
            if let Some(v) = update.start_time {
                run.start_time = Some(v);
            }
            if let Some(v) = update.end_time {
                run.end_time = Some(v);
            }
            if let Some(v) = update.duration_seconds {
                run.duration_seconds = Some(v);
            }
            Ok(run.clone())
        } else {
            Err(Error::NotFound("PlaybookRun not found".to_string()))
        }
    }

    async fn update_playbook_run_status(&self, id: i32, status: PlaybookRunStatus) -> Result<()> {
        let mut runs = self.playbook_runs.write().unwrap();
        if let Some(run) = runs.get_mut(&id) {
            run.status = status;
        }
        Ok(())
    }

    async fn delete_playbook_run(&self, id: i32, _project_id: i32) -> Result<()> {
        self.playbook_runs.write().unwrap().remove(&id);
        Ok(())
    }

    async fn get_playbook_run_stats(&self, playbook_id: i32) -> Result<PlaybookRunStats> {
        let runs: Vec<PlaybookRun> = self
            .playbook_runs
            .read()
            .unwrap()
            .values()
            .filter(|&r| r.playbook_id == playbook_id)
            .cloned()
            .collect();
        let total = runs.len() as i64;
        let success = runs
            .iter()
            .filter(|r| matches!(r.status, PlaybookRunStatus::Success))
            .count() as i64;
        let failed = runs
            .iter()
            .filter(|r| matches!(r.status, PlaybookRunStatus::Failed))
            .count() as i64;
        Ok(PlaybookRunStats {
            total_runs: total,
            success_runs: success,
            failed_runs: failed,
            avg_duration_seconds: None,
            last_run: runs.iter().max_by_key(|r| r.created).map(|r| r.created),
        })
    }
}

#[async_trait]
impl crate::db::store::WorkflowManager for MockStore {
    async fn get_workflows(
        &self,
        _project_id: i32,
    ) -> Result<Vec<crate::models::workflow::Workflow>> {
        Ok(Vec::new())
    }
    async fn get_workflow(
        &self,
        _id: i32,
        _project_id: i32,
    ) -> Result<crate::models::workflow::Workflow> {
        Err(Error::NotFound("Workflow not found".to_string()))
    }
    async fn create_workflow(
        &self,
        project_id: i32,
        payload: crate::models::workflow::WorkflowCreate,
    ) -> Result<crate::models::workflow::Workflow> {
        use crate::models::workflow::Workflow;
        use chrono::Utc;
        Ok(Workflow {
            id: 1,
            project_id,
            name: payload.name,
            description: payload.description,
            created: Utc::now(),
            updated: Utc::now(),
        })
    }
    async fn update_workflow(
        &self,
        _id: i32,
        _project_id: i32,
        payload: crate::models::workflow::WorkflowUpdate,
    ) -> Result<crate::models::workflow::Workflow> {
        use crate::models::workflow::Workflow;
        use chrono::Utc;
        Ok(Workflow {
            id: 1,
            project_id: 1,
            name: payload.name,
            description: payload.description,
            created: Utc::now(),
            updated: Utc::now(),
        })
    }
    async fn delete_workflow(&self, _id: i32, _project_id: i32) -> Result<()> {
        Ok(())
    }
    async fn get_workflow_nodes(
        &self,
        _workflow_id: i32,
    ) -> Result<Vec<crate::models::workflow::WorkflowNode>> {
        Ok(Vec::new())
    }
    async fn create_workflow_node(
        &self,
        workflow_id: i32,
        payload: crate::models::workflow::WorkflowNodeCreate,
    ) -> Result<crate::models::workflow::WorkflowNode> {
        use crate::models::workflow::WorkflowNode;
        Ok(WorkflowNode {
            id: 1,
            workflow_id,
            template_id: payload.template_id,
            name: payload.name,
            pos_x: payload.pos_x,
            pos_y: payload.pos_y,
            wave: payload.wave,
        })
    }
    async fn update_workflow_node(
        &self,
        id: i32,
        workflow_id: i32,
        payload: crate::models::workflow::WorkflowNodeUpdate,
    ) -> Result<crate::models::workflow::WorkflowNode> {
        use crate::models::workflow::WorkflowNode;
        Ok(WorkflowNode {
            id,
            workflow_id,
            template_id: 0,
            name: payload.name,
            pos_x: payload.pos_x,
            pos_y: payload.pos_y,
            wave: payload.wave,
        })
    }
    async fn delete_workflow_node(&self, _id: i32, _workflow_id: i32) -> Result<()> {
        Ok(())
    }
    async fn get_workflow_edges(
        &self,
        _workflow_id: i32,
    ) -> Result<Vec<crate::models::workflow::WorkflowEdge>> {
        Ok(Vec::new())
    }
    async fn create_workflow_edge(
        &self,
        workflow_id: i32,
        payload: crate::models::workflow::WorkflowEdgeCreate,
    ) -> Result<crate::models::workflow::WorkflowEdge> {
        use crate::models::workflow::WorkflowEdge;
        Ok(WorkflowEdge {
            id: 1,
            workflow_id,
            from_node_id: payload.from_node_id,
            to_node_id: payload.to_node_id,
            condition: payload.condition,
        })
    }
    async fn delete_workflow_edge(&self, _id: i32, _workflow_id: i32) -> Result<()> {
        Ok(())
    }
    async fn get_workflow_runs(
        &self,
        _workflow_id: i32,
        _project_id: i32,
    ) -> Result<Vec<crate::models::workflow::WorkflowRun>> {
        Ok(Vec::new())
    }
    async fn create_workflow_run(
        &self,
        _workflow_id: i32,
        _project_id: i32,
    ) -> Result<crate::models::workflow::WorkflowRun> {
        Err(Error::Other("not implemented".to_string()))
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
    ) -> Result<Vec<crate::models::notification::NotificationPolicy>> {
        Ok(Vec::new())
    }
    async fn get_notification_policy(
        &self,
        _id: i32,
        _project_id: i32,
    ) -> Result<crate::models::notification::NotificationPolicy> {
        Err(Error::NotFound("NotificationPolicy not found".to_string()))
    }
    async fn create_notification_policy(
        &self,
        _project_id: i32,
        _payload: crate::models::notification::NotificationPolicyCreate,
    ) -> Result<crate::models::notification::NotificationPolicy> {
        Err(Error::Other("not implemented".to_string()))
    }
    async fn update_notification_policy(
        &self,
        _id: i32,
        _project_id: i32,
        _payload: crate::models::notification::NotificationPolicyUpdate,
    ) -> Result<crate::models::notification::NotificationPolicy> {
        Err(Error::Other("not implemented".to_string()))
    }
    async fn delete_notification_policy(&self, _id: i32, _project_id: i32) -> Result<()> {
        Ok(())
    }
    async fn get_matching_policies(
        &self,
        _project_id: i32,
        _trigger: &str,
        _template_id: Option<i32>,
    ) -> Result<Vec<crate::models::notification::NotificationPolicy>> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl crate::db::store::CredentialTypeManager for MockStore {
    async fn get_credential_types(
        &self,
    ) -> Result<Vec<crate::models::credential_type::CredentialType>> {
        Ok(Vec::new())
    }
    async fn get_credential_type(
        &self,
        _id: i32,
    ) -> Result<crate::models::credential_type::CredentialType> {
        Err(Error::NotFound("CredentialType not found".to_string()))
    }
    async fn create_credential_type(
        &self,
        _payload: crate::models::credential_type::CredentialTypeCreate,
    ) -> Result<crate::models::credential_type::CredentialType> {
        Err(Error::Other("not implemented".to_string()))
    }
    async fn update_credential_type(
        &self,
        _id: i32,
        _payload: crate::models::credential_type::CredentialTypeUpdate,
    ) -> Result<crate::models::credential_type::CredentialType> {
        Err(Error::Other("not implemented".to_string()))
    }
    async fn delete_credential_type(&self, _id: i32) -> Result<()> {
        Ok(())
    }
    async fn get_credential_instances(
        &self,
        _project_id: i32,
    ) -> Result<Vec<crate::models::credential_type::CredentialInstance>> {
        Ok(Vec::new())
    }
    async fn get_credential_instance(
        &self,
        _id: i32,
        _project_id: i32,
    ) -> Result<crate::models::credential_type::CredentialInstance> {
        Err(Error::NotFound("CredentialInstance not found".to_string()))
    }
    async fn create_credential_instance(
        &self,
        _project_id: i32,
        _payload: crate::models::credential_type::CredentialInstanceCreate,
    ) -> Result<crate::models::credential_type::CredentialInstance> {
        Err(Error::Other("not implemented".to_string()))
    }
    async fn delete_credential_instance(&self, _id: i32, _project_id: i32) -> Result<()> {
        Ok(())
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
        Ok(Vec::new())
    }
    async fn get_snapshot(
        &self,
        _id: i32,
        _project_id: i32,
    ) -> Result<crate::models::snapshot::TaskSnapshot> {
        Err(Error::NotFound("Snapshot not found".to_string()))
    }
    async fn create_snapshot(
        &self,
        _project_id: i32,
        _payload: crate::models::snapshot::TaskSnapshotCreate,
    ) -> Result<crate::models::snapshot::TaskSnapshot> {
        Err(Error::Other("not implemented".to_string()))
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
        Ok(Vec::new())
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
        Err(crate::error::Error::Other("not implemented".to_string()))
    }
    async fn get_cost_summaries(
        &self,
        _project_id: i32,
    ) -> Result<Vec<crate::models::cost_estimate::CostSummary>> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl crate::db::store::TerraformStateManager for MockStore {
    async fn get_terraform_state(
        &self,
        _pid: i32,
        _ws: &str,
    ) -> Result<Option<crate::models::TerraformState>> {
        Ok(None)
    }
    async fn list_terraform_states(
        &self,
        _pid: i32,
        _ws: &str,
    ) -> Result<Vec<crate::models::TerraformStateSummary>> {
        Ok(Vec::new())
    }
    async fn get_terraform_state_by_serial(
        &self,
        _pid: i32,
        _ws: &str,
        _serial: i32,
    ) -> Result<Option<crate::models::TerraformState>> {
        Ok(None)
    }
    async fn create_terraform_state(
        &self,
        state: crate::models::TerraformState,
    ) -> Result<crate::models::TerraformState> {
        Ok(state)
    }
    async fn delete_terraform_state(&self, _pid: i32, _ws: &str) -> Result<()> {
        Ok(())
    }
    async fn delete_all_terraform_states(&self, _pid: i32, _ws: &str) -> Result<()> {
        Ok(())
    }
    async fn lock_terraform_state(
        &self,
        pid: i32,
        ws: &str,
        lock: crate::models::TerraformStateLock,
    ) -> Result<crate::models::TerraformStateLock> {
        let _ = (pid, ws);
        Ok(lock)
    }
    async fn unlock_terraform_state(&self, _pid: i32, _ws: &str, _lock_id: &str) -> Result<()> {
        Ok(())
    }
    async fn get_terraform_lock(
        &self,
        _pid: i32,
        _ws: &str,
    ) -> Result<Option<crate::models::TerraformStateLock>> {
        Ok(None)
    }
    async fn list_terraform_workspaces(&self, _pid: i32) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
    async fn purge_expired_terraform_locks(&self) -> Result<u64> {
        Ok(0)
    }
}

#[async_trait]
impl crate::db::store::PlanApprovalManager for MockStore {
    async fn create_plan(&self, plan: TerraformPlan) -> Result<TerraformPlan> {
        self.terraform_plans
            .write()
            .unwrap()
            .insert((plan.project_id, plan.task_id), plan.clone());
        Ok(plan)
    }
    async fn get_plan_by_task(
        &self,
        project_id: i32,
        task_id: i32,
    ) -> Result<Option<TerraformPlan>> {
        Ok(self
            .terraform_plans
            .read()
            .unwrap()
            .get(&(project_id, task_id))
            .cloned())
    }
    async fn list_pending_plans(
        &self,
        _project_id: i32,
    ) -> Result<Vec<crate::models::TerraformPlan>> {
        Ok(Vec::new())
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

#[async_trait::async_trait]
impl crate::db::store::DeploymentEnvironmentManager for MockStore {
    async fn get_deployment_environments(
        &self,
        _project_id: i32,
    ) -> Result<Vec<crate::models::DeploymentEnvironment>> {
        Ok(Vec::new())
    }
    async fn get_deployment_environment(
        &self,
        _id: i32,
        _project_id: i32,
    ) -> Result<crate::models::DeploymentEnvironment> {
        Err(crate::error::Error::NotFound("not found".into()))
    }
    async fn create_deployment_environment(
        &self,
        _project_id: i32,
        _payload: crate::models::DeploymentEnvironmentCreate,
    ) -> Result<crate::models::DeploymentEnvironment> {
        Err(crate::error::Error::Other("not implemented".into()))
    }
    async fn update_deployment_environment(
        &self,
        _id: i32,
        _project_id: i32,
        _payload: crate::models::DeploymentEnvironmentUpdate,
    ) -> Result<crate::models::DeploymentEnvironment> {
        Err(crate::error::Error::Other("not implemented".into()))
    }
    async fn delete_deployment_environment(&self, _id: i32, _project_id: i32) -> Result<()> {
        Ok(())
    }
    async fn get_deployment_history(
        &self,
        _env_id: i32,
        _project_id: i32,
    ) -> Result<Vec<crate::models::DeploymentRecord>> {
        Ok(Vec::new())
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

#[async_trait::async_trait]
impl crate::db::store::StructuredOutputManager for MockStore {
    async fn get_task_structured_outputs(
        &self,
        _task_id: i32,
        _project_id: i32,
    ) -> Result<Vec<crate::models::TaskStructuredOutput>> {
        Ok(Vec::new())
    }
    async fn get_task_outputs_map(
        &self,
        task_id: i32,
        _project_id: i32,
    ) -> Result<crate::models::TaskOutputsMap> {
        Ok(crate::models::TaskOutputsMap {
            task_id,
            outputs: Default::default(),
        })
    }
    async fn create_task_structured_output(
        &self,
        _task_id: i32,
        _project_id: i32,
        _payload: crate::models::TaskStructuredOutputCreate,
    ) -> Result<crate::models::TaskStructuredOutput> {
        Err(crate::error::Error::Other("not implemented".into()))
    }
    async fn create_task_structured_outputs_batch(
        &self,
        _task_id: i32,
        _project_id: i32,
        _outputs: Vec<crate::models::TaskStructuredOutputCreate>,
    ) -> Result<()> {
        Ok(())
    }
    async fn delete_task_structured_outputs(&self, _task_id: i32, _project_id: i32) -> Result<()> {
        Ok(())
    }
    async fn get_template_last_outputs(
        &self,
        _template_id: i32,
        _project_id: i32,
    ) -> Result<crate::models::TaskOutputsMap> {
        Ok(crate::models::TaskOutputsMap {
            task_id: 0,
            outputs: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // 1. Тесты для MockStore::new() и Default
    // ============================================================

    #[test]
    fn test_mock_store_new_is_empty() {
        let store = MockStore::new();
        assert_eq!(store.users.read().unwrap().len(), 0);
        assert_eq!(store.projects.read().unwrap().len(), 0);
        assert_eq!(store.tasks.read().unwrap().len(), 0);
        assert_eq!(store.templates.read().unwrap().len(), 0);
        assert_eq!(store.inventories.read().unwrap().len(), 0);
        assert_eq!(store.repositories.read().unwrap().len(), 0);
        assert_eq!(store.environments.read().unwrap().len(), 0);
        assert_eq!(store.playbook_runs.read().unwrap().len(), 0);
        assert_eq!(store.organizations.read().unwrap().len(), 0);
        assert_eq!(store.organization_users.read().unwrap().len(), 0);
        assert_eq!(store.terraform_plans.read().unwrap().len(), 0);
    }

    #[test]
    fn test_mock_store_default_is_empty() {
        let store = MockStore::default();
        assert_eq!(store.users.read().unwrap().len(), 0);
        assert_eq!(store.templates.read().unwrap().len(), 0);
        assert_eq!(store.inventories.read().unwrap().len(), 0);
    }

    // ============================================================
    // 2. Тесты для seed_* helper функций
    // ============================================================

    fn make_template(id: i32, project_id: i32) -> Template {
        Template {
            id,
            project_id,
            inventory_id: Some(1),
            repository_id: None,
            environment_id: None,
            name: format!("template_{id}"),
            playbook: "test.yml".to_string(),
            arguments: None,
            allow_override_args_in_task: false,
            allow_override_branch_in_task: false,
            allow_inventory_in_task: false,
            allow_parallel_tasks: false,
            suppress_success_alerts: false,
            require_approval: false,
            description: String::new(),
            r#type: TemplateType::Default,
            app: TemplateApp::Ansible,
            git_branch: None,
            created: Utc::now(),
            vault_key_id: None,
            view_id: None,
            build_template_id: None,
            autorun: false,
            task_params: None,
            survey_vars: None,
            vaults: None,
            parent_template_id: None,
            execution_image: None,
            pre_template_id: None,
            post_template_id: None,
            fail_template_id: None,
            deploy_environment_id: None,
        }
    }

    fn make_inventory(id: i32) -> Inventory {
        Inventory {
            id,
            project_id: 1,
            name: format!("inventory_{id}"),
            inventory_type: InventoryType::Static,
            inventory_data: "localhost".to_string(),
            key_id: None,
            secret_storage_id: None,
            ssh_login: String::new(),
            ssh_port: 22,
            extra_vars: None,
            ssh_key_id: None,
            become_key_id: None,
            vaults: None,
            created: None,
            runner_tag: None,
        }
    }

    fn make_repository(id: i32) -> Repository {
        Repository {
            id,
            project_id: 1,
            git_url: "https://github.com/test/repo.git".to_string(),
            git_branch: Some("main".to_string()),
            key_id: None,
            name: format!("repo_{id}"),
            git_type: RepositoryType::Https,
            git_path: None,
            created: None,
        }
    }

    fn make_environment(id: i32) -> Environment {
        Environment {
            id,
            project_id: 1,
            name: format!("env_{id}"),
            json: String::new(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: None,
            created: Some(Utc::now()),
        }
    }

    #[test]
    fn test_seed_template() {
        let store = MockStore::new();
        let tpl = make_template(1, 1);
        store.seed_template(tpl);
        assert_eq!(store.templates.read().unwrap().len(), 1);
        assert!(store.templates.read().unwrap().contains_key(&1));
    }

    #[test]
    fn test_seed_multiple_templates() {
        let store = MockStore::new();
        store.seed_template(make_template(1, 1));
        store.seed_template(make_template(2, 1));
        store.seed_template(make_template(3, 2));
        assert_eq!(store.templates.read().unwrap().len(), 3);
    }

    #[test]
    fn test_seed_inventory() {
        let store = MockStore::new();
        let inv = make_inventory(1);
        store.seed_inventory(inv);
        assert_eq!(store.inventories.read().unwrap().len(), 1);
        assert!(store.inventories.read().unwrap().contains_key(&1));
    }

    #[test]
    fn test_seed_repository() {
        let store = MockStore::new();
        let repo = make_repository(1);
        store.seed_repository(repo);
        assert_eq!(store.repositories.read().unwrap().len(), 1);
        assert!(store.repositories.read().unwrap().contains_key(&1));
    }

    #[test]
    fn test_seed_environment() {
        let store = MockStore::new();
        let env = make_environment(1);
        store.seed_environment(env);
        assert_eq!(store.environments.read().unwrap().len(), 1);
        assert!(store.environments.read().unwrap().contains_key(&1));
    }

    #[test]
    fn test_seed_playbook_run() {
        let store = MockStore::new();
        let run = PlaybookRun {
            id: 1,
            project_id: 1,
            playbook_id: 1,
            task_id: Some(1),
            template_id: Some(1),
            inventory_id: Some(1),
            environment_id: None,
            extra_vars: None,
            limit_hosts: None,
            tags: None,
            skip_tags: None,
            user_id: Some(1),
            status: PlaybookRunStatus::Waiting,
            output: None,
            error_message: None,
            created: Utc::now(),
            updated: Utc::now(),
            start_time: None,
            end_time: None,
            duration_seconds: None,
            hosts_total: Some(0),
            hosts_changed: Some(0),
            hosts_unreachable: Some(0),
            hosts_failed: Some(0),
        };
        store.seed_playbook_run(run);
        assert_eq!(store.playbook_runs.read().unwrap().len(), 1);
        assert!(store.playbook_runs.read().unwrap().contains_key(&1));
    }

    #[test]
    fn test_seed_terraform_plan() {
        let store = MockStore::new();
        let plan = TerraformPlan {
            id: 1,
            project_id: 1,
            task_id: 1,
            plan_output: "plan output".to_string(),
            plan_json: None,
            resources_added: 0,
            resources_changed: 0,
            resources_removed: 0,
            status: "pending".to_string(),
            created_at: Utc::now(),
            reviewed_at: None,
            reviewed_by: None,
            review_comment: None,
        };
        store.seed_terraform_plan(plan);
        assert_eq!(store.terraform_plans.read().unwrap().len(), 1);
        assert!(store.terraform_plans.read().unwrap().contains_key(&(1, 1)));
    }

    // ============================================================
    // 3. Тесты для моделей
    // ============================================================

    #[test]
    fn test_template_equality_by_id() {
        let tpl1 = make_template(1, 1);
        let tpl2 = make_template(1, 1);
        assert_eq!(tpl1.id, tpl2.id);
        assert_eq!(tpl1.project_id, tpl2.project_id);
    }

    #[test]
    fn test_inventory_type_static() {
        let inv = make_inventory(1);
        assert_eq!(inv.inventory_type, InventoryType::Static);
    }

    #[test]
    fn test_repository_fields() {
        let repo = make_repository(42);
        assert_eq!(repo.id, 42);
        assert_eq!(repo.git_branch, Some("main".to_string()));
        assert!(repo.git_url.contains("github.com"));
    }

    #[test]
    fn test_environment_optional_fields() {
        let env = make_environment(1);
        assert!(env.secret_storage_id.is_none());
        assert!(env.secret_storage_key_prefix.is_none());
        assert!(env.secrets.is_none());
    }

    #[test]
    fn test_playbook_run_initial_status() {
        let store = MockStore::new();
        let run = PlaybookRun {
            id: 1,
            project_id: 1,
            playbook_id: 1,
            task_id: None,
            template_id: Some(1),
            inventory_id: Some(1),
            environment_id: None,
            extra_vars: None,
            limit_hosts: None,
            tags: None,
            skip_tags: None,
            user_id: None,
            status: PlaybookRunStatus::Waiting,
            output: None,
            error_message: None,
            created: Utc::now(),
            updated: Utc::now(),
            start_time: None,
            end_time: None,
            duration_seconds: None,
            hosts_total: Some(0),
            hosts_changed: Some(0),
            hosts_unreachable: Some(0),
            hosts_failed: Some(0),
        };
        store.seed_playbook_run(run);
        let stored = store
            .playbook_runs
            .read()
            .unwrap()
            .get(&1)
            .cloned()
            .unwrap();
        assert_eq!(stored.status, PlaybookRunStatus::Waiting);
    }

    #[test]
    fn test_terraform_plan_key_is_composite() {
        let store = MockStore::new();
        let plan1 = TerraformPlan {
            id: 1,
            project_id: 1,
            task_id: 10,
            plan_output: "plan1".to_string(),
            plan_json: None,
            resources_added: 0,
            resources_changed: 0,
            resources_removed: 0,
            status: "pending".to_string(),
            created_at: Utc::now(),
            reviewed_at: None,
            reviewed_by: None,
            review_comment: None,
        };
        let plan2 = TerraformPlan {
            id: 2,
            project_id: 1,
            task_id: 20,
            plan_output: "plan2".to_string(),
            plan_json: None,
            resources_added: 0,
            resources_changed: 0,
            resources_removed: 0,
            status: "pending".to_string(),
            created_at: Utc::now(),
            reviewed_at: None,
            reviewed_by: None,
            review_comment: None,
        };
        store.seed_terraform_plan(plan1);
        store.seed_terraform_plan(plan2);
        assert_eq!(store.terraform_plans.read().unwrap().len(), 2);
        assert!(store.terraform_plans.read().unwrap().contains_key(&(1, 10)));
        assert!(store.terraform_plans.read().unwrap().contains_key(&(1, 20)));
    }

    // ============================================================
    // 4. Тесты для ConnectionManager
    // ============================================================

    #[tokio::test]
    async fn test_connection_manager_connect_returns_ok() {
        let store = MockStore::new();
        assert!(store.connect().await.is_ok());
    }

    #[tokio::test]
    async fn test_connection_manager_close_returns_ok() {
        let store = MockStore::new();
        assert!(store.close().await.is_ok());
    }

    #[tokio::test]
    async fn test_connection_manager_is_permanent() {
        let store = MockStore::new();
        assert!(store.is_permanent());
    }

    // ============================================================
    // 5. Тесты для MigrationManager
    // ============================================================

    #[tokio::test]
    async fn test_migration_manager_get_dialect() {
        let store = MockStore::new();
        assert_eq!(store.get_dialect(), "mock");
    }

    #[tokio::test]
    async fn test_migration_manager_is_initialized() {
        let store = MockStore::new();
        let result = store.is_initialized().await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_migration_manager_apply_migration() {
        let store = MockStore::new();
        assert!(store
            .apply_migration(1, "test_migration".to_string())
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_migration_manager_is_migration_applied() {
        let store = MockStore::new();
        let result = store.is_migration_applied(42).await.unwrap();
        assert!(result);
    }

    // ============================================================
    // 6. Тесты для OptionsManager
    // ============================================================

    #[tokio::test]
    async fn test_options_manager_get_options_empty() {
        let store = MockStore::new();
        let opts = store.get_options().await.unwrap();
        assert!(opts.is_empty());
    }

    #[tokio::test]
    async fn test_options_manager_get_option_returns_none() {
        let store = MockStore::new();
        let opt = store.get_option("some_key").await.unwrap();
        assert!(opt.is_none());
    }

    #[tokio::test]
    async fn test_options_manager_set_option_ok() {
        let store = MockStore::new();
        assert!(store.set_option("key", "value").await.is_ok());
        let opt = store.get_option("key").await.unwrap();
        assert!(opt.is_none());
    }

    #[tokio::test]
    async fn test_options_manager_delete_option_ok() {
        let store = MockStore::new();
        assert!(store.delete_option("nonexistent").await.is_ok());
    }

    // ============================================================
    // 7. Тесты для UserManager
    // ============================================================

    #[tokio::test]
    async fn test_user_manager_get_users_empty() {
        let store = MockStore::new();
        let users = store
            .get_users(RetrieveQueryParams::default())
            .await
            .unwrap();
        assert!(users.is_empty());
    }

    #[tokio::test]
    async fn test_user_manager_get_user_not_found() {
        let store = MockStore::new();
        let result = store.get_user(999).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::NotFound(msg) => assert!(msg.contains("999")),
            _ => panic!("Expected Error::NotFound"),
        }
    }

    #[tokio::test]
    async fn test_user_manager_get_all_admins_empty() {
        let store = MockStore::new();
        let admins = store.get_all_admins().await.unwrap();
        assert!(admins.is_empty());
    }

    #[tokio::test]
    async fn test_user_manager_get_user_count_empty() {
        let store = MockStore::new();
        let count = store.get_user_count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_user_manager_get_user_totp_none() {
        let store = MockStore::new();
        let totp = store.get_user_totp(1).await.unwrap();
        assert!(totp.is_none());
    }

    #[tokio::test]
    async fn test_user_manager_set_user_totp_ok() {
        let store = MockStore::new();
        let totp = UserTotp {
            id: 1,
            created: Utc::now(),
            user_id: 1,
            url: "otpauth://totp/test".to_string(),
            recovery_hash: "hash".to_string(),
            recovery_code: None,
        };
        assert!(store.set_user_totp(999, &totp).await.is_ok());
    }

    // ============================================================
    // 8. Тесты для ProjectStore
    // ============================================================

    #[tokio::test]
    async fn test_project_store_get_projects_empty() {
        let store = MockStore::new();
        let projects = store.get_projects(None).await.unwrap();
        assert!(projects.is_empty());
    }

    #[tokio::test]
    async fn test_project_store_get_project_not_found() {
        let store = MockStore::new();
        let result = store.get_project(1).await;
        assert!(result.is_err());
    }

    // ============================================================
    // 9. Тесты для TemplateManager
    // ============================================================

    #[tokio::test]
    async fn test_template_manager_get_templates_empty() {
        let store = MockStore::new();
        let templates = store.get_templates(1).await.unwrap();
        assert!(templates.is_empty());
    }

    #[tokio::test]
    async fn test_template_manager_get_template_not_found() {
        let store = MockStore::new();
        let result = store.get_template(1, 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_template_manager_get_template_wrong_project() {
        let store = MockStore::new();
        store.seed_template(make_template(1, 2));
        let result = store.get_template(1, 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_template_manager_get_template_found() {
        let store = MockStore::new();
        store.seed_template(make_template(1, 1));
        let tpl = store.get_template(1, 1).await.unwrap();
        assert_eq!(tpl.id, 1);
        assert_eq!(tpl.project_id, 1);
    }

    // ============================================================
    // 10. Тесты для InventoryManager
    // ============================================================

    #[tokio::test]
    async fn test_inventory_manager_get_inventory_not_found() {
        let store = MockStore::new();
        let result = store.get_inventory(1, 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_inventory_manager_get_inventory_found() {
        let store = MockStore::new();
        store.seed_inventory(make_inventory(5));
        let inv = store.get_inventory(1, 5).await.unwrap();
        assert_eq!(inv.id, 5);
    }

    // ============================================================
    // 11. Тесты для RepositoryManager
    // ============================================================

    #[tokio::test]
    async fn test_repository_manager_get_repository_not_found() {
        let store = MockStore::new();
        let result = store.get_repository(1, 1).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::NotFound(msg) => assert!(msg.contains("1")),
            _ => panic!("Expected Error::NotFound"),
        }
    }

    // ============================================================
    // 12. Тесты для EnvironmentManager
    // ============================================================

    #[tokio::test]
    async fn test_environment_manager_get_environment_not_found() {
        let store = MockStore::new();
        let result = store.get_environment(1, 42).await;
        assert!(result.is_err());
    }

    // ============================================================
    // 13. Тесты для AccessKeyManager
    // ============================================================

    #[tokio::test]
    async fn test_access_key_manager_get_access_keys_empty() {
        let store = MockStore::new();
        let keys = store.get_access_keys(1).await.unwrap();
        assert!(keys.is_empty());
    }

    #[tokio::test]
    async fn test_access_key_manager_get_access_key_not_found() {
        let store = MockStore::new();
        let result = store.get_access_key(1, 1).await;
        assert!(result.is_err());
    }

    // ============================================================
    // 14. Тесты для ScheduleManager
    // ============================================================

    #[tokio::test]
    async fn test_schedule_manager_get_schedules_empty() {
        let store = MockStore::new();
        let schedules = store.get_schedules(1).await.unwrap();
        assert!(schedules.is_empty());
    }

    #[tokio::test]
    async fn test_schedule_manager_get_all_schedules_empty() {
        let store = MockStore::new();
        let schedules = store.get_all_schedules().await.unwrap();
        assert!(schedules.is_empty());
    }

    // ============================================================
    // 15. Тесты для SessionManager
    // ============================================================

    #[tokio::test]
    async fn test_session_manager_get_session_not_found() {
        let store = MockStore::new();
        let result = store.get_session(1, 1).await;
        assert!(result.is_err());
    }

    // ============================================================
    // 16. Тесты для TokenManager
    // ============================================================

    #[tokio::test]
    async fn test_token_manager_get_api_tokens_empty() {
        let store = MockStore::new();
        let tokens = store.get_api_tokens(1).await.unwrap();
        assert!(tokens.is_empty());
    }

    #[tokio::test]
    async fn test_token_manager_get_api_token_not_found() {
        let store = MockStore::new();
        let result = store.get_api_token(1).await;
        assert!(result.is_err());
    }

    // ============================================================
    // 17. Тесты для EventManager
    // ============================================================

    #[tokio::test]
    async fn test_event_manager_get_events_empty() {
        let store = MockStore::new();
        let events = store.get_events(None, 10).await.unwrap();
        assert!(events.is_empty());
    }

    // ============================================================
    // 18. Тесты для RunnerManager
    // ============================================================

    #[tokio::test]
    async fn test_runner_manager_get_runners_empty() {
        let store = MockStore::new();
        let runners = store.get_runners(None).await.unwrap();
        assert!(runners.is_empty());
    }

    #[tokio::test]
    async fn test_runner_manager_get_runner_not_found() {
        let store = MockStore::new();
        let result = store.get_runner(1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_runner_manager_get_runners_count_zero() {
        let store = MockStore::new();
        let count = store.get_runners_count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_runner_manager_get_active_runners_count_zero() {
        let store = MockStore::new();
        let count = store.get_active_runners_count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_runner_manager_find_runner_by_token_not_found() {
        let store = MockStore::new();
        let result = store.find_runner_by_token("some_token").await;
        assert!(result.is_err());
    }

    // ============================================================
    // 19. Тесты для ViewManager
    // ============================================================

    #[tokio::test]
    async fn test_view_manager_get_views_empty() {
        let store = MockStore::new();
        let views = store.get_views(1).await.unwrap();
        assert!(views.is_empty());
    }

    // ============================================================
    // 20. Тесты для IntegrationManager
    // ============================================================

    #[tokio::test]
    async fn test_integration_manager_get_integrations_empty() {
        let store = MockStore::new();
        let integrations = store.get_integrations(1).await.unwrap();
        assert!(integrations.is_empty());
    }

    // ============================================================
    // 21. Тесты для SecretStorageManager
    // ============================================================

    #[tokio::test]
    async fn test_secret_storage_manager_get_secret_storages_empty() {
        let store = MockStore::new();
        let storages = store.get_secret_storages(1).await.unwrap();
        assert!(storages.is_empty());
    }

    // ============================================================
    // 22. Тесты для AuditLogManager
    // ============================================================

    #[tokio::test]
    async fn test_audit_log_manager_search_returns_empty_result() {
        let store = MockStore::new();
        let filter = AuditLogFilter::default();
        let result = store.search_audit_logs(&filter).await.unwrap();
        assert!(result.records.is_empty());
        assert_eq!(result.total, 0);
    }

    // ============================================================
    // 23. Тесты для OrganizationManager
    // ============================================================

    #[tokio::test]
    async fn test_organization_manager_get_organizations_empty() {
        let store = MockStore::new();
        let orgs = store.get_organizations().await.unwrap();
        assert!(orgs.is_empty());
    }

    #[tokio::test]
    async fn test_organization_manager_get_organization_not_found() {
        let store = MockStore::new();
        let result = store.get_organization(1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_organization_manager_get_organizations_after_create() {
        let store = MockStore::new();
        let payload = OrganizationCreate {
            name: "Test Org".to_string(),
            slug: None,
            description: None,
            settings: None,
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
        };
        let org = store.create_organization(payload).await.unwrap();
        assert_eq!(org.name, "Test Org");
        assert!(org.active);

        let orgs = store.get_organizations().await.unwrap();
        assert_eq!(orgs.len(), 1);
    }

    #[tokio::test]
    async fn test_organization_manager_check_quota_projects() {
        let store = MockStore::new();
        let payload = OrganizationCreate {
            name: "Quota Org".to_string(),
            slug: Some("quota-org".to_string()),
            description: None,
            settings: None,
            quota_max_projects: Some(5),
            quota_max_users: None,
            quota_max_tasks_per_month: None,
        };
        let org = store.create_organization(payload).await.unwrap();
        let allowed = store
            .check_organization_quota(org.id, "projects")
            .await
            .unwrap();
        assert!(allowed);
    }

    #[tokio::test]
    async fn test_organization_manager_add_and_get_users() {
        let store = MockStore::new();
        let org_payload = OrganizationCreate {
            name: "Org With Users".to_string(),
            slug: Some("org-users".to_string()),
            description: None,
            settings: None,
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
        };
        let org = store.create_organization(org_payload).await.unwrap();

        let user_payload = OrganizationUserCreate {
            org_id: org.id,
            user_id: 10,
            role: "member".to_string(),
        };
        let ou = store.add_user_to_organization(user_payload).await.unwrap();
        assert_eq!(ou.org_id, org.id);
        assert_eq!(ou.user_id, 10);
        assert_eq!(ou.role, "member");

        let org_users = store.get_organization_users(org.id).await.unwrap();
        assert_eq!(org_users.len(), 1);
        assert_eq!(org_users[0].user_id, 10);
    }

    #[tokio::test]
    async fn test_organization_manager_remove_user() {
        let store = MockStore::new();
        let org_payload = OrganizationCreate {
            name: "Remove User Org".to_string(),
            slug: Some("remove-user".to_string()),
            description: None,
            settings: None,
            quota_max_projects: None,
            quota_max_users: None,
            quota_max_tasks_per_month: None,
        };
        let org = store.create_organization(org_payload).await.unwrap();
        store
            .add_user_to_organization(OrganizationUserCreate {
                org_id: org.id,
                user_id: 20,
                role: "admin".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(store.get_organization_users(org.id).await.unwrap().len(), 1);
        store
            .remove_user_from_organization(org.id, 20)
            .await
            .unwrap();
        assert_eq!(store.get_organization_users(org.id).await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_organization_manager_update_role() {
        let store = MockStore::new();
        let org = store
            .create_organization(OrganizationCreate {
                name: "Role Update Org".to_string(),
                slug: Some("role-update".to_string()),
                description: None,
                settings: None,
                quota_max_projects: None,
                quota_max_users: None,
                quota_max_tasks_per_month: None,
            })
            .await
            .unwrap();

        store
            .add_user_to_organization(OrganizationUserCreate {
                org_id: org.id,
                user_id: 30,
                role: "member".to_string(),
            })
            .await
            .unwrap();

        store
            .update_user_organization_role(org.id, 30, "owner")
            .await
            .unwrap();

        let users = store.get_organization_users(org.id).await.unwrap();
        assert_eq!(users[0].role, "owner");
    }

    #[tokio::test]
    async fn test_organization_manager_delete_organization() {
        let store = MockStore::new();
        let org = store
            .create_organization(OrganizationCreate {
                name: "To Delete".to_string(),
                slug: Some("to-delete".to_string()),
                description: None,
                settings: None,
                quota_max_projects: None,
                quota_max_users: None,
                quota_max_tasks_per_month: None,
            })
            .await
            .unwrap();

        assert_eq!(store.get_organizations().await.unwrap().len(), 1);
        store.delete_organization(org.id).await.unwrap();
        assert_eq!(store.get_organizations().await.unwrap().len(), 0);
    }

    // ============================================================
    // 24. Тесты для PlaybookRunManager
    // ============================================================

    #[tokio::test]
    async fn test_playbook_run_manager_get_runs_empty() {
        let store = MockStore::new();
        let filter = PlaybookRunFilter::default();
        let runs = store.get_playbook_runs(filter).await.unwrap();
        assert!(runs.is_empty());
    }

    #[tokio::test]
    async fn test_playbook_run_manager_get_run_not_found() {
        let store = MockStore::new();
        let result = store.get_playbook_run(1, 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_playbook_run_manager_get_stats_empty() {
        let store = MockStore::new();
        let stats = store.get_playbook_run_stats(1).await.unwrap();
        assert_eq!(stats.total_runs, 0);
        assert_eq!(stats.success_runs, 0);
        assert_eq!(stats.failed_runs, 0);
    }

    // ============================================================
    // 25. Тесты для TaskManager
    // ============================================================

    #[tokio::test]
    async fn test_task_manager_get_tasks_empty() {
        let store = MockStore::new();
        let tasks = store.get_tasks(1, None).await.unwrap();
        assert!(tasks.is_empty());
    }

    #[tokio::test]
    async fn test_task_manager_get_global_tasks_empty() {
        let store = MockStore::new();
        let tasks = store.get_global_tasks(None, None).await.unwrap();
        assert!(tasks.is_empty());
    }

    #[tokio::test]
    async fn test_task_manager_get_task_not_found() {
        let store = MockStore::new();
        let result = store.get_task(1, 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_task_manager_get_running_tasks_count_zero() {
        let store = MockStore::new();
        let count = store.get_running_tasks_count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_task_manager_get_waiting_tasks_count_zero() {
        let store = MockStore::new();
        let count = store.get_waiting_tasks_count().await.unwrap();
        assert_eq!(count, 0);
    }

    // ============================================================
    // 26. Тесты для HookManager
    // ============================================================

    #[tokio::test]
    async fn test_hook_manager_get_hooks_empty() {
        let store = MockStore::new();
        let hooks = store.get_hooks_by_template(1).await.unwrap();
        assert!(hooks.is_empty());
    }

    // ============================================================
    // 27. Тесты для ProjectInviteManager
    // ============================================================

    #[tokio::test]
    async fn test_project_invite_manager_get_invites_empty() {
        let store = MockStore::new();
        let invites = store
            .get_project_invites(1, RetrieveQueryParams::default())
            .await
            .unwrap();
        assert!(invites.is_empty());
    }

    // ============================================================
    // 28. Тесты для TerraformInventoryManager
    // ============================================================

    #[tokio::test]
    async fn test_terraform_inventory_get_aliases_empty() {
        let store = MockStore::new();
        let aliases = store.get_terraform_inventory_aliases(1, 1).await.unwrap();
        assert!(aliases.is_empty());
    }

    #[tokio::test]
    async fn test_terraform_inventory_get_state_count_zero() {
        let store = MockStore::new();
        let count = store.get_terraform_state_count().await.unwrap();
        assert_eq!(count, 0);
    }

    // ============================================================
    // 29. Тесты для TerraformStateManager
    // ============================================================

    #[tokio::test]
    async fn test_terraform_state_get_none() {
        let store = MockStore::new();
        let state = store.get_terraform_state(1, "default").await.unwrap();
        assert!(state.is_none());
    }

    #[tokio::test]
    async fn test_terraform_state_list_empty() {
        let store = MockStore::new();
        let states = store.list_terraform_states(1, "default").await.unwrap();
        assert!(states.is_empty());
    }

    #[tokio::test]
    async fn test_terraform_state_list_workspaces_empty() {
        let store = MockStore::new();
        let workspaces = store.list_terraform_workspaces(1).await.unwrap();
        assert!(workspaces.is_empty());
    }

    // ============================================================
    // 30. Тесты для PlanApprovalManager
    // ============================================================

    #[tokio::test]
    async fn test_plan_approval_get_plan_by_task_none() {
        let store = MockStore::new();
        let plan = store.get_plan_by_task(1, 1).await.unwrap();
        assert!(plan.is_none());
    }

    #[tokio::test]
    async fn test_plan_approval_list_pending_empty() {
        let store = MockStore::new();
        let plans = store.list_pending_plans(1).await.unwrap();
        assert!(plans.is_empty());
    }

    #[tokio::test]
    async fn test_plan_approval_create_and_get() {
        let store = MockStore::new();
        let plan = TerraformPlan {
            id: 100,
            project_id: 5,
            task_id: 50,
            plan_output: "plan output".to_string(),
            plan_json: None,
            resources_added: 2,
            resources_changed: 1,
            resources_removed: 0,
            status: "pending".to_string(),
            created_at: Utc::now(),
            reviewed_at: None,
            reviewed_by: None,
            review_comment: None,
        };
        let created = store.create_plan(plan.clone()).await.unwrap();
        assert_eq!(created.id, 100);

        let found = store.get_plan_by_task(5, 50).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 100);
    }

    #[tokio::test]
    async fn test_plan_approval_approve_ok() {
        let store = MockStore::new();
        assert!(store
            .approve_plan(1, 2, Some("ok".to_string()))
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_plan_approval_reject_ok() {
        let store = MockStore::new();
        assert!(store
            .reject_plan(1, 2, Some("no".to_string()))
            .await
            .is_ok());
    }

    // ============================================================
    // 31. Тесты для CostEstimateManager
    // ============================================================

    #[tokio::test]
    async fn test_cost_estimate_get_estimates_empty() {
        let store = MockStore::new();
        let estimates = store.get_cost_estimates(1, 10).await.unwrap();
        assert!(estimates.is_empty());
    }

    #[tokio::test]
    async fn test_cost_estimate_get_for_task_none() {
        let store = MockStore::new();
        let opt = store.get_cost_estimate_for_task(1, 1).await.unwrap();
        assert!(opt.is_none());
    }

    // ============================================================
    // 32. Тесты для SnapshotManager
    // ============================================================

    #[tokio::test]
    async fn test_snapshot_manager_get_snapshots_empty() {
        let store = MockStore::new();
        let snapshots = store.get_snapshots(1, None, 10).await.unwrap();
        assert!(snapshots.is_empty());
    }

    // ============================================================
    // 33. Тесты для CredentialTypeManager
    // ============================================================

    #[tokio::test]
    async fn test_credential_type_manager_get_types_empty() {
        let store = MockStore::new();
        let types = store.get_credential_types().await.unwrap();
        assert!(types.is_empty());
    }

    // ============================================================
    // 34. Тесты для NotificationPolicyManager
    // ============================================================

    #[tokio::test]
    async fn test_notification_policy_get_matching_empty() {
        let store = MockStore::new();
        let policies = store
            .get_matching_policies(1, "task_success", None)
            .await
            .unwrap();
        assert!(policies.is_empty());
    }

    // ============================================================
    // 35. Тесты для DriftManager
    // ============================================================

    #[tokio::test]
    async fn test_drift_manager_get_configs_empty() {
        let store = MockStore::new();
        let configs = store.get_drift_configs(1).await.unwrap();
        assert!(configs.is_empty());
    }

    // ============================================================
    // 36. Тесты для LdapGroupMappingManager
    // ============================================================

    #[tokio::test]
    async fn test_ldap_group_mapping_get_empty() {
        let store = MockStore::new();
        let mappings = store.get_ldap_group_mappings().await.unwrap();
        assert!(mappings.is_empty());
    }

    // ============================================================
    // 37. Тесты для StructuredOutputManager
    // ============================================================

    #[tokio::test]
    async fn test_structured_output_manager_get_outputs_map() {
        let store = MockStore::new();
        let map = store.get_task_outputs_map(42, 1).await.unwrap();
        assert_eq!(map.task_id, 42);
        assert!(map.outputs.is_empty());
    }

    #[tokio::test]
    async fn test_structured_output_manager_get_template_last_outputs() {
        let store = MockStore::new();
        let map = store.get_template_last_outputs(1, 1).await.unwrap();
        assert_eq!(map.task_id, 0);
    }

    // ============================================================
    // 38. Тесты для WebhookManager
    // ============================================================

    #[tokio::test]
    async fn test_webhook_manager_get_logs_empty() {
        let store = MockStore::new();
        let logs = store.get_webhook_logs(1).await.unwrap();
        assert!(logs.is_empty());
    }

    // ============================================================
    // 39. Тесты для DeploymentEnvironmentManager
    // ============================================================

    #[tokio::test]
    async fn test_deployment_env_get_history_empty() {
        let store = MockStore::new();
        let records = store.get_deployment_history(1, 1).await.unwrap();
        assert!(records.is_empty());
    }

    // ============================================================
    // 40. Тесты для WorkflowManager
    // ============================================================

    #[tokio::test]
    async fn test_workflow_manager_get_workflows_empty() {
        let store = MockStore::new();
        let workflows = store.get_workflows(1).await.unwrap();
        assert!(workflows.is_empty());
    }

    #[tokio::test]
    async fn test_workflow_manager_get_nodes_empty() {
        let store = MockStore::new();
        let nodes = store.get_workflow_nodes(1).await.unwrap();
        assert!(nodes.is_empty());
    }

    #[tokio::test]
    async fn test_workflow_manager_get_edges_empty() {
        let store = MockStore::new();
        let edges = store.get_workflow_edges(1).await.unwrap();
        assert!(edges.is_empty());
    }

    #[tokio::test]
    async fn test_workflow_manager_get_runs_empty() {
        let store = MockStore::new();
        let runs = store.get_workflow_runs(1, 1).await.unwrap();
        assert!(runs.is_empty());
    }

    // ============================================================
    // 41. Тесты для PlaybookManager
    // ============================================================

    #[tokio::test]
    async fn test_playbook_manager_get_playbooks_empty() {
        let store = MockStore::new();
        let playbooks = store.get_playbooks(1).await.unwrap();
        assert!(playbooks.is_empty());
    }

    // ============================================================
    // 42. Стресс-тест: множественные seed операции
    // ============================================================

    #[test]
    fn test_bulk_seed_operations() {
        let store = MockStore::new();
        for i in 1..=50 {
            store.seed_template(make_template(i, 1));
        }
        assert_eq!(store.templates.read().unwrap().len(), 50);

        for i in 1..=30 {
            store.seed_inventory(make_inventory(i));
        }
        assert_eq!(store.inventories.read().unwrap().len(), 30);

        for i in 1..=20 {
            store.seed_repository(make_repository(i));
        }
        assert_eq!(store.repositories.read().unwrap().len(), 20);
    }

    #[test]
    fn test_seed_overwrite_same_id() {
        let store = MockStore::new();
        store.seed_template(make_template(1, 1));
        let tpl2 = Template {
            id: 1,
            project_id: 2,
            name: "overwritten".to_string(),
            playbook: "new.yml".to_string(),
            inventory_id: Some(1),
            repository_id: None,
            environment_id: None,
            arguments: None,
            allow_override_args_in_task: false,
            allow_override_branch_in_task: false,
            allow_inventory_in_task: false,
            allow_parallel_tasks: false,
            suppress_success_alerts: false,
            require_approval: false,
            description: String::new(),
            r#type: TemplateType::Default,
            app: TemplateApp::Ansible,
            git_branch: None,
            created: Utc::now(),
            vault_key_id: None,
            view_id: None,
            build_template_id: None,
            autorun: false,
            task_params: None,
            survey_vars: None,
            vaults: None,
            parent_template_id: None,
            execution_image: None,
            pre_template_id: None,
            post_template_id: None,
            fail_template_id: None,
            deploy_environment_id: None,
        };
        store.seed_template(tpl2);
        assert_eq!(store.templates.read().unwrap().len(), 1);
        assert_eq!(
            store.templates.read().unwrap().get(&1).unwrap().name,
            "overwritten"
        );
    }
}
