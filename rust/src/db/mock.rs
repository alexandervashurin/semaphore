//! Mock-реализация Store для тестов

use crate::db::store::*;
use crate::models::*;
use crate::error::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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
impl Store for MockStore {
    fn connection_manager(&self) -> Arc<dyn ConnectionManager> {
        Arc::new(MockConnectionManager)
    }

    fn migration_manager(&self) -> Arc<dyn MigrationManager> {
        Arc::new(MockMigrationManager)
    }

    fn options_manager(&self) -> Arc<dyn OptionsManager> {
        Arc::new(MockOptionsManager)
    }

    fn user_manager(&self) -> Arc<dyn UserManager> {
        Arc::new(MockUserManager {
            users: self.users.clone(),
        })
    }

    fn project_store(&self) -> Arc<dyn ProjectStore> {
        Arc::new(MockProjectStore {
            projects: self.projects.clone(),
        })
    }

    fn template_manager(&self) -> Arc<dyn TemplateManager> {
        Arc::new(MockTemplateManager {
            templates: self.templates.clone(),
        })
    }

    fn inventory_manager(&self) -> Arc<dyn InventoryManager> {
        Arc::new(MockInventoryManager)
    }

    fn repository_manager(&self) -> Arc<dyn RepositoryManager> {
        Arc::new(MockRepositoryManager)
    }

    fn environment_manager(&self) -> Arc<dyn EnvironmentManager> {
        Arc::new(MockEnvironmentManager)
    }

    fn access_key_manager(&self) -> Arc<dyn AccessKeyManager> {
        Arc::new(MockAccessKeyManager)
    }

    fn task_manager(&self) -> Arc<dyn TaskManager> {
        Arc::new(MockTaskManager {
            tasks: self.tasks.clone(),
        })
    }

    fn schedule_manager(&self) -> Arc<dyn ScheduleManager> {
        Arc::new(MockScheduleManager)
    }

    fn session_manager(&self) -> Arc<dyn SessionManager> {
        Arc::new(MockSessionManager)
    }

    fn token_manager(&self) -> Arc<dyn TokenManager> {
        Arc::new(MockTokenManager)
    }

    fn event_manager(&self) -> Arc<dyn EventManager> {
        Arc::new(MockEventManager)
    }

    fn runner_manager(&self) -> Arc<dyn RunnerManager> {
        Arc::new(MockRunnerManager)
    }

    fn view_manager(&self) -> Arc<dyn ViewManager> {
        Arc::new(MockViewManager)
    }

    fn integration_manager(&self) -> Arc<dyn IntegrationManager> {
        Arc::new(MockIntegrationManager)
    }
}

// Mock реализации для всех менеджеров

struct MockConnectionManager;
#[async_trait]
impl ConnectionManager for MockConnectionManager {
    async fn connect(&self) -> Result<()> { Ok(()) }
    async fn close(&self) -> Result<()> { Ok(()) }
    fn is_permanent(&self) -> bool { true }
}

struct MockMigrationManager;
#[async_trait]
impl MigrationManager for MockMigrationManager {
    fn get_dialect(&self) -> &str { "mock" }
    async fn is_initialized(&self) -> Result<bool> { Ok(true) }
    async fn initialize(&self) -> Result<()> { Ok(()) }
    async fn verify(&self) -> Result<bool> { Ok(true) }
    async fn migrate(&self) -> Result<bool> { Ok(true) }
    async fn rollback(&self) -> Result<bool> { Ok(true) }
}

struct MockOptionsManager;
#[async_trait]
impl OptionsManager for MockOptionsManager {
    async fn get_option(&self, _key: &str) -> Result<Option<String>> { Ok(None) }
    async fn set_option(&self, _key: &str, _value: &str) -> Result<()> { Ok(()) }
    async fn delete_option(&self, _key: &str) -> Result<bool> { Ok(false) }
}

struct MockUserManager {
    users: RwLock<HashMap<i32, User>>,
}
#[async_trait]
impl UserManager for MockUserManager {
    async fn get_users(&self, _params: RetrieveQueryParams) -> Result<Vec<User>> {
        Ok(self.users.read().unwrap().values().cloned().collect())
    }
    async fn get_user(&self, id: i32) -> Result<Option<User>> {
        Ok(self.users.read().unwrap().get(&id).cloned())
    }
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        Ok(self.users.read().unwrap().values().find(|u| u.username == username).cloned())
    }
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        Ok(self.users.read().unwrap().values().find(|u| u.email == email).cloned())
    }
    async fn create_user(&self, user: User) -> Result<User> {
        self.users.write().unwrap().insert(user.id, user.clone());
        Ok(user)
    }
    async fn update_user(&self, user: User) -> Result<User> {
        self.users.write().unwrap().insert(user.id, user.clone());
        Ok(user)
    }
    async fn delete_user(&self, _id: i32) -> Result<bool> { Ok(true) }
    async fn get_user_count(&self) -> Result<usize> {
        Ok(self.users.read().unwrap().len())
    }
}

