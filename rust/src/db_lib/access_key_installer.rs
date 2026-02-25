//! AccessKey Installer
//!
//! Модуль для установки ключей доступа (SSH, login/password)
//! Полная замена Go db_lib/AccessKeyInstaller.go

use crate::error::{Error, Result};
use crate::services::ssh_agent::{
    AccessKeyInstallation, AccessKeyRole, AccessKeyType, AccessKey,
    SshKeyData, LoginPasswordData, KeyInstaller,
};
use crate::services::task_logger::TaskLogger;

// ============================================================================
// Типы данных (аналог Go db.AccessKey)
// ============================================================================

/// Тип ключа доступа (аналог Go AccessKeyType)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbAccessKeyType {
    Ssh,
    None,
    LoginPassword,
    String,
}

/// Владелец ключа (аналог Go AccessKeyOwner)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbAccessKeyOwner {
    Environment,
    Variable,
    SecretStorage,
    Shared,
}

/// Тип хранилища (аналог Go AccessKeySourceStorageType)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbAccessKeySourceStorageType {
    Vault,
    Env,
    File,
}

/// Роль ключа доступа (аналог Go AccessKeyRole)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbAccessKeyRole {
    AnsibleUser = 0,
    AnsibleBecomeUser = 1,
    AnsiblePasswordVault = 2,
    Git = 3,
}

impl From<DbAccessKeyRole> for AccessKeyRole {
    fn from(role: DbAccessKeyRole) -> Self {
        match role {
            DbAccessKeyRole::AnsibleUser => AccessKeyRole::AnsibleUser,
            DbAccessKeyRole::AnsibleBecomeUser => AccessKeyRole::AnsibleBecomeUser,
            DbAccessKeyRole::AnsiblePasswordVault => AccessKeyRole::AnsiblePasswordVault,
            DbAccessKeyRole::Git => AccessKeyRole::Git,
        }
    }
}

/// Данные SSH ключа (аналог Go SshKey)
#[derive(Debug, Clone)]
pub struct DbSshKey {
    pub login: String,
    pub passphrase: String,
    pub private_key: String,
}

/// Данные логина/пароля (аналог Go LoginPassword)
#[derive(Debug, Clone)]
pub struct DbLoginPassword {
    pub login: String,
    pub password: String,
}

/// Ключ доступа (аналог Go db.AccessKey)
#[derive(Debug, Clone)]
pub struct DbAccessKey {
    pub id: i32,
    pub name: String,
    pub key_type: DbAccessKeyType,
    pub project_id: Option<i32>,
    pub secret: Option<String>,
    pub plain: Option<String>,
    pub string_value: Option<String>,
    pub login_password: Option<DbLoginPassword>,
    pub ssh_key: Option<DbSshKey>,
    pub override_secret: bool,
    pub storage_id: Option<i32>,
    pub environment_id: Option<i32>,
    pub user_id: Option<i32>,
    pub empty: bool,
    pub owner: DbAccessKeyOwner,
    pub source_storage_id: Option<i32>,
    pub source_storage_key: Option<String>,
    pub source_storage_type: Option<DbAccessKeySourceStorageType>,
}

impl DbAccessKey {
    /// Проверяет, пуст ли ключ
    pub fn is_empty(&self) -> bool {
        if self.key_type == DbAccessKeyType::None {
            return false;
        }

        if let Some(storage_type) = self.source_storage_type {
            match storage_type {
                DbAccessKeySourceStorageType::Env | DbAccessKeySourceStorageType::File => {
                    return self.source_storage_key.as_ref().map_or(true, |k| k.is_empty());
                }
                DbAccessKeySourceStorageType::Vault => {
                    return self.source_storage_id.is_none();
                }
            }
        }

        if let Some(secret) = &self.secret {
            if !secret.is_empty() {
                return false;
            }
        }

        match self.key_type {
            DbAccessKeyType::String => self.string_value.as_ref().map_or(true, |s| s.is_empty()),
            DbAccessKeyType::Ssh => self.ssh_key.as_ref().map_or(true, |k| k.private_key.is_empty()),
            DbAccessKeyType::LoginPassword => {
                self.login_password.as_ref().map_or(true, |k| k.password.is_empty())
            }
            DbAccessKeyType::None => true,
        }
    }

