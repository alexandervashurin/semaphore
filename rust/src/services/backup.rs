//! Backup - экспорт и импорт проектов
//!
//! Аналог services/project/backup.go из Go версии

use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};

use crate::error::{Error, Result};
use crate::models::*;

/// BackupFormat - формат backup проекта
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFormat {
    #[serde(default = "default_backup_version")]
    pub version: String,
    #[serde(alias = "meta")]
    pub project: BackupProject,
    #[serde(default)]
    pub templates: Vec<BackupTemplate>,
    #[serde(default)]
    pub repositories: Vec<BackupRepository>,
    #[serde(default)]
    pub inventories: Vec<BackupInventory>,
    #[serde(default)]
    pub environments: Vec<BackupEnvironment>,
    #[serde(alias = "keys", default)]
    pub access_keys: Vec<BackupAccessKey>,
    #[serde(default)]
    pub schedules: Vec<BackupSchedule>,
    #[serde(default)]
    pub integrations: Vec<BackupIntegration>,
    #[serde(default)]
    pub views: Vec<BackupView>,
}

fn default_backup_version() -> String {
    "1.0".to_string()
}

/// BackupProject - информация о проекте
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupProject {
    pub name: String,
    pub alert: Option<bool>,
    pub alert_chat: Option<String>,
    pub max_parallel_tasks: Option<i32>,
}

/// BackupTemplate - шаблон для backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupTemplate {
    pub name: String,
    #[serde(default)]
    pub playbook: String,
    pub arguments: Option<String>,
    #[serde(alias = "type", default)]
    pub template_type: String,
    pub inventory: Option<String>,
    pub repository: Option<String>,
    pub environment: Option<String>,
    pub cron: Option<String>,
}

/// BackupRepository - репозиторий для backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRepository {
    pub name: String,
    #[serde(default)]
    pub git_url: String,
    #[serde(default)]
    pub git_branch: String,
    pub ssh_key: Option<String>,
}

/// BackupInventory - инвентарь для backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInventory {
    pub name: String,
    #[serde(alias = "type", default = "default_inventory_type")]
    pub inventory_type: String,
    #[serde(default)]
    pub inventory: String,
    pub ssh_key: Option<String>,
    pub become_key: Option<String>,
}

fn default_inventory_type() -> String {
    "static".to_string()
}

/// BackupEnvironment - окружение для backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEnvironment {
    pub name: String,
    pub json: String,
}

/// BackupAccessKey - ключ доступа для backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupAccessKey {
    pub name: String,
    #[serde(alias = "type", default = "default_key_type")]
    pub key_type: String,
    #[serde(default)]
    pub owner: String,
    pub ssh_key: Option<BackupSshKey>,
    pub login_password: Option<BackupLoginPassword>,
}

fn default_key_type() -> String {
    "none".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSshKey {
    pub private_key: String,
    pub passphrase: Option<String>,
    pub login: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupLoginPassword {
    pub login: String,
    pub password: String,
}

/// BackupSchedule - расписание для backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSchedule {
    pub template: String,
    pub cron_format: String,
    pub active: bool,
}

/// BackupIntegration - интеграция для backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupIntegration {
    pub name: String,
    pub template_id: Option<i32>,
}

/// BackupView - представление для backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupView {
    #[serde(alias = "title")]
    pub name: String,
    #[serde(default)]
    pub position: i32,
}

/// BackupDB - загрузчик backup из БД
#[derive(Default)]
pub struct BackupDB {
    templates: Vec<Template>,
    repositories: Vec<Repository>,
    inventories: Vec<Inventory>,
    environments: Vec<Environment>,
    access_keys: Vec<AccessKey>,
    schedules: Vec<Schedule>,
    integrations: Vec<Integration>,
    views: Vec<View>,
}

impl BackupDB {
    /// Создаёт новый BackupDB
    pub fn new() -> Self {
        Self::default()
    }

    /// Загружает данные из БД
    pub async fn load(&mut self, project_id: i32, store: &dyn crate::db::Store) -> Result<()> {
        self.templates = store.get_templates(project_id).await?;
        self.repositories = store.get_repositories(project_id).await?;
        self.inventories = store.get_inventories(project_id).await?;
        self.environments = store.get_environments(project_id).await?;
        self.access_keys = store.get_access_keys(project_id).await?;
        self.schedules = store.get_schedules(project_id).await?;
        self.integrations = store.get_integrations(project_id).await?;
        self.views = store.get_views(project_id).await?;

        Ok(())
    }

    /// Уникализирует имена
    pub fn make_unique_names(&mut self) {
        make_unique_names(
            &mut self.templates,
            |item| &item.name,
            |item, name| item.name = name,
        );
        make_unique_names(
            &mut self.repositories,
            |item| &item.name,
            |item, name| item.name = name,
        );
        make_unique_names(
            &mut self.inventories,
            |item| &item.name,
            |item, name| item.name = name,
        );
        make_unique_names(
            &mut self.environments,
            |item| &item.name,
            |item, name| item.name = name,
        );
        make_unique_names(
            &mut self.access_keys,
            |item| &item.name,
            |item, name| item.name = name,
        );
    }

