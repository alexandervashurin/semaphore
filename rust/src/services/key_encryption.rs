//! Access Key Encryption Service
//!
//! Сервис шифрования ключей доступа с помощью AES-256-GCM.
//! Если переменная окружения SEMAPHORE_ACCESS_KEY_ENCRYPTION задана,
//! все секретные поля ключей шифруются при сохранении и дешифруются при чтении.

use std::sync::Arc;
use crate::error::{Error, Result};
use crate::models::AccessKey;
use crate::db::store::Store;

/// Маркер зашифрованного значения
const ENC_PREFIX: &str = "$enc$";

/// Возвращает 32-байтный ключ шифрования из переменной окружения
fn get_encryption_key() -> Option<[u8; 32]> {
    use std::env;
    let raw = env::var("SEMAPHORE_ACCESS_KEY_ENCRYPTION").ok()?;
    if raw.is_empty() {
        return None;
    }
    // SHA-256 хэш строки → 32 байта
    use sha2::{Sha256, Digest};
    let hash = Sha256::digest(raw.as_bytes());
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash);
    Some(key)
}

/// Шифрует строку, возвращает "$enc$<base64>"
fn encrypt_value(plaintext: &str, key: &[u8; 32]) -> Result<String> {
    let encrypted = crate::utils::encryption::aes256_encrypt(plaintext.as_bytes(), key)
        .map_err(|e| Error::Other(e.to_string()))?;
    Ok(format!("{}{}", ENC_PREFIX, encrypted))
}

/// Дешифрует строку вида "$enc$<base64>"
fn decrypt_value(ciphertext: &str, key: &[u8; 32]) -> Result<String> {
    let data = ciphertext.trim_start_matches(ENC_PREFIX);
    let plaintext = crate::utils::encryption::aes256_decrypt(data, key)
        .map_err(|e| Error::Other(e.to_string()))?;
    String::from_utf8(plaintext).map_err(|e| Error::Other(e.to_string()))
}

/// Шифрует поле: если ключ не задан или значение уже зашифровано — пропускает
fn maybe_encrypt(value: &Option<String>, key: &[u8; 32]) -> Result<Option<String>> {
    match value {
        None => Ok(None),
        Some(s) if s.is_empty() => Ok(Some(s.clone())),
        Some(s) if s.starts_with(ENC_PREFIX) => Ok(Some(s.clone())), // уже зашифровано
        Some(s) => Ok(Some(encrypt_value(s, key)?)),
    }
}

/// Дешифрует поле: если значение не зашифровано — возвращает как есть
fn maybe_decrypt(value: &Option<String>, key: &[u8; 32]) -> Result<Option<String>> {
    match value {
        None => Ok(None),
        Some(s) if s.is_empty() => Ok(Some(s.clone())),
        Some(s) if s.starts_with(ENC_PREFIX) => Ok(Some(decrypt_value(s, key)?)),
        Some(s) => Ok(Some(s.clone())), // plaintext (старые данные без шифрования)
    }
}

// ============================================================================
// Trait
// ============================================================================

/// Сервис шифрования ключей доступа
pub trait AccessKeyEncryptionService: Send + Sync {
    /// Шифрует секретные поля перед сохранением
    fn serialize_secret(&self, key: &mut AccessKey) -> Result<()>;

    /// Дешифрует секретные поля после загрузки
    fn deserialize_secret(&self, key: &mut AccessKey) -> Result<()>;

    /// Заполняет секреты окружения
    fn fill_environment_secrets(&self, env: &mut crate::models::Environment, deserialize_secret: bool) -> Result<()>;

    /// Удаляет секрет (no-op для DB-хранилища)
    fn delete_secret(&self, key: &AccessKey) -> Result<()>;
}

// ============================================================================
// Implementation
// ============================================================================

/// Реализация сервиса шифрования
pub struct AccessKeyEncryptionServiceImpl {
    #[allow(dead_code)]
    access_key_repo: Arc<dyn Store + Send + Sync>,
    #[allow(dead_code)]
    environment_repo: Arc<dyn Store + Send + Sync>,
    #[allow(dead_code)]
    secret_storage_repo: Arc<dyn Store + Send + Sync>,
}

impl AccessKeyEncryptionServiceImpl {
    /// Создаёт новый сервис
    pub fn new(
        access_key_repo: Arc<dyn Store + Send + Sync>,
        environment_repo: Arc<dyn Store + Send + Sync>,
        secret_storage_repo: Arc<dyn Store + Send + Sync>,
    ) -> Self {
        Self {
            access_key_repo,
            environment_repo,
            secret_storage_repo,
        }
    }
}

