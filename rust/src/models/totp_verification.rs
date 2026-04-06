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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_verification_serialization() {
        let totp = TotpVerification {
            secret: "JBSWY3DPEHPK3PXP".to_string(),
            recovery_hash: "hash123".to_string(),
            recovery_codes: Some(vec!["code1".to_string(), "code2".to_string()]),
        };
        let json = serde_json::to_string(&totp).unwrap();
        assert!(json.contains("\"secret\":\"JBSWY3DPEHPK3PXP\""));
        assert!(json.contains("\"recovery_hash\":\"hash123\""));
    }

    #[test]
    fn test_totp_verification_no_recovery_codes() {
        let totp = TotpVerification {
            secret: "secret".to_string(),
            recovery_hash: "hash".to_string(),
            recovery_codes: None,
        };
        let json = serde_json::to_string(&totp).unwrap();
        // TotpVerification doesn't have skip_serializing_if on recovery_codes
        assert!(json.contains("\"recovery_codes\":null"));
    }
}
