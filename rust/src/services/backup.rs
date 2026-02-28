//! Backup - экспорт и импорт проектов
//!
//! Аналог services/project/backup.go из Go версии

use serde::{Deserialize, Serialize};
use rand::RngCore;
use std::collections::HashMap;
use tracing::{info, warn, error};

use crate::error::{Error, Result};
use crate::models::*;

/// BackupFormat - формат backup проекта
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFormat {
    pub version: String,
    pub project: BackupProject,
    pub templates: Vec<BackupTemplate>,
    pub repositories: Vec<BackupRepository>,
    pub inventories: Vec<BackupInventory>,
    pub environments: Vec<BackupEnvironment>,
    pub access_keys: Vec<BackupAccessKey>,
    pub schedules: Vec<BackupSchedule>,
    pub integrations: Vec<BackupIntegration>,
    pub views: Vec<BackupView>,
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
    pub playbook: String,
    pub arguments: Option<String>,
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
    pub git_url: String,
    pub git_branch: String,
    pub ssh_key: Option<String>,
}

/// BackupInventory - инвентарь для backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInventory {
    pub name: String,
    pub inventory_type: String,
    pub inventory: String,
    pub ssh_key: Option<String>,
    pub become_key: Option<String>,
}

/// BackupEnvironment - окружение для backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEnvironment {
    pub name: String,
    pub json: String,
    pub env: Option<String>,
}

/// BackupAccessKey - ключ доступа для backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupAccessKey {
    pub name: String,
    pub key_type: String,
    pub owner: String,
    pub ssh_key: Option<BackupSshKey>,
    pub login_password: Option<BackupLoginPassword>,
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
    pub name: String,
    pub position: i32,
}

