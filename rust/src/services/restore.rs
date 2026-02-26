//! Restore - восстановление проектов из backup
//!
//! Аналог services/project/restore.go из Go версии

use rand::RngCore;
use tracing::{info, warn, error};

use crate::error::{Error, Result};
use crate::models::*;
use crate::services::backup::*;

/// Trait для сущностей restore
pub trait RestoreEntry {
    type Output;
    
    fn get_name(&self) -> &str;
    fn verify(&self, backup: &BackupFormat) -> Result<()>;
    fn restore(&self, store: &dyn crate::db::Store, backup_db: &mut RestoreDB) -> Result<Self::Output>;
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
pub fn get_entry_by_name<T: RestoreEntry>(name: &Option<String>, items: &[T]) -> Option<&T> {
    if name.is_none() {
        return None;
    }
    
    let target_name = name.as_ref().unwrap();
    for item in items {
        if item.get_name() == target_name {
            return Some(item);
        }
    }
    None
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
    
    fn restore(&self, store: &dyn crate::db::Store, backup_db: &mut RestoreDB) -> Result<Self::Output> {
        let mut env = self.clone();
        env.project_id = backup_db.meta.id;
        
        let new_env = store.create_environment(env)?;
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
    
    fn restore(&self, store: &dyn crate::db::Store, backup_db: &mut RestoreDB) -> Result<Self::Output> {
        let mut view = self.clone();
        view.project_id = backup_db.meta.id;
        
        let new_view = store.create_view(view)?;
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
    
    fn restore(&self, store: &dyn crate::db::Store, backup_db: &mut RestoreDB) -> Result<Self::Output> {
        // Находим шаблон по имени
        let template = backup_db.templates.iter()
            .find(|t| t.name == self.template)
            .ok_or_else(|| Error::NotFound(format!("Template {} not found", self.template)))?;
        
        let schedule = Schedule {
            id: 0,
            project_id: backup_db.meta.id,
            template_id: template.id,
            cron_format: self.cron_format.clone(),
            active: self.active,
            last_commit_hash: None,
            repository_id: None,
        };
        
        let new_schedule = store.create_schedule(schedule)?;
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
    
    fn restore(&self, store: &dyn crate::db::Store, backup_db: &mut RestoreDB) -> Result<Self::Output> {
        let mut key = AccessKey {
            id: 0,
            project_id: Some(backup_db.meta.id),
            name: self.name.clone(),
            owner: self.owner.parse().unwrap_or(AccessKeyOwner::Shared),
            key_type: self.key_type.parse().unwrap_or(AccessKeyType::None),
            ssh_key: None,
            login_password: None,
            created: chrono::Utc::now(),
            override_secret: false,
            environment_id: None,
        };
        
        if let Some(ref ssh) = self.ssh_key {
            key.ssh_key = Some(SshKeyData {
                private_key: ssh.private_key.clone(),
                passphrase: ssh.passphrase.clone(),
                login: ssh.login.clone(),
            });
        }
        
        if let Some(ref lp) = self.login_password {
            key.login_password = Some(LoginPasswordData {
                login: lp.login.clone(),
                password: lp.password.clone(),
            });
        }
        
        let new_key = store.create_access_key(key)?;
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
    
    fn restore(&self, store: &dyn crate::db::Store, backup_db: &mut RestoreDB) -> Result<Self::Output> {
        let mut inv = Inventory {
            id: 0,
            project_id: backup_db.meta.id,
            name: self.name.clone(),
            inventory_type: self.inventory_type.parse().unwrap_or(InventoryType::Static),
            inventory: self.inventory.clone(),
            ssh_key_id: None,
            become_key_id: None,
            vaults: Vec::new(),
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
        
        let new_inv = store.create_inventory(inv)?;
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
    
    fn restore(&self, store: &dyn crate::db::Store, backup_db: &mut RestoreDB) -> Result<Self::Output> {
        let mut repo = Repository {
            id: 0,
            project_id: backup_db.meta.id,
            name: self.name.clone(),
            git_url: self.git_url.clone(),
            git_branch: self.git_branch.clone(),
            ssh_key_id: None,
        };
        
        // Находим SSH ключ по имени
        if let Some(ref ssh_key_name) = self.ssh_key {
            if let Some(ssh_key) = backup_db.access_keys.iter().find(|k| &k.name == ssh_key_name) {
                repo.ssh_key_id = Some(ssh_key.id);
            }
        }
        
        let new_repo = store.create_repository(repo)?;
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
    
    fn restore(&self, store: &dyn crate::db::Store, backup_db: &mut RestoreDB) -> Result<Self::Output> {
        let mut tpl = Template {
            id: 0,
            project_id: backup_db.meta.id,
            name: self.name.clone(),
            playbook: self.playbook.clone(),
            arguments: self.arguments.clone(),
            template_type: self.template_type.parse().unwrap_or(TemplateType::Task),
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            start_version: None,
            build_version: None,
            description: None,
            survey_vars: None,
            vaults: Vec::new(),
            tasks: 0,
            created: chrono::Utc::now(),
        };
        
        // Находим инвентарь по имени
        if let Some(ref inv_name) = self.inventory {
            if let Some(inv) = backup_db.inventories.iter().find(|i| &i.name == inv_name) {
                tpl.inventory_id = Some(inv.id);
            }
        }
        
        // Находим репозиторий по имени
        if let Some(ref repo_name) = self.repository {
            if let Some(repo) = backup_db.repositories.iter().find(|r| &r.name == repo_name) {
                tpl.repository_id = Some(repo.id);
            }
        }
        
        // Находим окружение по имени
        if let Some(ref env_name) = self.environment {
            if let Some(env) = backup_db.environments.iter().find(|e| &e.name == env_name) {
                tpl.environment_id = Some(env.id);
            }
        }
        
        let new_tpl = store.create_template(tpl)?;
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
    pub fn restore(&self, user: &User, store: &dyn crate::db::Store) -> Result<Project> {
        info!("Starting project restore from backup...");
        
        // Создаём новый проект
        let mut project = Project {
            id: 0,
            name: format!("{} (Restored)", self.project.name),
            created: chrono::Utc::now(),
            alert: self.project.alert,
            alert_chat: self.project.alert_chat.clone(),
            max_parallel_tasks: self.project.max_parallel_tasks,
        };
        
        let new_project = store.create_project(project)?;
        info!("Project {} created with ID {}", new_project.name, new_project.id);
        
        // Создаём базу данных для восстановления
        let mut restore_db = RestoreDB::new(new_project.clone());
        
        // Восстанавливаем сущности в правильном порядке
        // 1. Окружения
        for env in &self.environments {
            env.restore(store, &mut restore_db)?;
        }
        info!("Environments restored: {}", restore_db.environments.len());
        
        // 2. Представления
        for view in &self.views {
            view.restore(store, &mut restore_db)?;
        }
        info!("Views restored: {}", restore_db.views.len());
        
        // 3. Ключи доступа
        for key in &self.access_keys {
            key.restore(store, &mut restore_db)?;
        }
        info!("Access keys restored: {}", restore_db.access_keys.len());
        
        // 4. Инвентари
        for inv in &self.inventories {
            inv.restore(store, &mut restore_db)?;
        }
        info!("Inventories restored: {}", restore_db.inventories.len());
        
        // 5. Репозитории
        for repo in &self.repositories {
            repo.restore(store, &mut restore_db)?;
        }
        info!("Repositories restored: {}", restore_db.repositories.len());
        
        // 6. Шаблоны
        for tpl in &self.templates {
            tpl.restore(store, &mut restore_db)?;
        }
        info!("Templates restored: {}", restore_db.templates.len());
        
        // 7. Расписания
        for schedule in &self.schedules {
            schedule.restore(store, &mut restore_db)?;
        }
        info!("Schedules restored: {}", restore_db.schedules.len());
        
        // 8. Интеграции
        for integration in &self.integrations {
            // TODO: Реализовать восстановление интеграций
            warn!("Integration {} not restored (not implemented)", integration.name);
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
            BackupEnvironment { name: "Test".to_string(), json: String::new(), env: None },
            BackupEnvironment { name: "Test".to_string(), json: String::new(), env: None },
        ];
        
        let result = verify_duplicate("Test", &items);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_entry_by_name() {
        let items = vec![
            BackupEnvironment { name: "Test1".to_string(), json: String::new(), env: None },
            BackupEnvironment { name: "Test2".to_string(), json: String::new(), env: None },
        ];
        
        let result = get_entry_by_name(&Some("Test1".to_string()), &items);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Test1");
    }
}
