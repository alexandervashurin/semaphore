//! Session CRUD Operations
//!
//! Операции с сессиями в SQL

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::{Session, SessionVerificationMethod};
use sqlx::Row;

impl SqlDb {
    fn pg_pool_session(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает сессию по ID
    pub async fn get_session(&self, _user_id: i32, session_id: i32) -> Result<Session> {
        let row = sqlx::query("SELECT * FROM session WHERE id = $1")
            .bind(session_id)
            .fetch_optional(self.pg_pool_session()?)
            .await
            .map_err(Error::Database)?
            .ok_or_else(|| Error::NotFound("Сессия не найдена".to_string()))?;

        Ok(Session {
            id: row.get("id"),
            user_id: row.get("user_id"),
            created: row.get("created"),
            last_active: row.get("last_active"),
            ip: row.try_get("ip").ok().unwrap_or_default(),
            user_agent: row.try_get("user_agent").ok().unwrap_or_default(),
            expired: row.get("expired"),
            verification_method: row
                .try_get("verification_method")
                .ok()
                .unwrap_or(SessionVerificationMethod::None),
            verified: row.try_get("verified").ok().unwrap_or(false),
        })
    }

    /// Создаёт сессию
    pub async fn create_session(&self, mut session: Session) -> Result<Session> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO session (user_id, created, last_active, ip, user_agent, expired, \
             verification_method, verified) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
        )
        .bind(session.user_id)
        .bind(session.created)
        .bind(session.last_active)
        .bind(&session.ip)
        .bind(&session.user_agent)
        .bind(session.expired)
        .bind(&session.verification_method)
        .bind(session.verified)
        .fetch_one(self.pg_pool_session()?)
        .await
        .map_err(Error::Database)?;

        session.id = id;
        Ok(session)
    }

    /// Истекает сессию
    pub async fn expire_session(&self, _user_id: i32, session_id: i32) -> Result<()> {
        sqlx::query("UPDATE session SET expired = TRUE WHERE id = $1")
            .bind(session_id)
            .execute(self.pg_pool_session()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_session_verification_method_none_display() {
        let json = serde_json::to_string(&SessionVerificationMethod::None).unwrap();
        assert_eq!(json, "\"none\"");
    }

    #[test]
    fn test_session_verification_method_totp_display() {
        let json = serde_json::to_string(&SessionVerificationMethod::Totp).unwrap();
        assert_eq!(json, "\"totp\"");
    }

    #[test]
    fn test_session_verification_method_email_otp_display() {
        let json = serde_json::to_string(&SessionVerificationMethod::EmailOtp).unwrap();
        assert_eq!(json, "\"email_otp\"");
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
        assert!(json.contains("\"expired\":false"));
    }

    #[test]
    fn test_session_deserialization() {
        let json = r#"{"id":5,"user_id":20,"created":"2024-01-01T00:00:00Z","last_active":"2024-01-01T01:00:00Z","ip":"10.0.0.1","user_agent":"Test","expired":false,"verification_method":"email_otp","verified":true}"#;
        let session: Session = serde_json::from_str(json).unwrap();
        assert_eq!(session.id, 5);
        assert_eq!(session.user_id, 20);
        assert_eq!(session.ip, "10.0.0.1");
        assert_eq!(session.verification_method, SessionVerificationMethod::EmailOtp);
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
    fn test_session_debug_format() {
        let session = Session {
            id: 42,
            user_id: 1,
            created: Utc::now(),
            last_active: Utc::now(),
            ip: "10.0.0.1".to_string(),
            user_agent: "Debug".to_string(),
            expired: false,
            verification_method: SessionVerificationMethod::None,
            verified: false,
        };
        let debug_str = format!("{:?}", session);
        assert!(debug_str.contains("Session"));
        assert!(debug_str.contains("Debug"));
    }

    #[test]
    fn test_session_expired_flag() {
        let session = Session {
            id: 1,
            user_id: 1,
            created: Utc::now(),
            last_active: Utc::now(),
            ip: String::new(),
            user_agent: String::new(),
            expired: true,
            verification_method: SessionVerificationMethod::None,
            verified: false,
        };
        assert!(session.expired);
    }

    #[test]
    fn test_session_serialization_roundtrip() {
        let original = Session {
            id: 100,
            user_id: 50,
            created: Utc::now(),
            last_active: Utc::now(),
            ip: "172.16.0.1".to_string(),
            user_agent: "Roundtrip".to_string(),
            expired: false,
            verification_method: SessionVerificationMethod::Totp,
            verified: true,
        };
        let json = serde_json::to_string(&original).unwrap();
        let decoded: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, original.id);
        assert_eq!(decoded.user_id, original.user_id);
        assert_eq!(decoded.ip, original.ip);
        assert_eq!(decoded.verification_method, original.verification_method);
    }

    #[test]
    fn test_session_empty_strings() {
        let session = Session {
            id: 1,
            user_id: 1,
            created: Utc::now(),
            last_active: Utc::now(),
            ip: "".to_string(),
            user_agent: "".to_string(),
            expired: false,
            verification_method: SessionVerificationMethod::None,
            verified: false,
        };
        assert!(session.ip.is_empty());
        assert!(session.user_agent.is_empty());
    }

    #[test]
    fn test_session_verification_method_deserialization() {
        let none: SessionVerificationMethod = serde_json::from_str("\"none\"").unwrap();
        assert_eq!(none, SessionVerificationMethod::None);

        let totp: SessionVerificationMethod = serde_json::from_str("\"totp\"").unwrap();
        assert_eq!(totp, SessionVerificationMethod::Totp);

        let email_otp: SessionVerificationMethod = serde_json::from_str("\"email_otp\"").unwrap();
        assert_eq!(email_otp, SessionVerificationMethod::EmailOtp);
    }
}
