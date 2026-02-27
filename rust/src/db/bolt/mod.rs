//! BoltDB-хранилище (ключ-значение на базе sled)
//!
//! Это реализация хранилища данных, совместимая с оригинальной BoltDB-версией Semaphore.

mod bolt_db;
mod event;
mod user;
mod project_invite;
mod task;
mod template;
mod project;
mod schedule;
mod session;
mod inventory_repository_environment;
mod access_key;
mod view_option;
mod environment;
mod inventory;
mod repository;
mod role;
mod option;
mod view;
mod integration;
mod secret_storage;
mod runner;
mod migration;
mod template_vault;
mod integration_alias;
mod global_runner;
mod public_alias;
mod migration_system;

use crate::db::store::*;
use crate::models::{User, Project, Task, TaskWithTpl, TaskOutput, TaskStage, Template, TemplateFilter, Inventory, Repository, Environment, AccessKey, Integration, Schedule, Session, APIToken, Event, Runner, View, Role, ProjectInvite, ProjectInviteWithUser, RetrieveQueryParams, ObjectReferrers, OptionItem, SecretStorage, Hook, GetAccessKeyOptions};
use crate::error::{Error, Result};
use async_trait::async_trait;
use std::collections::HashMap;

/// BoltDB-хранилище данных
pub struct BoltStore {
    db: sled::Db,
}

impl BoltStore {
    /// Создаёт новое BoltDB-хранилище
    pub fn new(path: &str) -> Result<Self> {
        let db = sled::open(path)
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;

        Ok(Self { db })
    }

    /// Сериализует объект в JSON
    fn serialize<T: serde::Serialize>(&self, obj: &T) -> Result<Vec<u8>> {
        serde_json::to_vec(obj).map_err(|e| Error::Json(e))
    }

    /// Десериализует объект из JSON
    #[allow(dead_code)]
    fn deserialize<T: serde::de::DeserializeOwned>(&self, bytes: &[u8]) -> Result<T> {
        serde_json::from_slice(bytes).map_err(|e| Error::Json(e))
    }
}

#[async_trait]
impl ConnectionManager for BoltStore {
    async fn connect(&self) -> Result<()> {
        // Уже подключено при создании
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        self.db.flush().map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;
        Ok(())
    }

    fn is_permanent(&self) -> bool {
        false // BoltDB не держит постоянное подключение
    }
}

#[async_trait]
impl MigrationManager for BoltStore {
    fn get_dialect(&self) -> &str {
        "bolt"
    }

    async fn is_initialized(&self) -> Result<bool> {
        let tree = self.db.open_tree("migration")
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;
        Ok(!tree.is_empty())
    }

    async fn apply_migration(&self, version: i64, name: String) -> Result<()> {
        let tree = self.db.open_tree("migration")
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;
        
        let key = version.to_be_bytes();
        let value = self.serialize(&name)?;
        
        tree.insert(key, value)
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;
        
        Ok(())
    }

    async fn is_migration_applied(&self, version: i64) -> Result<bool> {
        let tree = self.db.open_tree("migration")
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;
        
        let key = version.to_be_bytes();
        Ok(tree.contains_key(key)
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?)
    }
}

#[async_trait]
impl OptionsManager for BoltStore {
    async fn get_options(&self) -> Result<HashMap<String, String>> {
        let tree = self.db.open_tree("options")
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;
        
        let mut options = HashMap::new();
        for item in tree.iter() {
            let (key, value) = item
                .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;
            
            let key_str = String::from_utf8_lossy(&key).to_string();
            let value_str = String::from_utf8_lossy(&value).to_string();
            options.insert(key_str, value_str);
        }
        
        Ok(options)
    }

    async fn get_option(&self, key: &str) -> Result<Option<String>> {
        let tree = self.db.open_tree("options")
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;
        
        if let Some(value) = tree.get(key.as_bytes())
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))? {
            Ok(Some(String::from_utf8_lossy(&value).to_string()))
        } else {
            Ok(None)
        }
    }

    async fn set_option(&self, key: &str, value: &str) -> Result<()> {
        let tree = self.db.open_tree("options")
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;
        
        tree.insert(key.as_bytes(), value.as_bytes())
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;
        
        Ok(())
    }

    async fn delete_option(&self, key: &str) -> Result<()> {
        let tree = self.db.open_tree("options")
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;
        
        tree.remove(key.as_bytes())
            .map_err(|e| Error::Database(sqlx::Error::Protocol(e.to_string())))?;
        
        Ok(())
    }
}

// Заглушки для остальных трейтов (реализация аналогична SQL)
#[async_trait]
impl UserManager for BoltStore {
    async fn get_users(&self, params: RetrieveQueryParams) -> Result<Vec<User>> {
        self.get_users(params).await
    }

