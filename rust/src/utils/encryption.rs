//! Модуль криптографии
//!
//! Предоставляет функции для генерации RSA ключей

use pem::{encode, Pem};
use rand::rngs::OsRng;
use rsa::{pkcs1::EncodeRsaPrivateKey, pkcs1::EncodeRsaPublicKey, RsaPrivateKey, RsaPublicKey};
use std::io::Write;
use thiserror::Error;

/// Типы ошибок encryption
#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error("Ошибка генерации ключа: {0}")]
    KeyGeneration(String),

    #[error("Ошибка кодирования ключа: {0}")]
    Encoding(String),

    #[error("Ошибка записи: {0}")]
    WriteError(String),
}

impl From<rsa::Error> for EncryptionError {
    fn from(err: rsa::Error) -> Self {
        EncryptionError::KeyGeneration(err.to_string())
    }
}

impl From<pkcs1::Error> for EncryptionError {
    fn from(err: pkcs1::Error) -> Self {
        EncryptionError::Encoding(err.to_string())
    }
}

impl From<std::io::Error> for EncryptionError {
    fn from(err: std::io::Error) -> Self {
        EncryptionError::WriteError(err.to_string())
    }
}

/// Результат генерации ключа
pub struct KeyPair {
    /// Публичный ключ в PEM формате
    pub public_key: String,
}

/// Генерирует RSA приватный ключ (2048 бит) и записывает его в файл
///
/// Возвращает публичный ключ в PEM формате
///
/// # Пример
///
/// ```ignore
/// let mut file = File::create("private_key.pem")?;
/// let keypair = generate_private_key(&mut file)?;
/// println!("Public key: {}", keypair.public_key);
/// ```
pub fn generate_private_key<W: Write>(private_key_file: &mut W) -> Result<KeyPair, EncryptionError> {
    // 1. Генерация RSA приватного ключа (2048 бита)
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048)?;

    // 2. Кодирование приватного ключа в PKCS#1 ASN.1 PEM
    let private_key_bytes = private_key.to_pkcs1_der()?;
    let private_key_pem = Pem::new("RSA PRIVATE KEY", private_key_bytes.as_bytes());
    let private_key_pem_string = encode(&private_key_pem);

    // 3. Запись приватного ключа в файл
    write!(private_key_file, "{}", private_key_pem_string)?;

    // 4. Кодирование публичного ключа
    let public_key = private_key.to_public_key();
    let public_key_bytes = public_key.to_pkcs1_der()?;
    let public_key_pem = Pem::new("PUBLIC KEY", public_key_bytes.as_bytes());
    let public_key_pem_string = encode(&public_key_pem);

    Ok(KeyPair {
        public_key: public_key_pem_string,
    })
}

// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_generate_private_key() {
        let mut buffer = Cursor::new(Vec::new());
        let result = generate_private_key(&mut buffer);

        assert!(result.is_ok());
        let keypair = result.unwrap();

        // Проверяем что приватный ключ записан
        let private_key_string = String::from_utf8(buffer.into_inner()).unwrap();
        assert!(private_key_string.contains("-----BEGIN RSA PRIVATE KEY-----"));
        assert!(private_key_string.contains("-----END RSA PRIVATE KEY-----"));

        // Проверяем что публичный ключ корректный
        assert!(keypair.public_key.contains("-----BEGIN PUBLIC KEY-----"));
        assert!(keypair.public_key.contains("-----END PUBLIC KEY-----"));
    }

    #[test]
    fn test_generate_private_key_format() {
        let mut buffer = Cursor::new(Vec::new());
        let result = generate_private_key(&mut buffer).unwrap();

        // Проверяем PEM формат
        let lines: Vec<&str> = result.public_key.lines().collect();
        assert!(lines.len() > 2);
        assert_eq!(lines[0], "-----BEGIN PUBLIC KEY-----");
        assert!(lines.last().unwrap().contains(&"-----END PUBLIC KEY-----"));
    }

    #[test]
    fn test_key_size() {
        let mut buffer = Cursor::new(Vec::new());
        let result = generate_private_key(&mut buffer).unwrap();

        // Приблизительная проверка размера ключа по длине PEM
        // 2048-bit ключ в PEM формате PKCS#1 должен быть примерно 451 символ
        let private_key_string = String::from_utf8(buffer.into_inner()).unwrap();
        // Удаляем заголовки и переводы строк для проверки длины
        let key_body: String = private_key_string
            .lines()
            .filter(|line| !line.starts_with("-----"))
            .collect();
        
        // Длина должна быть около 344 символов (2048 бит = 256 байт в base64)
        assert!(key_body.len() > 300);
    }
}
