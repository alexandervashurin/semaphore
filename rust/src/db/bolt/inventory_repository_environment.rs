//! Inventory, Repository, Environment - операции в BoltDB
//!
//! Аналог db/bolt/inventory.go, repository.go, environment.go из Go версии

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::Result;
use crate::models::{AccessKey, InventoryType, RetrieveQueryParams};

// ============================================================================
// Inventory Operations (refs only - main CRUD in inventory.rs)
// ============================================================================

impl BoltStore {
    /// Получает рефереры инвентаря
    pub async fn get_inventory_refs(&self, project_id: i32, inventory_id: i32) -> Result<crate::models::ObjectReferrers> {
        self.get_object_refs(project_id, "inventories", inventory_id).await
    }
}

// ============================================================================
// Repository Operations (refs only - main CRUD in repository.rs)
// ============================================================================

impl BoltStore {
    /// Получает рефереры репозитория
    pub async fn get_repository_refs(&self, project_id: i32, repository_id: i32) -> Result<crate::models::ObjectReferrers> {
        self.get_object_refs(project_id, "repositories", repository_id).await
    }
}

// ============================================================================
// Environment Operations (refs only - main CRUD in environment.rs)
// ============================================================================

impl BoltStore {
    /// Получает рефереры окружения
    pub async fn get_environment_refs(&self, project_id: i32, environment_id: i32) -> Result<crate::models::ObjectReferrers> {
        self.get_object_refs(project_id, "environments", environment_id).await
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
    use std::path::PathBuf;

    fn create_test_bolt_db() -> BoltStore {
        let path = PathBuf::from("/tmp/test_inv_repo_env.db");
        BoltStore::new(path).unwrap()
    }

    // Tests for ref methods would go here
    #[test]
    fn test_placeholder() {
        assert!(true);
    }
}
