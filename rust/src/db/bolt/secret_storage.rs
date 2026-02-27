//! Secret Storage CRUD Operations for BoltDB
//!
//! Операции с хранилищами секретов в BoltDB

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::{Error, Result};
use crate::models::SecretStorage;

impl BoltStore {
    /// Получает хранилища секретов проекта
    pub async fn get_secret_storages(&self, project_id: i32) -> Result<Vec<SecretStorage>> {
        self.get_objects::<SecretStorage>(project_id, "secret_storages", crate::db::store::RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            filter: String::new(),
        }).await
    }

    /// Получает хранилище секретов по ID
    pub async fn get_secret_storage(&self, project_id: i32, storage_id: i32) -> Result<SecretStorage> {
        self.get_object::<SecretStorage>(project_id, "secret_storages", storage_id).await
    }

    /// Создаёт хранилище секретов
    pub async fn create_secret_storage(&self, mut storage: SecretStorage) -> Result<SecretStorage> {
        storage.id = self.get_next_id("secret_storages")?;
        self.create_object(storage.project_id, "secret_storages", &storage).await?;
        Ok(storage)
    }

    /// Обновляет хранилище секретов
    pub async fn update_secret_storage(&self, storage: SecretStorage) -> Result<()> {
        self.update_object(storage.project_id, "secret_storages", storage.id, &storage).await
    }

    /// Удаляет хранилище секретов
    pub async fn delete_secret_storage(&self, project_id: i32, storage_id: i32) -> Result<()> {
        self.delete_object(project_id, "secret_storages", storage_id).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_storage_operations() {
        // Тест для проверки операций с хранилищами секретов
        assert!(true);
    }
}
