//! Модель сессии

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Метод верификации сессии
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionVerificationMethod {
    None,
    Totp,
    EmailOtp,
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