    /// Валидация ключа
    pub fn validate(&self, validate_secret_fields: bool) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::Validation("Name cannot be empty".to_string()));
        }

        if validate_secret_fields {
            match self.key_type {
                DbAccessKeyType::Ssh => {
                    if let Some(ref ssh) = self.ssh_key {
                        if ssh.private_key.is_empty() {
                            return Err(Error::Validation(
                                "Private key cannot be empty".to_string(),
                            ));
                        }
                    }
                }
                DbAccessKeyType::LoginPassword => {
                    if let Some(ref lp) = self.login_password {
                        if lp.password.is_empty() {
                            return Err(Error::Validation(
                                "Password cannot be empty".to_string(),
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Конвертация в AccessKey для ssh_agent
    pub fn to_ssh_access_key(&self) -> Result<AccessKey> {
        let project_id = self.project_id.map(|id| id as i64);

        match self.key_type {
            DbAccessKeyType::Ssh => {
                let ssh = self.ssh_key.as_ref().ok_or_else(|| {
                    Error::Validation("SSH key data missing".to_string())
                })?;

                Ok(AccessKey::new_ssh(
                    self.id as i64,
                    ssh.private_key.clone(),
                    ssh.passphrase.clone(),
                    ssh.login.clone(),
                    project_id,
                ))
            }
            DbAccessKeyType::LoginPassword => {
                let lp = self.login_password.as_ref().ok_or_else(|| {
                    Error::Validation("Login/Password data missing".to_string())
                })?;

                Ok(AccessKey::new_login_password(
                    self.id as i64,
                    lp.login.clone(),
                    lp.password.clone(),
                    project_id,
                ))
            }
            DbAccessKeyType::None => Ok(AccessKey::new_none(self.id as i64, project_id)),
            DbAccessKeyType::String => Err(Error::Validation(
                "String key type not supported for SSH operations".to_string(),
            )),
        }
    }
}

// ============================================================================
// AccessKeyInstaller trait (аналог Go AccessKeyInstaller interface)
// ============================================================================

/// Трейт для установщика ключей доступа
pub trait AccessKeyInstallerTrait: Send + Sync {
    /// Устанавливает ключ доступа
    fn install(
        &self,
        key: &DbAccessKey,
        usage: DbAccessKeyRole,
        logger: &dyn TaskLogger,
    ) -> Result<AccessKeyInstallation>;
}

/// Реализация установщика
pub struct AccessKeyInstallerImpl {
    key_installer: KeyInstaller,
}

impl AccessKeyInstallerImpl {
    pub fn new() -> Self {
        Self {
            key_installer: KeyInstaller::new(),
        }
    }
}

impl Default for AccessKeyInstallerImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessKeyInstallerTrait for AccessKeyInstallerImpl {
    fn install(
        &self,
        key: &DbAccessKey,
        usage: DbAccessKeyRole,
        logger: &dyn TaskLogger,
    ) -> Result<AccessKeyInstallation> {
        // Конвертируем DbAccessKeyRole в AccessKeyRole
        let role: AccessKeyRole = usage.into();

        // Конвертируем DbAccessKey в AccessKey
        let access_key = key.to_ssh_access_key()?;

        // Устанавливаем ключ через KeyInstaller
        self.key_installer.install(&access_key, role, logger)
    }
}

// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::task_logger::BasicLogger;

    #[test]
    fn test_db_access_key_type_conversion() {
        let ssh_key = DbAccessKey {
            id: 1,
            name: "Test Key".to_string(),
            key_type: DbAccessKeyType::Ssh,
            project_id: Some(1),
            secret: None,
            plain: None,
            string_value: None,
            login_password: None,
            ssh_key: Some(DbSshKey {
                login: "user".to_string(),
                passphrase: "".to_string(),
                private_key: "test_key".to_string(),
            }),
            override_secret: false,
            storage_id: None,
            environment_id: None,
            user_id: None,
            empty: false,
            owner: DbAccessKeyOwner::Shared,
            source_storage_id: None,
            source_storage_key: None,
            source_storage_type: None,
        };

        assert!(!ssh_key.is_empty());
        assert_eq!(ssh_key.key_type, DbAccessKeyType::Ssh);
    }

    #[test]
    fn test_db_access_key_validate() {
        let key = DbAccessKey {
            id: 1,
            name: "".to_string(),
            key_type: DbAccessKeyType::Ssh,
            project_id: None,
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
            owner: DbAccessKeyOwner::Shared,
            source_storage_id: None,
            source_storage_key: None,
            source_storage_type: None,
        };

        assert!(key.validate(true).is_err());
    }

    #[test]
    fn test_db_access_key_to_ssh_access_key() {
        let key = DbAccessKey {
            id: 1,
            name: "Test".to_string(),
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
            owner: DbAccessKeyOwner::Shared,
            source_storage_id: None,
            source_storage_key: None,
            source_storage_type: None,
        };

        let ssh_key = key.to_ssh_access_key().unwrap();
        assert_eq!(ssh_key.id, 1);
        assert_eq!(ssh_key.key_type, AccessKeyType::Ssh);
    }

    #[test]
    fn test_access_key_installer_install_git() {
        let installer = AccessKeyInstallerImpl::new();
        let logger = BasicLogger::new();

        let key = DbAccessKey {
            id: 1,
            name: "Test Git Key".to_string(),
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
            owner: DbAccessKeyOwner::Shared,
            source_storage_id: None,
            source_storage_key: None,
            source_storage_type: None,
        };

        let result = installer.install(&key, DbAccessKeyRole::Git, &logger);
        assert!(result.is_ok());

        let installation = result.unwrap();
        assert!(installation.ssh_agent.is_some());
    }

    #[test]
    fn test_db_access_key_role_conversion() {
        assert_eq!(
            AccessKeyRole::from(DbAccessKeyRole::Git),
            AccessKeyRole::Git
        );
        assert_eq!(
            AccessKeyRole::from(DbAccessKeyRole::AnsibleUser),
            AccessKeyRole::AnsibleUser
        );
    }
}
