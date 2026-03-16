//! Restore - восстановление проектов из backup
//!
//! Аналог services/project/restore.go из Go версии

use rand::RngCore;
use tracing::{info, warn, error};

use crate::error::{Error, Result};
use crate::models::*;
use crate::services::backup::*;

use async_trait::async_trait;

/// Trait для сущностей restore
pub trait RestoreEntry {
    type Output;

    fn get_name(&self) -> &str;
    fn verify(&self, backup: &BackupFormat) -> Result<()>;
}

/// Trait для асинхронного restore
#[async_trait::async_trait]
pub trait RestoreEntryAsync: RestoreEntry {
    async fn restore_async(&self, store: &dyn crate::db::Store, backup_db: &mut RestoreDB) -> Result<Self::Output>;
}

/// RestoreDB - база данных для восстановления
pub struct RestoreDB {
    pub meta: Project,
    pub environments: Vec<Environment>,
    pub access_keys: Vec<AccessKey>,
    pub repositories: Vec<Repository>,
    pub inventories: Vec<Inventory>,
    pub templates: Vec<Template>,
    pub schedules: Vec<Schedule>,
    pub integrations: Vec<Integration>,
    pub views: Vec<View>,
    pub roles: Vec<Role>,
    pub secret_storages: Vec<SecretStorage>,
}

impl RestoreDB {
    /// Создаёт новую RestoreDB
    pub fn new(project: Project) -> Self {
        Self {
            meta: project,
            environments: Vec::new(),
            access_keys: Vec::new(),
            repositories: Vec::new(),
            inventories: Vec::new(),
            templates: Vec::new(),
            schedules: Vec::new(),
            integrations: Vec::new(),
            views: Vec::new(),
            roles: Vec::new(),
            secret_storages: Vec::new(),
        }
    }
}

/// Получает сущность по имени
pub fn get_entry_by_name<'a, T: RestoreEntry>(name: &'a Option<String>, items: &'a [T]) -> Option<&'a T> {
    if name.is_none() {
        return None;
    }
    
    let target_name = name.as_ref().unwrap();
    items.iter().find(|&item| item.get_name() == target_name).map(|v| v as _)
}

/// Проверяет на дубликаты
pub fn verify_duplicate<T: RestoreEntry>(name: &str, items: &[T]) -> Result<()> {
    let count = items.iter()
        .filter(|item| item.get_name() == name)
        .count();
    
    if count > 1 {
        return Err(Error::Other(format!("{} is duplicate", name)));
    }
    
    Ok(())
}

/// Генерирует случайный slug
pub fn generate_random_slug() -> String {
    let mut rng = rand::thread_rng();
    let mut random_bytes = [0u8; 16];
    rng.fill_bytes(&mut random_bytes);
    hex::encode(random_bytes)
}

// ============================================================================
// Restore методы для сущностей
// ============================================================================

impl RestoreEntry for BackupEnvironment {
    type Output = Environment;

    fn get_name(&self) -> &str {
        &self.name
    }

    fn verify(&self, backup: &BackupFormat) -> Result<()> {
        verify_duplicate(&self.name, &backup.environments)
    }
}

#[async_trait::async_trait]
impl RestoreEntryAsync for BackupEnvironment {
    async fn restore_async(&self, store: &dyn crate::db::Store, backup_db: &mut RestoreDB) -> Result<Self::Output> {
        let env = Environment {
            id: 0,
            project_id: backup_db.meta.id,
            name: self.name.clone(),
            json: self.json.clone(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: None,
            created: None,
        };

        let new_env = store.create_environment(env).await?;
        backup_db.environments.push(new_env.clone());

        Ok(new_env)
    }
}

impl RestoreEntry for BackupView {
    type Output = View;

    fn get_name(&self) -> &str {
        &self.name
    }

