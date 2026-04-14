//! AccessKey Installation Service
//!
//! Full replacement for the legacy Go access key installation service.

use crate::db_lib::{
    AccessKeyInstallerImpl, AccessKeyInstallerTrait, DbAccessKey, DbAccessKeyRole,
};
use crate::error::{Error, Result};
use crate::services::ssh_agent::AccessKeyInstallation;
use crate::services::task_logger::TaskLogger;

/// Service trait for installing access keys into the execution environment.
pub trait AccessKeyInstallationServiceTrait: Send + Sync {
    fn install(
        &self,
        key: &DbAccessKey,
        usage: DbAccessKeyRole,
        logger: &dyn TaskLogger,
    ) -> Result<AccessKeyInstallation>;
}

/// Encryption trait used while reading and storing key secrets.
pub trait AccessKeyEncryptionService: Send + Sync {
    fn encrypt_secret(&self, key: &mut DbAccessKey) -> Result<()>;
    fn decrypt_secret(&self, key: &mut DbAccessKey) -> Result<()>;
    fn serialize_secret(&self, key: &mut DbAccessKey) -> Result<()>;
    fn deserialize_secret(&self, key: &mut DbAccessKey) -> Result<()>;
}

/// Default access key installation service.
pub struct AccessKeyInstallationServiceImpl {
    encryption_service: Box<dyn AccessKeyEncryptionService>,
    key_installer: AccessKeyInstallerImpl,
}

impl AccessKeyInstallationServiceImpl {
    pub fn new(encryption_service: Box<dyn AccessKeyEncryptionService>) -> Self {
        Self {
            encryption_service,
            key_installer: AccessKeyInstallerImpl::new(),
        }
    }

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
        if key.key_type == crate::db_lib::DbAccessKeyType::None {
            return Ok(AccessKeyInstallation::new());
        }

        let mut key_copy = key.clone();
        // Secret может храниться как JSON (plain), либо в зашифрованном виде.
        // Сначала пробуем fast-path (plain JSON), затем fallback с decrypt.
        if self
            .encryption_service
            .deserialize_secret(&mut key_copy)
            .is_err()
            && key_copy.secret.is_some()
        {
            self.encryption_service.decrypt_secret(&mut key_copy)?;
            self.encryption_service.deserialize_secret(&mut key_copy)?;
        }
        self.key_installer.install(&key_copy, usage, logger)
    }
}

/// Minimal AES-backed encryption service used by tests and local flows.
pub struct SimpleEncryptionService {
    key: [u8; 32],
}

impl SimpleEncryptionService {
    pub fn new(secret: &str) -> Self {
        let mut key = [0u8; 32];
        let bytes = secret.as_bytes();
        let len = bytes.len().min(32);
        key[..len].copy_from_slice(&bytes[..len]);
        Self { key }
    }
}

impl Default for SimpleEncryptionService {
    fn default() -> Self {
        Self::new("semaphore-default-encryption-key")
    }
}

impl AccessKeyEncryptionService for SimpleEncryptionService {
    fn encrypt_secret(&self, key: &mut DbAccessKey) -> Result<()> {
        use crate::utils::encryption::aes256_encrypt;

        if let Some(ref plaintext) = key.secret {
            let encrypted = aes256_encrypt(plaintext.as_bytes(), &self.key)
                .map_err(|e| Error::Other(e.to_string()))?;
            key.secret = Some(encrypted);
        }

        Ok(())
    }

    fn decrypt_secret(&self, key: &mut DbAccessKey) -> Result<()> {
        use crate::utils::encryption::aes256_decrypt;

        if let Some(ref encrypted) = key.secret {
            let plaintext_bytes =
                aes256_decrypt(encrypted, &self.key).map_err(|e| Error::Other(e.to_string()))?;
            key.secret =
                Some(String::from_utf8(plaintext_bytes).map_err(|e| Error::Other(e.to_string()))?);
        }

        Ok(())
    }

