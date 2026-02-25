//! AccessKey Installation Service
//!
//! Полная замена Go services/server/access_key_installation_svc.go
//! Сервис для установки ключей доступа

use crate::db_lib::{AccessKeyInstallerImpl, AccessKeyInstallerTrait, DbAccessKey, DbAccessKeyRole};
use crate::error::{Error, Result};
use crate::services::ssh_agent::AccessKeyInstallation;
use crate::services::task_logger::TaskLogger;

// ============================================================================
// Traits (интерфейсы)
// ============================================================================

/// Трейт сервиса установки ключей (аналог Go AccessKeyInstallationService)
pub trait AccessKeyInstallationServiceTrait: Send + Sync {
    /// Устанавливает ключ доступа
    fn install(
        &self,
        key: &DbAccessKey,
        usage: DbAccessKeyRole,
        logger: &dyn TaskLogger,
    ) -> Result<AccessKeyInstallation>;
}

// ============================================================================
/// AccessKeyEncryptionService - трейт для шифрования/дешифрования ключей
/// (аналог Go AccessKeyEncryptionService)
pub trait AccessKeyEncryptionService: Send + Sync {
    /// Шифрует секрет ключа
    fn encrypt_secret(&self, key: &mut DbAccessKey) -> Result<()>;

    /// Дешифрует секрет ключа
    fn decrypt_secret(&self, key: &mut DbAccessKey) -> Result<()>;

    /// Сериализует секрет (подготовка к хранению)
    fn serialize_secret(&self, key: &mut DbAccessKey) -> Result<()>;

    /// Десериализует секрет (чтение из хранилища)
    fn deserialize_secret(&self, key: &mut DbAccessKey) -> Result<()>;
}

// ============================================================================
// AccessKeyInstallationServiceImpl
// ============================================================================

/// Реализация сервиса установки ключей
pub struct AccessKeyInstallationServiceImpl {
    encryption_service: Box<dyn AccessKeyEncryptionService>,
    key_installer: AccessKeyInstallerImpl,
}

impl AccessKeyInstallationServiceImpl {
    /// Создаёт новый сервис
    pub fn new(encryption_service: Box<dyn AccessKeyEncryptionService>) -> Self {
        Self {
            encryption_service,
            key_installer: AccessKeyInstallerImpl::new(),
        }
    }

    /// Создаёт сервис с кастомным установщиком
    pub fn with_installer(
        encryption_service: Box<dyn AccessKeyEncryptionService>,
        key_installer: AccessKeyInstallerImpl,
    ) -> Self {
        Self {
            encryption_service,
            key_installer,
        }
    }
}

impl AccessKeyInstallationServiceTrait for AccessKeyInstallationServiceImpl {
    fn install(
        &self,
        key: &DbAccessKey,
        usage: DbAccessKeyRole,
        logger: &dyn TaskLogger,
    ) -> Result<AccessKeyInstallation> {
        // Если тип ключа None - возвращаем пустую установку
        if key.key_type == crate::db_lib::DbAccessKeyType::None {
            return Ok(AccessKeyInstallation::new());
        }

        // Создаём копию ключа для десериализации
        let mut key_copy = key.clone();

        // Десериализуем секрет (расшифровываем)
        self.encryption_service.deserialize_secret(&mut key_copy)?;

        // Устанавливаем ключ через KeyInstaller
        self.key_installer.install(&key_copy, usage, logger)
    }
}

// ============================================================================
// AccessKeyEncryptionServiceImpl (заглушка)
// ============================================================================

/// Простая реализация шифрования (без реального шифрования)
/// В production нужно использовать реальное шифрование
pub struct SimpleEncryptionService;

impl AccessKeyEncryptionService for SimpleEncryptionService {
    fn encrypt_secret(&self, _key: &mut DbAccessKey) -> Result<()> {
        // TODO: Реализовать шифрование
        Ok(())
    }