    async fn get_user(&self, user_id: i32) -> Result<User> {
        self.get_user(user_id).await
    }

    async fn get_user_by_login_or_email(&self, login: &str, email: &str) -> Result<User> {
        self.get_user_by_login_or_email(login, email).await
    }

    async fn create_user(&self, user: User, password: &str) -> Result<User> {
        self.create_user(user, password).await
    }

    async fn update_user(&self, user: User) -> Result<()> {
        self.update_user(user).await
    }

    async fn delete_user(&self, user_id: i32) -> Result<()> {
        self.delete_user(user_id).await
    }

    async fn set_user_password(&self, user_id: i32, password: &str) -> Result<()> {
        self.set_user_password(user_id, password).await
    }

    async fn get_all_admins(&self) -> Result<Vec<User>> {
        self.get_all_admins().await
    }

    async fn get_user_count(&self) -> Result<usize> {
        self.get_user_count().await
    }
}

#[async_trait]
impl ProjectStore for BoltStore {
    async fn get_projects(&self, user_id: Option<i32>) -> Result<Vec<Project>> {
        match user_id {
            Some(uid) => self.get_projects(uid).await,
            None => self.get_all_projects().await,
        }
    }

    async fn get_project(&self, project_id: i32) -> Result<Project> {
        self.get_project(project_id).await
    }

    async fn create_project(&self, project: Project) -> Result<Project> {
        self.create_project(project).await
    }

    async fn update_project(&self, project: Project) -> Result<()> {
        self.update_project(project).await
    }

    async fn delete_project(&self, project_id: i32) -> Result<()> {
        self.delete_project(project_id).await
    }
}

#[async_trait]
impl TemplateManager for BoltStore {
    async fn get_templates(&self, project_id: i32) -> Result<Vec<Template>> {
        let params = RetrieveQueryParams {
            offset: 0,
            count: 1000,
            filter: String::new(),
        };
        
        self.get_templates(project_id, TemplateFilter { view_id: None }, params).await
    }

    async fn get_template(&self, project_id: i32, template_id: i32) -> Result<Template> {
        self.get_template(project_id, template_id).await
    }

    async fn create_template(&self, template: Template) -> Result<Template> {
        self.create_template(template).await
    }

    async fn update_template(&self, template: Template) -> Result<()> {
        self.update_template(template).await
    }

    async fn delete_template(&self, project_id: i32, template_id: i32) -> Result<()> {
        self.delete_template(project_id, template_id).await
    }
}

#[async_trait]
impl InventoryManager for BoltStore {
    async fn get_inventories(&self, project_id: i32) -> Result<Vec<Inventory>> {
        let params = RetrieveQueryParams {
            offset: 0,
            count: 1000,
            filter: String::new(),
        };
        self.get_inventories(project_id, params, vec![]).await
    }

    async fn get_inventory(&self, project_id: i32, inventory_id: i32) -> Result<Inventory> {
        self.get_inventory(project_id, inventory_id).await
    }

    async fn create_inventory(&self, inventory: Inventory) -> Result<Inventory> {
        self.create_inventory(inventory).await
    }

    async fn update_inventory(&self, inventory: Inventory) -> Result<()> {
        self.update_inventory(inventory).await
    }

    async fn delete_inventory(&self, project_id: i32, inventory_id: i32) -> Result<()> {
        self.delete_inventory(project_id, inventory_id).await
    }
}

#[async_trait]
impl RepositoryManager for BoltStore {
    async fn get_repositories(&self, project_id: i32) -> Result<Vec<Repository>> {
        let params = RetrieveQueryParams {
            offset: 0,
            count: 1000,
            filter: String::new(),
        };
        self.get_repositories(project_id, params).await
    }

    async fn get_repository(&self, project_id: i32, repository_id: i32) -> Result<Repository> {
        self.get_repository(project_id, repository_id).await
    }

    async fn create_repository(&self, repository: Repository) -> Result<Repository> {
        self.create_repository(repository).await
    }

    async fn update_repository(&self, repository: Repository) -> Result<()> {
        self.update_repository(repository).await
    }

    async fn delete_repository(&self, project_id: i32, repository_id: i32) -> Result<()> {
        self.delete_repository(project_id, repository_id).await
    }
}

#[async_trait]
impl EnvironmentManager for BoltStore {
    async fn get_environments(&self, project_id: i32) -> Result<Vec<Environment>> {
        let params = RetrieveQueryParams {
            offset: 0,
            count: 1000,
            filter: String::new(),
        };
        self.get_environments(project_id, params).await
    }

    async fn get_environment(&self, project_id: i32, environment_id: i32) -> Result<Environment> {
        self.get_environment(project_id, environment_id).await
    }

    async fn create_environment(&self, environment: Environment) -> Result<Environment> {
        self.create_environment(environment).await
    }

    async fn update_environment(&self, environment: Environment) -> Result<()> {
        self.update_environment(environment).await
    }

