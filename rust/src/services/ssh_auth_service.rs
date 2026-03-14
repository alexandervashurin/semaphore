//! Сервис SSH аутентификации для Git
//!
//! Этот модуль предоставляет функциональность для аутентификации
//! в Git репозиториях через SSH ключи.

use crate::db::store::AccessKeyManager;
use crate::error::{Error, Result};
use crate::models::access_key::{AccessKey, AccessKeyType};
use git2::{Cred, RemoteCallbacks};
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use tracing::{info, warn};

/// Сервис для SSH аутентификации в Git
pub struct SshAuthService;

impl SshAuthService {
    /// Создает RemoteCallbacks для аутентификации в Git через SSH
    ///
    /// # Arguments
    /// * `access_key` - SSH ключ доступа
    ///
    /// # Returns
    /// * `Result<RemoteCallbacks<'static>>` - Callbacks для git2
    pub fn create_ssh_callbacks(access_key: &AccessKey) -> Result<RemoteCallbacks<'static>> {
        // Проверяем тип ключа
        if access_key.r#type != AccessKeyType::SSH {
            return Err(Error::Validation(
                "Ключ не является SSH ключом".to_string(),
            ));
        }

        // Получаем SSH ключ
        let ssh_key = access_key.ssh_key.clone().ok_or_else(|| {
            Error::Validation("SSH ключ не найден".to_string())
        })?;

        let passphrase = access_key.ssh_passphrase.clone();

        // Создаем временную директорию для ключа
        let temp_dir = Arc::new(TempDir::new().map_err(|e| {
            Error::Other(format!("Не удалось создать временную директорию: {}", e))
        })?);

        // Записываем ключ в файл
        let key_path = temp_dir.path().join("ssh_key");
        std::fs::write(&key_path, &ssh_key).map_err(|e| {
            Error::Other(format!("Не удалось записать SSH ключ: {}", e))
        })?;

        // Устанавливаем правильные права (только для владельца)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&key_path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&key_path, perms)?;
        }

        let key_path = Arc::new(key_path);

        // Создаем RemoteCallbacks
        let mut callbacks = RemoteCallbacks::new();

        // Устанавливаем credentials callback
        callbacks.credentials(move |url, username_from_url, allowed_types| {
            info!("Git credentials callback: URL={}, username={:?}", url, username_from_url);

            // Проверяем разрешенные типы аутентификации
            if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                info!("Используем SSH ключ аутентификацию");

                let username = username_from_url.unwrap_or("git");

                // Создаем credentials из SSH ключа
                if let Some(ref passphrase) = passphrase {
                    Cred::ssh_key_from_memory(username, None, &ssh_key, Some(passphrase))
                } else {
                    Cred::ssh_key_from_memory(username, None, &ssh_key, None)
                }
            } else {
                info!("Используем SSH агент или default credentials");
                Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
            }
        });

        // Устанавливаем certificate check callback (для самоподписанных сертификатов)
        callbacks.certificate_check(|cert, host| {
            info!("Certificate check for host: {}", host);
            // Разрешаем все сертификаты (можно добавить проверку)
            Ok(git2::CertificateCheckStatus::CertificateOk)
        });

        Ok(callbacks)
    }

    /// Получает SSH ключ из AccessKeyManager и создает callbacks
    ///
    /// # Arguments
    /// * `key_id` - ID ключа доступа
    /// * `store` - Менеджер ключей доступа
    ///
    /// # Returns
    /// * `Result<RemoteCallbacks<'static>>` - Callbacks для git2
    pub async fn create_callbacks_from_key_id<S>(
        key_id: i32,
        store: &S,
    ) -> Result<RemoteCallbacks<'static>>
    where
        S: AccessKeyManager,
    {
        // Получаем ключ из БД
        // TODO: Нужен метод get_access_key(id: i32)
        
        // Временно возвращаем ошибку
        Err(Error::NotFound(format!(
            "SSH аутентификация требует реализации get_access_key({})",
            key_id
        )))
    }

    /// Проверяет валидность SSH ключа
    ///
    /// # Arguments
    /// * `ssh_key` - SSH ключ (PEM формат)
    ///
    /// # Returns
    /// * `Result<()>` - Ok если ключ валиден
    pub fn validate_ssh_key(ssh_key: &str) -> Result<()> {
        // Простая проверка формата
        if !ssh_key.contains("BEGIN") || !ssh_key.contains("END") {
            return Err(Error::Validation(
                "Неверный формат SSH ключа".to_string(),
            ));
        }

        // Проверяем поддерживаемые типы ключей
        let supported_types = ["RSA", "ED25519", "EC", "OPENSSH"];
        if !supported_types.iter().any(|t| ssh_key.contains(t)) {
            return Err(Error::Validation(
                "Неподдерживаемый тип SSH ключа".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_ssh_key() {
        let key = "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----";
        assert!(SshAuthService::validate_ssh_key(key).is_ok());
    }

    #[test]
    fn test_validate_invalid_ssh_key() {
        let key = "invalid key";
        assert!(SshAuthService::validate_ssh_key(key).is_err());
    }

    #[test]
    fn test_validate_ed25519_key() {
        let key = "-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAA...\n-----END OPENSSH PRIVATE KEY-----";
        assert!(SshAuthService::validate_ssh_key(key).is_ok());
    }
}
