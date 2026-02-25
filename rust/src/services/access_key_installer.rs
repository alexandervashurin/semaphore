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
use tracing::{info, warn, debug};

use crate::error::{Error, Result};
use crate::models::AccessKey;
use crate::models::access_key::AccessKeyType;

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
            fs::remove_file(&self.private_key_path).map_err(|e| {
                Error::Other(format!("Ошибка удаления приватного ключа: {}", e))
            })?;
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

    /// Создаёт установщик по умолчанию
    pub fn default() -> Self {
        Self {
            temp_dir: PathBuf::from("/tmp/semaphore/keys"),
        }
    }

    /// Устанавливает ключ
    pub fn install(
        &self,
        key: &AccessKey,
        role: AccessKeyRole,
    ) -> Result<SshKeyInstallation> {
        match key.r#type {
            AccessKeyType::SSH => self.install_ssh_key(key),
            AccessKeyType::LoginPassword => {
                // Для login/password SSH ключ не нужен
                Err(Error::Other("Login/Password ключ не требует установки SSH ключа".to_string()))
            }
            AccessKeyType::AccessKey => {
                // Access Key (AWS и т.д.) не требует SSH ключа
                Err(Error::Other("Access Key не требует установки SSH ключа".to_string()))
            }
            AccessKeyType::None => {
                Err(Error::Other("Ключ не настроен".to_string()))
            }
        }
    }

    /// Устанавливает SSH ключ
    fn install_ssh_key(&self, key: &AccessKey) -> Result<SshKeyInstallation> {
        info!("Установка SSH ключа: {}", key.name);

        // Получаем приватный ключ
        let private_key = key.ssh_key.as_ref()
            .ok_or_else(|| Error::Other("SSH ключ не настроен".to_string()))?;

        // Создаём временную директорию для ключа
        let key_dir = self.temp_dir.join(format!("key_{}", key.id));
        fs::create_dir_all(&key_dir).map_err(|e| {
            Error::Other(format!("Ошибка создания директории: {}", e))
        })?;

        // Записываем приватный ключ
        let private_key_path = key_dir.join("id_rsa");
        let mut file = File::create(&private_key_path).map_err(|e| {
            Error::Other(format!("Ошибка создания файла ключа: {}", e))
        })?;

        file.write_all(private_key.as_bytes()).map_err(|e| {
            Error::Other(format!("Ошибка записи ключа: {}", e))
        })?;

        // Устанавливаем правильные права (0o600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&private_key_path, fs::Permissions::from_mode(0o600)).map_err(|e| {
                Error::Other(format!("Ошибка установки прав на ключ: {}", e))
            })?;
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
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_ssh_key() -> AccessKey {
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
        
        let installation = SshKeyInstallation::new(private_path.clone())
            .with_public_key(public_path.clone());
        
        assert_eq!(installation.private_key_path, private_path);
        assert!(installation.public_key_path.is_some());
        assert_eq!(installation.public_key_path.unwrap(), public_path);
    }

    #[test]
    fn test_ssh_key_installation_with_passphrase() {
        let path = PathBuf::from("/tmp/test_key");
        let installation = SshKeyInstallation::new(path.clone())
            .with_passphrase("test".to_string());
        
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
        
        assert!(installer.temp_dir.display().to_string().contains("semaphore"));
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
        };

        let installer = AccessKeyInstaller::default();
        let result = installer.install(&key, AccessKeyRole::Git);
        
        assert!(result.is_err());
    }
}