    /// Форматирует backup
    pub fn format(&self, project: &Project) -> Result<BackupFormat> {
        let mut backup = BackupFormat {
            version: "1.0".to_string(),
            project: BackupProject {
                name: project.name.clone(),
                alert: Some(project.alert),
                alert_chat: project.alert_chat.clone(),
                max_parallel_tasks: Some(project.max_parallel_tasks),
            },
            templates: Vec::new(),
            repositories: Vec::new(),
            inventories: Vec::new(),
            environments: Vec::new(),
            access_keys: Vec::new(),
            schedules: Vec::new(),
            integrations: Vec::new(),
            views: Vec::new(),
        };

        // Создаём мапы для поиска
        let mut inventory_map = HashMap::new();
        for inv in &self.inventories {
            inventory_map.insert(inv.id, inv.name.clone());
        }

        let mut repository_map = HashMap::new();
        for repo in &self.repositories {
            repository_map.insert(repo.id, repo.name.clone());
        }

        let mut environment_map = HashMap::new();
        for env in &self.environments {
            environment_map.insert(env.id, env.name.clone());
        }

        let mut access_key_map = HashMap::new();
        for key in &self.access_keys {
            access_key_map.insert(key.id, key.name.clone());
        }

        // Конвертируем шаблоны
        for tpl in &self.templates {
            let schedule = get_schedule_by_template(tpl.id, &self.schedules);

            backup.templates.push(BackupTemplate {
                name: tpl.name.clone(),
                playbook: tpl.playbook.clone(),
                arguments: tpl.arguments.clone(),
                template_type: tpl.r#type.to_string(),
                inventory: tpl
                    .inventory_id
                    .and_then(|id| inventory_map.get(&id).cloned()),
                repository: tpl
                    .repository_id
                    .and_then(|id| repository_map.get(&id).cloned()),
                environment: tpl
                    .environment_id
                    .and_then(|id| environment_map.get(&id).cloned()),
                cron: schedule,
            });
        }

        // Конвертируем репозитории
        for repo in &self.repositories {
            backup.repositories.push(BackupRepository {
                name: repo.name.clone(),
                git_url: repo.git_url.clone(),
                git_branch: repo.git_branch.clone().unwrap_or_default(),
                ssh_key: repo.key_id.and_then(|id| access_key_map.get(&id).cloned()),
            });
        }

        // Конвертируем инвентари
        for inv in &self.inventories {
            backup.inventories.push(BackupInventory {
                name: inv.name.clone(),
                inventory_type: inv.inventory_type.to_string(),
                inventory: inv.inventory_data.clone(),
                ssh_key: inv
                    .ssh_key_id
                    .and_then(|id| access_key_map.get(&id).cloned()),
                become_key: inv
                    .become_key_id
                    .and_then(|id| access_key_map.get(&id).cloned()),
            });
        }

        // Конвертируем окружения
        for env in &self.environments {
            backup.environments.push(BackupEnvironment {
                name: env.name.clone(),
                json: env.json.clone(),
            });
        }

        // Конвертируем ключи доступа
        for key in &self.access_keys {
            let mut backup_key = BackupAccessKey {
                name: key.name.clone(),
                key_type: key.r#type.to_string(),
                owner: key
                    .owner
                    .as_ref()
                    .map(|o| o.to_string())
                    .unwrap_or_default(),
                ssh_key: None,
                login_password: None,
            };

            use crate::models::access_key::AccessKeyType;
            match key.r#type {
                AccessKeyType::SSH => {
                    if let Some(ref private_key) = key.ssh_key {
                        backup_key.ssh_key = Some(BackupSshKey {
                            private_key: private_key.clone(),
                            passphrase: key.ssh_passphrase.clone(),
                            login: key.login_password_login.clone(),
                        });
                    }
                }
                AccessKeyType::LoginPassword => {
                    if let Some(ref login) = key.login_password_login {
                        backup_key.login_password = Some(BackupLoginPassword {
                            login: login.clone(),
                            password: key.login_password_password.clone().unwrap_or_default(),
                        });
                    }
                }
                _ => {}
            }

            backup.access_keys.push(backup_key);
        }

        // Конвертируем расписания
        for schedule in &self.schedules {
            if let Some(tpl_name) = get_template_name_by_id(schedule.template_id, &self.templates) {
                backup.schedules.push(BackupSchedule {
                    template: tpl_name,
                    cron_format: schedule.cron_format.clone().unwrap_or_default(),
                    active: schedule.active,
                });
            }
        }

        // Конвертируем интеграции
        for integration in &self.integrations {
            backup.integrations.push(BackupIntegration {
                name: integration.name.clone(),
                template_id: Some(integration.template_id),
            });
        }

        // Конвертируем представления
        for view in &self.views {
            backup.views.push(BackupView {
                name: view.title.clone(),
                position: view.position,
            });
        }

        Ok(backup)
    }
}

/// Вспомогательная функция для поиска по slug
pub fn find_name_by_slug<T: BackupSluggedEntity>(slug: &str, items: &[T]) -> Option<String> {
    for item in items {
        if item.get_slug() == slug {
            return Some(item.get_name());
        }
    }
    None
}

/// Вспомогательная функция для поиска по ID
pub fn find_name_by_id<T: BackupEntity>(id: i32, items: &[T]) -> Option<String> {
    for item in items {
        if item.get_id() == id {
            return Some(item.get_name());
        }
    }
    None
}

/// Вспомогательная функция для поиска сущности по имени
pub fn find_entity_by_name<'a, T: BackupEntity>(name: &'a str, items: &'a [T]) -> Option<&'a T> {
    items
        .iter()
        .find(|&item| item.get_name() == name)
        .map(|v| v as _)
}

/// Получает расписания по проекту
pub fn get_schedules_by_project(project_id: i32, schedules: &[Schedule]) -> Vec<Schedule> {
    schedules
        .iter()
        .filter(|s| s.project_id == project_id)
        .cloned()
        .collect()
}

/// Получает cron формат по шаблону
pub fn get_schedule_by_template(template_id: i32, schedules: &[Schedule]) -> Option<String> {
    schedules
        .iter()
        .find(|s| s.template_id == template_id)
        .and_then(|s| s.cron_format.clone())
}

/// Генерирует случайное имя
pub fn get_random_name(name: &str) -> String {
    let mut rng = rand::thread_rng();
    let mut random_bytes = [0u8; 10];
    rng.fill_bytes(&mut random_bytes);
    format!("{} - {}", name, hex::encode(random_bytes))
}

/// Уникализирует имена
pub fn make_unique_names<T>(
    items: &mut [T],
    getter: impl Fn(&T) -> &String,
    setter: impl Fn(&mut T, String),
) {
    for i in (0..items.len()).rev() {
        for k in 0..i {
            let name = getter(&items[i]);
            if name == getter(&items[k]) {
                let random_name = get_random_name(name);
                setter(&mut items[i], random_name);
                break;
            }
        }
    }
}

/// Получает имя шаблона по ID
fn get_template_name_by_id(template_id: i32, templates: &[Template]) -> Option<String> {
    templates
        .iter()
        .find(|t| t.id == template_id)
        .map(|t| t.name.clone())
}

/// Trait для сущностей backup
pub trait BackupEntity {
    fn get_id(&self) -> i32;
    fn get_name(&self) -> String;
}

/// Trait для сущностей backup с slug
pub trait BackupSluggedEntity: BackupEntity {
    fn get_slug(&self) -> String;
}

