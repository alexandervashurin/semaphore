//! TokenManager - управление API токенами

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::APIToken;
use async_trait::async_trait;
use chrono::Utc;
use sqlx::Row;

#[async_trait]
impl TokenManager for SqlStore {
    async fn get_api_tokens(&self, user_id: i32) -> Result<Vec<APIToken>> {
        let query = "SELECT * FROM api_token WHERE user_id = $1 ORDER BY created DESC";
            let rows = sqlx::query(query).bind(user_id).fetch_all(self.get_postgres_pool()?).await.map_err(Error::Database)?;
            Ok(rows.into_iter().map(|row| APIToken {
                id: row.get("id"),
                user_id: row.get("user_id"),
                name: row.get("name"),
                token: row.get("token"),
                created: row.get("created"),
                expired: row.get("expired"),
            }).collect())
    }

    async fn create_api_token(&self, mut token: APIToken) -> Result<APIToken> {
        let query = "INSERT INTO api_token (user_id, name, token, created, expired) VALUES ($1, $2, $3, $4, $5) RETURNING id";
            let id: i32 = sqlx::query_scalar(query)
                .bind(token.user_id)
                .bind(&token.name)
                .bind(&token.token)
                .bind(token.created)
                .bind(token.expired)
                .fetch_one(self.get_postgres_pool()?).await.map_err(Error::Database)?;
            token.id = id;
            Ok(token)
    }

    async fn get_api_token(&self, token_id: i32) -> Result<APIToken> {
        let query = "SELECT * FROM api_token WHERE id = $1";
            let row = sqlx::query(query).bind(token_id).fetch_one(self.get_postgres_pool()?).await.map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound("Токен не найден".to_string()),
                _ => Error::Database(e),
            })?;
            Ok(APIToken {
                id: row.get("id"),
                user_id: row.get("user_id"),
                name: row.get("name"),
                token: row.get("token"),
                created: row.get("created"),
                expired: row.get("expired"),
            })
    }

    async fn expire_api_token(&self, _user_id: i32, token_id: i32) -> Result<()> {
        let query = "UPDATE api_token SET expired = TRUE WHERE id = $1";
            sqlx::query(query).bind(token_id).execute(self.get_postgres_pool()?).await.map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_api_token(&self, _user_id: i32, token_id: i32) -> Result<()> {
        let query = "DELETE FROM api_token WHERE id = $1";
            sqlx::query(query).bind(token_id).execute(self.get_postgres_pool()?).await.map_err(Error::Database)?;
        Ok(())
    }
}