struct MockProjectStore {
    projects: RwLock<HashMap<i32, Project>>,
}
#[async_trait]
impl ProjectStore for MockProjectStore {
    async fn get_projects(&self, _params: RetrieveQueryParams) -> Result<Vec<Project>> {
        Ok(self.projects.read().unwrap().values().cloned().collect())
    }
    async fn get_project(&self, id: i32) -> Result<Option<Project>> {
        Ok(self.projects.read().unwrap().get(&id).cloned())
    }
    async fn create_project(&self, project: Project) -> Result<Project> {
        self.projects.write().unwrap().insert(project.id, project.clone());
        Ok(project)
    }
    async fn update_project(&self, project: Project) -> Result<Project> {
        self.projects.write().unwrap().insert(project.id, project.clone());
        Ok(project)
    }
    async fn delete_project(&self, _id: i32) -> Result<bool> { Ok(true) }
}

struct MockTemplateManager {
    templates: RwLock<HashMap<i32, Template>>,
}
#[async_trait]
impl TemplateManager for MockTemplateManager {
    async fn get_templates(&self, _project_id: i32, _params: RetrieveQueryParams) -> Result<Vec<Template>> {
        Ok(self.templates.read().unwrap().values().cloned().collect())
    }
    async fn get_template(&self, id: i32) -> Result<Option<Template>> {
        Ok(self.templates.read().unwrap().get(&id).cloned())
    }
    async fn create_template(&self, template: Template) -> Result<Template> {
        self.templates.write().unwrap().insert(template.id, template.clone());
        Ok(template)
    }
    async fn update_template(&self, template: Template) -> Result<Template> {
        self.templates.write().unwrap().insert(template.id, template.clone());
        Ok(template)
    }
    async fn delete_template(&self, _id: i32) -> Result<bool> { Ok(true) }
}

struct MockInventoryManager;
#[async_trait]
impl InventoryManager for MockInventoryManager {
    async fn get_inventories(&self, _project_id: i32, _params: RetrieveQueryParams) -> Result<Vec<Inventory>> { Ok(vec![]) }
    async fn get_inventory(&self, _id: i32) -> Result<Option<Inventory>> { Ok(None) }
    async fn create_inventory(&self, inv: Inventory) -> Result<Inventory> { Ok(inv) }
    async fn update_inventory(&self, inv: Inventory) -> Result<Inventory> { Ok(inv) }
    async fn delete_inventory(&self, _id: i32) -> Result<bool> { Ok(true) }
}

struct MockRepositoryManager;
#[async_trait]
impl RepositoryManager for MockRepositoryManager {
    async fn get_repositories(&self, _project_id: i32, _params: RetrieveQueryParams) -> Result<Vec<Repository>> { Ok(vec![]) }
    async fn get_repository(&self, _id: i32) -> Result<Option<Repository>> { Ok(None) }
    async fn create_repository(&self, repo: Repository) -> Result<Repository> { Ok(repo) }
    async fn update_repository(&self, repo: Repository) -> Result<Repository> { Ok(repo) }
    async fn delete_repository(&self, _id: i32) -> Result<bool> { Ok(true) }
}

struct MockEnvironmentManager;
#[async_trait]
impl EnvironmentManager for MockEnvironmentManager {
    async fn get_environments(&self, _project_id: i32, _params: RetrieveQueryParams) -> Result<Vec<Environment>> { Ok(vec![]) }
    async fn get_environment(&self, _id: i32) -> Result<Option<Environment>> { Ok(None) }
    async fn create_environment(&self, env: Environment) -> Result<Environment> { Ok(env) }
    async fn update_environment(&self, env: Environment) -> Result<Environment> { Ok(env) }
    async fn delete_environment(&self, _id: i32) -> Result<bool> { Ok(true) }
}

struct MockAccessKeyManager;
#[async_trait]
impl AccessKeyManager for MockAccessKeyManager {
    async fn get_access_keys(&self, _project_id: i32, _params: RetrieveQueryParams) -> Result<Vec<AccessKey>> { Ok(vec![]) }
    async fn get_access_key(&self, _id: i32) -> Result<Option<AccessKey>> { Ok(None) }
    async fn create_access_key(&self, key: AccessKey) -> Result<AccessKey> { Ok(key) }
    async fn update_access_key(&self, key: AccessKey) -> Result<AccessKey> { Ok(key) }
    async fn delete_access_key(&self, _id: i32) -> Result<bool> { Ok(true) }
}

