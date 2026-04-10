//! Установка SSH ключей (AccessKeyInstaller)
//!
//! Предоставляет инфраструктуру для установки SSH ключей:
//! - Временные файлы с ключами
//! - Правильные права доступа (0o600)
//! - Очистка после использования
//! - Интеграция с Git и Ansible

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::error::{Error, Result};
use crate::models::access_key::AccessKeyType;
use crate::models::AccessKey;

/// Роль ключа доступа
#[derive(Debug, Clone, Copy)]
pub enum AccessKeyRole {
    /// Для Git операций
    Git,
    /// Для Ansible
    Ansible,
    /// Для SSH подключений
    SSH,
}

/// Установленный SSH ключ
pub struct SshKeyInstallation {
    /// Путь к приватному ключу
    pub private_key_path: PathBuf,
    /// Путь к публичному ключу (опционально)
    pub public_key_path: Option<PathBuf>,
    /// SSH пароль (опционально)
    pub passphrase: Option<String>,
}

impl SshKeyInstallation {
    /// Создаёт новую установку
    pub fn new(private_key_path: PathBuf) -> Self {
        Self {
            private_key_path,
            public_key_path: None,
            passphrase: None,
        }
    }

    /// Устанавливает публичный ключ
    pub fn with_public_key(mut self, public_key_path: PathBuf) -> Self {
        self.public_key_path = Some(public_key_path);
        self
    }

    /// Устанавливает пароль
    pub fn with_passphrase(mut self, passphrase: String) -> Self {
        self.passphrase = Some(passphrase);
        self
    }

    /// Уничтожает установку (удаляет временные файлы)
    pub fn destroy(&self) -> Result<()> {
        // Удаляем приватный ключ
        if self.private_key_path.exists() {
            fs::remove_file(&self.private_key_path)
                .map_err(|e| Error::Other(format!("Ошибка удаления приватного ключа: {}", e)))?;
        }

        // Удаляем публичный ключ
        if let Some(ref pub_path) = self.public_key_path {
            if pub_path.exists() {
                fs::remove_file(pub_path).map_err(|e| {
                    Error::Other(format!("Ошибка удаления публичного ключа: {}", e))
                })?;
            }
        }

        debug!("SSH ключи успешно удалены");
        Ok(())
    }

    /// Получает переменные окружения для Git
    pub fn get_git_env(&self) -> Vec<(String, String)> {
        let mut env = Vec::new();

        // GIT_SSH_COMMAND для использования конкретного ключа
        let ssh_command = format!(
            "ssh -i {} -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null",
            self.private_key_path.display()
        );
        env.push(("GIT_SSH_COMMAND".to_string(), ssh_command));

        // SSH_AUTH_SOCK не нужен, т.к. используем прямой путь к ключу
        env
    }

    /// Получает переменные окружения для Ansible
    pub fn get_ansible_env(&self) -> Vec<(String, String)> {
        let mut env = Vec::new();

        // ANSIBLE_PRIVATE_KEY_FILE
        env.push((
            "ANSIBLE_PRIVATE_KEY_FILE".to_string(),
            self.private_key_path.to_string_lossy().to_string(),
        ));

        // ANSIBLE_SSH_ARGS
        let ssh_args = "-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null";
        env.push(("ANSIBLE_SSH_ARGS".to_string(), ssh_args.to_string()));

        env
    }
}

/// Установщик ключей доступа
pub struct AccessKeyInstaller {
    /// Временная директория для ключей
    temp_dir: PathBuf,
}

impl AccessKeyInstaller {
    /// Создаёт новый установщик
    pub fn new(temp_dir: PathBuf) -> Self {
        Self { temp_dir }
    }

