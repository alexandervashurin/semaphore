//! Модель сессии

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type, database::Database, decode::Decode, encode::Encode};

/// Метод верификации сессии
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionVerificationMethod {
    None,
    Totp,
    EmailOtp,
}

impl<DB: Database> Type<DB> for SessionVerificationMethod
where
    String: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for SessionVerificationMethod
where
    String: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as Decode<'r, DB>>::decode(value)?;
        Ok(match s.as_str() {
            "totp" => SessionVerificationMethod::Totp,
            "email_otp" => SessionVerificationMethod::EmailOtp,
            _ => SessionVerificationMethod::None,
        })
    }
}

impl<'q, DB: Database> Encode<'q, DB> for SessionVerificationMethod
where
    DB: 'q,
    String: Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s: String = match self {
            SessionVerificationMethod::None => "none",
            SessionVerificationMethod::Totp => "totp",
            SessionVerificationMethod::EmailOtp => "email_otp",
        }
        .to_string();
        <String as Encode<'q, DB>>::encode(s, buf)
    }
}

/// Сессия пользователя
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: i32,
    pub user_id: i32,
    pub created: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub ip: String,
    pub user_agent: String,
    pub expired: bool,
    pub verification_method: SessionVerificationMethod,
    pub verified: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_verification_method_display() {
        assert_eq!(
            serde_json::to_string(&SessionVerificationMethod::None).unwrap(),
            "\"none\""
        );
        assert_eq!(
            serde_json::to_string(&SessionVerificationMethod::Totp).unwrap(),
            "\"totp\""
        );
        assert_eq!(
            serde_json::to_string(&SessionVerificationMethod::EmailOtp).unwrap(),
            "\"email_otp\""
        );
    }

    #[test]
    fn test_session_serialization() {
        let session = Session {
            id: 1,
            user_id: 10,
            created: Utc::now(),
            last_active: Utc::now(),
            ip: "192.168.1.1".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
            expired: false,
            verification_method: SessionVerificationMethod::Totp,
            verified: true,
        };
        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("\"user_id\":10"));
        assert!(json.contains("\"ip\":\"192.168.1.1\""));
        assert!(json.contains("\"verified\":true"));
    }

    #[test]
    fn test_session_clone() {
        let session = Session {
            id: 1,
            user_id: 1,
            created: Utc::now(),
            last_active: Utc::now(),
            ip: "127.0.0.1".to_string(),
            user_agent: "Test Agent".to_string(),
            expired: false,
            verification_method: SessionVerificationMethod::None,
            verified: false,
        };
        let cloned = session.clone();
        assert_eq!(cloned.ip, session.ip);
        assert_eq!(cloned.verification_method, session.verification_method);
    }

    #[test]
    fn test_session_verification_method_deserialization() {
        let none_method: SessionVerificationMethod = serde_json::from_str("\"none\"").unwrap();
        let totp_method: SessionVerificationMethod = serde_json::from_str("\"totp\"").unwrap();
        let email_method: SessionVerificationMethod =
            serde_json::from_str("\"email_otp\"").unwrap();
        assert_eq!(none_method, SessionVerificationMethod::None);
        assert_eq!(totp_method, SessionVerificationMethod::Totp);
        assert_eq!(email_method, SessionVerificationMethod::EmailOtp);
    }

    #[test]
    fn test_session_verification_method_unknown_deserialization() {
        let result: Result<SessionVerificationMethod, _> = serde_json::from_str("\"unknown\"");
        assert!(result.is_err()); // unknown variant is rejected
    }

    #[test]
    fn test_session_debug() {
        let session = Session {
            id: 1,
            user_id: 1,
            created: Utc::now(),
            last_active: Utc::now(),
            ip: "127.0.0.1".to_string(),
            user_agent: "Test".to_string(),
            expired: false,
            verification_method: SessionVerificationMethod::None,
            verified: false,
        };
        let debug_str = format!("{:?}", session);
        assert!(debug_str.contains("Session"));
    }

    #[test]
    fn test_session_expired_true() {
        let session = Session {
            id: 1,
            user_id: 1,
            created: Utc::now(),
            last_active: Utc::now(),
            ip: "10.0.0.1".to_string(),
            user_agent: "Browser".to_string(),
            expired: true,
            verification_method: SessionVerificationMethod::EmailOtp,
            verified: true,
        };
        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("\"expired\":true"));
        assert!(json.contains("\"verified\":true"));
    }

    #[test]
    fn test_session_full_serialization() {
        let session = Session {
            id: 100,
            user_id: 50,
            created: Utc::now(),
            last_active: Utc::now(),
            ip: "192.168.0.1".to_string(),
            user_agent: "Chrome/120.0".to_string(),
            expired: false,
            verification_method: SessionVerificationMethod::Totp,
            verified: true,
        };
        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("\"id\":100"));
        assert!(json.contains("\"user_id\":50"));
        assert!(json.contains("\"ip\":\"192.168.0.1\""));
    }

    #[test]
    fn test_session_deserialization_full() {
        let json = r#"{"id":5,"user_id":10,"created":"2024-01-01T00:00:00Z","last_active":"2024-01-01T01:00:00Z","ip":"10.0.0.1","user_agent":"Firefox","expired":false,"verification_method":"totp","verified":true}"#;
        let session: Session = serde_json::from_str(json).unwrap();
        assert_eq!(session.id, 5);
        assert_eq!(session.user_id, 10);
        assert_eq!(session.ip, "10.0.0.1");
        assert_eq!(session.verification_method, SessionVerificationMethod::Totp);
    }

    #[test]
    fn test_session_verification_method_clone() {
        let method = SessionVerificationMethod::EmailOtp;
        let cloned = method.clone();
        assert_eq!(cloned, method);
    }

    #[test]
    fn test_session_unicode_ip_and_user_agent() {
        let session = Session {
            id: 1,
            user_id: 1,
            created: Utc::now(),
            last_active: Utc::now(),
            ip: "0:0:0:0:0:0:0:1".to_string(),
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64)".to_string(),
            expired: false,
            verification_method: SessionVerificationMethod::None,
            verified: false,
        };
        let json = serde_json::to_string(&session).unwrap();
        let restored: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.ip, "0:0:0:0:0:0:0:1");
    }

    #[test]
    fn test_session_clone_independence() {
        let mut session = Session {
            id: 1,
            user_id: 1,
            created: Utc::now(),
            last_active: Utc::now(),
            ip: "127.0.0.1".to_string(),
            user_agent: "Test".to_string(),
            expired: false,
            verification_method: SessionVerificationMethod::None,
            verified: false,
        };
        let cloned = session.clone();
        session.ip = "192.168.1.1".to_string();
        assert_eq!(cloned.ip, "127.0.0.1");
    }

    #[test]
    fn test_session_debug_contains_fields() {
        let session = Session {
            id: 42,
            user_id: 10,
            created: Utc::now(),
            last_active: Utc::now(),
            ip: "10.0.0.1".to_string(),
            user_agent: "DebugAgent".to_string(),
            expired: true,
            verification_method: SessionVerificationMethod::Totp,
            verified: true,
        };
        let debug_str = format!("{:?}", session);
        assert!(debug_str.contains("42"));
        assert!(debug_str.contains("DebugAgent"));
        assert!(debug_str.contains("Session"));
    }

    #[test]
    fn test_session_all_verification_methods() {
        for method in [
            SessionVerificationMethod::None,
            SessionVerificationMethod::Totp,
            SessionVerificationMethod::EmailOtp,
        ] {
            let session = Session {
                id: 1,
                user_id: 1,
                created: Utc::now(),
                last_active: Utc::now(),
                ip: "127.0.0.1".to_string(),
                user_agent: "Test".to_string(),
                expired: false,
                verification_method: method.clone(),
                verified: true,
            };
            let json = serde_json::to_string(&session).unwrap();
            assert!(json.contains("\"verification_method\":"));
        }
    }

    #[test]
    fn test_session_roundtrip() {
        let original = Session {
            id: 42,
            user_id: 10,
            created: Utc::now(),
            last_active: Utc::now(),
            ip: "192.168.1.100".to_string(),
            user_agent: "TestAgent/2.0".to_string(),
            expired: false,
            verification_method: SessionVerificationMethod::Totp,
            verified: true,
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(original.id, restored.id);
        assert_eq!(original.ip, restored.ip);
        assert_eq!(original.verification_method, restored.verification_method);
    }
}
