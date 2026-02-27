//! TOTP Verification модель

use serde::{Deserialize, Serialize};

/// TOTP верификация
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpVerification {
    /// Секретный ключ
    pub secret: String,
    /// Хеш кода восстановления
    pub recovery_hash: String,
    /// Коды восстановления (опционально)
    pub recovery_codes: Option<Vec<String>>,
}
