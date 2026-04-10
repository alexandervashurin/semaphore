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

    #[test]
    fn test_totp_verification_debug() {
        let totp = TotpVerification {
            secret: "debug_secret".to_string(),
            recovery_hash: "debug_hash".to_string(),
            recovery_codes: None,
        };
        let debug_str = format!("{:?}", totp);
        assert!(debug_str.contains("TotpVerification"));
        assert!(debug_str.contains("debug_secret"));
    }

    #[test]
    fn test_totp_verification_deserialization_no_codes() {
        let json = r#"{"secret":"NOCODES","recovery_hash":"hash1","recovery_codes":null}"#;
        let totp: TotpVerification = serde_json::from_str(json).unwrap();
        assert_eq!(totp.secret, "NOCODES");
        assert!(totp.recovery_codes.is_none());
    }

    #[test]
    fn test_totp_verification_empty_recovery_codes() {
        let totp = TotpVerification {
            secret: "empty_codes".to_string(),
            recovery_hash: "hash".to_string(),
            recovery_codes: Some(vec![]),
        };
        let json = serde_json::to_string(&totp).unwrap();
        assert!(json.contains("\"recovery_codes\":[]"));
    }

    #[test]
    fn test_totp_verification_multiple_codes() {
        let codes = vec!["code1".to_string(), "code2".to_string(), "code3".to_string(), "code4".to_string(), "code5".to_string()];
        let totp = TotpVerification {
            secret: "multi".to_string(),
            recovery_hash: "hash_multi".to_string(),
            recovery_codes: Some(codes),
        };
        let json = serde_json::to_string(&totp).unwrap();
        assert!(json.contains("\"recovery_codes\":["));
        let deserialized: TotpVerification = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.recovery_codes.as_ref().unwrap().len(), 5);
    }

    #[test]
    fn test_totp_verification_clone_with_codes() {
        let totp = TotpVerification {
            secret: "clone_codes".to_string(),
            recovery_hash: "hash_clone".to_string(),
            recovery_codes: Some(vec!["a".to_string(), "b".to_string()]),
        };
        let cloned = totp.clone();
        assert_eq!(cloned.recovery_codes.as_ref().unwrap().len(), 2);
        assert_eq!(cloned.recovery_codes, totp.recovery_codes);
    }

    #[test]
    fn test_totp_verification_special_chars() {
        let totp = TotpVerification {
            secret: "JBSWY3DPEHPK3PXP+/==".to_string(),
            recovery_hash: "hash$%^&*".to_string(),
            recovery_codes: None,
        };
        let json = serde_json::to_string(&totp).unwrap();
        let deserialized: TotpVerification = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.secret, "JBSWY3DPEHPK3PXP+/==");
    }

    #[test]
    fn test_totp_verification_roundtrip() {
        let original = TotpVerification {
            secret: "roundtrip".to_string(),
            recovery_hash: "roundtrip_hash".to_string(),
            recovery_codes: Some(vec!["rc1".to_string(), "rc2".to_string()]),
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: TotpVerification = serde_json::from_str(&json).unwrap();
        assert_eq!(original.secret, restored.secret);
        assert_eq!(original.recovery_hash, restored.recovery_hash);
        assert_eq!(original.recovery_codes, restored.recovery_codes);
    }
}
