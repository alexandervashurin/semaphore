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
                verification_method: row.try_get("verification_method").ok().unwrap_or(SessionVerificationMethod::None),
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
                .fetch_one(self.get_postgres_pool()?).await.map_err(Error::Database)?;
            session.id = id;
            Ok(session)
    }

    async fn expire_session(&self, _user_id: i32, session_id: i32) -> Result<()> {
        let query = "UPDATE session SET expired = TRUE WHERE id = $1";
            sqlx::query(query).bind(session_id).execute(self.get_postgres_pool()?).await.map_err(Error::Database)?;
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
            sqlx::query(query).bind(Utc::now()).bind(session_id).execute(self.get_postgres_pool()?).await.map_err(Error::Database)?;
        Ok(())
    }
}

