//! SessionManager - управление сессиями

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::{Session, SessionVerificationMethod};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

#[async_trait]
impl SessionManager for SqlStore {
    async fn get_session(&self, _user_id: i32, session_id: i32) -> Result<Session> {
        let query = "SELECT * FROM session WHERE id = $1";
        let row = sqlx::query(query)
            .bind(session_id)
            .fetch_optional(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

        let row = row.ok_or_else(|| Error::NotFound("Сессия не найдена".to_string()))?;

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

    async fn create_session(&self, mut session: Session) -> Result<Session> {
        let query = "INSERT INTO session (user_id, created, last_active, ip, user_agent, expired, verification_method, verified) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id";
        let id: i32 = sqlx::query_scalar(query)
            .bind(session.user_id)
            .bind(session.created)
            .bind(session.last_active)
            .bind(&session.ip)
            .bind(&session.user_agent)
            .bind(session.expired)
            .bind(&session.verification_method)
            .bind(session.verified)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        session.id = id;
        Ok(session)
    }

    async fn expire_session(&self, _user_id: i32, session_id: i32) -> Result<()> {
        let query = "UPDATE session SET expired = TRUE WHERE id = $1";
        sqlx::query(query)
            .bind(session_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn verify_session(&self, _user_id: i32, session_id: i32) -> Result<()> {
        sqlx::query("UPDATE session SET verified = TRUE WHERE id = $1")
            .bind(session_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn touch_session(&self, _user_id: i32, session_id: i32) -> Result<()> {
        let query = "UPDATE session SET last_active = $1 WHERE id = $2";
        sqlx::query(query)
            .bind(Utc::now())
            .bind(session_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{Session, SessionVerificationMethod};
    use chrono::Utc;

    #[test]
    fn test_session_verification_method_serialization() {
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
            id: 1, user_id: 10, created: Utc::now(), last_active: Utc::now(),
            ip: "192.168.1.1".to_string(), user_agent: "Mozilla/5.0".to_string(),
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
            id: 1, user_id: 1, created: Utc::now(), last_active: Utc::now(),
            ip: "127.0.0.1".to_string(), user_agent: "Test".to_string(),
            expired: false, verification_method: SessionVerificationMethod::None,
            verified: false,
        };
        let cloned = session.clone();
        assert_eq!(cloned.ip, session.ip);
        assert_eq!(cloned.verification_method, session.verification_method);
    }

    #[test]
    fn test_session_expired() {
        let session = Session {
            id: 1, user_id: 1, created: Utc::now(), last_active: Utc::now(),
            ip: "10.0.0.1".to_string(), user_agent: "Agent".to_string(),
            expired: true, verification_method: SessionVerificationMethod::None,
            verified: false,
        };
        assert!(session.expired);
    }

    #[test]
    fn test_session_verified() {
        let session = Session {
            id: 1, user_id: 1, created: Utc::now(), last_active: Utc::now(),
            ip: String::new(), user_agent: String::new(),
            expired: false, verification_method: SessionVerificationMethod::EmailOtp,
            verified: true,
        };
        assert!(session.verified);
        assert_eq!(session.verification_method, SessionVerificationMethod::EmailOtp);
    }

    #[test]
    fn test_session_deserialization() {
        let json = r#"{"id":5,"user_id":20,"created":"2024-01-01T00:00:00Z","last_active":"2024-01-01T01:00:00Z","ip":"10.0.0.5","user_agent":"Chrome","expired":false,"verification_method":"totp","verified":true}"#;
        let session: Session = serde_json::from_str(json).unwrap();
        assert_eq!(session.id, 5);
        assert_eq!(session.user_id, 20);
        assert_eq!(session.ip, "10.0.0.5");
        assert_eq!(session.verification_method, SessionVerificationMethod::Totp);
    }

    #[test]
    fn test_session_all_verification_methods() {
        let methods = [
            SessionVerificationMethod::None,
            SessionVerificationMethod::Totp,
            SessionVerificationMethod::EmailOtp,
        ];
        for method in &methods {
            let json = serde_json::to_string(method).unwrap();
            assert!(json.starts_with('"') && json.ends_with('"'));
        }
    }

    #[test]
    fn test_session_empty_strings() {
        let session = Session {
            id: 1, user_id: 1, created: Utc::now(), last_active: Utc::now(),
            ip: String::new(), user_agent: String::new(),
            expired: false, verification_method: SessionVerificationMethod::None,
            verified: false,
        };
        assert!(session.ip.is_empty());
        assert!(session.user_agent.is_empty());
    }
}