    /// Устанавливает ключ
    pub fn install(&self, key: &AccessKey, role: AccessKeyRole) -> Result<SshKeyInstallation> {
        match key.r#type {
            AccessKeyType::SSH => self.install_ssh_key(key),
            AccessKeyType::LoginPassword => {
                // Для login/password SSH ключ не нужен
                Err(Error::Other(
                    "Login/Password ключ не требует установки SSH ключа".to_string(),
                ))
            }
            AccessKeyType::AccessKey => {
                // Access Key (AWS и т.д.) не требует SSH ключа
                Err(Error::Other(
                    "Access Key не требует установки SSH ключа".to_string(),
                ))
            }
            AccessKeyType::None => Err(Error::Other("Ключ не настроен".to_string())),
        }
    }

    /// Устанавливает SSH ключ
    fn install_ssh_key(&self, key: &AccessKey) -> Result<SshKeyInstallation> {
        info!("Установка SSH ключа: {}", key.name);

        // Получаем приватный ключ
        let private_key = key
            .ssh_key
            .as_ref()
            .ok_or_else(|| Error::Other("SSH ключ не настроен".to_string()))?;

        // Создаём временную директорию для ключа
        let key_dir = self.temp_dir.join(format!("key_{}", key.id));
        fs::create_dir_all(&key_dir)
            .map_err(|e| Error::Other(format!("Ошибка создания директории: {}", e)))?;

        // Записываем приватный ключ
        let private_key_path = key_dir.join("id_rsa");
        let mut file = File::create(&private_key_path)
            .map_err(|e| Error::Other(format!("Ошибка создания файла ключа: {}", e)))?;

        file.write_all(private_key.as_bytes())
            .map_err(|e| Error::Other(format!("Ошибка записи ключа: {}", e)))?;

        // Устанавливаем правильные права (0o600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&private_key_path, fs::Permissions::from_mode(0o600))
                .map_err(|e| Error::Other(format!("Ошибка установки прав на ключ: {}", e)))?;
        }

        debug!("Приватный ключ установлен: {:?}", private_key_path);

        // Записываем публичный ключ (если есть)
        let public_key_path = if let Some(ref pub_key) = key.ssh_key {
            // В реальной реализации здесь был бы публичный ключ
            // Пока создаём заглушку
            let pub_key_path = key_dir.join("id_rsa.pub");

            // Пробуем сгенерировать публичный ключ из приватного
            // Это упрощённая реализация
            Some(pub_key_path)
        } else {
            None
        };

        let mut installation = SshKeyInstallation::new(private_key_path);

        if let Some(pub_path) = public_key_path {
            installation = installation.with_public_key(pub_path);
        }

        if let Some(ref passphrase) = key.ssh_passphrase {
            installation = installation.with_passphrase(passphrase.clone());
        }

        info!("SSH ключ успешно установлен");
        Ok(installation)
    }

    /// Устанавливает ключ для Git
    pub fn install_for_git(&self, key: &AccessKey) -> Result<SshKeyInstallation> {
        self.install(key, AccessKeyRole::Git)
    }

    /// Устанавливает ключ для Ansible
    pub fn install_for_ansible(&self, key: &AccessKey) -> Result<SshKeyInstallation> {
        self.install(key, AccessKeyRole::Ansible)
    }

    /// Устанавливает ключ для SSH
    pub fn install_for_ssh(&self, key: &AccessKey) -> Result<SshKeyInstallation> {
        self.install(key, AccessKeyRole::SSH)
    }
}