struct MockTaskManager {
    tasks: RwLock<HashMap<i32, Task>>,
}
#[async_trait]
impl TaskManager for MockTaskManager {
    async fn get_tasks(&self, _project_id: i32, _params: RetrieveQueryParams) -> Result<Vec<Task>> {
        Ok(self.tasks.read().unwrap().values().cloned().collect())
    }
    async fn get_task(&self, id: i32) -> Result<Option<Task>> {
        Ok(self.tasks.read().unwrap().get(&id).cloned())
    }
    async fn create_task(&self, task: Task) -> Result<Task> {
        self.tasks.write().unwrap().insert(task.id, task.clone());
        Ok(task)
    }
    async fn update_task(&self, task: Task) -> Result<Task> {
        self.tasks.write().unwrap().insert(task.id, task.clone());
        Ok(task)
    }
    async fn delete_task(&self, _id: i32) -> Result<bool> { Ok(true) }
    async fn get_task_for_template(&self, _template_id: i32, _limit: i64) -> Result<Vec<Task>> { Ok(vec![]) }
}

struct MockScheduleManager;
#[async_trait]
impl ScheduleManager for MockScheduleManager {
    async fn get_schedules(&self, _project_id: i32, _params: RetrieveQueryParams) -> Result<Vec<Schedule>> { Ok(vec![]) }
    async fn get_schedule(&self, _id: i32) -> Result<Option<Schedule>> { Ok(None) }
    async fn create_schedule(&self, sched: Schedule) -> Result<Schedule> { Ok(sched) }
    async fn update_schedule(&self, sched: Schedule) -> Result<Schedule> { Ok(sched) }
    async fn delete_schedule(&self, _id: i32) -> Result<bool> { Ok(true) }
}

struct MockSessionManager;
#[async_trait]
impl SessionManager for MockSessionManager {
    async fn get_session(&self, _id: i32) -> Result<Option<Session>> { Ok(None) }
    async fn create_session(&self, sess: Session) -> Result<Session> { Ok(sess) }
    async fn delete_session(&self, _id: i32) -> Result<bool> { Ok(true) }
    async fn expire_sessions(&self) -> Result<()> { Ok(()) }
}

struct MockTokenManager;
#[async_trait]
impl TokenManager for MockTokenManager {
    async fn get_token(&self, _id: i32) -> Result<Option<APIToken>> { Ok(None) }
    async fn get_token_by_key(&self, _key: &str) -> Result<Option<APIToken>> { Ok(None) }
    async fn create_token(&self, token: APIToken) -> Result<APIToken> { Ok(token) }
    async fn delete_token(&self, _id: i32) -> Result<bool> { Ok(true) }
}

struct MockEventManager;
#[async_trait]
impl EventManager for MockEventManager {
    async fn get_events(&self, _project_id: i32, _params: RetrieveQueryParams) -> Result<Vec<Event>> { Ok(vec![]) }
    async fn create_event(&self, event: Event) -> Result<Event> { Ok(event) }
}

struct MockRunnerManager;
#[async_trait]
impl RunnerManager for MockRunnerManager {
    async fn get_runners(&self, _params: RetrieveQueryParams) -> Result<Vec<Runner>> { Ok(vec![]) }
    async fn get_runner(&self, _id: i32) -> Result<Option<Runner>> { Ok(None) }
    async fn create_runner(&self, runner: Runner) -> Result<Runner> { Ok(runner) }
    async fn update_runner(&self, runner: Runner) -> Result<Runner> { Ok(runner) }
    async fn delete_runner(&self, _id: i32) -> Result<bool> { Ok(true) }
}

struct MockViewManager;
#[async_trait]
impl ViewManager for MockViewManager {
    async fn get_views(&self, _project_id: i32, _params: RetrieveQueryParams) -> Result<Vec<View>> { Ok(vec![]) }
    async fn get_view(&self, _id: i32) -> Result<Option<View>> { Ok(None) }
    async fn create_view(&self, view: View) -> Result<View> { Ok(view) }
    async fn update_view(&self, view: View) -> Result<View> { Ok(view) }
    async fn delete_view(&self, _id: i32) -> Result<bool> { Ok(true) }
}

struct MockIntegrationManager;
#[async_trait]
impl IntegrationManager for MockIntegrationManager {
    async fn get_integrations(&self, _project_id: i32, _params: RetrieveQueryParams) -> Result<Vec<Integration>> { Ok(vec![]) }
    async fn get_integration(&self, _id: i32) -> Result<Option<Integration>> { Ok(None) }
    async fn create_integration(&self, int: Integration) -> Result<Integration> { Ok(int) }
    async fn update_integration(&self, int: Integration) -> Result<Integration> { Ok(int) }
    async fn delete_integration(&self, _id: i32) -> Result<bool> { Ok(true) }
}
