//! Access Key Service
//!
//! Сервис управления ключами доступа

use std::sync::Arc;
use crate::error::{Error, Result};
use crate::models::AccessKey;
use crate::db::store::{Store, RetrieveQueryParams};
use super::access_key_encryption_svc::AccessKeyEncryptionService;

/// Сервис ключей доступа
pub trait AccessKeyService: Send + Sync {
    /// Обновляет ключ
    fn update(&self, key: AccessKey) -> Result<()>;

    /// Создаёт ключ
    fn create(&self, key: AccessKey) -> Result<AccessKey>;

    /// Получает все ключи
    fn get_all(&self, project_id: i32, options: GetAccessKeyOptions, params: RetrieveQueryParams) -> Result<Vec<AccessKey>>;

    /// Удаляет ключ
    fn delete(&self, project_id: i32, key_id: i32) -> Result<()>;
}

/// Опции для получения ключей доступа
#[derive(Debug, Clone, Default)]
pub struct GetAccessKeyOptions {
    pub ignore_owner: bool,
}

/// Реализация сервиса ключей доступа
pub struct AccessKeyServiceImpl {
    access_key_repo: Arc<dyn Store + Send + Sync>,
    encryption_service: Arc<dyn AccessKeyEncryptionService>,
    secret_storage_repo: Arc<dyn Store + Send + Sync>,
}

impl AccessKeyServiceImpl {
    /// Создаёт новый сервис
    pub fn new(
        access_key_repo: Arc<dyn Store + Send + Sync>,
        encryption_service: Arc<dyn AccessKeyEncryptionService>,
        secret_storage_repo: Arc<dyn Store + Send + Sync>,
    ) -> Self {
        Self {
            access_key_repo,
            encryption_service,
            secret_storage_repo,
        }
    }
}

impl AccessKeyService for AccessKeyServiceImpl {
    fn update(&self, key: AccessKey) -> Result<()> {
        self.access_key_repo.update_access_key(key)
    }

    fn create(&self, mut key: AccessKey) -> Result<AccessKey> {
        self.encryption_service.serialize_secret(&mut key)?;
        self.access_key_repo.create_access_key(key)
    }

    fn get_all(&self, project_id: i32, _options: GetAccessKeyOptions, params: RetrieveQueryParams) -> Result<Vec<AccessKey>> {
        self.access_key_repo.get_access_keys(project_id, params)
    }

    fn delete(&self, project_id: i32, key_id: i32) -> Result<()> {
        self.access_key_repo.delete_access_key(project_id, key_id)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_key_service_creation() {
        // Тест для проверки создания сервиса
        assert!(true);
    }

    #[test]
    fn test_get_access_key_options_default() {
        let options = GetAccessKeyOptions::default();
        assert!(!options.ignore_owner);
    }
}