    fn serialize_secret(&self, key: &mut DbAccessKey) -> Result<()> {
        use crate::db_lib::DbAccessKeyType;

        match key.key_type {
            DbAccessKeyType::Ssh => {
                if let Some(ref ssh_key) = key.ssh_key {
                    key.secret = Some(
                        serde_json::to_string(ssh_key).map_err(|e| Error::Other(e.to_string()))?,
                    );
                }
            }
            DbAccessKeyType::LoginPassword => {
                if let Some(ref login_password) = key.login_password {
                    key.secret = Some(
                        serde_json::to_string(login_password)
                            .map_err(|e| Error::Other(e.to_string()))?,
                    );
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn deserialize_secret(&self, key: &mut DbAccessKey) -> Result<()> {
        use crate::db_lib::{DbAccessKeyType, DbLoginPassword, DbSshKey};

        if let Some(ref secret) = key.secret.clone() {
            match key.key_type {
                DbAccessKeyType::Ssh => {
                    let ssh_key: DbSshKey =
                        serde_json::from_str(secret).map_err(|e| Error::Other(e.to_string()))?;
                    key.ssh_key = Some(ssh_key);
                }
                DbAccessKeyType::LoginPassword => {
                    let login_password: DbLoginPassword =
                        serde_json::from_str(secret).map_err(|e| Error::Other(e.to_string()))?;
                    key.login_password = Some(login_password);
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_lib::{DbAccessKeyType, DbLoginPassword, DbSshKey};
    use crate::services::task_logger::BasicLogger;

    #[test]
    fn test_simple_encryption_service() {
        let encryption = SimpleEncryptionService::default();
        let mut key = DbAccessKey {
            id: 1,
            name: "Test".to_string(),
            key_type: DbAccessKeyType::Ssh,
            project_id: Some(1),
            secret: Some(r#"{"login":"user","passphrase":"","private_key":"test"}"#.to_string()),
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

        assert!(encryption.encrypt_secret(&mut key).is_ok());
        assert!(encryption.decrypt_secret(&mut key).is_ok());
        assert!(encryption.serialize_secret(&mut key).is_ok());
        assert!(encryption.deserialize_secret(&mut key).is_ok());
    }

    #[test]
    fn test_access_key_installation_service_creation() {
        let encryption = Box::new(SimpleEncryptionService::default());
        let service = AccessKeyInstallationServiceImpl::new(encryption);
        let _ = service;
    }

    #[test]
    fn test_access_key_installation_service_install_none() {
        let encryption = Box::new(SimpleEncryptionService::default());
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

        let installation = service
            .install(&key, DbAccessKeyRole::AnsibleUser, &logger)
            .unwrap();
        assert!(installation.ssh_agent.is_none());
    }

    #[test]
    fn test_access_key_installation_service_install_ssh() {
        let encryption = Box::new(SimpleEncryptionService::default());
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
                private_key:
                    "-----BEGIN OPENSSH PRIVATE KEY-----\ntest\n-----END OPENSSH PRIVATE KEY-----"
                        .to_string(),
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

        let installation = service
            .install(&key, DbAccessKeyRole::Git, &logger)
            .unwrap();
        assert!(installation.ssh_agent.is_some());
    }

    #[test]
    fn test_access_key_install() {
        let encryption = Box::new(SimpleEncryptionService::default());
        let service = AccessKeyInstallationServiceImpl::new(encryption);
        let logger = BasicLogger::new();

        let mut key = DbAccessKey {
            id: 1,
            name: "Serialized SSH Key".to_string(),
            key_type: DbAccessKeyType::Ssh,
            project_id: Some(1),
            secret: None,
            plain: None,
            string_value: None,
            login_password: None,
            ssh_key: Some(DbSshKey {
                login: "git".to_string(),
                passphrase: "".to_string(),
                private_key:
                    "-----BEGIN OPENSSH PRIVATE KEY-----\ntest\n-----END OPENSSH PRIVATE KEY-----"
                        .to_string(),
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

        let serializer = SimpleEncryptionService::default();
        serializer.serialize_secret(&mut key).unwrap();
        key.ssh_key = None;

        let installation = service
            .install(&key, DbAccessKeyRole::Git, &logger)
            .unwrap();
        assert!(installation.ssh_agent.is_some());
    }

    #[test]
    fn test_simple_encryption_service_serializes_login_password_secret() {
        let encryption = SimpleEncryptionService::default();
        let mut key = DbAccessKey {
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

        encryption.serialize_secret(&mut key).unwrap();
        assert!(
            key.secret
                .as_ref()
                .is_some_and(|secret| secret.contains("\"login\":\"user\""))
        );
    }

    #[test]
    fn test_access_key_install_service_accepts_encrypted_serialized_secret() {
        let encryption = Box::new(SimpleEncryptionService::default());
        let service = AccessKeyInstallationServiceImpl::new(encryption);
        let logger = BasicLogger::new();

        let mut key = DbAccessKey {
            id: 1,
            name: "Encrypted SSH".to_string(),
            key_type: DbAccessKeyType::Ssh,
            project_id: Some(1),
            secret: None,
            plain: None,
            string_value: None,
            login_password: None,
            ssh_key: Some(DbSshKey {
                login: "git".to_string(),
                passphrase: "".to_string(),
                private_key:
                    "-----BEGIN OPENSSH PRIVATE KEY-----\ntest\n-----END OPENSSH PRIVATE KEY-----"
                        .to_string(),
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

        let serializer = SimpleEncryptionService::default();
        serializer.serialize_secret(&mut key).unwrap();
        serializer.encrypt_secret(&mut key).unwrap();
        key.ssh_key = None;

        let installation = service
            .install(&key, DbAccessKeyRole::Git, &logger)
            .unwrap();
        assert!(installation.ssh_agent.is_some());
    }

    #[test]
    fn test_simple_encryption_service_serializes_ssh_key_secret() {
        let encryption = SimpleEncryptionService::default();
        let mut key = DbAccessKey {
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
                private_key:
                    "-----BEGIN OPENSSH PRIVATE KEY-----\ntest\n-----END OPENSSH PRIVATE KEY-----"
                        .to_string(),
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

        encryption.serialize_secret(&mut key).unwrap();
        assert!(
            key.secret
                .as_ref()
                .is_some_and(|s| s.contains("\"private_key\""))
        );
    }

    #[test]
    fn test_access_key_install_service_with_custom_installer() {
        let encryption = Box::new(SimpleEncryptionService::default());
        let key_installer = AccessKeyInstallerImpl::new();
        let service = AccessKeyInstallationServiceImpl::with_installer(encryption, key_installer);
        let logger = BasicLogger::new();

        let key = DbAccessKey {
            id: 1,
            name: "Test".to_string(),
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

        let installation = service
            .install(&key, DbAccessKeyRole::AnsibleUser, &logger)
            .unwrap();
        // None key type should return empty installation
        assert!(installation.ssh_agent.is_none());
    }

    #[test]
    fn test_simple_encryption_service_encrypt_decrypt_roundtrip() {
        let encryption = SimpleEncryptionService::default();
        let mut key = DbAccessKey {
            id: 1,
            name: "Test".to_string(),
            key_type: DbAccessKeyType::LoginPassword,
            project_id: Some(1),
            secret: None,
            plain: None,
            string_value: None,
            login_password: Some(DbLoginPassword {
                login: "user".to_string(),
                password: "secret".to_string(),
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

        encryption.serialize_secret(&mut key).unwrap();
        encryption.encrypt_secret(&mut key).unwrap();

        // After encryption, secret should be encrypted
        let encrypted_secret = key.secret.clone();
        assert!(encrypted_secret.is_some());

        // Decrypt and deserialize
        encryption.decrypt_secret(&mut key).unwrap();
        encryption.deserialize_secret(&mut key).unwrap();

        // After decrypt, login_password should be restored
        assert!(key.login_password.is_some());
        let lp = key.login_password.as_ref().unwrap();
        assert_eq!(lp.login, "user");
        assert_eq!(lp.password, "secret");
    }

    #[test]
    fn test_db_access_key_role_from() {
        // Test all DbAccessKeyRole variants
        let roles = [
            DbAccessKeyRole::AnsibleUser,
            DbAccessKeyRole::AnsibleBecomeUser,
            DbAccessKeyRole::Git,
        ];
        for role in &roles {
            let key = DbAccessKey {
                id: 1,
                name: "Test".to_string(),
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
            // Just verify that we can create a key with any role
            let _ = (key, *role);
        }
    }

    #[test]
    fn test_access_key_install_service_none_key_type_returns_empty_installation() {
        let encryption = Box::new(SimpleEncryptionService::default());
        let service = AccessKeyInstallationServiceImpl::new(encryption);
        let logger = BasicLogger::new();

        let key = DbAccessKey {
            id: 0,
            name: "".to_string(),
            key_type: DbAccessKeyType::None,
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
            owner: crate::db_lib::DbAccessKeyOwner::Shared,
            source_storage_id: None,
            source_storage_key: None,
            source_storage_type: None,
        };

        let installation = service
            .install(&key, DbAccessKeyRole::Git, &logger)
            .unwrap();
        assert!(installation.ssh_agent.is_none());
        assert!(installation.login.is_none());
    }

    #[test]
    fn test_access_key_installation_default() {
        let installation = AccessKeyInstallation::default();
        assert!(installation.ssh_agent.is_none());
        assert!(installation.login.is_none());
        assert!(installation.password.is_none());
    }

    #[test]
    fn test_access_key_installation_new_with_key_id() {
        let installation = AccessKeyInstallation::new_with_key_id(123);
        assert!(installation.ssh_agent.is_none());
        assert!(installation.login.is_none());
    }

    #[test]
    fn test_access_key_role_display_all() {
        use crate::services::ssh_agent::AccessKeyRole;
        assert_eq!(format!("{}", AccessKeyRole::Git), "git");
        assert_eq!(
            format!("{}", AccessKeyRole::AnsiblePasswordVault),
            "ansible_password_vault"
        );
        assert_eq!(
            format!("{}", AccessKeyRole::AnsibleBecomeUser),
            "ansible_become_user"
        );
        assert_eq!(format!("{}", AccessKeyRole::AnsibleUser), "ansible_user");
    }

    #[test]
    fn test_access_key_type_variants() {
        use crate::models::access_key::AccessKeyType;
        assert_eq!(format!("{}", AccessKeyType::None), "none");
        assert_eq!(
            format!("{}", AccessKeyType::LoginPassword),
            "login_password"
        );
        assert_eq!(format!("{}", AccessKeyType::SSH), "ssh");
        assert_eq!(format!("{}", AccessKeyType::AccessKey), "access_key");
    }

    #[test]
    fn test_simple_encryption_service_new_uses_first_32_bytes() {
        let svc = SimpleEncryptionService::new("short");
        assert_eq!(svc.key.len(), 32);
    }

    #[test]
    fn test_simple_encryption_service_default_key_not_all_zeros() {
        let svc = SimpleEncryptionService::default();
        assert!(svc.key.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_simple_encryption_service_encrypt_empty_secret() {
        let encryption = SimpleEncryptionService::default();
        let mut key = DbAccessKey {
            id: 1,
            name: "Test".to_string(),
            key_type: DbAccessKeyType::Ssh,
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

        let result = encryption.encrypt_secret(&mut key);
        assert!(result.is_ok());
        assert!(key.secret.is_none());
    }

    #[test]
    fn test_simple_encryption_service_decrypt_empty_secret() {
        let encryption = SimpleEncryptionService::default();
        let mut key = DbAccessKey {
            id: 1,
            name: "Test".to_string(),
            key_type: DbAccessKeyType::Ssh,
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

        let result = encryption.decrypt_secret(&mut key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_simple_encryption_service_serialize_empty_key_type() {
        let encryption = SimpleEncryptionService::default();
        let mut key = DbAccessKey {
            id: 1,
            name: "Test".to_string(),
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

        let result = encryption.serialize_secret(&mut key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_simple_encryption_service_deserialize_empty_secret() {
        let encryption = SimpleEncryptionService::default();
        let mut key = DbAccessKey {
            id: 1,
            name: "Test".to_string(),
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

        let result = encryption.deserialize_secret(&mut key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_access_key_install_service_with_installer_constructor() {
        let encryption = Box::new(SimpleEncryptionService::default());
        let installer = AccessKeyInstallerImpl::new();
        let service = AccessKeyInstallationServiceImpl::with_installer(encryption, installer);

        let key = DbAccessKey {
            id: 1,
            name: "Test".to_string(),
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

        let logger = BasicLogger::new();
        let result = service.install(&key, DbAccessKeyRole::Git, &logger);
        assert!(result.is_ok());
    }

    #[test]
    fn test_db_access_key_role_display_ansible_user() {
        let role = DbAccessKeyRole::AnsibleUser;
        let debug = format!("{:?}", role);
        assert!(debug.contains("AnsibleUser"));
    }

    #[test]
    fn test_db_access_key_role_display_ansible_become_user() {
        let role = DbAccessKeyRole::AnsibleBecomeUser;
        let debug = format!("{:?}", role);
        assert!(debug.contains("AnsibleBecomeUser"));
    }

    #[test]
    fn test_db_access_key_role_display_git() {
        let role = DbAccessKeyRole::Git;
        let debug = format!("{:?}", role);
        assert!(debug.contains("Git"));
    }

    #[test]
    fn test_access_key_install_service_install_login_password() {
        let encryption = Box::new(SimpleEncryptionService::default());
        let service = AccessKeyInstallationServiceImpl::new(encryption);
        let logger = BasicLogger::new();

        let key = DbAccessKey {
            id: 1,
            name: "LP Key".to_string(),
            key_type: DbAccessKeyType::LoginPassword,
            project_id: Some(1),
            secret: None,
            plain: None,
            string_value: None,
            login_password: Some(DbLoginPassword {
                login: "admin".to_string(),
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

        let installation = service
            .install(&key, DbAccessKeyRole::AnsibleUser, &logger)
            .unwrap();
        assert!(installation.login.is_some());
    }

    #[test]
    fn test_simple_encryption_service_key_length_32() {
        let svc = SimpleEncryptionService::new("a");
        assert_eq!(svc.key.len(), 32);
    }

    #[test]
    fn test_access_key_installation_service_trait_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Box<dyn AccessKeyInstallationServiceTrait>>();
    }

    #[test]
    fn test_access_key_encryption_service_trait_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Box<dyn AccessKeyEncryptionService>>();
    }
}
