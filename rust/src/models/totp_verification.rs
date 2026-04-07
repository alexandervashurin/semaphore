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

    #[test]
    fn test_totp_verification_deserialization() {
        let json = r#"{"secret":"ABC123","recovery_hash":"hash456","recovery_codes":["rc1","rc2"]}"#;
        let totp: TotpVerification = serde_json::from_str(json).unwrap();
        assert_eq!(totp.secret, "ABC123");
        assert_eq!(totp.recovery_codes.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_totp_verification_clone() {
        let totp = TotpVerification {
            secret: "clone_test".to_string(),
            recovery_hash: "clone_hash".to_string(),
            recovery_codes: Some(vec!["code1".to_string()]),
        };
        let cloned = totp.clone();
        assert_eq!(cloned.secret, totp.secret);
        assert_eq!(cloned.recovery_hash, totp.recovery_hash);
    }
}
