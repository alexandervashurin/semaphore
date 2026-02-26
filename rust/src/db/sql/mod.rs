//! SQL-хранилище (SQLite)

pub mod runner;
pub mod types;
pub mod init;
pub mod migrations;
pub mod queries;
pub mod utils;
pub mod template_crud;
pub mod template_vault;
pub mod template_roles;
pub mod template_utils;

use crate::db::store::*;
use crate::models::*;
use crate::error::{Error, Result};
use async_trait::async_trait;
use sqlx::{SqlitePool, Row};
use std::collections::HashMap;

/// SQL-хранилище данных (на базе SQLite)
pub struct SqlStore {
    pool: SqlitePool,
}

impl SqlStore {
    /// Создаёт новое SQL-хранилище
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url)
            .await
            .map_err(|e| Error::Database(e))?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl ConnectionManager for SqlStore {
    async fn connect(&self) -> Result<()> {
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }

    fn is_permanent(&self) -> bool {
        true
    }
}

#[async_trait]
impl MigrationManager for SqlStore {
    fn get_dialect(&self) -> &str {
        "sqlite"
    }

    async fn is_initialized(&self) -> Result<bool> {
        let query = "SELECT name FROM sqlite_master WHERE type='table' AND name='migration'";
        let result = sqlx::query(query)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        Ok(result.is_some())
    }

    async fn apply_migration(&self, version: i64, name: String) -> Result<()> {
        let query = "INSERT INTO migration (version, name) VALUES (?, ?)";
        sqlx::query(query)
            .bind(version)
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        Ok(())
    }

    async fn is_migration_applied(&self, version: i64) -> Result<bool> {
        let query = "SELECT COUNT(*) FROM migration WHERE version = ?";
        let count: i64 = sqlx::query_scalar(query)
            .bind(version)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        Ok(count > 0)
    }
}

#[async_trait]
impl OptionsManager for SqlStore {
    async fn get_options(&self) -> Result<HashMap<String, String>> {
        let query = "SELECT key, value FROM option";
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(rows.into_iter().map(|row| {
            let key: String = row.get("key");
            let value: String = row.get("value");
            (key, value)
        }).collect())
    }

    async fn get_option(&self, key: &str) -> Result<Option<String>> {
        let query = "SELECT value FROM option WHERE key = ?";
        let result = sqlx::query_scalar::<_, String>(query)
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        Ok(result)
    }

    async fn set_option(&self, key: &str, value: &str) -> Result<()> {
        let query = "INSERT OR REPLACE INTO option (key, value) VALUES (?, ?)";
        sqlx::query(query)
            .bind(key)
            .bind(value)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        Ok(())
    }

    async fn delete_option(&self, key: &str) -> Result<()> {
        let query = "DELETE FROM option WHERE key = ?";
        sqlx::query(query)
            .bind(key)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        Ok(())
    }
}

#[async_trait]
impl UserManager for SqlStore {
    async fn get_users(&self, _params: RetrieveQueryParams) -> Result<Vec<User>> {
        let query = "SELECT * FROM user ORDER BY id";
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(rows.into_iter().map(|row| User {
            id: row.get("id"),
            created: row.get("created"),
            username: row.get("username"),
            name: row.get("name"),
            email: row.get("email"),
            password: row.get("password"),
            admin: row.get("admin"),
            external: row.get("external"),
            alert: row.get("alert"),
            pro: row.get("pro"),
            totp: None,
            email_otp: None,
        }).collect())
    }