// Реализация трейтов для моделей
impl BackupEntity for Template {
    fn get_id(&self) -> i32 {
        self.id
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl BackupEntity for Repository {
    fn get_id(&self) -> i32 {
        self.id
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl BackupEntity for Inventory {
    fn get_id(&self) -> i32 {
        self.id
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl BackupEntity for Environment {
    fn get_id(&self) -> i32 {
        self.id
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl BackupEntity for AccessKey {
    fn get_id(&self) -> i32 {
        self.id
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl BackupSluggedEntity for Repository {
    fn get_slug(&self) -> String {
        self.git_url.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_format_creation() {
        let backup = BackupFormat {
            version: "1.0".to_string(),
            project: BackupProject {
                name: "Test Project".to_string(),
                alert: Some(false),
                alert_chat: None,
                max_parallel_tasks: Some(5),
            },
            templates: Vec::new(),
            repositories: Vec::new(),
            inventories: Vec::new(),
            environments: Vec::new(),
            access_keys: Vec::new(),
            schedules: Vec::new(),
            integrations: Vec::new(),
            views: Vec::new(),
        };

        assert_eq!(backup.version, "1.0");
        assert_eq!(backup.project.name, "Test Project");
    }

    #[test]
    fn test_get_random_name() {
        let name = get_random_name("Test");
        assert!(name.starts_with("Test - "));
        assert!(name.len() > 10);
    }

    #[test]
    fn test_make_unique_names() {
        let mut items = vec![
            BackupTemplate {
                name: "Test".to_string(),
                playbook: String::new(),
                arguments: None,
                template_type: String::new(),
                inventory: None,
                repository: None,
                environment: None,
                cron: None,
            },
            BackupTemplate {
                name: "Test".to_string(),
                playbook: String::new(),
                arguments: None,
                template_type: String::new(),
                inventory: None,
                repository: None,
                environment: None,
                cron: None,
            },
        ];

        make_unique_names(&mut items, |item| &item.name, |item, name| item.name = name);

        assert_ne!(items[0].name, items[1].name);
    }

    #[test]
    fn test_get_random_name_uniqueness() {
        let name1 = get_random_name("Test");
        let name2 = get_random_name("Test");
        // Оба начинаются с "Test - ", но имеют разные случайные части
        assert!(name1.starts_with("Test - "));
        assert!(name2.starts_with("Test - "));
        assert_ne!(name1, name2);
    }

    #[test]
    fn test_make_unique_names_no_duplicates() {
        let mut items = vec![
            BackupTemplate {
                name: "Unique1".to_string(),
                playbook: String::new(),
                arguments: None,
                template_type: String::new(),
                inventory: None,
                repository: None,
                environment: None,
                cron: None,
            },
            BackupTemplate {
                name: "Unique2".to_string(),
                playbook: String::new(),
                arguments: None,
                template_type: String::new(),
                inventory: None,
                repository: None,
                environment: None,
                cron: None,
            },
        ];

        make_unique_names(&mut items, |item| &item.name, |item, name| item.name = name);

        assert_eq!(items[0].name, "Unique1");
        assert_eq!(items[1].name, "Unique2");
    }

    #[test]
    fn test_backup_project_serialization() {
        let project = BackupProject {
            name: "My Project".to_string(),
            alert: Some(true),
            alert_chat: Some("chat123".to_string()),
            max_parallel_tasks: Some(10),
        };
        let json = serde_json::to_string(&project).unwrap();
        assert!(json.contains("\"name\":\"My Project\""));
        assert!(json.contains("\"alert\":true"));
        assert!(json.contains("\"alert_chat\":\"chat123\""));
        assert!(json.contains("\"max_parallel_tasks\":10"));
    }

    #[test]
    fn test_backup_project_null_fields() {
        let project = BackupProject {
            name: "Minimal".to_string(),
            alert: None,
            alert_chat: None,
            max_parallel_tasks: None,
        };
        let json = serde_json::to_string(&project).unwrap();
        assert!(json.contains("\"alert\":null"));
        assert!(json.contains("\"alert_chat\":null"));
        assert!(json.contains("\"max_parallel_tasks\":null"));
    }

    #[test]
    fn test_backup_template_serialization() {
        let template = BackupTemplate {
            name: "Deploy".to_string(),
            playbook: "deploy.yml".to_string(),
            arguments: Some("--limit=web".to_string()),
            template_type: "ansible".to_string(),
            inventory: Some("Production".to_string()),
            repository: Some("main-repo".to_string()),
            environment: Some("prod-env".to_string()),
            cron: None,
        };
        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("\"name\":\"Deploy\""));
        assert!(json.contains("\"playbook\":\"deploy.yml\""));
        assert!(json.contains("\"arguments\":\"--limit=web\""));
        assert!(json.contains("\"inventory\":\"Production\""));
    }

    #[test]
    fn test_backup_repository_serialization() {
        let repo = BackupRepository {
            name: "My Repo".to_string(),
            git_url: "https://github.com/test/repo.git".to_string(),
            git_branch: "main".to_string(),
            ssh_key: None,
        };
        let json = serde_json::to_string(&repo).unwrap();
        assert!(json.contains("\"name\":\"My Repo\""));
        assert!(json.contains("\"git_url\":\"https://github.com/test/repo.git\""));
        assert!(json.contains("\"git_branch\":\"main\""));
    }

    #[test]
    fn test_backup_environment_serialization() {
        let env = BackupEnvironment {
            name: "Production".to_string(),
            json: r#"{"DB_HOST":"localhost"}"#.to_string(),
        };
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains("\"name\":\"Production\""));
        assert!(json.contains("\"json\":\""));
    }

    #[test]
    fn test_make_unique_names_three_duplicates() {
        let mut items = vec![
            BackupTemplate {
                name: "Same".to_string(),
                playbook: String::new(),
                arguments: None,
                template_type: String::new(),
                inventory: None,
                repository: None,
                environment: None,
                cron: None,
            },
            BackupTemplate {
                name: "Same".to_string(),
                playbook: String::new(),
                arguments: None,
                template_type: String::new(),
                inventory: None,
                repository: None,
                environment: None,
                cron: None,
            },
            BackupTemplate {
                name: "Same".to_string(),
                playbook: String::new(),
                arguments: None,
                template_type: String::new(),
                inventory: None,
                repository: None,
                environment: None,
                cron: None,
            },
        ];

        make_unique_names(&mut items, |item| &item.name, |item, name| item.name = name);

        // All names should be unique
        assert_ne!(items[0].name, items[1].name);
        assert_ne!(items[0].name, items[2].name);
        assert_ne!(items[1].name, items[2].name);
        // First item keeps original name
        assert_eq!(items[0].name, "Same");
    }

    #[test]
    fn test_default_backup_version() {
        assert_eq!(default_backup_version(), "1.0");
    }

    #[test]
    fn test_backup_format_clone() {
        let backup = BackupFormat {
            version: "1.0".to_string(),
            project: BackupProject {
                name: "Clone Test".to_string(),
                alert: Some(true),
                alert_chat: None,
                max_parallel_tasks: Some(5),
            },
            templates: Vec::new(),
            repositories: Vec::new(),
            inventories: Vec::new(),
            environments: Vec::new(),
            access_keys: Vec::new(),
            schedules: Vec::new(),
            integrations: Vec::new(),
            views: Vec::new(),
        };
        let cloned = backup.clone();
        assert_eq!(cloned.project.name, backup.project.name);
        assert_eq!(cloned.version, backup.version);
    }

    #[test]
    fn test_backup_schedule_serialization() {
        let schedule = BackupSchedule {
            template: "Deploy".to_string(),
            cron_format: "0 0 * * *".to_string(),
            active: true,
        };
        let json = serde_json::to_string(&schedule).unwrap();
        assert!(json.contains("\"template\":\"Deploy\""));
        assert!(json.contains("\"cron_format\":\"0 0 * * *\""));
        assert!(json.contains("\"active\":true"));
    }

    #[test]
    fn test_backup_integration_serialization() {
        let integration = BackupIntegration {
            name: "Slack".to_string(),
            template_id: Some(42),
        };
        let json = serde_json::to_string(&integration).unwrap();
        assert!(json.contains("\"name\":\"Slack\""));
        assert!(json.contains("\"template_id\":42"));
    }

    #[test]
    fn test_backup_view_serialization() {
        let view = BackupView {
            name: "Main View".to_string(),
            position: 1,
        };
        let json = serde_json::to_string(&view).unwrap();
        assert!(json.contains("\"name\":\"Main View\""));
        assert!(json.contains("\"position\":1"));
    }

    #[test]
    fn test_backup_view_with_title_alias() {
        let json = r#"{"title":"Alias View","position":5}"#;
        let view: BackupView = serde_json::from_str(json).unwrap();
        assert_eq!(view.name, "Alias View");
        assert_eq!(view.position, 5);
    }

    #[test]
    fn test_backup_ssh_key_serialization() {
        let ssh_key = BackupSshKey {
            private_key: "-----BEGIN RSA PRIVATE KEY-----".to_string(),
            passphrase: Some("secret".to_string()),
            login: Some("admin".to_string()),
        };
        let json = serde_json::to_string(&ssh_key).unwrap();
        assert!(json.contains("\"private_key\":\"-----BEGIN RSA PRIVATE KEY-----\""));
        assert!(json.contains("\"passphrase\":\"secret\""));
        assert!(json.contains("\"login\":\"admin\""));
    }

    #[test]
    fn test_backup_login_password_serialization() {
        let creds = BackupLoginPassword {
            login: "user".to_string(),
            password: "pass123".to_string(),
        };
        let json = serde_json::to_string(&creds).unwrap();
        assert!(json.contains("\"login\":\"user\""));
        assert!(json.contains("\"password\":\"pass123\""));
    }

    #[test]
    fn test_backup_access_key_ssh_type() {
        let key = BackupAccessKey {
            name: "SSH Key".to_string(),
            key_type: "ssh".to_string(),
            owner: "admin".to_string(),
            ssh_key: Some(BackupSshKey {
                private_key: "key_data".to_string(),
                passphrase: None,
                login: Some("deploy".to_string()),
            }),
            login_password: None,
        };
        let json = serde_json::to_string(&key).unwrap();
        assert!(json.contains("\"name\":\"SSH Key\""));
        assert!(json.contains("\"key_type\":\"ssh\""));
        assert!(json.contains("\"owner\":\"admin\""));
        assert!(json.contains("\"private_key\":\"key_data\""));
    }

    #[test]
    fn test_backup_access_key_login_password_type() {
        let key = BackupAccessKey {
            name: "Login Key".to_string(),
            key_type: "login_password".to_string(),
            owner: String::new(),
            ssh_key: None,
            login_password: Some(BackupLoginPassword {
                login: "root".to_string(),
                password: "secret".to_string(),
            }),
        };
        let json = serde_json::to_string(&key).unwrap();
        assert!(json.contains("\"name\":\"Login Key\""));
        assert!(json.contains("\"key_type\":\"login_password\""));
        assert!(json.contains("\"login\":\"root\""));
    }

    #[test]
    fn test_backup_access_key_none_type() {
        let key = BackupAccessKey {
            name: "None Key".to_string(),
            key_type: "none".to_string(),
            owner: String::new(),
            ssh_key: None,
            login_password: None,
        };
        assert_eq!(key.name, "None Key");
        assert_eq!(key.key_type, "none");
        assert!(key.ssh_key.is_none());
        assert!(key.login_password.is_none());
    }

    #[test]
    fn test_backup_inventory_serialization() {
        let inv = BackupInventory {
            name: "Production".to_string(),
            inventory_type: "static".to_string(),
            inventory: "[web]\n192.168.1.1".to_string(),
            ssh_key: None,
            become_key: None,
        };
        let json = serde_json::to_string(&inv).unwrap();
        assert!(json.contains("\"name\":\"Production\""));
        assert!(json.contains("\"inventory_type\":\"static\""));
        assert!(json.contains("\"inventory\":\"[web]\\n192.168.1.1\""));
    }

    #[test]
    fn test_backup_inventory_with_keys() {
        let inv = BackupInventory {
            name: "With Keys".to_string(),
            inventory_type: "file".to_string(),
            inventory: "/path/to/inventory".to_string(),
            ssh_key: Some("ssh-key-name".to_string()),
            become_key: Some("become-key-name".to_string()),
        };
        assert_eq!(inv.name, "With Keys");
        assert_eq!(inv.inventory_type, "file");
        assert_eq!(inv.ssh_key, Some("ssh-key-name".to_string()));
        assert_eq!(inv.become_key, Some("become-key-name".to_string()));
    }

    #[test]
    fn test_default_inventory_type() {
        assert_eq!(default_inventory_type(), "static");
    }

    #[test]
    fn test_default_key_type() {
        assert_eq!(default_key_type(), "none");
    }

    #[test]
    fn test_backup_format_serialization_roundtrip() {
        let backup = BackupFormat {
            version: "1.0".to_string(),
            project: BackupProject {
                name: "Roundtrip".to_string(),
                alert: Some(true),
                alert_chat: Some("chat".to_string()),
                max_parallel_tasks: Some(3),
            },
            templates: vec![BackupTemplate {
                name: "Tpl".to_string(),
                playbook: "play.yml".to_string(),
                arguments: None,
                template_type: "ansible".to_string(),
                inventory: None,
                repository: None,
                environment: None,
                cron: None,
            }],
            repositories: Vec::new(),
            inventories: Vec::new(),
            environments: Vec::new(),
            access_keys: Vec::new(),
            schedules: Vec::new(),
            integrations: Vec::new(),
            views: Vec::new(),
        };

        let json = serde_json::to_string(&backup).unwrap();
        let restored: BackupFormat = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.project.name, "Roundtrip");
        assert_eq!(restored.templates.len(), 1);
        assert_eq!(restored.templates[0].name, "Tpl");
        assert_eq!(restored.templates[0].playbook, "play.yml");
    }

    #[test]
    fn test_backup_format_empty_collections() {
        let backup = BackupFormat {
            version: "1.0".to_string(),
            project: BackupProject {
                name: "Empty".to_string(),
                alert: None,
                alert_chat: None,
                max_parallel_tasks: None,
            },
            templates: Vec::new(),
            repositories: Vec::new(),
            inventories: Vec::new(),
            environments: Vec::new(),
            access_keys: Vec::new(),
            schedules: Vec::new(),
            integrations: Vec::new(),
            views: Vec::new(),
        };

        assert!(backup.templates.is_empty());
        assert!(backup.repositories.is_empty());
        assert!(backup.inventories.is_empty());
        assert!(backup.environments.is_empty());
        assert!(backup.access_keys.is_empty());
        assert!(backup.schedules.is_empty());
        assert!(backup.integrations.is_empty());
        assert!(backup.views.is_empty());
    }

    #[test]
    fn test_backup_project_meta_alias() {
        let json = r#"{"meta":{"name":"Aliased"},"templates":[]}"#;
        let result: std::result::Result<BackupFormat, _> = serde_json::from_str(json);
        // meta - это alias для project, поэтому десериализация должна работать
        assert!(result.is_ok());
        let backup = result.unwrap();
        assert_eq!(backup.project.name, "Aliased");
    }

    #[test]
    fn test_backup_access_keys_alias() {
        let json = r#"{"project":{"name":"Test"},"keys":[],"templates":[]}"#;
        let backup: BackupFormat = serde_json::from_str(json).unwrap();
        assert_eq!(backup.project.name, "Test");
        assert!(backup.access_keys.is_empty());
    }

    #[test]
    fn test_make_unique_names_empty_slice() {
        let mut items: Vec<BackupTemplate> = Vec::new();
        make_unique_names(&mut items, |item| &item.name, |item, name| item.name = name);
        assert!(items.is_empty());
    }

    #[test]
    fn test_make_unique_names_single_item() {
        let mut items = vec![BackupTemplate {
            name: "Single".to_string(),
            playbook: String::new(),
            arguments: None,
            template_type: String::new(),
            inventory: None,
            repository: None,
            environment: None,
            cron: None,
        }];

        make_unique_names(&mut items, |item| &item.name, |item, name| item.name = name);

        assert_eq!(items[0].name, "Single");
    }

    #[test]
    fn test_backup_template_default_fields() {
        let template = BackupTemplate {
            name: "Minimal".to_string(),
            playbook: String::new(),
            arguments: None,
            template_type: String::new(),
            inventory: None,
            repository: None,
            environment: None,
            cron: None,
        };
        assert_eq!(template.playbook, "");
        assert!(template.arguments.is_none());
        assert!(template.inventory.is_none());
    }

    #[test]
    fn test_backup_repository_default_branch() {
        let repo = BackupRepository {
            name: "Test".to_string(),
            git_url: "https://example.com/repo.git".to_string(),
            git_branch: String::new(),
            ssh_key: None,
        };
        assert_eq!(repo.git_branch, "");
    }

    #[test]
    fn test_backup_schedule_inactive() {
        let schedule = BackupSchedule {
            template: "Old".to_string(),
            cron_format: "0 0 1 1 *".to_string(),
            active: false,
        };
        assert!(!schedule.active);
        assert_eq!(schedule.template, "Old");
    }

    #[test]
    fn test_backup_integration_null_template_id() {
        let integration = BackupIntegration {
            name: "Global".to_string(),
            template_id: None,
        };
        assert!(integration.template_id.is_none());
        assert_eq!(integration.name, "Global");
    }

    #[test]
    fn test_backup_view_default_position() {
        let json = r#"{"name":"NoPos"}"#;
        let view: BackupView = serde_json::from_str(json).unwrap();
        assert_eq!(view.name, "NoPos");
        assert_eq!(view.position, 0);
    }

    #[test]
    fn test_backup_format_version_default() {
        let json = r#"{"project":{"name":"Test"},"templates":[]}"#;
        let backup: BackupFormat = serde_json::from_str(json).unwrap();
        assert_eq!(backup.version, "1.0");
    }

    #[test]
    fn test_backup_debug_traits() {
        let project = BackupProject {
            name: "Debug".to_string(),
            alert: None,
            alert_chat: None,
            max_parallel_tasks: None,
        };
        let debug_str = format!("{:?}", project);
        assert!(debug_str.contains("Debug"));
        assert!(debug_str.contains("BackupProject"));
    }

    #[test]
    fn test_backup_multiple_templates_and_repos() {
        let backup = BackupFormat {
            version: "1.0".to_string(),
            project: BackupProject {
                name: "Multi".to_string(),
                alert: None,
                alert_chat: None,
                max_parallel_tasks: None,
            },
            templates: vec![
                BackupTemplate {
                    name: "Deploy".to_string(),
                    playbook: "deploy.yml".to_string(),
                    arguments: None,
                    template_type: String::new(),
                    inventory: None,
                    repository: None,
                    environment: None,
                    cron: None,
                },
                BackupTemplate {
                    name: "Test".to_string(),
                    playbook: "test.yml".to_string(),
                    arguments: None,
                    template_type: String::new(),
                    inventory: None,
                    repository: None,
                    environment: None,
                    cron: None,
                },
            ],
            repositories: vec![
                BackupRepository {
                    name: "Repo1".to_string(),
                    git_url: "https://github.com/repo1.git".to_string(),
                    git_branch: "main".to_string(),
                    ssh_key: None,
                },
                BackupRepository {
                    name: "Repo2".to_string(),
                    git_url: "https://github.com/repo2.git".to_string(),
                    git_branch: "develop".to_string(),
                    ssh_key: None,
                },
            ],
            inventories: Vec::new(),
            environments: Vec::new(),
            access_keys: Vec::new(),
            schedules: Vec::new(),
            integrations: Vec::new(),
            views: Vec::new(),
        };

        assert_eq!(backup.templates.len(), 2);
        assert_eq!(backup.repositories.len(), 2);
        assert_eq!(backup.templates[0].name, "Deploy");
        assert_eq!(backup.templates[1].name, "Test");
        assert_eq!(backup.repositories[0].name, "Repo1");
        assert_eq!(backup.repositories[1].name, "Repo2");
    }

    #[test]
    fn test_backup_format_deserialize_minimal() {
        let json = r#"{"project":{"name":"Minimal"},"templates":[]}"#;
        let backup: BackupFormat = serde_json::from_str(json).unwrap();
        assert_eq!(backup.version, "1.0");
        assert_eq!(backup.project.name, "Minimal");
        assert!(backup.templates.is_empty());
    }

    #[test]
    fn test_backup_project_clone() {
        let project = BackupProject {
            name: "CloneTest".to_string(),
            alert: Some(true),
            alert_chat: Some("chat".to_string()),
            max_parallel_tasks: Some(10),
        };
        let cloned = project.clone();
        assert_eq!(cloned.name, project.name);
        assert_eq!(cloned.alert, project.alert);
        assert_eq!(cloned.alert_chat, project.alert_chat);
        assert_eq!(cloned.max_parallel_tasks, project.max_parallel_tasks);
    }

    #[test]
    fn test_backup_template_clone() {
        let tpl = BackupTemplate {
            name: "Clone".to_string(),
            playbook: "clone.yml".to_string(),
            arguments: Some("--verbose".to_string()),
            template_type: "build".to_string(),
            inventory: Some("inv".to_string()),
            repository: Some("repo".to_string()),
            environment: Some("env".to_string()),
            cron: Some("0 5 * * *".to_string()),
        };
        let cloned = tpl.clone();
        assert_eq!(cloned.name, tpl.name);
        assert_eq!(cloned.playbook, tpl.playbook);
        assert_eq!(cloned.arguments, tpl.arguments);
        assert_eq!(cloned.cron, tpl.cron);
    }

    #[test]
    fn test_backup_repository_clone() {
        let repo = BackupRepository {
            name: "Repo".to_string(),
            git_url: "https://example.com/repo.git".to_string(),
            git_branch: "master".to_string(),
            ssh_key: Some("my-key".to_string()),
        };
        let cloned = repo.clone();
        assert_eq!(cloned.name, repo.name);
        assert_eq!(cloned.git_url, repo.git_url);
        assert_eq!(cloned.ssh_key, repo.ssh_key);
    }

    #[test]
    fn test_backup_environment_clone() {
        let env = BackupEnvironment {
            name: "Staging".to_string(),
            json: r#"{"KEY":"VALUE"}"#.to_string(),
        };
        let cloned = env.clone();
        assert_eq!(cloned.name, env.name);
        assert_eq!(cloned.json, env.json);
    }

    #[test]
    fn test_backup_access_key_clone() {
        let key = BackupAccessKey {
            name: "Key".to_string(),
            key_type: "ssh".to_string(),
            owner: "user".to_string(),
            ssh_key: Some(BackupSshKey {
                private_key: "key".to_string(),
                passphrase: Some("pass".to_string()),
                login: Some("admin".to_string()),
            }),
            login_password: None,
        };
        let cloned = key.clone();
        assert_eq!(cloned.name, key.name);
        assert_eq!(cloned.ssh_key.as_ref().unwrap().private_key, "key");
    }

    #[test]
    fn test_backup_schedule_clone() {
        let sched = BackupSchedule {
            template: "Nightly".to_string(),
            cron_format: "0 0 * * *".to_string(),
            active: true,
        };
        let cloned = sched.clone();
        assert_eq!(cloned.template, sched.template);
        assert_eq!(cloned.cron_format, sched.cron_format);
        assert_eq!(cloned.active, sched.active);
    }

    #[test]
    fn test_backup_integration_clone() {
        let integration = BackupIntegration {
            name: "Webhook".to_string(),
            template_id: Some(10),
        };
        let cloned = integration.clone();
        assert_eq!(cloned.name, integration.name);
        assert_eq!(cloned.template_id, integration.template_id);
    }

    #[test]
    fn test_backup_view_clone() {
        let view = BackupView {
            name: "Overview".to_string(),
            position: 3,
        };
        let cloned = view.clone();
        assert_eq!(cloned.name, view.name);
        assert_eq!(cloned.position, view.position);
    }

    #[test]
    fn test_backup_ssh_key_clone() {
        let ssh_key = BackupSshKey {
            private_key: "-----BEGIN KEY-----".to_string(),
            passphrase: None,
            login: Some("root".to_string()),
        };
        let cloned = ssh_key.clone();
        assert_eq!(cloned.private_key, ssh_key.private_key);
        assert_eq!(cloned.login, ssh_key.login);
    }

    #[test]
    fn test_backup_login_password_clone() {
        let creds = BackupLoginPassword {
            login: "admin".to_string(),
            password: "secret".to_string(),
        };
        let cloned = creds.clone();
        assert_eq!(cloned.login, creds.login);
        assert_eq!(cloned.password, creds.password);
    }

    #[test]
    fn test_backup_format_with_all_entity_types() {
        let backup = BackupFormat {
            version: "2.0".to_string(),
            project: BackupProject {
                name: "Full".to_string(),
                alert: Some(true),
                alert_chat: Some("#alerts".to_string()),
                max_parallel_tasks: Some(5),
            },
            templates: vec![BackupTemplate {
                name: "Deploy".to_string(),
                playbook: "deploy.yml".to_string(),
                arguments: None,
                template_type: "deploy".to_string(),
                inventory: Some("prod".to_string()),
                repository: Some("main".to_string()),
                environment: Some("production".to_string()),
                cron: Some("0 2 * * 1".to_string()),
            }],
            repositories: vec![BackupRepository {
                name: "main".to_string(),
                git_url: "https://github.com/org/main.git".to_string(),
                git_branch: "main".to_string(),
                ssh_key: Some("deploy-key".to_string()),
            }],
            inventories: vec![BackupInventory {
                name: "prod".to_string(),
                inventory_type: "static".to_string(),
                inventory: "[servers]\n10.0.0.1".to_string(),
                ssh_key: Some("deploy-key".to_string()),
                become_key: Some("sudo-key".to_string()),
            }],
            environments: vec![BackupEnvironment {
                name: "production".to_string(),
                json: r#"{"ENV":"prod"}"#.to_string(),
            }],
            access_keys: vec![
                BackupAccessKey {
                    name: "deploy-key".to_string(),
                    key_type: "ssh".to_string(),
                    owner: "shared".to_string(),
                    ssh_key: Some(BackupSshKey {
                        private_key: "key".to_string(),
                        passphrase: None,
                        login: Some("deploy".to_string()),
                    }),
                    login_password: None,
                },
                BackupAccessKey {
                    name: "sudo-key".to_string(),
                    key_type: "login_password".to_string(),
                    owner: "shared".to_string(),
                    ssh_key: None,
                    login_password: Some(BackupLoginPassword {
                        login: "root".to_string(),
                        password: "secret".to_string(),
                    }),
                },
            ],
            schedules: vec![BackupSchedule {
                template: "Deploy".to_string(),
                cron_format: "0 2 * * 1".to_string(),
                active: true,
            }],
            integrations: vec![BackupIntegration {
                name: "Slack".to_string(),
                template_id: Some(1),
            }],
            views: vec![BackupView {
                name: "Production View".to_string(),
                position: 0,
            }],
        };

        let json = serde_json::to_string(&backup).unwrap();
        let restored: BackupFormat = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.project.name, "Full");
        assert_eq!(restored.templates.len(), 1);
        assert_eq!(restored.repositories.len(), 1);
        assert_eq!(restored.inventories.len(), 1);
        assert_eq!(restored.environments.len(), 1);
        assert_eq!(restored.access_keys.len(), 2);
        assert_eq!(restored.schedules.len(), 1);
        assert_eq!(restored.integrations.len(), 1);
        assert_eq!(restored.views.len(), 1);
    }

    #[test]
    fn test_backup_format_serde_roundtrip_with_special_chars() {
        let backup = BackupFormat {
            version: "1.0".to_string(),
            project: BackupProject {
                name: "Project with \"quotes\" & <special> chars".to_string(),
                alert: Some(true),
                alert_chat: Some("#channel/with-slash".to_string()),
                max_parallel_tasks: Some(0),
            },
            templates: vec![],
            repositories: vec![],
            inventories: vec![],
            environments: vec![],
            access_keys: vec![],
            schedules: vec![],
            integrations: vec![],
            views: vec![],
        };

        let json = serde_json::to_string(&backup).unwrap();
        let restored: BackupFormat = serde_json::from_str(&json).unwrap();

        assert_eq!(
            restored.project.name,
            "Project with \"quotes\" & <special> chars"
        );
        assert_eq!(
            restored.project.alert_chat,
            Some("#channel/with-slash".to_string())
        );
    }

    #[test]
    fn test_make_unique_names_preserves_first() {
        let mut items = vec![
            BackupTemplate {
                name: "Original".to_string(),
                playbook: String::new(),
                arguments: None,
                template_type: String::new(),
                inventory: None,
                repository: None,
                environment: None,
                cron: None,
            },
            BackupTemplate {
                name: "Original".to_string(),
                playbook: String::new(),
                arguments: None,
                template_type: String::new(),
                inventory: None,
                repository: None,
                environment: None,
                cron: None,
            },
        ];

        make_unique_names(&mut items, |item| &item.name, |item, name| item.name = name);

        assert_eq!(items[0].name, "Original");
        assert_ne!(items[1].name, "Original");
    }

    #[test]
    fn test_make_unique_names_all_different() {
        let mut items = vec![
            BackupTemplate {
                name: "A".to_string(),
                playbook: String::new(),
                arguments: None,
                template_type: String::new(),
                inventory: None,
                repository: None,
                environment: None,
                cron: None,
            },
            BackupTemplate {
                name: "B".to_string(),
                playbook: String::new(),
                arguments: None,
                template_type: String::new(),
                inventory: None,
                repository: None,
                environment: None,
                cron: None,
            },
            BackupTemplate {
                name: "C".to_string(),
                playbook: String::new(),
                arguments: None,
                template_type: String::new(),
                inventory: None,
                repository: None,
                environment: None,
                cron: None,
            },
        ];

        make_unique_names(&mut items, |item| &item.name, |item, name| item.name = name);

        assert_eq!(items[0].name, "A");
        assert_eq!(items[1].name, "B");
        assert_eq!(items[2].name, "C");
    }

    #[test]
    fn test_get_random_name_contains_original() {
        let name = get_random_name("MyTemplate");
        assert!(name.starts_with("MyTemplate - "));
    }

    #[test]
    fn test_get_random_name_empty_string() {
        let name = get_random_name("");
        assert!(name.starts_with(" - "));
        assert_eq!(name.len(), 3 + 20); // " - " + 10 bytes hex = 23
    }

    #[test]
    fn test_backup_inventory_become_key_only() {
        let inv = BackupInventory {
            name: "Only Become".to_string(),
            inventory_type: "static".to_string(),
            inventory: "localhost".to_string(),
            ssh_key: None,
            become_key: Some("become".to_string()),
        };
        assert!(inv.ssh_key.is_none());
        assert_eq!(inv.become_key, Some("become".to_string()));
    }

    #[test]
    fn test_backup_inventory_ssh_key_only() {
        let inv = BackupInventory {
            name: "Only SSH".to_string(),
            inventory_type: "file".to_string(),
            inventory: "/etc/ansible/hosts".to_string(),
            ssh_key: Some("ssh".to_string()),
            become_key: None,
        };
        assert_eq!(inv.ssh_key, Some("ssh".to_string()));
        assert!(inv.become_key.is_none());
    }

    #[test]
    fn test_backup_repository_with_ssh_key_string() {
        let repo = BackupRepository {
            name: "Secure Repo".to_string(),
            git_url: "git@github.com:org/repo.git".to_string(),
            git_branch: "develop".to_string(),
            ssh_key: Some("my-ssh-key".to_string()),
        };
        assert_eq!(repo.ssh_key, Some("my-ssh-key".to_string()));
    }

    #[test]
    fn test_backup_template_with_all_refs() {
        let tpl = BackupTemplate {
            name: "Full Template".to_string(),
            playbook: "site.yml".to_string(),
            arguments: Some("--forks=10".to_string()),
            template_type: "deploy".to_string(),
            inventory: Some("prod-inv".to_string()),
            repository: Some("prod-repo".to_string()),
            environment: Some("prod-env".to_string()),
            cron: Some("0 3 * * *".to_string()),
        };
        assert_eq!(tpl.inventory, Some("prod-inv".to_string()));
        assert_eq!(tpl.repository, Some("prod-repo".to_string()));
        assert_eq!(tpl.environment, Some("prod-env".to_string()));
        assert_eq!(tpl.cron, Some("0 3 * * *".to_string()));
    }

    #[test]
    fn test_backup_template_no_refs() {
        let tpl = BackupTemplate {
            name: "Standalone".to_string(),
            playbook: "standalone.yml".to_string(),
            arguments: None,
            template_type: "default".to_string(),
            inventory: None,
            repository: None,
            environment: None,
            cron: None,
        };
        assert!(tpl.inventory.is_none());
        assert!(tpl.repository.is_none());
        assert!(tpl.environment.is_none());
        assert!(tpl.cron.is_none());
    }

    #[test]
    fn test_backup_format_debug_output() {
        let backup = BackupFormat {
            version: "1.0".to_string(),
            project: BackupProject {
                name: "DebugTest".to_string(),
                alert: None,
                alert_chat: None,
                max_parallel_tasks: None,
            },
            templates: vec![],
            repositories: vec![],
            inventories: vec![],
            environments: vec![],
            access_keys: vec![],
            schedules: vec![],
            integrations: vec![],
            views: vec![],
        };
        let debug_str = format!("{:?}", backup);
        assert!(debug_str.contains("BackupFormat"));
        assert!(debug_str.contains("DebugTest"));
    }

    #[test]
    fn test_backup_schedule_debug_output() {
        let sched = BackupSchedule {
            template: "Test".to_string(),
            cron_format: "*/5 * * * *".to_string(),
            active: false,
        };
        let debug_str = format!("{:?}", sched);
        assert!(debug_str.contains("BackupSchedule"));
        assert!(debug_str.contains("Test"));
    }

    #[test]
    fn test_backup_integration_debug_output() {
        let integration = BackupIntegration {
            name: "PagerDuty".to_string(),
            template_id: Some(7),
        };
        let debug_str = format!("{:?}", integration);
        assert!(debug_str.contains("BackupIntegration"));
        assert!(debug_str.contains("PagerDuty"));
    }

    #[test]
    fn test_backup_view_debug_output() {
        let view = BackupView {
            name: "TestView".to_string(),
            position: 42,
        };
        let debug_str = format!("{:?}", view);
        assert!(debug_str.contains("BackupView"));
        assert!(debug_str.contains("TestView"));
    }

    #[test]
    fn test_backup_ssh_key_debug_output() {
        let ssh_key = BackupSshKey {
            private_key: "KEY_DATA".to_string(),
            passphrase: Some("secret".to_string()),
            login: Some("user".to_string()),
        };
        let debug_str = format!("{:?}", ssh_key);
        assert!(debug_str.contains("BackupSshKey"));
        assert!(debug_str.contains("KEY_DATA"));
    }

    #[test]
    fn test_backup_login_password_debug_output() {
        let creds = BackupLoginPassword {
            login: "admin".to_string(),
            password: "pass".to_string(),
        };
        let debug_str = format!("{:?}", creds);
        assert!(debug_str.contains("BackupLoginPassword"));
        assert!(debug_str.contains("admin"));
    }

    #[test]
    fn test_backup_format_deserialize_with_keys_alias() {
        let json = r#"{
            "project": {"name": "Test"},
            "keys": [
                {"name": "Key1", "type": "none", "owner": "shared"}
            ],
            "templates": []
        }"#;
        let backup: BackupFormat = serde_json::from_str(json).unwrap();
        assert_eq!(backup.access_keys.len(), 1);
        assert_eq!(backup.access_keys[0].name, "Key1");
    }

    #[test]
    fn test_backup_format_deserialize_with_meta_alias() {
        let json = r#"{
            "meta": {"name": "MetaProject", "alert": true},
            "templates": [],
            "repositories": []
        }"#;
        let backup: BackupFormat = serde_json::from_str(json).unwrap();
        assert_eq!(backup.project.name, "MetaProject");
        assert_eq!(backup.project.alert, Some(true));
    }

    #[test]
    fn test_backup_project_max_parallel_tasks_zero() {
        let project = BackupProject {
            name: "Zero Parallel".to_string(),
            alert: None,
            alert_chat: None,
            max_parallel_tasks: Some(0),
        };
        assert_eq!(project.max_parallel_tasks, Some(0));
    }

    #[test]
    fn test_backup_project_max_parallel_tasks_negative() {
        let project = BackupProject {
            name: "Negative Parallel".to_string(),
            alert: None,
            alert_chat: None,
            max_parallel_tasks: Some(-1),
        };
        assert_eq!(project.max_parallel_tasks, Some(-1));
    }

    #[test]
    fn test_backup_environment_empty_json() {
        let env = BackupEnvironment {
            name: "Empty".to_string(),
            json: String::new(),
        };
        assert_eq!(env.json, "");
        let json_str = serde_json::to_string(&env).unwrap();
        assert!(json_str.contains("\"json\":\"\""));
    }

    #[test]
    fn test_backup_environment_complex_json() {
        let env = BackupEnvironment {
            name: "Complex".to_string(),
            json: r#"{"nested":{"key":"value"},"array":[1,2,3],"bool":true}"#.to_string(),
        };
        let json_str = serde_json::to_string(&env).unwrap();
        let restored: BackupEnvironment = serde_json::from_str(&json_str).unwrap();
        assert_eq!(restored.json, env.json);
    }

    #[test]
    fn test_backup_format_large_backup() {
        let mut templates = Vec::new();
        for i in 0..50 {
            templates.push(BackupTemplate {
                name: format!("Template-{}", i),
                playbook: format!("playbook-{}.yml", i),
                arguments: None,
                template_type: "default".to_string(),
                inventory: None,
                repository: None,
                environment: None,
                cron: None,
            });
        }

        let backup = BackupFormat {
            version: "1.0".to_string(),
            project: BackupProject {
                name: "Large".to_string(),
                alert: None,
                alert_chat: None,
                max_parallel_tasks: None,
            },
            templates,
            repositories: vec![],
            inventories: vec![],
            environments: vec![],
            access_keys: vec![],
            schedules: vec![],
            integrations: vec![],
            views: vec![],
        };

        assert_eq!(backup.templates.len(), 50);
        assert_eq!(backup.templates[49].name, "Template-49");

        let json = serde_json::to_string(&backup).unwrap();
        let restored: BackupFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.templates.len(), 50);
    }
}
