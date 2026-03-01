//! Модель сессии

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type, decode::Decode, encode::Encode, database::Database};

/// Метод верификации сессии
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionVerificationMethod {
    None,
    Totp,
    EmailOtp,
}

impl<DB: Database> Type<DB> for SessionVerificationMethod {
    fn type_info() -> DB::TypeInfo {
        String::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        String::compatible(ty)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for SessionVerificationMethod {
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = String::decode(value)?;
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
{
    fn encode_by_ref(&self, buf: &mut <DB as Database>::ArgumentBuffer<'q>) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s: String = match self {
            SessionVerificationMethod::None => "none",
            SessionVerificationMethod::Totp => "totp",
            SessionVerificationMethod::EmailOtp => "email_otp",
        }.to_string();
        Encode::encode(s, buf)
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
