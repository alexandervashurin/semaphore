//! Inventory, Repository, Environment - операции в BoltDB
//!
//! Аналог db/bolt/inventory.go, repository.go, environment.go из Go версии

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::Result;
use crate::models::{Inventory, Repository, Environment, AccessKey, InventoryType, RetrieveQueryParams};

// ============================================================================
// Inventory Operations
// ============================================================================

impl BoltStore {
    /// Получает инвентарь по ID
    pub async fn get_inventory(&self, project_id: i32, inventory_id: i32) -> Result<Inventory> {
        self.get_object(project_id, "inventories", inventory_id).await
    }

    /// Получает инвентари проекта
    pub async fn get_inventories(&self, project_id: i32, params: RetrieveQueryParams, types: Vec<InventoryType>) -> Result<Vec<Inventory>> {
        let mut inventories = Vec::new();
        
        let all_inventories = self.get_objects::<Inventory>(project_id, "inventories", params).await?;
        
        for inventory in all_inventories {
            if types.is_empty() {
                inventories.push(inventory);
            } else {
                for t in &types {
                    if inventory.inventory_type == *t {
                        inventories.push(inventory);
                        break;
                    }
                }
            }
        }
        
        Ok(inventories)
    }

    /// Получает рефереры инвентаря
    pub async fn get_inventory_refs(&self, project_id: i32, inventory_id: i32) -> Result<crate::models::ObjectReferrers> {
        self.get_object_refs(project_id, "inventories", inventory_id).await
    }

    /// Удаляет инвентарь
    pub async fn delete_inventory(&self, project_id: i32, inventory_id: i32) -> Result<()> {
        self.delete_object(project_id, "inventories", inventory_id).await
    }

    /// Обновляет инвентарь
    pub async fn update_inventory(&self, inventory: Inventory) -> Result<()> {
        self.update_object(inventory.project_id, "inventories", inventory).await
    }

    /// Создаёт инвентарь
    pub async fn create_inventory(&self, mut inventory: Inventory) -> Result<Inventory> {
        inventory.created = chrono::Utc::now();
        
        let inventory_clone = inventory.clone();
        
        let new_inventory = self.db.update(|tx| {
            let bucket = tx.create_bucket_if_not_exists(b"inventories")?;
            
            let str = serde_json::to_vec(&inventory_clone)?;
            
            let id = bucket.next_sequence()?;
            let id = i64::MAX - id as i64;
            
            let mut inventory_with_id = inventory_clone;
            inventory_with_id.id = id as i32;
            
            let str = serde_json::to_vec(&inventory_with_id)?;
            bucket.put(id.to_be_bytes(), str)?;
            
            Ok(inventory_with_id)
        }).await?;
        
        Ok(new_inventory)
    }
}

// ============================================================================
// Repository Operations
// ============================================================================

impl BoltStore {
    /// Получает репозиторий по ID
    pub async fn get_repository(&self, project_id: i32, repository_id: i32) -> Result<Repository> {
        let mut repository = self.get_object(project_id, "repositories", repository_id).await?;
        
        // Получаем SSH ключ
        if repository.ssh_key_id != 0 {
            repository.ssh_key = Some(Box::new(self.get_access_key(project_id, repository.ssh_key_id).await?));
        }
        
        Ok(repository)
    }

    /// Получает рефереры репозитория
    pub async fn get_repository_refs(&self, project_id: i32, repository_id: i32) -> Result<crate::models::ObjectReferrers> {
        self.get_object_refs(project_id, "repositories", repository_id).await
    }

    /// Получает репозитории проекта
    pub async fn get_repositories(&self, project_id: i32, params: RetrieveQueryParams) -> Result<Vec<Repository>> {
        self.get_objects::<Repository>(project_id, "repositories", params).await
    }

    /// Обновляет репозиторий
    pub async fn update_repository(&self, repository: Repository) -> Result<()> {
        repository.validate()?;
        self.update_object(repository.project_id, "repositories", repository).await
    }

    /// Создаёт репозиторий
    pub async fn create_repository(&self, mut repository: Repository) -> Result<Repository> {
        repository.validate()?;
        repository.created = chrono::Utc::now();
        
        let repository_clone = repository.clone();
        
        let new_repository = self.db.update(|tx| {
            let bucket = tx.create_bucket_if_not_exists(b"repositories")?;
            
            let str = serde_json::to_vec(&repository_clone)?;
            
            let id = bucket.next_sequence()?;
            let id = i64::MAX - id as i64;
            
            let mut repository_with_id = repository_clone;
            repository_with_id.id = id as i32;
            
            let str = serde_json::to_vec(&repository_with_id)?;
            bucket.put(id.to_be_bytes(), str)?;
            
            Ok(repository_with_id)
        }).await?;
        
        Ok(new_repository)
    }

    /// Удаляет репозиторий
    pub async fn delete_repository(&self, project_id: i32, repository_id: i32) -> Result<()> {
        self.delete_object(project_id, "repositories", repository_id).await
    }
}

// ============================================================================
// Environment Operations
// ============================================================================

impl BoltStore {
    /// Получает окружение по ID
    pub async fn get_environment(&self, project_id: i32, environment_id: i32) -> Result<Environment> {
        self.get_object(project_id, "environments", environment_id).await
    }