    async fn delete_environment(&self, project_id: i32, environment_id: i32) -> Result<()> {
        self.delete_environment(project_id, environment_id).await
    }
}

#[async_trait]
impl AccessKeyManager for BoltStore {
    async fn get_access_keys(&self, project_id: i32, options: GetAccessKeyOptions) -> Result<Vec<AccessKey>> {
        let params = RetrieveQueryParams {
            offset: 0,
            count: 1000,
            filter: String::new(),
        };
        self.get_access_keys(project_id, options, params).await
    }

    async fn get_access_key(&self, project_id: i32, access_key_id: i32) -> Result<AccessKey> {
        self.get_access_key(project_id, access_key_id).await
    }

    async fn create_access_key(&self, key: AccessKey) -> Result<AccessKey> {
        self.create_access_key(key).await
    }

    async fn update_access_key(&self, key: AccessKey) -> Result<()> {
        self.update_access_key(key).await
    }

    async fn delete_access_key(&self, project_id: i32, access_key_id: i32) -> Result<()> {
        self.delete_access_key(project_id, access_key_id).await
    }
}

#[async_trait]
impl TaskManager for BoltStore {
    async fn get_tasks(&self, project_id: i32, template_id: Option<i32>) -> Result<Vec<TaskWithTpl>> {
        let params = RetrieveQueryParams {
            offset: 0,
            count: 1000,
            filter: String::new(),
        };
        
        match template_id {
            Some(tid) => self.get_template_tasks(project_id, tid, params).await,
            None => self.get_project_tasks(project_id, params).await,
        }
    }

    async fn get_task(&self, project_id: i32, task_id: i32) -> Result<Task> {
        self.get_task(project_id, task_id).await
    }

    async fn create_task(&self, task: Task) -> Result<Task> {
        self.create_task(task, 0).await
    }

    async fn update_task(&self, task: Task) -> Result<()> {
        self.update_task(task).await
    }

    async fn delete_task(&self, project_id: i32, task_id: i32) -> Result<()> {
        self.delete_task_with_outputs(project_id, task_id).await
    }

    async fn get_task_outputs(&self, task_id: i32) -> Result<Vec<TaskOutput>> {
        // Получаем project_id из задачи
        // Для упрощения используем заглушку
        Ok(vec![])
    }

    async fn create_task_output(&self, output: TaskOutput) -> Result<TaskOutput> {
        self.create_task_output(output).await
    }
}

#[async_trait]
impl ScheduleManager for BoltStore {
    async fn get_schedules(&self, _project_id: i32) -> Result<Vec<Schedule>> {
        self.get_schedules().await
    }

    async fn get_schedule(&self, project_id: i32, schedule_id: i32) -> Result<Schedule> {
        self.get_schedule(project_id, schedule_id).await
    }

    async fn create_schedule(&self, schedule: Schedule) -> Result<Schedule> {
        self.create_schedule(schedule).await
    }

    async fn update_schedule(&self, schedule: Schedule) -> Result<()> {
        self.update_schedule(schedule).await
    }

    async fn delete_schedule(&self, project_id: i32, schedule_id: i32) -> Result<()> {
        self.delete_schedule(project_id, schedule_id).await
    }

    async fn set_schedule_active(&self, project_id: i32, schedule_id: i32, active: bool) -> Result<()> {
        self.set_schedule_active(project_id, schedule_id, active).await
    }

    async fn set_schedule_commit_hash(&self, project_id: i32, schedule_id: i32, hash: &str) -> Result<()> {
        self.set_schedule_commit_hash(project_id, schedule_id, hash).await
    }
}

#[async_trait]
impl SessionManager for BoltStore {
    async fn get_session(&self, user_id: i32, session_id: i32) -> Result<Session> {
        self.get_session(user_id, session_id).await
    }

    async fn create_session(&self, session: Session) -> Result<Session> {
        self.create_session(session).await
    }

    async fn expire_session(&self, user_id: i32, session_id: i32) -> Result<()> {
        self.expire_session(user_id, session_id).await
    }

    async fn verify_session(&self, user_id: i32, session_id: i32) -> Result<()> {
        self.verify_session(user_id, session_id).await
    }

    async fn touch_session(&self, user_id: i32, session_id: i32) -> Result<()> {
        self.touch_session(user_id, session_id).await
    }
}

#[async_trait]
impl TokenManager for BoltStore {
    async fn get_api_tokens(&self, user_id: i32) -> Result<Vec<APIToken>> {
        self.get_api_tokens(user_id).await
    }

    async fn create_api_token(&self, token: APIToken) -> Result<APIToken> {
        self.create_api_token(token).await
    }

    async fn get_api_token(&self, token_id: &str) -> Result<APIToken> {
        self.get_api_token(token_id).await
    }