    async fn get_user(&self, user_id: i32) -> Result<User> {
        let query = "SELECT * FROM user WHERE id = ?";
        let row = sqlx::query(query)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound("Пользователь не найден".to_string()),
                _ => Error::Database(e),
            })?;
        
        Ok(User {
            id: row.get("id"),
            created: row.get("created"),
            username: row.get("username"),
            name: row.get("name"),
            email: row.get("email"),
            password: row.get("password"),
            admin: row.get("admin"),
            external: row.get("external"),
            alert: row.get("alert"),
            pro: row.get("pro"),
            totp: None,
            email_otp: None,
        })
    }

    async fn get_user_by_login_or_email(&self, login: &str, email: &str) -> Result<User> {
        let query = "SELECT * FROM user WHERE username = ? OR email = ?";
        let row = sqlx::query(query)
            .bind(login)
            .bind(email)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound("Пользователь не найден".to_string()),
                _ => Error::Database(e),
            })?;
        
        Ok(User {
            id: row.get("id"),
            created: row.get("created"),
            username: row.get("username"),
            name: row.get("name"),
            email: row.get("email"),
            password: row.get("password"),
            admin: row.get("admin"),
            external: row.get("external"),
            alert: row.get("alert"),
            pro: row.get("pro"),
            totp: None,
            email_otp: None,
        })
    }

    async fn create_user(&self, user: User, password: &str) -> Result<User> {
        let query = "INSERT INTO user (username, name, email, password, admin, external, alert, pro, created) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";
        sqlx::query(query)
            .bind(&user.username)
            .bind(&user.name)
            .bind(&user.email)
            .bind(password)
            .bind(user.admin)
            .bind(user.external)
            .bind(user.alert)
            .bind(user.pro)
            .bind(user.created)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        self.get_user_by_login_or_email(&user.username, &user.email).await
    }

    async fn update_user(&self, user: User) -> Result<()> {
        let query = "UPDATE user SET username = ?, name = ?, email = ?, admin = ?, external = ?, alert = ?, pro = ? WHERE id = ?";
        sqlx::query(query)
            .bind(&user.username)
            .bind(&user.name)
            .bind(&user.email)
            .bind(user.admin)
            .bind(user.external)
            .bind(user.alert)
            .bind(user.pro)
            .bind(user.id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        Ok(())
    }

    async fn delete_user(&self, user_id: i32) -> Result<()> {
        let query = "DELETE FROM user WHERE id = ?";
        sqlx::query(query)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        Ok(())
    }

    async fn set_user_password(&self, user_id: i32, password: &str) -> Result<()> {
        let query = "UPDATE user SET password = ? WHERE id = ?";
        sqlx::query(query)
            .bind(password)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        Ok(())
    }

    async fn get_all_admins(&self) -> Result<Vec<User>> {
        let query = "SELECT * FROM user WHERE admin = 1";
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(rows.into_iter().map(|row| User {
            id: row.get("id"),
            created: row.get("created"),
            username: row.get("username"),
            name: row.get("name"),
            email: row.get("email"),
            password: row.get("password"),
            admin: row.get("admin"),
            external: row.get("external"),
            alert: row.get("alert"),
            pro: row.get("pro"),
            totp: None,
            email_otp: None,
        }).collect())
    }

    async fn get_user_count(&self) -> Result<usize> {
        let query = "SELECT COUNT(*) FROM user";
        let count: i64 = sqlx::query_scalar(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        Ok(count as usize)
    }
}

#[async_trait]
impl ProjectStore for SqlStore {
    async fn get_projects(&self, user_id: Option<i32>) -> Result<Vec<Project>> {
        let (query, bind_user_id) = if let Some(uid) = user_id {
            ("SELECT p.* FROM project p JOIN project__user pu ON p.id = pu.project_id WHERE pu.user_id = ?", Some(uid))
        } else {
            ("SELECT * FROM project", None)
        };

        let mut q = sqlx::query(query);
        if let Some(uid) = bind_user_id {
            q = q.bind(uid);
        }

        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        Ok(rows.into_iter().map(|row| Project {
            id: row.get("id"),
            created: row.get("created"),
            name: row.get("name"),
            alert: row.get("alert"),
            alert_chat: row.get("alert_chat"),
            max_parallel_tasks: row.get("max_parallel_tasks"),
            r#type: row.get("type"),
            default_secret_storage_id: row.get("default_secret_storage_id"),
        }).collect())
    }

    async fn get_project(&self, project_id: i32) -> Result<Project> {
        let query = "SELECT * FROM project WHERE id = ?";
        let row = sqlx::query(query)
            .bind(project_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound("Проект не найден".to_string()),
                _ => Error::Database(e),
            })?;
        
        Ok(Project {
            id: row.get("id"),
            created: row.get("created"),
            name: row.get("name"),
            alert: row.get("alert"),
            alert_chat: row.get("alert_chat"),
            max_parallel_tasks: row.get("max_parallel_tasks"),
            r#type: row.get("type"),
            default_secret_storage_id: row.get("default_secret_storage_id"),
        })
    }

    async fn create_project(&self, mut project: Project) -> Result<Project> {
        let query = "INSERT INTO project (name, created, alert, alert_chat, max_parallel_tasks, type, default_secret_storage_id) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id";
        let id: i32 = sqlx::query_scalar(query)
            .bind(&project.name)
            .bind(project.created)
            .bind(project.alert)
            .bind(&project.alert_chat)
            .bind(project.max_parallel_tasks)
            .bind(&project.r#type)
            .bind(&project.default_secret_storage_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        
        project.id = id;
        Ok(project)
    }

    async fn update_project(&self, project: Project) -> Result<()> {
        let query = "UPDATE project SET name = ?, alert = ?, alert_chat = ?, max_parallel_tasks = ?, type = ?, default_secret_storage_id = ? WHERE id = ?";
        sqlx::query(query)
            .bind(&project.name)
            .bind(project.alert)
            .bind(&project.alert_chat)
            .bind(project.max_parallel_tasks)
            .bind(&project.r#type)
            .bind(&project.default_secret_storage_id)
            .bind(project.id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        Ok(())
    }

    async fn delete_project(&self, project_id: i32) -> Result<()> {
        let query = "DELETE FROM project WHERE id = ?";
        sqlx::query(query)
            .bind(project_id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e))?;
        Ok(())
    }
}

// Заглушки для остальных трейтов
#[async_trait]
impl TemplateManager for SqlStore {
    async fn get_templates(&self, _project_id: i32) -> Result<Vec<Template>> {
        Ok(vec![])
    }
    async fn get_template(&self, _project_id: i32, _template_id: i32) -> Result<Template> {
        Err(Error::NotFound("Шаблон не найден".to_string()))
    }
    async fn create_template(&self, _template: Template) -> Result<Template> {
        Err(Error::Other("Не реализовано".to_string()))
    }
    async fn update_template(&self, _template: Template) -> Result<()> {
        Err(Error::Other("Не реализовано".to_string()))
    }
    async fn delete_template(&self, _project_id: i32, _template_id: i32) -> Result<()> {
        Err(Error::Other("Не реализовано".to_string()))
    }
}

#[async_trait]
impl InventoryManager for SqlStore {
    async fn get_inventories(&self, _project_id: i32) -> Result<Vec<Inventory>> { Ok(vec![]) }
    async fn get_inventory(&self, _project_id: i32, _inventory_id: i32) -> Result<Inventory> { Err(Error::NotFound("Инвентарь не найден".to_string())) }
    async fn create_inventory(&self, _inventory: Inventory) -> Result<Inventory> { Err(Error::Other("Не реализовано".to_string())) }
    async fn update_inventory(&self, _inventory: Inventory) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn delete_inventory(&self, _project_id: i32, _inventory_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl RepositoryManager for SqlStore {
    async fn get_repositories(&self, _project_id: i32) -> Result<Vec<Repository>> { Ok(vec![]) }
    async fn get_repository(&self, _project_id: i32, _repository_id: i32) -> Result<Repository> { Err(Error::NotFound("Репозиторий не найден".to_string())) }
    async fn create_repository(&self, _repository: Repository) -> Result<Repository> { Err(Error::Other("Не реализовано".to_string())) }
    async fn update_repository(&self, _repository: Repository) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn delete_repository(&self, _project_id: i32, _repository_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl EnvironmentManager for SqlStore {
    async fn get_environments(&self, _project_id: i32) -> Result<Vec<Environment>> { Ok(vec![]) }
    async fn get_environment(&self, _project_id: i32, _environment_id: i32) -> Result<Environment> { Err(Error::NotFound("Окружение не найдено".to_string())) }
    async fn create_environment(&self, _environment: Environment) -> Result<Environment> { Err(Error::Other("Не реализовано".to_string())) }
    async fn update_environment(&self, _environment: Environment) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn delete_environment(&self, _project_id: i32, _environment_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl AccessKeyManager for SqlStore {
    async fn get_access_keys(&self, _project_id: i32) -> Result<Vec<AccessKey>> { Ok(vec![]) }
    async fn get_access_key(&self, _project_id: i32, _key_id: i32) -> Result<AccessKey> { Err(Error::NotFound("Ключ доступа не найден".to_string())) }
    async fn create_access_key(&self, _key: AccessKey) -> Result<AccessKey> { Err(Error::Other("Не реализовано".to_string())) }
    async fn update_access_key(&self, _key: AccessKey) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn delete_access_key(&self, _project_id: i32, _key_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl TaskManager for SqlStore {
    async fn get_tasks(&self, _project_id: i32, _template_id: Option<i32>) -> Result<Vec<TaskWithTpl>> { Ok(vec![]) }
    async fn get_task(&self, _project_id: i32, _task_id: i32) -> Result<Task> { Err(Error::NotFound("Задача не найдена".to_string())) }
    async fn create_task(&self, _task: Task) -> Result<Task> { Err(Error::Other("Не реализовано".to_string())) }
    async fn update_task(&self, _task: Task) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn delete_task(&self, _project_id: i32, _task_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn get_task_outputs(&self, _task_id: i32) -> Result<Vec<TaskOutput>> { Ok(vec![]) }
    async fn create_task_output(&self, _output: TaskOutput) -> Result<TaskOutput> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl ScheduleManager for SqlStore {
    async fn get_schedules(&self, _project_id: i32) -> Result<Vec<Schedule>> { Ok(vec![]) }
    async fn get_schedule(&self, _project_id: i32, _schedule_id: i32) -> Result<Schedule> { Err(Error::NotFound("Расписание не найдено".to_string())) }
    async fn create_schedule(&self, _schedule: Schedule) -> Result<Schedule> { Err(Error::Other("Не реализовано".to_string())) }
    async fn update_schedule(&self, _schedule: Schedule) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn delete_schedule(&self, _project_id: i32, _schedule_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl SessionManager for SqlStore {
    async fn get_session(&self, _user_id: i32, _session_id: i32) -> Result<Session> { Err(Error::NotFound("Сессия не найдена".to_string())) }
    async fn create_session(&self, _session: Session) -> Result<Session> { Err(Error::Other("Не реализовано".to_string())) }
    async fn expire_session(&self, _user_id: i32, _session_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl TokenManager for SqlStore {
    async fn get_api_tokens(&self, _user_id: i32) -> Result<Vec<APIToken>> { Ok(vec![]) }
    async fn create_api_token(&self, _token: APIToken) -> Result<APIToken> { Err(Error::Other("Не реализовано".to_string())) }
    async fn get_api_token(&self, _token_id: &str) -> Result<APIToken> { Err(Error::NotFound("Токен не найден".to_string())) }
    async fn expire_api_token(&self, _user_id: i32, _token_id: &str) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl EventManager for SqlStore {
    async fn get_events(&self, _project_id: Option<i32>, _limit: usize) -> Result<Vec<Event>> { Ok(vec![]) }
    async fn create_event(&self, _event: Event) -> Result<Event> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl RunnerManager for SqlStore {
    async fn get_runners(&self, _project_id: Option<i32>) -> Result<Vec<Runner>> { Ok(vec![]) }
    async fn get_runner(&self, _runner_id: i32) -> Result<Runner> { Err(Error::NotFound("Раннер не найден".to_string())) }
    async fn create_runner(&self, _runner: Runner) -> Result<Runner> { Err(Error::Other("Не реализовано".to_string())) }
    async fn update_runner(&self, _runner: Runner) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn delete_runner(&self, _runner_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl ViewManager for SqlStore {
    async fn get_views(&self, _project_id: i32) -> Result<Vec<View>> { Ok(vec![]) }
    async fn get_view(&self, _project_id: i32, _view_id: i32) -> Result<View> { Err(Error::NotFound("Представление не найдено".to_string())) }
    async fn create_view(&self, _view: View) -> Result<View> { Err(Error::Other("Не реализовано".to_string())) }
    async fn update_view(&self, _view: View) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn delete_view(&self, _project_id: i32, _view_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl IntegrationManager for SqlStore {
    async fn get_integrations(&self, _project_id: i32) -> Result<Vec<Integration>> { Ok(vec![]) }
    async fn get_integration(&self, _project_id: i32, _integration_id: i32) -> Result<Integration> { Err(Error::NotFound("Интеграция не найдена".to_string())) }
    async fn create_integration(&self, _integration: Integration) -> Result<Integration> { Err(Error::Other("Не реализовано".to_string())) }
    async fn update_integration(&self, _integration: Integration) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
    async fn delete_integration(&self, _project_id: i32, _integration_id: i32) -> Result<()> { Err(Error::Other("Не реализовано".to_string())) }
}

#[async_trait]
impl Store for SqlStore {}