    /// Получает рефереры окружения
    pub async fn get_environment_refs(&self, project_id: i32, environment_id: i32) -> Result<crate::models::ObjectReferrers> {
        self.get_object_refs(project_id, "environments", environment_id).await
    }

    /// Получает окружения проекта
    pub async fn get_environments(&self, project_id: i32, params: RetrieveQueryParams) -> Result<Vec<Environment>> {
        self.get_objects::<Environment>(project_id, "environments", params).await
    }

    /// Обновляет окружение
    pub async fn update_environment(&self, env: Environment) -> Result<()> {
        env.validate()?;
        self.update_object(env.project_id, "environments", env).await
    }

    /// Создаёт окружение
    pub async fn create_environment(&self, mut env: Environment) -> Result<Environment> {
        env.validate()?;
        env.created = chrono::Utc::now();
        
        let env_clone = env.clone();
        
        let new_env = self.db.update(|tx| {
            let bucket = tx.create_bucket_if_not_exists(b"environments")?;
            
            let str = serde_json::to_vec(&env_clone)?;
            
            let id = bucket.next_sequence()?;
            let id = i64::MAX - id as i64;
            
            let mut env_with_id = env_clone;
            env_with_id.id = id as i32;
            
            let str = serde_json::to_vec(&env_with_id)?;
            bucket.put(id.to_be_bytes(), str)?;
            
            Ok(env_with_id)
        }).await?;
        
        Ok(new_env)
    }

    /// Удаляет окружение
    pub async fn delete_environment(&self, project_id: i32, environment_id: i32) -> Result<()> {
        self.delete_object(project_id, "environments", environment_id).await
    }

    /// Получает секреты окружения
    pub async fn get_environment_secrets(&self, project_id: i32, environment_id: i32) -> Result<Vec<AccessKey>> {
        let mut keys = Vec::new();
        
        let all_keys = self.get_objects::<AccessKey>(project_id, "access_keys", RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            filter: None,
            sort_by: None,
            sort_inverted: false,
        }).await?;
        
        for key in all_keys {
            if let Some(env_id) = key.environment_id {
                if env_id == environment_id {
                    keys.push(key);
                }
            }
        }
        
        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;

    fn create_test_bolt_db() -> BoltStore {
        let path = PathBuf::from("/tmp/test_inv_repo_env.db");
        BoltStore::new(path).unwrap()
    }

    fn create_test_inventory(project_id: i32, name: &str) -> Inventory {
        Inventory {
            id: 0,
            project_id,
            name: name.to_string(),
            inventory_type: InventoryType::Static,
            inventory: "localhost".to_string(),
            ssh_key_id: None,
            become_key_id: None,
            vaults: vec![],
        }
    }

    fn create_test_repository(project_id: i32, name: &str) -> Repository {
        Repository {
            id: 0,
            project_id,
            name: name.to_string(),
            git_url: "https://github.com/test/test.git".to_string(),
            git_branch: "main".to_string(),
            ssh_key_id: None,
        }
    }

    fn create_test_environment(project_id: i32, name: &str) -> Environment {
        Environment {
            id: 0,
            project_id,
            name: name.to_string(),
            json: "{}".to_string(),
            env: None,
            secrets: vec![],
            created: Utc::now(),
        }
    }

    // Inventory Tests
    #[tokio::test]
    async fn test_create_inventory() {
        let db = create_test_bolt_db();
        let inventory = create_test_inventory(1, "Test Inventory");
        
        let result = db.create_inventory(inventory).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_inventory() {
        let db = create_test_bolt_db();
        let inventory = create_test_inventory(1, "Test Inventory");
        let created = db.create_inventory(inventory).await.unwrap();
        
        let retrieved = db.get_inventory(1, created.id).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().name, "Test Inventory");
    }

    // Repository Tests
    #[tokio::test]
    async fn test_create_repository() {
        let db = create_test_bolt_db();
        let repository = create_test_repository(1, "Test Repo");
        
        let result = db.create_repository(repository).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_repository() {
        let db = create_test_bolt_db();
        let repository = create_test_repository(1, "Test Repo");
        let created = db.create_repository(repository).await.unwrap();
        
        let retrieved = db.get_repository(1, created.id).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().name, "Test Repo");
    }

    // Environment Tests
    #[tokio::test]
    async fn test_create_environment() {
        let db = create_test_bolt_db();
        let environment = create_test_environment(1, "Test Env");
        
        let result = db.create_environment(environment).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_environment() {
        let db = create_test_bolt_db();
        let environment = create_test_environment(1, "Test Env");
        let created = db.create_environment(environment).await.unwrap();
        
        let retrieved = db.get_environment(1, created.id).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().name, "Test Env");
    }

    #[tokio::test]
    async fn test_get_environments() {
        let db = create_test_bolt_db();
        
        // Создаём несколько окружений
        for i in 0..5 {
            let environment = create_test_environment(1, &format!("Env {}", i));
            db.create_environment(environment).await.unwrap();
        }
        
        let params = RetrieveQueryParams {
            offset: 0,
            count: Some(10),
            filter: None,
            sort_by: None,
            sort_inverted: false,
        };
        
        let environments = db.get_environments(1, params).await;
        assert!(environments.is_ok());
        assert!(environments.unwrap().len() >= 5);
    }
}