impl AccessKeyEncryptionService for AccessKeyEncryptionServiceImpl {
    fn serialize_secret(&self, key: &mut AccessKey) -> Result<()> {
        let enc_key = match get_encryption_key() {
            Some(k) => k,
            None => return Ok(()), // шифрование отключено
        };
        key.ssh_key = maybe_encrypt(&key.ssh_key, &enc_key)?;
        key.ssh_passphrase = maybe_encrypt(&key.ssh_passphrase, &enc_key)?;
        key.login_password_password = maybe_encrypt(&key.login_password_password, &enc_key)?;
        key.access_key_secret_key = maybe_encrypt(&key.access_key_secret_key, &enc_key)?;
        Ok(())
    }

    fn deserialize_secret(&self, key: &mut AccessKey) -> Result<()> {
        let enc_key = match get_encryption_key() {
            Some(k) => k,
            None => return Ok(()), // шифрование отключено
        };
        key.ssh_key = maybe_decrypt(&key.ssh_key, &enc_key)?;
        key.ssh_passphrase = maybe_decrypt(&key.ssh_passphrase, &enc_key)?;
        key.login_password_password = maybe_decrypt(&key.login_password_password, &enc_key)?;
        key.access_key_secret_key = maybe_decrypt(&key.access_key_secret_key, &enc_key)?;
        Ok(())
    }

    fn fill_environment_secrets(&self, _env: &mut crate::models::Environment, _deserialize_secret: bool) -> Result<()> {
        Ok(())
    }

    fn delete_secret(&self, _key: &AccessKey) -> Result<()> {
        Ok(())
    }
}

// ============================================================================
// Helper functions (pub для использования в handlers)
// ============================================================================

/// Шифрует секретные поля ключа перед записью в БД
pub fn encrypt_key_secrets(key: &mut AccessKey) {
    let enc_key = match get_encryption_key() {
        Some(k) => k,
        None => return,
    };
    if let Ok(v) = maybe_encrypt(&key.ssh_key, &enc_key) { key.ssh_key = v; }
    if let Ok(v) = maybe_encrypt(&key.ssh_passphrase, &enc_key) { key.ssh_passphrase = v; }
    if let Ok(v) = maybe_encrypt(&key.login_password_password, &enc_key) { key.login_password_password = v; }
    if let Ok(v) = maybe_encrypt(&key.access_key_secret_key, &enc_key) { key.access_key_secret_key = v; }
}

/// Дешифрует секретные поля ключа после загрузки из БД
pub fn decrypt_key_secrets(key: &mut AccessKey) {
    let enc_key = match get_encryption_key() {
        Some(k) => k,
        None => return,
    };
    if let Ok(v) = maybe_decrypt(&key.ssh_key, &enc_key) { key.ssh_key = v; }
    if let Ok(v) = maybe_decrypt(&key.ssh_passphrase, &enc_key) { key.ssh_passphrase = v; }
    if let Ok(v) = maybe_decrypt(&key.login_password_password, &enc_key) { key.login_password_password = v; }
    if let Ok(v) = maybe_decrypt(&key.access_key_secret_key, &enc_key) { key.access_key_secret_key = v; }
}

/// Маскирует секретные поля для API-ответа (никогда не возвращать plaintext)
pub fn mask_key_secrets(key: &mut AccessKey) {
    if key.ssh_key.is_some() {
        key.ssh_key = Some("**SECRET**".to_string());
    }
    if key.ssh_passphrase.is_some() {
        key.ssh_passphrase = Some("**SECRET**".to_string());
    }
    if key.login_password_password.is_some() {
        key.login_password_password = Some("**SECRET**".to_string());
    }
    if key.access_key_secret_key.is_some() {
        key.access_key_secret_key = Some("**SECRET**".to_string());
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let plaintext = "my_secret_password";
        let encrypted = encrypt_value(plaintext, &key).unwrap();
        assert!(encrypted.starts_with(ENC_PREFIX));
        let decrypted = decrypt_value(&encrypted, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_maybe_encrypt_skip_already_encrypted() {
        let key = [42u8; 32];
        let already = Some(format!("{}somedata", ENC_PREFIX));
        let result = maybe_encrypt(&already, &key).unwrap();
        assert_eq!(result, already); // не меняем
    }

    #[test]
    fn test_maybe_decrypt_passthrough_plaintext() {
        let key = [42u8; 32];
        let plaintext = Some("not_encrypted".to_string());
        let result = maybe_decrypt(&plaintext, &key).unwrap();
        assert_eq!(result, plaintext);
    }

    #[test]
    fn test_maybe_encrypt_none() {
        let key = [42u8; 32];
        let result = maybe_encrypt(&None, &key).unwrap();
        assert_eq!(result, None);
    }
}