/// BackupDB - загрузчик backup из БД
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
        Self {
            templates: Vec::new(),
            repositories: Vec::new(),
            inventories: Vec::new(),
            environments: Vec::new(),
            access_keys: Vec::new(),
            schedules: Vec::new(),
            integrations: Vec::new(),
            views: Vec::new(),
        }
    }

    /// Загружает данные из БД
    pub async fn load(&mut self, project_id: i32, store: &dyn crate::db::Store) -> Result<()> {
        self.templates = store.get_templates(project_id).await?;
        self.repositories = store.get_repositories(project_id).await?;
        self.inventories = store.get_inventories(project_id).await?;
        self.environments = store.get_environments(project_id).await?;
        self.access_keys = store.get_access_keys(project_id, crate::models::GetAccessKeyOptions::default()).await?;
        self.schedules = store.get_project_schedules(project_id, false, false).await?;
        self.integrations = store.get_integrations(project_id).await?;
        self.views = store.get_views(project_id).await?;
        
        Ok(())
    }

    /// Уникализирует имена
    pub fn make_unique_names(&mut self) {
        make_unique_names(&mut self.templates, |item| &item.name, |item, name| item.name = name);
        make_unique_names(&mut self.repositories, |item| &item.name, |item, name| item.name = name);
        make_unique_names(&mut self.inventories, |item| &item.name, |item, name| item.name = name);
        make_unique_names(&mut self.environments, |item| &item.name, |item, name| item.name = name);
        make_unique_names(&mut self.access_keys, |item| &item.name, |item, name| item.name = name);
    }

    /// Форматирует backup
    pub fn format(&self, project: &Project) -> Result<BackupFormat> {
        let mut backup = BackupFormat {
            version: "1.0".to_string(),
            project: BackupProject {
                name: project.name.clone(),
                alert: project.alert,
                alert_chat: project.alert_chat.clone(),
                max_parallel_tasks: project.max_parallel_tasks,
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
                template_type: tpl.template_type.to_string(),
                inventory: tpl.inventory_id.and_then(|id| inventory_map.get(&id).cloned()),
                repository: tpl.repository_id.and_then(|id| repository_map.get(&id).cloned()),
                environment: tpl.environment_id.and_then(|id| environment_map.get(&id).cloned()),
                cron: schedule,
            });
        }

        // Конвертируем репозитории
        for repo in &self.repositories {
            backup.repositories.push(BackupRepository {
                name: repo.name.clone(),
                git_url: repo.git_url.clone(),
                git_branch: repo.git_branch.clone(),
                ssh_key: repo.ssh_key_id.and_then(|id| access_key_map.get(&id).cloned()),
            });
        }

        // Конвертируем инвентари
        for inv in &self.inventories {
            backup.inventories.push(BackupInventory {
                name: inv.name.clone(),
                inventory_type: inv.inventory_type.to_string(),
                inventory: inv.inventory.clone(),
                ssh_key: inv.ssh_key_id.and_then(|id| access_key_map.get(&id).cloned()),
                become_key: inv.become_key_id.and_then(|id| access_key_map.get(&id).cloned()),
            });
        }

        // Конвертируем окружения
        for env in &self.environments {
            backup.environments.push(BackupEnvironment {
                name: env.name.clone(),
                json: env.json.clone(),
                env: env.env.clone(),
            });
        }

        // Конвертируем ключи доступа
        for key in &self.access_keys {
            let mut backup_key = BackupAccessKey {
                name: key.name.clone(),
                key_type: key.key_type.to_string(),
                owner: key.owner.to_string(),
                ssh_key: None,
                login_password: None,
            };

            if let Some(ref ssh) = key.ssh_key {
                backup_key.ssh_key = Some(BackupSshKey {
                    private_key: ssh.private_key.clone(),
                    passphrase: ssh.passphrase.clone(),
                    login: ssh.login.clone(),
                });
            }

            if let Some(ref lp) = key.login_password {
                backup_key.login_password = Some(BackupLoginPassword {
                    login: lp.login.clone(),
                    password: lp.password.clone(),
                });
            }

            backup.access_keys.push(backup_key);
        }

        // Конвертируем расписания
        for schedule in &self.schedules {
            if let Some(tpl_name) = get_template_name_by_id(schedule.template_id, &self.templates) {
                backup.schedules.push(BackupSchedule {
                    template: tpl_name,
                    cron_format: schedule.cron_format.clone(),
                    active: schedule.active,
                });
            }
        }

        // Конвертируем интеграции
        for integration in &self.integrations {
            backup.integrations.push(BackupIntegration {
                name: integration.name.clone(),
                template_id: integration.template_id,
            });
        }

        // Конвертируем представления
        for view in &self.views {
            backup.views.push(BackupView {
                name: view.name.clone(),
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
    for item in items {
        if item.get_name() == name {
            return Some(item);
        }
    }
    None
}

/// Получает расписания по проекту
pub fn get_schedules_by_project(project_id: i32, schedules: &[Schedule]) -> Vec<Schedule> {
    schedules.iter()
        .filter(|s| s.project_id == project_id)
        .cloned()
        .collect()
}

/// Получает cron формат по шаблону
pub fn get_schedule_by_template(template_id: i32, schedules: &[Schedule]) -> Option<String> {
    schedules.iter()
        .find(|s| s.template_id == template_id)
        .map(|s| s.cron_format.clone())
}

/// Генерирует случайное имя
pub fn get_random_name(name: &str) -> String {
    let mut rng = rand::thread_rng();
    let mut random_bytes = [0u8; 10];
    rng.fill_bytes(&mut random_bytes);
    format!("{} - {}", name, hex::encode(random_bytes))
}

/// Уникализирует имена
pub fn make_unique_names<T>(items: &mut [T], getter: impl Fn(&T) -> &String, setter: impl Fn(&mut T, String)) {
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
    templates.iter()
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
    fn get_id(&self) -> i32 { self.id }
    fn get_name(&self) -> String { self.name.clone() }
}

impl BackupEntity for Repository {
    fn get_id(&self) -> i32 { self.id }
    fn get_name(&self) -> String { self.name.clone() }
}

impl BackupEntity for Inventory {
    fn get_id(&self) -> i32 { self.id }
    fn get_name(&self) -> String { self.name.clone() }
}

impl BackupEntity for Environment {
    fn get_id(&self) -> i32 { self.id }
    fn get_name(&self) -> String { self.name.clone() }
}

impl BackupEntity for AccessKey {
    fn get_id(&self) -> i32 { self.id }
    fn get_name(&self) -> String { self.name.clone() }
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
            BackupTemplate { name: "Test".to_string(), playbook: String::new(), arguments: None, template_type: String::new(), inventory: None, repository: None, environment: None, cron: None },
            BackupTemplate { name: "Test".to_string(), playbook: String::new(), arguments: None, template_type: String::new(), inventory: None, repository: None, environment: None, cron: None },
        ];

        make_unique_names(&mut items, |item| &item.name, |item, name| item.name = name);

        assert_ne!(items[0].name, items[1].name);
    }
}