    fn verify(&self, backup: &BackupFormat) -> Result<()> {
        verify_duplicate(&self.name, &backup.views)
    }
}

#[async_trait::async_trait]
impl RestoreEntryAsync for BackupView {
    async fn restore_async(&self, store: &dyn crate::db::Store, backup_db: &mut RestoreDB) -> Result<Self::Output> {
        let view = View {
            id: 0,
            project_id: backup_db.meta.id,
            title: self.name.clone(),
            position: self.position,
        };

        let new_view = store.create_view(view).await?;
        backup_db.views.push(new_view.clone());

        Ok(new_view)
    }
}

impl RestoreEntry for BackupSchedule {
    type Output = Schedule;

    fn get_name(&self) -> &str {
        &self.template
    }

    fn verify(&self, backup: &BackupFormat) -> Result<()> {
        // Проверка на дубликаты расписаний для шаблона
        let count = backup.schedules.iter()
            .filter(|s| s.template == self.template)
            .count();

        if count > 1 {
            return Err(Error::Other(format!("Schedule for template {} is duplicate", self.template)));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl RestoreEntryAsync for BackupSchedule {
    async fn restore_async(&self, store: &dyn crate::db::Store, backup_db: &mut RestoreDB) -> Result<Self::Output> {
        // Находим шаблон по имени
        let template = backup_db.templates.iter()
            .find(|t| t.name == self.template)
            .ok_or_else(|| Error::NotFound(format!("Template {} not found", self.template)))?;

        let schedule = Schedule {
            id: 0,
            project_id: backup_db.meta.id,
            template_id: template.id,
            cron: String::new(),
            cron_format: Some(self.cron_format.clone()),
            name: self.template.clone(),
            active: self.active,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };

        let new_schedule = store.create_schedule(schedule).await?;
        backup_db.schedules.push(new_schedule.clone());

        Ok(new_schedule)
    }
}

impl RestoreEntry for BackupAccessKey {
    type Output = AccessKey;
    
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn verify(&self, backup: &BackupFormat) -> Result<()> {
        verify_duplicate(&self.name, &backup.access_keys)
    }
}

#[async_trait::async_trait]
impl RestoreEntryAsync for BackupAccessKey {
    async fn restore_async(&self, store: &dyn crate::db::Store, backup_db: &mut RestoreDB) -> Result<Self::Output> {
        let key = AccessKey {
            id: 0,
            project_id: Some(backup_db.meta.id),
            name: self.name.clone(),
            r#type: self.key_type.parse().unwrap_or(AccessKeyType::None),
            owner: Some(self.owner.parse().unwrap_or(AccessKeyOwner::Shared)),
            user_id: None,
            login_password_login: None,
            login_password_password: None,
            ssh_key: None,
            ssh_passphrase: None,
            access_key_access_key: None,
            access_key_secret_key: None,
            secret_storage_id: None,
            source_storage_type: None,
            source_storage_id: None,
            source_key: None,
            environment_id: None,
            created: None,
        };

        let new_key = store.create_access_key(key).await?;
        backup_db.access_keys.push(new_key.clone());

        Ok(new_key)
    }
}

impl RestoreEntry for BackupInventory {
    type Output = Inventory;

    fn get_name(&self) -> &str {
        &self.name
    }

    fn verify(&self, backup: &BackupFormat) -> Result<()> {
        verify_duplicate(&self.name, &backup.inventories)
    }
}

#[async_trait::async_trait]
impl RestoreEntryAsync for BackupInventory {
    async fn restore_async(&self, store: &dyn crate::db::Store, backup_db: &mut RestoreDB) -> Result<Self::Output> {
        let mut inv = Inventory {
            id: 0,
            project_id: backup_db.meta.id,
            name: self.name.clone(),
            inventory_type: self.inventory_type.parse().unwrap_or(InventoryType::Static),
            inventory_data: self.inventory.clone(),
            key_id: None,
            secret_storage_id: None,
            ssh_login: "root".to_string(),
            ssh_port: 22,
            extra_vars: None,
            ssh_key_id: None,
            become_key_id: None,
            vaults: None,
            created: Some(chrono::Utc::now()),
            runner_tag: None,
        };

        // Находим SSH ключ по имени
        if let Some(ref ssh_key_name) = self.ssh_key {
            if let Some(ssh_key) = backup_db.access_keys.iter().find(|k| &k.name == ssh_key_name) {
                inv.ssh_key_id = Some(ssh_key.id);
            }
        }

        // Находим Become ключ по имени
        if let Some(ref become_key_name) = self.become_key {
            if let Some(become_key) = backup_db.access_keys.iter().find(|k| &k.name == become_key_name) {
                inv.become_key_id = Some(become_key.id);
            }
        }

        let new_inv = store.create_inventory(inv).await?;
        backup_db.inventories.push(new_inv.clone());

        Ok(new_inv)
    }
}

impl RestoreEntry for BackupRepository {
    type Output = Repository;

    fn get_name(&self) -> &str {
        &self.name
    }

    fn verify(&self, backup: &BackupFormat) -> Result<()> {
        verify_duplicate(&self.name, &backup.repositories)
    }
}

#[async_trait::async_trait]
impl RestoreEntryAsync for BackupRepository {
    async fn restore_async(&self, store: &dyn crate::db::Store, backup_db: &mut RestoreDB) -> Result<Self::Output> {
        let repo = Repository {
            id: 0,
            project_id: backup_db.meta.id,
            name: self.name.clone(),
            git_url: self.git_url.clone(),
            // git_type: self.git_type.parse().unwrap_or(RepositoryType::Git),  // поле удалено
            git_type: RepositoryType::Git,
            git_branch: self.git_branch.clone().into(),
            key_id: None,
            git_path: None,
            created: None,
        };

        let new_repo = store.create_repository(repo).await?;
        backup_db.repositories.push(new_repo.clone());

        Ok(new_repo)
    }
}

impl RestoreEntry for BackupTemplate {
    type Output = Template;

    fn get_name(&self) -> &str {
        &self.name
    }

    fn verify(&self, backup: &BackupFormat) -> Result<()> {
        verify_duplicate(&self.name, &backup.templates)
    }
}

#[async_trait::async_trait]
impl RestoreEntryAsync for BackupTemplate {
    async fn restore_async(&self, store: &dyn crate::db::Store, backup_db: &mut RestoreDB) -> Result<Self::Output> {
        let tpl = Template {
            id: 0,
            project_id: backup_db.meta.id,
            name: self.name.clone(),
            playbook: self.playbook.clone(),
            // description: self.description.clone().unwrap_or_default(),  // поле удалено
            description: String::new(),
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            r#type: self.template_type.parse().unwrap_or(TemplateType::Default),
            app: TemplateApp::Default,
            git_branch: None,
            created: chrono::Utc::now(),
            arguments: self.arguments.clone(),
            vault_key_id: None,
            view_id: None,
            build_template_id: None,
            autorun: false,
            allow_override_args_in_task: false,
            allow_override_branch_in_task: false,
            allow_inventory_in_task: false,
            allow_parallel_tasks: false,
            suppress_success_alerts: false,
            task_params: None,
            survey_vars: None,
            vaults: None,
        };

        let new_tpl = store.create_template(tpl).await?;
        backup_db.templates.push(new_tpl.clone());

        Ok(new_tpl)
    }
}

// ============================================================================
// Методы восстановления
// ============================================================================

impl BackupFormat {
    /// Проверяет backup на корректность
    pub fn verify(&self) -> Result<()> {
        // Проверяем все сущности на дубликаты
        for env in &self.environments {
            env.verify(self)?;
        }

        for view in &self.views {
            view.verify(self)?;
        }

        for schedule in &self.schedules {
            schedule.verify(self)?;
        }

        for key in &self.access_keys {
            key.verify(self)?;
        }

        for inv in &self.inventories {
            inv.verify(self)?;
        }

        for repo in &self.repositories {
            repo.verify(self)?;
        }

        for tpl in &self.templates {
            tpl.verify(self)?;
        }

        Ok(())
    }

    /// Восстанавливает проект из backup
    pub async fn restore(&self, user: &User, store: &dyn crate::db::Store) -> Result<Project> {
        info!("Starting project restore from backup...");

        // Создаём новый проект
        let mut project = Project {
            id: 0,
            name: format!("{} (Restored)", self.project.name),
            created: chrono::Utc::now(),
            alert: self.project.alert.unwrap_or(false),
            alert_chat: self.project.alert_chat.clone(),
            max_parallel_tasks: self.project.max_parallel_tasks.unwrap_or(0),
            r#type: "default".to_string(),
            default_secret_storage_id: None,
        };

        let new_project = store.create_project(project).await?;
        info!("Project {} created with ID {}", new_project.name, new_project.id);

        // Создаём базу данных для восстановления
        let mut restore_db = RestoreDB::new(new_project.clone());

        // Восстанавливаем сущности в правильном порядке
        // 1. Окружения
        for env in &self.environments {
            env.restore_async(store, &mut restore_db).await?;
        }
        info!("Environments restored: {}", restore_db.environments.len());

        // 2. Представления
        for view in &self.views {
            view.restore_async(store, &mut restore_db).await?;
        }
        info!("Views restored: {}", restore_db.views.len());

        // 3. Ключи доступа
        for key in &self.access_keys {
            key.restore_async(store, &mut restore_db).await?;
        }
        info!("Access keys restored: {}", restore_db.access_keys.len());

        // 4. Инвентари
        for inv in &self.inventories {
            inv.restore_async(store, &mut restore_db).await?;
        }
        info!("Inventories restored: {}", restore_db.inventories.len());

        // 5. Репозитории
        for repo in &self.repositories {
            repo.restore_async(store, &mut restore_db).await?;
        }
        info!("Repositories restored: {}", restore_db.repositories.len());

        // 6. Шаблоны
        for tpl in &self.templates {
            tpl.restore_async(store, &mut restore_db).await?;
        }
        info!("Templates restored: {}", restore_db.templates.len());

        // 7. Расписания
        for schedule in &self.schedules {
            schedule.restore_async(store, &mut restore_db).await?;
        }
        info!("Schedules restored: {}", restore_db.schedules.len());

        // 8. Интеграции
        for integration in &self.integrations {
            // Map old template_id to restored template: use first available template as fallback
            let template_id = restore_db.templates.first().map(|t| t.id).unwrap_or(0);

            let new_integration = crate::models::Integration {
                id: 0,
                project_id: restore_db.meta.id,
                name: integration.name.clone(),
                template_id,
                auth_method: String::new(),
                auth_header: None,
                auth_secret_id: None,
            };

            match store.create_integration(new_integration).await {
                Ok(created) => {
                    restore_db.integrations.push(created);
                }
                Err(e) => {
                    warn!("Failed to restore integration {}: {}", integration.name, e);
                }
            }
        }

        info!("Project restore completed successfully!");

        Ok(new_project)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_slug() {
        let slug1 = generate_random_slug();
        let slug2 = generate_random_slug();
        
        assert_eq!(slug1.len(), 32);
        assert_eq!(slug2.len(), 32);
        assert_ne!(slug1, slug2);
    }

    #[test]
    fn test_verify_duplicate() {
        let items = vec![
            BackupEnvironment { name: "Test".to_string(), json: String::new() },
            BackupEnvironment { name: "Test".to_string(), json: String::new() },
        ];
        
        let result = verify_duplicate("Test", &items);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_entry_by_name() {
        let items = vec![
            BackupEnvironment { name: "Test1".to_string(), json: String::new() },
            BackupEnvironment { name: "Test2".to_string(), json: String::new() },
        ];
        
        let name = Some("Test1".to_string());
        let result = get_entry_by_name(&name, &items);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Test1");
    }
}
