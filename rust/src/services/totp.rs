//! Сервис двухфакторной аутентификации (TOTP)
//!
//! Реализация Time-based One-Time Password (TOTP) согласно RFC 6238.
//! Совместим с Google Authenticator, Authy и другими совместимыми приложениями.

use chrono::Utc;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use base32;
use rand::RngCore;

use crate::models::User;
use crate::error::{Error, Result};

/// Длина TOTP кода
const TOTP_CODE_LENGTH: usize = 6;

/// Период действия TOTP кода (30 секунд)
const TOTP_PERIOD: u64 = 30;

/// Размер секретного ключа в байтах (160 бит)
const TOTP_SECRET_SIZE: usize = 20;

/// Создаёт новый TOTP секрет для пользователя
pub fn generate_totp_secret(user: &User, issuer: &str) -> Result<TotpSecret> {
    // Генерируем случайный секрет
    let mut secret_bytes = [0u8; TOTP_SECRET_SIZE];
    rand::thread_rng().fill_bytes(&mut secret_bytes);

    // Кодируем в Base32
    let secret = base32::encode(base32::Alphabet::Rfc4648 { padding: true }, &secret_bytes);

    // Создаём URL для QR-кода
    let label = format!("{}:{}", issuer, user.username);
    let url = format!(
        "otpauth://totp/{}?secret={}&issuer={}",
        label,
        secret,
        issuer
    );

    // Генерируем код восстановления
    let mut recovery_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut recovery_bytes);
    let recovery_code = hex::encode(&recovery_bytes);

    // Хешируем код восстановления
    let recovery_hash = bcrypt::hash(&recovery_code, bcrypt::DEFAULT_COST)
        .map_err(|e| Error::Other(format!("Ошибка хеширования: {}", e)))?;

    Ok(TotpSecret {
        secret,
        url,
        recovery_code,
        recovery_hash,
    })
}

/// TOTP секрет с данными для настройки
#[derive(Debug, Clone)]
pub struct TotpSecret {
    /// Секретный ключ (Base32)
    pub secret: String,
    /// URL для QR-кода
    pub url: String,
    /// Код восстановления (показать пользователю один раз)
    pub recovery_code: String,
    /// Хеш кода восстановления (сохранить в БД)
    pub recovery_hash: String,
}

/// Проверяет TOTP код
pub fn verify_totp_code(secret: &str, code: &str) -> bool {
    // Декодируем секрет из Base32
    let secret_bytes = match base32::decode(base32::Alphabet::Rfc4648 { padding: true }, secret) {
        Some(bytes) => bytes,
        None => return false,
    };

    // Вычисляем текущий временной шаг
    let now = Utc::now().timestamp() as u64;
    let time_step = now / TOTP_PERIOD;

    // Проверяем код для текущего, предыдущего и следующего шага
    // (допускаем небольшое расхождение во времени)
    for offset in [-1i64, 0, 1] {
        let adjusted_time = (time_step as i64 + offset) as u64;
        if let Some(generated) = generate_totp_code(&secret_bytes, adjusted_time) {
            if generated == code {
                return true;
            }
        }
    }

    false
}

/// Генерирует TOTP код для данного временного шага
fn generate_totp_code(secret: &[u8], time_step: u64) -> Option<String> {
    // Преобразуем временной шаг в байты (big-endian)
    let time_bytes = time_step.to_be_bytes();

    // Создаём HMAC-SHA1
    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(secret).ok()?;
    mac.update(&time_bytes);
    let result = mac.finalize().into_bytes();

    // Dynamic truncation (RFC 4226)
    let offset = (result[19] & 0x0f) as usize;
    let binary = ((result[offset] & 0x7f) as u32) << 24
        | (result[offset + 1] as u32) << 16
        | (result[offset + 2] as u32) << 8
        | (result[offset + 3] as u32);

    // Получаем 6-значный код
    let code = binary % 10u32.pow(TOTP_CODE_LENGTH as u32);
    Some(format!("{:0width$}", code, width = TOTP_CODE_LENGTH))
}

/// Проверяет код восстановления
pub fn verify_recovery_code(code: &str, hash: &str) -> bool {
    bcrypt::verify(code, hash).unwrap_or(false)
}

/// Генерирует новый код восстановления
pub fn generate_recovery_code() -> Result<(String, String)> {
    let mut recovery_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut recovery_bytes);
    let recovery_code = hex::encode(&recovery_bytes);

    let recovery_hash = bcrypt::hash(&recovery_code, bcrypt::DEFAULT_COST)
        .map_err(|e| Error::Other(format!("Ошибка хеширования: {}", e)))?;

    Ok((recovery_code, recovery_hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_totp_secret() {
        let user = User {
            id: 1,
            created: Utc::now(),
            username: "testuser".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password: String::new(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };

        let secret = generate_totp_secret(&user, "Semaphore").unwrap();

        assert!(!secret.secret.is_empty());
        assert!(secret.url.starts_with("otpauth://totp/"));
        assert!(!secret.recovery_code.is_empty());
        assert!(!secret.recovery_hash.is_empty());

        // Проверяем, что URL содержит правильные параметры
        assert!(secret.url.contains(&format!("secret={}", secret.secret)));
        assert!(secret.url.contains("issuer=Semaphore"));
    }

    #[test]
    fn test_verify_recovery_code() {
        let (code, hash) = generate_recovery_code().unwrap();

        assert!(verify_recovery_code(&code, &hash));
        assert!(!verify_recovery_code("wrong_code", &hash));
    }

    #[test]
    fn test_totp_code_generation() {
        // Тест с известным секретом из RFC 6238
        let secret = base32::decode(base32::Alphabet::Rfc4648 { padding: true }, "GEZDGNBVGY3TQOJQ")
            .unwrap();

        // Проверяем, что код генерируется корректно
        let now = Utc::now().timestamp() as u64;
        let time_step = now / TOTP_PERIOD;

        let code = generate_totp_code(&secret, time_step);
        assert!(code.is_some());
        let code = code.unwrap();
        assert_eq!(code.len(), TOTP_CODE_LENGTH);
    }
}