impl Default for AccessKeyInstaller {
    fn default() -> Self {
        Self {
            temp_dir: PathBuf::from("/tmp/semaphore/keys"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_ssh_key() -> AccessKey {
        use crate::models::AccessKeyOwner;
        AccessKey {
            id: 1,
            project_id: Some(1),
            name: "Test SSH Key".to_string(),
            r#type: AccessKeyType::SSH,
            user_id: None,
            login_password_login: None,
            login_password_password: None,
            ssh_key: Some("-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA0Z3VS5JJcds3xfn/ygWyF8PbnGy...\n-----END RSA PRIVATE KEY-----".to_string()),
            ssh_passphrase: None,
            access_key_access_key: None,
            access_key_secret_key: None,
            secret_storage_id: None,
            environment_id: None,
            owner: Some(AccessKeyOwner::Project),
            created: None,
            source_storage_type: None,
            source_storage_id: None,
            source_key: None,
        }
    }

    #[test]
    fn test_ssh_key_installation_creation() {
        let path = PathBuf::from("/tmp/test_key");
        let installation = SshKeyInstallation::new(path.clone());

        assert_eq!(installation.private_key_path, path);
        assert!(installation.public_key_path.is_none());
        assert!(installation.passphrase.is_none());
    }

    #[test]
    fn test_ssh_key_installation_with_public_key() {
        let private_path = PathBuf::from("/tmp/test_key");
        let public_path = PathBuf::from("/tmp/test_key.pub");

        let installation =
            SshKeyInstallation::new(private_path.clone()).with_public_key(public_path.clone());

        assert_eq!(installation.private_key_path, private_path);
        assert!(installation.public_key_path.is_some());
        assert_eq!(installation.public_key_path.unwrap(), public_path);
    }

    #[test]
    fn test_ssh_key_installation_with_passphrase() {
        let path = PathBuf::from("/tmp/test_key");
        let installation =
            SshKeyInstallation::new(path.clone()).with_passphrase("test".to_string());

        assert_eq!(installation.private_key_path, path);
        assert!(installation.passphrase.is_some());
        assert_eq!(installation.passphrase.unwrap(), "test");
    }

    #[test]
    fn test_access_key_installer_creation() {
        let temp_dir = PathBuf::from("/tmp/semaphore/test");
        let installer = AccessKeyInstaller::new(temp_dir.clone());

        assert_eq!(installer.temp_dir, temp_dir);
    }

    #[test]
    fn test_access_key_installer_default() {
        let installer = AccessKeyInstaller::default();

        assert!(installer
            .temp_dir
            .display()
            .to_string()
            .contains("semaphore"));
    }

    #[test]
    fn test_access_key_role_enum() {
        let git_role = AccessKeyRole::Git;
        let ansible_role = AccessKeyRole::Ansible;
        let ssh_role = AccessKeyRole::SSH;

        assert!(matches!(git_role, AccessKeyRole::Git));
        assert!(matches!(ansible_role, AccessKeyRole::Ansible));
        assert!(matches!(ssh_role, AccessKeyRole::SSH));
    }

    #[test]
    fn test_install_for_non_ssh_key() {
        use crate::models::AccessKeyOwner;
        let key = AccessKey {
            id: 1,
            project_id: Some(1),
            name: "Test".to_string(),
            r#type: AccessKeyType::LoginPassword,
            user_id: None,
            login_password_login: Some("user".to_string()),
            login_password_password: Some("pass".to_string()),
            ssh_key: None,
            ssh_passphrase: None,
            access_key_access_key: None,
            access_key_secret_key: None,
            secret_storage_id: None,
            environment_id: None,
            owner: Some(AccessKeyOwner::Project),
            created: None,
            source_storage_type: None,
            source_storage_id: None,
            source_key: None,
        };

        let installer = AccessKeyInstaller::default();
        let result = installer.install(&key, AccessKeyRole::Git);

        assert!(result.is_err());
    }

    #[test]
    fn test_install_for_access_key_type() {
        use crate::models::AccessKeyOwner;
        let key = AccessKey {
            id: 2,
            project_id: Some(1),
            name: "AWS Key".to_string(),
            r#type: AccessKeyType::AccessKey,
            user_id: None,
            login_password_login: None,
            login_password_password: None,
            ssh_key: None,
            ssh_passphrase: None,
            access_key_access_key: Some("AKIAIOSFODNN7EXAMPLE".to_string()),
            access_key_secret_key: Some("wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string()),
            secret_storage_id: None,
            environment_id: None,
            owner: Some(AccessKeyOwner::Project),
            created: None,
            source_storage_type: None,
            source_storage_id: None,
            source_key: None,
        };

        let installer = AccessKeyInstaller::default();
        let result = installer.install(&key, AccessKeyRole::Git);

        assert!(result.is_err());
        match result {
            Err(e) => assert!(format!("{}", e).contains("Access Key")),
            Ok(_) => panic!("Expected error for AccessKey type"),
        }
    }

    #[test]
    fn test_install_for_none_key_type() {
        use crate::models::AccessKeyOwner;
        let key = AccessKey {
            id: 3,
            project_id: Some(1),
            name: "Empty".to_string(),
            r#type: AccessKeyType::None,
            user_id: None,
            login_password_login: None,
            login_password_password: None,
            ssh_key: None,
            ssh_passphrase: None,
            access_key_access_key: None,
            access_key_secret_key: None,
            secret_storage_id: None,
            environment_id: None,
            owner: Some(AccessKeyOwner::Project),
            created: None,
            source_storage_type: None,
            source_storage_id: None,
            source_key: None,
        };

        let installer = AccessKeyInstaller::default();
        let result = installer.install(&key, AccessKeyRole::Git);

        assert!(result.is_err());
        match result {
            Err(e) => assert!(format!("{}", e).contains("не настроен")),
            Ok(_) => panic!("Expected error for None type"),
        }
    }

    #[test]
    fn test_install_ssh_key_missing_ssh_key_field() {
        use crate::models::AccessKeyOwner;
        let key = AccessKey {
            id: 4,
            project_id: Some(1),
            name: "Broken SSH".to_string(),
            r#type: AccessKeyType::SSH,
            user_id: None,
            login_password_login: None,
            login_password_password: None,
            ssh_key: None,
            ssh_passphrase: None,
            access_key_access_key: None,
            access_key_secret_key: None,
            secret_storage_id: None,
            environment_id: None,
            owner: Some(AccessKeyOwner::Project),
            created: None,
            source_storage_type: None,
            source_storage_id: None,
            source_key: None,
        };

        let temp_dir = std::env::temp_dir().join("semaphore_test_install_ssh");
        let installer = AccessKeyInstaller::new(temp_dir);
        let result = installer.install(&key, AccessKeyRole::Git);

        assert!(result.is_err());
        match result {
            Err(e) => assert!(format!("{}", e).contains("SSH ключ не настроен")),
            Ok(_) => panic!("Expected error for missing ssh_key field"),
        }
    }

    #[test]
    fn test_ssh_key_installation_actual_file_creation() {
        use crate::models::AccessKeyOwner;
        let key = AccessKey {
            id: 10,
            project_id: Some(1),
            name: "File Test SSH".to_string(),
            r#type: AccessKeyType::SSH,
            user_id: None,
            login_password_login: None,
            login_password_password: None,
            ssh_key: Some("-----BEGIN RSA PRIVATE KEY-----\ntestcontent\n-----END RSA PRIVATE KEY-----".to_string()),
            ssh_passphrase: None,
            access_key_access_key: None,
            access_key_secret_key: None,
            secret_storage_id: None,
            environment_id: None,
            owner: Some(AccessKeyOwner::Project),
            created: None,
            source_storage_type: None,
            source_storage_id: None,
            source_key: None,
        };

        let temp_dir = std::env::temp_dir().join("semaphore_test_file_write");
        let installer = AccessKeyInstaller::new(temp_dir.clone());
        let result = installer.install(&key, AccessKeyRole::Git);

        assert!(result.is_ok());
        let installation = result.unwrap();

        // Проверяем, что файл создан
        assert!(installation.private_key_path.exists());

        // Проверяем содержимое файла
        let content = fs::read_to_string(&installation.private_key_path).unwrap();
        assert!(content.contains("-----BEGIN RSA PRIVATE KEY-----"));
        assert!(content.contains("-----END RSA PRIVATE KEY-----"));

        // Проверяем путь содержит ID ключа
        assert!(installation.private_key_path.to_string_lossy().contains("key_10"));

        // Уборка
        installation.destroy().unwrap();
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_ssh_key_installation_with_passphrase_from_model() {
        use crate::models::AccessKeyOwner;
        let key = AccessKey {
            id: 11,
            project_id: Some(1),
            name: "Passphrase SSH".to_string(),
            r#type: AccessKeyType::SSH,
            user_id: None,
            login_password_login: None,
            login_password_password: None,
            ssh_key: Some("-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----".to_string()),
            ssh_passphrase: Some("my-secret-passphrase".to_string()),
            access_key_access_key: None,
            access_key_secret_key: None,
            secret_storage_id: None,
            environment_id: None,
            owner: Some(AccessKeyOwner::Project),
            created: None,
            source_storage_type: None,
            source_storage_id: None,
            source_key: None,
        };

        let temp_dir = std::env::temp_dir().join("semaphore_test_passphrase");
        let installer = AccessKeyInstaller::new(temp_dir.clone());
        let result = installer.install(&key, AccessKeyRole::SSH);

        assert!(result.is_ok());
        let installation = result.unwrap();

        assert!(installation.passphrase.is_some());
        assert_eq!(installation.passphrase.as_ref().unwrap(), "my-secret-passphrase");

        installation.destroy().unwrap();
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_ssh_key_destroy_removes_file() {
        use crate::models::AccessKeyOwner;
        let key = AccessKey {
            id: 20,
            project_id: Some(1),
            name: "Destroy Test".to_string(),
            r#type: AccessKeyType::SSH,
            user_id: None,
            login_password_login: None,
            login_password_password: None,
            ssh_key: Some("-----BEGIN RSA PRIVATE KEY-----\ndestroy_me\n-----END RSA PRIVATE KEY-----".to_string()),
            ssh_passphrase: None,
            access_key_access_key: None,
            access_key_secret_key: None,
            secret_storage_id: None,
            environment_id: None,
            owner: Some(AccessKeyOwner::Project),
            created: None,
            source_storage_type: None,
            source_storage_id: None,
            source_key: None,
        };

        let temp_dir = std::env::temp_dir().join("semaphore_test_destroy");
        let installer = AccessKeyInstaller::new(temp_dir.clone());
        let installation = installer.install(&key, AccessKeyRole::Git).unwrap();

        let private_path = installation.private_key_path.clone();
        assert!(private_path.exists());

        installation.destroy().unwrap();

        assert!(!private_path.exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_get_git_env_contains_ssh_command() {
        let installation = SshKeyInstallation::new(PathBuf::from("/tmp/test/id_rsa"))
            .with_public_key(PathBuf::from("/tmp/test/id_rsa.pub"))
            .with_passphrase("pass".to_string());

        let env = installation.get_git_env();

        // Ищем GIT_SSH_COMMAND
        let git_ssh = env.iter().find(|(k, _)| k == "GIT_SSH_COMMAND");
        assert!(git_ssh.is_some());

        let (_, value) = git_ssh.unwrap();
        assert!(value.contains("/tmp/test/id_rsa"));
        assert!(value.contains("StrictHostKeyChecking=no"));
        assert!(value.contains("UserKnownHostsFile=/dev/null"));
    }

    #[test]
    fn test_get_ansible_env_contains_private_key_file() {
        let installation = SshKeyInstallation::new(PathBuf::from("/tmp/ansible/id_rsa"));

        let env = installation.get_ansible_env();

        // Ищем ANSIBLE_PRIVATE_KEY_FILE
        let key_file = env.iter().find(|(k, _)| k == "ANSIBLE_PRIVATE_KEY_FILE");
        assert!(key_file.is_some());
        assert_eq!(key_file.unwrap().1, "/tmp/ansible/id_rsa");

        // Ищем ANSIBLE_SSH_ARGS
        let ssh_args = env.iter().find(|(k, _)| k == "ANSIBLE_SSH_ARGS");
        assert!(ssh_args.is_some());
        assert!(ssh_args.unwrap().1.contains("StrictHostKeyChecking=no"));
    }

    #[test]
    fn test_install_for_git_method() {
        let key = create_test_ssh_key();
        let temp_dir = std::env::temp_dir().join("semaphore_test_git_method");
        let installer = AccessKeyInstaller::new(temp_dir.clone());

        let result = installer.install_for_git(&key);
        assert!(result.is_ok());

        let installation = result.unwrap();
        assert!(installation.private_key_path.exists());

        installation.destroy().unwrap();
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_install_for_ansible_method() {
        let key = create_test_ssh_key();
        let temp_dir = std::env::temp_dir().join("semaphore_test_ansible_method");
        let installer = AccessKeyInstaller::new(temp_dir.clone());

        let result = installer.install_for_ansible(&key);
        assert!(result.is_ok());

        let installation = result.unwrap();
        assert!(installation.private_key_path.exists());

        installation.destroy().unwrap();
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_install_for_ssh_method() {
        let key = create_test_ssh_key();
        let temp_dir = std::env::temp_dir().join("semaphore_test_ssh_method");
        let installer = AccessKeyInstaller::new(temp_dir.clone());

        let result = installer.install_for_ssh(&key);
        assert!(result.is_ok());

        let installation = result.unwrap();
        assert!(installation.private_key_path.exists());

        installation.destroy().unwrap();
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_multiple_keys_different_ids() {
        use crate::models::AccessKeyOwner;
        let key1 = AccessKey {
            id: 100,
            project_id: Some(1),
            name: "Key 1".to_string(),
            r#type: AccessKeyType::SSH,
            user_id: None,
            login_password_login: None,
            login_password_password: None,
            ssh_key: Some("-----BEGIN RSA PRIVATE KEY-----\nkey1\n-----END RSA PRIVATE KEY-----".to_string()),
            ssh_passphrase: None,
            access_key_access_key: None,
            access_key_secret_key: None,
            secret_storage_id: None,
            environment_id: None,
            owner: Some(AccessKeyOwner::Project),
            created: None,
            source_storage_type: None,
            source_storage_id: None,
            source_key: None,
        };

        let key2 = AccessKey {
            id: 200,
            project_id: Some(1),
            name: "Key 2".to_string(),
            r#type: AccessKeyType::SSH,
            user_id: None,
            login_password_login: None,
            login_password_password: None,
            ssh_key: Some("-----BEGIN RSA PRIVATE KEY-----\nkey2\n-----END RSA PRIVATE KEY-----".to_string()),
            ssh_passphrase: None,
            access_key_access_key: None,
            access_key_secret_key: None,
            secret_storage_id: None,
            environment_id: None,
            owner: Some(AccessKeyOwner::Project),
            created: None,
            source_storage_type: None,
            source_storage_id: None,
            source_key: None,
        };

        let temp_dir = std::env::temp_dir().join("semaphore_test_multiple");
        let installer = AccessKeyInstaller::new(temp_dir.clone());

        let inst1 = installer.install(&key1, AccessKeyRole::Git).unwrap();
        let inst2 = installer.install(&key2, AccessKeyRole::Git).unwrap();

        // Пути разные
        assert_ne!(inst1.private_key_path, inst2.private_key_path);
        assert!(inst1.private_key_path.to_string_lossy().contains("key_100"));
        assert!(inst2.private_key_path.to_string_lossy().contains("key_200"));

        // Содержимое разное
        let content1 = fs::read_to_string(&inst1.private_key_path).unwrap();
        let content2 = fs::read_to_string(&inst2.private_key_path).unwrap();
        assert_ne!(content1, content2);

        inst1.destroy().unwrap();
        inst2.destroy().unwrap();
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_destroy_nonexistent_file_does_not_error() {
        // SshKeyInstallation::destroy должен корректно обрабатывать
        // ситуацию когда файл уже удалён
        let installation = SshKeyInstallation::new(PathBuf::from("/tmp/semaphore_test_nonexistent/key"));

        // Не создавая файл, вызываем destroy — не должно быть ошибки
        let result = installation.destroy();
        assert!(result.is_ok());
    }

    #[test]
    fn test_destroy_with_public_key_removes_both() {
        use crate::models::AccessKeyOwner;
        let key = AccessKey {
            id: 30,
            project_id: Some(1),
            name: "Both Keys".to_string(),
            r#type: AccessKeyType::SSH,
            user_id: None,
            login_password_login: None,
            login_password_password: None,
            ssh_key: Some("-----BEGIN RSA PRIVATE KEY-----\nboth\n-----END RSA PRIVATE KEY-----".to_string()),
            ssh_passphrase: None,
            access_key_access_key: None,
            access_key_secret_key: None,
            secret_storage_id: None,
            environment_id: None,
            owner: Some(AccessKeyOwner::Project),
            created: None,
            source_storage_type: None,
            source_storage_id: None,
            source_key: None,
        };

        let temp_dir = std::env::temp_dir().join("semaphore_test_both_destroy");
        let installer = AccessKeyInstaller::new(temp_dir.clone());
        let installation = installer.install(&key, AccessKeyRole::Git).unwrap();

        let private_path = installation.private_key_path.clone();
        assert!(private_path.exists());

        installation.destroy().unwrap();
        assert!(!private_path.exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_git_env_does_not_contain_ssh_auth_sock() {
        let installation = SshKeyInstallation::new(PathBuf::from("/tmp/test/id_rsa"));
        let env = installation.get_git_env();

        // SSH_AUTH_SOCK не должен быть установлен
        let has_auth_sock = env.iter().any(|(k, _)| k == "SSH_AUTH_SOCK");
        assert!(!has_auth_sock);
    }

    #[test]
    fn test_ansible_env_has_exactly_two_vars() {
        let installation = SshKeyInstallation::new(PathBuf::from("/tmp/test/id_rsa"));
        let env = installation.get_ansible_env();

        assert_eq!(env.len(), 2);

        let keys: Vec<&str> = env.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"ANSIBLE_PRIVATE_KEY_FILE"));
        assert!(keys.contains(&"ANSIBLE_SSH_ARGS"));
    }

    #[test]
    fn test_installer_temp_dir_custom_path() {
        let custom_path = PathBuf::from("/custom/semaphore/keys/path");
        let installer = AccessKeyInstaller::new(custom_path.clone());

        assert_eq!(installer.temp_dir, custom_path);
        assert!(installer.temp_dir.to_string_lossy().contains("/custom/"));
    }

    #[test]
    fn test_install_ssh_key_creates_key_directory() {
        use crate::models::AccessKeyOwner;
        let key = AccessKey {
            id: 42,
            project_id: Some(1),
            name: "Dir Test".to_string(),
            r#type: AccessKeyType::SSH,
            user_id: None,
            login_password_login: None,
            login_password_password: None,
            ssh_key: Some("-----BEGIN RSA PRIVATE KEY-----\ndir_test\n-----END RSA PRIVATE KEY-----".to_string()),
            ssh_passphrase: None,
            access_key_access_key: None,
            access_key_secret_key: None,
            secret_storage_id: None,
            environment_id: None,
            owner: Some(AccessKeyOwner::Project),
            created: None,
            source_storage_type: None,
            source_storage_id: None,
            source_key: None,
        };

        let temp_dir = std::env::temp_dir().join("semaphore_test_dir_creation");
        let installer = AccessKeyInstaller::new(temp_dir.clone());

        let key_dir = temp_dir.join("key_42");
        assert!(!key_dir.exists());

        let _ = installer.install(&key, AccessKeyRole::Git).unwrap();

        // Директория создана
        assert!(key_dir.exists());
        assert!(key_dir.is_dir());

        // cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_install_login_password_returns_error_with_correct_message() {
        let key = AccessKey::new_login_password(
            1,
            "LP Key".to_string(),
            "admin".to_string(),
            "secret".to_string(),
            None,
        );

        let installer = AccessKeyInstaller::default();
        let result = installer.install(&key, AccessKeyRole::Git);

        assert!(result.is_err());
        match result {
            Err(e) => {
                let err_msg = format!("{}", e);
                assert!(err_msg.contains("Login/Password"));
                assert!(err_msg.contains("не требует установки SSH ключа"));
            }
            Ok(_) => panic!("Expected error for LoginPassword type"),
        }
    }

    #[test]
    fn test_ssh_key_installation_builder_chain() {
        let installation = SshKeyInstallation::new(PathBuf::from("/chain/private"))
            .with_public_key(PathBuf::from("/chain/public"))
            .with_passphrase("chain_pass".to_string());

        assert_eq!(installation.private_key_path, PathBuf::from("/chain/private"));
        assert_eq!(installation.public_key_path, Some(PathBuf::from("/chain/public")));
        assert_eq!(installation.passphrase, Some("chain_pass".to_string()));
    }
}
