//! Модель сессии

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{database::Database, decode::Decode, encode::Encode, FromRow, Type};

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
}