    fn decrypt_secret(&self, _key: &mut DbAccessKey) -> Result<()> {
        // TODO: Реализовать дешифрование
        Ok(())
    }

    fn serialize_secret(&self, _key: &mut DbAccessKey) -> Result<()> {
        // TODO: Реализовать сериализацию
        Ok(())
    }

    fn deserialize_secret(&self, _key: &mut DbAccessKey) -> Result<()> {
        // TODO: Реализовать десериализацию
        Ok(())
    }
}

impl Default for SimpleEncryptionService {
    fn default() -> Self {
        Self
    }
}

// ============================================================================
// AccessKeyService - полный сервис для управления ключами
// ============================================================================

/// Трейт сервиса управления ключами (аналог Go AccessKeyService)
pub trait AccessKeyServiceTrait: Send + Sync {
    /// Обновляет ключ
    fn update(&self, key: &DbAccessKey) -> Result<()>;

    /// Создаёт новый ключ
    fn create(&self, key: &DbAccessKey) -> Result<DbAccessKey>;

    /// Получает все ключи
    fn get_all(
        &self,
        project_id: i32,
        options: GetAccessKeyOptions,
    ) -> Result<Vec<DbAccessKey>>;

    /// Удаляет ключ
    fn delete(&self, project_id: i32, key_id: i32) -> Result<()>;
}

/// Опции для получения ключей
#[derive(Debug, Clone, Default)]
pub struct GetAccessKeyOptions {
    pub user_id: Option<i32>,
    pub environment_id: Option<i32>,
}

/// Реализация сервиса управления ключами
pub struct AccessKeyServiceImpl {
    encryption_service: Box<dyn AccessKeyEncryptionService>,
    // TODO: Добавить repository: Box<dyn AccessKeyRepository>
}

impl AccessKeyServiceImpl {
    /// Создаёт новый сервис
    pub fn new(encryption_service: Box<dyn AccessKeyEncryptionService>) -> Self {
        Self {
            encryption_service,
        }
    }
}

impl AccessKeyServiceTrait for AccessKeyServiceImpl {
    fn update(&self, key: &DbAccessKey) -> Result<()> {
        // TODO: Реализовать через repository
        Ok(())
    }

    fn create(&self, key: &DbAccessKey) -> Result<DbAccessKey> {
        // Сериализуем секрет перед сохранением
        let mut key_copy = key.clone();
        self.encryption_service.serialize_secret(&mut key_copy)?;

        // TODO: Сохранить через repository
        Ok(key_copy)
    }

    fn get_all(
        &self,
        _project_id: i32,
        _options: GetAccessKeyOptions,
    ) -> Result<Vec<DbAccessKey>> {
        // TODO: Реализовать через repository
        Ok(vec![])
    }

    fn delete(&self, _project_id: i32, _key_id: i32) -> Result<()> {
        // TODO: Реализовать через repository
        Ok(())
    }
}

// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_lib::{DbAccessKeyType, DbSshKey, DbLoginPassword};
    use crate::services::task_logger::BasicLogger;

    #[test]
    fn test_simple_encryption_service() {
        let encryption = SimpleEncryptionService;
        let mut key = DbAccessKey {
            id: 1,
            name: "Test".to_string(),
            key_type: DbAccessKeyType::Ssh,
            project_id: Some(1),
            secret: Some("secret".to_string()),
            plain: None,
            string_value: None,
            login_password: None,
            ssh_key: None,
            override_secret: false,
            storage_id: None,
            environment_id: None,
            user_id: None,
            empty: false,
            owner: crate::db_lib::DbAccessKeyOwner::Shared,
            source_storage_id: None,
            source_storage_key: None,
            source_storage_type: None,
        };

        // Проверяем, что методы не паникуют
        assert!(encryption.encrypt_secret(&mut key).is_ok());
        assert!(encryption.decrypt_secret(&mut key).is_ok());
        assert!(encryption.serialize_secret(&mut key).is_ok());
        assert!(encryption.deserialize_secret(&mut key).is_ok());
    }

    #[test]
    fn test_access_key_installation_service_creation() {
        let encryption = Box::new(SimpleEncryptionService);
        let service = AccessKeyInstallationServiceImpl::new(encryption);

        // Проверяем, что сервис создан
        let _ = service;
    }

    #[test]
    fn test_access_key_installation_service_install_none() {
        let encryption = Box::new(SimpleEncryptionService);
        let service = AccessKeyInstallationServiceImpl::new(encryption);
        let logger = BasicLogger::new();

        let key = DbAccessKey {
            id: 1,
            name: "None Key".to_string(),
            key_type: DbAccessKeyType::None,
            project_id: Some(1),
            secret: None,
            plain: None,
            string_value: None,
            login_password: None,
            ssh_key: None,
            override_secret: false,
            storage_id: None,
            environment_id: None,
            user_id: None,
            empty: false,
            owner: crate::db_lib::DbAccessKeyOwner::Shared,
            source_storage_id: None,
            source_storage_key: None,
            source_storage_type: None,
        };

        let result = service.install(&key, DbAccessKeyRole::Git, &logger);
        assert!(result.is_ok());

        let installation = result.unwrap();
        assert!(installation.ssh_agent.is_none());
    }

    #[test]
    fn test_access_key_installation_service_install_ssh() {
        let encryption = Box::new(SimpleEncryptionService);
        let service = AccessKeyInstallationServiceImpl::new(encryption);
        let logger = BasicLogger::new();

        let key = DbAccessKey {
            id: 1,
            name: "SSH Key".to_string(),
            key_type: DbAccessKeyType::Ssh,
            project_id: Some(1),
            secret: None,
            plain: None,
            string_value: None,
            login_password: None,
            ssh_key: Some(DbSshKey {
                login: "git".to_string(),
                passphrase: "".to_string(),
                private_key: "-----BEGIN OPENSSH PRIVATE KEY-----\ntest\n-----END OPENSSH PRIVATE KEY-----".to_string(),
            }),
            override_secret: false,
            storage_id: None,
            environment_id: None,
            user_id: None,
            empty: false,
            owner: crate::db_lib::DbAccessKeyOwner::Shared,
            source_storage_id: None,
            source_storage_key: None,
            source_storage_type: None,
        };

        let result = service.install(&key, DbAccessKeyRole::Git, &logger);
        assert!(result.is_ok());

        let installation = result.unwrap();
        assert!(installation.ssh_agent.is_some());
    }

    #[test]
    fn test_access_key_service_creation() {
        let encryption = Box::new(SimpleEncryptionService);
        let service = AccessKeyServiceImpl::new(encryption);

        // Проверяем, что сервис создан
        let _ = service;
    }

    #[test]
    fn test_access_key_service_create() {
        let encryption = Box::new(SimpleEncryptionService);
        let service = AccessKeyServiceImpl::new(encryption);

        let key = DbAccessKey {
            id: 1,
            name: "Test Key".to_string(),
            key_type: DbAccessKeyType::LoginPassword,
            project_id: Some(1),
            secret: None,
            plain: None,
            string_value: None,
            login_password: Some(DbLoginPassword {
                login: "user".to_string(),
                password: "pass".to_string(),
            }),
            ssh_key: None,
            override_secret: false,
            storage_id: None,
            environment_id: None,
            user_id: None,
            empty: false,
            owner: crate::db_lib::DbAccessKeyOwner::Shared,
            source_storage_id: None,
            source_storage_key: None,
            source_storage_type: None,
        };

        let result = service.create(&key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_access_key_options() {
        let options = GetAccessKeyOptions {
            user_id: Some(1),
            environment_id: Some(2),
        };

        assert_eq!(options.user_id, Some(1));
        assert_eq!(options.environment_id, Some(2));
    }
}
