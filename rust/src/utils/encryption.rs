//! Модуль криптографии
//!
//! AES-256-GCM для симметричного шифрования секретов.
//! ECDSA P-256 для генерации ключевых пар (runner keypairs).
//!
//! Замена RSA → P-256: RUSTSEC-2023-0071 (Marvin Attack).

use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng as AesOsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use p256::ecdsa::SigningKey;
use p256::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use std::io::Write;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error("Ошибка генерации ключа: {0}")]
    KeyGeneration(String),
    #[error("Ошибка кодирования ключа: {0}")]
    Encoding(String),
    #[error("Ошибка записи: {0}")]
    WriteError(String),
}

impl From<std::io::Error> for EncryptionError {
    fn from(err: std::io::Error) -> Self {
        EncryptionError::WriteError(err.to_string())
    }
}

pub struct KeyPair {
    pub public_key: String,
}

pub fn aes256_encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<String, EncryptionError> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Aes256Gcm::generate_nonce(&mut AesOsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| EncryptionError::Encoding(e.to_string()))?;
    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(BASE64.encode(combined))
}

pub fn aes256_decrypt(encoded: &str, key: &[u8; 32]) -> Result<Vec<u8>, EncryptionError> {
    let data = BASE64
        .decode(encoded)
        .map_err(|e| EncryptionError::Encoding(e.to_string()))?;
    if data.len() < 12 {
        return Err(EncryptionError::Encoding("Ciphertext too short".to_string()));
    }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| EncryptionError::Encoding(e.to_string()))
}

/// Генерирует ECDSA P-256 приватный ключ и записывает его в файл (PKCS#8 PEM).
/// Заменяет RSA-2048 — устраняет RUSTSEC-2023-0071 (Marvin Attack).
pub fn generate_private_key<W: Write>(
    private_key_file: &mut W,
) -> Result<KeyPair, EncryptionError> {
    let signing_key = SigningKey::random(&mut rand::rngs::OsRng);
    let private_pem = signing_key
        .to_pkcs8_pem(LineEnding::LF)
        .map_err(|e| EncryptionError::Encoding(e.to_string()))?;
    write!(private_key_file, "{}", private_pem.as_str())?;
    let public_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .map_err(|e| EncryptionError::Encoding(e.to_string()))?;
    Ok(KeyPair { public_key: public_pem })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_generate_private_key() {
        let mut buf = Cursor::new(Vec::new());
        let kp = generate_private_key(&mut buf).unwrap();
        let priv_str = String::from_utf8(buf.into_inner()).unwrap();
        assert!(priv_str.contains("-----BEGIN PRIVATE KEY-----"));
        assert!(kp.public_key.contains("-----BEGIN PUBLIC KEY-----"));
    }

    #[test]
    fn test_key_uniqueness() {
        let mut b1 = Cursor::new(Vec::new());
        let mut b2 = Cursor::new(Vec::new());
        let kp1 = generate_private_key(&mut b1).unwrap();
        let kp2 = generate_private_key(&mut b2).unwrap();
        assert_ne!(kp1.public_key, kp2.public_key);
    }

    #[test]
    fn test_aes256_roundtrip() {
        let key = [0u8; 32];
        let plaintext = b"hello secret world";
        let enc = aes256_encrypt(plaintext, &key).unwrap();
        let dec = aes256_decrypt(&enc, &key).unwrap();
        assert_eq!(dec, plaintext);
    }

    #[test]
    fn test_aes256_different_nonces() {
        let key = [42u8; 32];
        let enc1 = aes256_encrypt(b"same", &key).unwrap();
        let enc2 = aes256_encrypt(b"same", &key).unwrap();
        assert_ne!(enc1, enc2);
    }

    #[test]
    fn test_aes256_wrong_key_fails() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let enc = aes256_encrypt(b"secret", &key1).unwrap();
        assert!(aes256_decrypt(&enc, &key2).is_err());
    }

    #[test]
    fn test_aes256_decrypt_invalid_base64() {
        let key = [0u8; 32];
        let result = aes256_decrypt("not-valid-base64!@#$%", &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_aes256_decrypt_too_short() {
        let key = [0u8; 32];
        // Base64 строка "YWJj" декодируется в 3 байта, что меньше 12 байт nonce
        let result = aes256_decrypt("YWJj", &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_encryption_error_display() {
        let err = EncryptionError::KeyGeneration("test error".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("test error"));
    }

    #[test]
    fn test_aes256_empty_plaintext() {
        let key = [0u8; 32];
        let enc = aes256_encrypt(b"", &key).unwrap();
        let dec = aes256_decrypt(&enc, &key).unwrap();
        assert_eq!(dec, b"");
    }
}