    async fn expire_api_token(&self, user_id: i32, token_id: &str) -> Result<()> {
        self.expire_api_token(user_id, token_id).await
    }

    async fn delete_api_token(&self, user_id: i32, token_id: &str) -> Result<()> {
        self.delete_api_token(user_id, token_id).await
    }
}

#[async_trait]
impl EventManager for BoltStore {
    async fn get_events(&self, project_id: Option<i32>, limit: usize) -> Result<Vec<Event>> {
        let params = RetrieveQueryParams {
            offset: 0,
            count: limit,
            filter: String::new(),
        };
        
        match project_id {
            Some(pid) => self.get_events(pid, params).await,
            None => self.get_all_events(params).await,
        }
    }

    async fn create_event(&self, event: Event) -> Result<Event> {
        self.create_event(event).await
    }
}

#[async_trait]
impl RunnerManager for BoltStore {
    async fn get_runners(&self, _project_id: Option<i32>) -> Result<Vec<Runner>> {
        Ok(vec![])
    }

    async fn get_runner(&self, _runner_id: i32) -> Result<Runner> {
        Err(Error::NotFound("Раннер не найден".to_string()))
    }

    async fn create_runner(&self, _runner: Runner) -> Result<Runner> {
        Err(Error::Other("Не реализовано".to_string()))
    }

    async fn update_runner(&self, _runner: Runner) -> Result<()> {
        Err(Error::Other("Не реализовано".to_string()))
    }

    async fn delete_runner(&self, _runner_id: i32) -> Result<()> {
        Err(Error::Other("Не реализовано".to_string()))
    }
}

#[async_trait]
impl ViewManager for BoltStore {
    async fn get_views(&self, project_id: i32) -> Result<Vec<View>> {
        self.get_views(project_id).await
    }

    async fn get_view(&self, project_id: i32, view_id: i32) -> Result<View> {
        self.get_view(project_id, view_id).await
    }

    async fn create_view(&self, view: View) -> Result<View> {
        self.create_view(view).await
    }

    async fn update_view(&self, view: View) -> Result<()> {
        self.update_view(view).await
    }

    async fn delete_view(&self, project_id: i32, view_id: i32) -> Result<()> {
        self.delete_view(project_id, view_id).await
    }
}

#[async_trait]
impl OptionsManager for BoltStore {
    async fn get_options(&self) -> Result<HashMap<String, String>> {
        let params = RetrieveQueryParams {
            offset: 0,
            count: 1000,
            filter: String::new(),
        };
        self.get_options(params).await
    }

    async fn get_option(&self, key: &str) -> Result<Option<String>> {
        match self.get_option(key).await {
            Ok(value) => Ok(Some(value)),
            Err(Error::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn set_option(&self, key: &str, value: &str) -> Result<()> {
        self.set_option(key, value).await
    }

    async fn delete_option(&self, key: &str) -> Result<()> {
        self.delete_option(key).await
    }
}

#[async_trait]
impl IntegrationManager for BoltStore {
    async fn get_integrations(&self, _project_id: i32) -> Result<Vec<Integration>> {
        Ok(vec![])
    }

    async fn get_integration(&self, _project_id: i32, _integration_id: i32) -> Result<Integration> {
        Err(Error::NotFound("Интеграция не найдена".to_string()))
    }

    async fn create_integration(&self, _integration: Integration) -> Result<Integration> {
        Err(Error::Other("Не реализовано".to_string()))
    }

    async fn update_integration(&self, _integration: Integration) -> Result<()> {
        Err(Error::Other("Не реализовано".to_string()))
    }

    async fn delete_integration(&self, _project_id: i32, _integration_id: i32) -> Result<()> {
        Err(Error::Other("Не реализовано".to_string()))
    }
}

#[async_trait]
impl ProjectInviteManager for BoltStore {
    async fn get_project_invites(&self, project_id: i32, params: RetrieveQueryParams) -> Result<Vec<ProjectInviteWithUser>> {
        self.get_project_invites(project_id, params).await
    }

    async fn create_project_invite(&self, invite: ProjectInvite) -> Result<ProjectInvite> {
        self.create_project_invite(invite).await
    }

    async fn get_project_invite(&self, project_id: i32, invite_id: i32) -> Result<ProjectInvite> {
        self.get_project_invite(project_id, invite_id).await
    }

    async fn get_project_invite_by_token(&self, token: &str) -> Result<ProjectInvite> {
        self.get_project_invite_by_token(token).await
    }

    async fn update_project_invite(&self, invite: ProjectInvite) -> Result<()> {
        self.update_project_invite(invite).await
    }

    async fn delete_project_invite(&self, project_id: i32, invite_id: i32) -> Result<()> {
        self.delete_project_invite(project_id, invite_id).await
    }
}

#[async_trait]
impl Store for BoltStore {}
