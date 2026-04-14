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
        let rows = sqlx::query(query)
            .bind(user_id)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(rows
            .into_iter()
            .map(|row| APIToken {
                id: row.get("id"),
                user_id: row.get("user_id"),
                name: row.get("name"),
                token: row.get("token"),
                created: row.get("created"),
                expired: row.get("expired"),
            })
            .collect())
    }

    async fn create_api_token(&self, mut token: APIToken) -> Result<APIToken> {
        let query = "INSERT INTO api_token (user_id, name, token, created, expired) VALUES ($1, $2, $3, $4, $5) RETURNING id";
        let id: i32 = sqlx::query_scalar(query)
            .bind(token.user_id)
            .bind(&token.name)
            .bind(&token.token)
            .bind(token.created)
            .bind(token.expired)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        token.id = id;
        Ok(token)
    }

    async fn get_api_token(&self, token_id: i32) -> Result<APIToken> {
        let query = "SELECT * FROM api_token WHERE id = $1";
        let row = sqlx::query(query)
            .bind(token_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(|e| match e {
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
        sqlx::query(query)
            .bind(token_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_api_token(&self, _user_id: i32, token_id: i32) -> Result<()> {
        let query = "DELETE FROM api_token WHERE id = $1";
        sqlx::query(query)
            .bind(token_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::models::APIToken;
    use chrono::Utc;

    #[test]
    fn test_api_token_serialization() {
        let token = APIToken {
            id: 1,
            user_id: 10,
            name: "My Token".to_string(),
            token: "secret_token_value".to_string(),
            created: Utc::now(),
            expired: false,
        };
        let json = serde_json::to_string(&token).unwrap();
        assert!(json.contains("\"name\":\"My Token\""));
        assert!(json.contains("\"user_id\":10"));
        assert!(json.contains("\"expired\":false"));
    }

    #[test]
    fn test_api_token_deserialize() {
        let json = r#"{"id":5,"user_id":20,"name":"Test Token","token":"abc123","created":"2024-01-01T00:00:00Z","expired":true}"#;
        let token: APIToken = serde_json::from_str(json).unwrap();
        assert_eq!(token.id, 5);
        assert_eq!(token.name, "Test Token");
        assert!(token.expired);
    }

    #[test]
    fn test_api_token_clone() {
        let token = APIToken {
            id: 1,
            user_id: 1,
            name: "Clone".to_string(),
            token: "val".to_string(),
            created: Utc::now(),
            expired: false,
        };
        let cloned = token.clone();
        assert_eq!(cloned.name, token.name);
        assert_eq!(cloned.user_id, token.user_id);
    }

    #[test]
    fn test_api_token_expired() {
        let token = APIToken {
            id: 1,
            user_id: 1,
            name: "Expired".to_string(),
            token: "expired".to_string(),
            created: Utc::now() - chrono::Duration::days(30),
            expired: true,
        };
        let json = serde_json::to_string(&token).unwrap();
        assert!(json.contains("\"expired\":true"));
    }

    #[test]
    fn test_api_token_not_expired() {
        let token = APIToken {
            id: 2,
            user_id: 1,
            name: "Active".to_string(),
            token: "active_token".to_string(),
            created: Utc::now(),
            expired: false,
        };
        assert!(!token.expired);
    }

    #[test]
    fn test_api_token_deserialize_full() {
        let json = r#"{"id":10,"user_id":50,"name":"Full Token","token":"secret123","created":"2024-06-01T12:00:00Z","expired":false}"#;
        let token: APIToken = serde_json::from_str(json).unwrap();
        assert_eq!(token.id, 10);
        assert_eq!(token.user_id, 50);
        assert!(!token.expired);
    }

    #[test]
    fn test_api_token_empty_name() {
        let token = APIToken {
            id: 1,
            user_id: 1,
            name: String::new(),
            token: "tok".to_string(),
            created: Utc::now(),
            expired: false,
        };
        assert!(token.name.is_empty());
    }

    #[test]
    fn test_api_token_different_users() {
        let token1 = APIToken {
            id: 1,
            user_id: 1,
            name: "User1".to_string(),
            token: "t1".to_string(),
            created: Utc::now(),
            expired: false,
        };
        let token2 = APIToken {
            id: 2,
            user_id: 2,
            name: "User2".to_string(),
            token: "t2".to_string(),
            created: Utc::now(),
            expired: false,
        };
        assert_ne!(token1.user_id, token2.user_id);
        assert_ne!(token1.name, token2.name);
    }

    #[test]
    fn test_api_token_serialization_with_long_name() {
        let token = APIToken {
            id: 1,
            user_id: 1,
            name: "A very long token name that describes the purpose of this token for CI/CD pipeline deployment".to_string(),
            token: "token_value".to_string(),
            created: Utc::now(),
            expired: false,
        };
        let json = serde_json::to_string(&token).unwrap();
        let deserialized: APIToken = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name.len(), 93);
    }

    #[test]
    fn test_api_token_serialization_unicode_name() {
        let token = APIToken {
            id: 1,
            user_id: 1,
            name: "Токен для деплоя".to_string(),
            token: "значение".to_string(),
            created: Utc::now(),
            expired: false,
        };
        let json = serde_json::to_string(&token).unwrap();
        assert!(json.contains("Токен"));

        let deserialized: APIToken = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Токен для деплоя");
    }

    #[test]
    fn test_api_token_minimal_values() {
        let token = APIToken {
            id: 0,
            user_id: 0,
            name: String::new(),
            token: String::new(),
            created: Utc::now(),
            expired: false,
        };
        assert_eq!(token.id, 0);
        assert_eq!(token.user_id, 0);
        assert!(token.name.is_empty());
        assert!(token.token.is_empty());
    }

    #[test]
    fn test_api_token_max_values() {
        let token = APIToken {
            id: i32::MAX,
            user_id: i32::MAX,
            name: "max".to_string(),
            token: "max_token".to_string(),
            created: Utc::now(),
            expired: true,
        };
        assert_eq!(token.id, i32::MAX);
        assert_eq!(token.user_id, i32::MAX);
    }

    #[test]
    fn test_api_token_debug_format() {
        let token = APIToken {
            id: 1,
            user_id: 42,
            name: "Debug Test".to_string(),
            token: "debug_val".to_string(),
            created: Utc::now(),
            expired: false,
        };
        let debug_str = format!("{:?}", token);
        assert!(debug_str.contains("Debug Test"));
        assert!(debug_str.contains("APIToken"));
    }

    #[test]
    fn test_api_token_partial_eq() {
        let token1 = APIToken {
            id: 1,
            user_id: 1,
            name: "Same".to_string(),
            token: "same".to_string(),
            created: Utc::now(),
            expired: false,
        };
        let token2 = token1.clone();
        // APIToken may not implement PartialEq, so test field-by-field
        assert_eq!(token1.id, token2.id);
        assert_eq!(token1.user_id, token2.user_id);
        assert_eq!(token1.name, token2.name);
    }

    #[test]
    fn test_api_token_deserialize_invalid_json() {
        let result = serde_json::from_str::<APIToken>("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_api_token_deserialize_missing_fields() {
        let json = r#"{"id":1}"#;
        let result = serde_json::from_str::<APIToken>(json);
        // Should fail due to missing required fields
        assert!(result.is_err());
    }

    #[test]
    fn test_api_token_created_at_past() {
        let past_time = Utc::now() - chrono::Duration::hours(24);
        let token = APIToken {
            id: 1,
            user_id: 1,
            name: "Past Token".to_string(),
            token: "past".to_string(),
            created: past_time,
            expired: false,
        };
        let json = serde_json::to_string(&token).unwrap();
        assert!(json.contains("created"));
    }

    #[test]
    fn test_api_token_multiple_tokens_same_user() {
        let now = Utc::now();
        let token1 = APIToken {
            id: 1,
            user_id: 10,
            name: "First".to_string(),
            token: "t1".to_string(),
            created: now,
            expired: false,
        };
        let token2 = APIToken {
            id: 2,
            user_id: 10,
            name: "Second".to_string(),
            token: "t2".to_string(),
            created: now,
            expired: false,
        };

        assert_eq!(token1.user_id, token2.user_id);
        assert_ne!(token1.id, token2.id);
        assert_ne!(token1.name, token2.name);
    }

    #[test]
    fn test_api_token_json_roundtrip() {
        let original = APIToken {
            id: 99,
            user_id: 999,
            name: "Roundtrip".to_string(),
            token: "roundtrip_value".to_string(),
            created: Utc::now(),
            expired: false,
        };

        let json = serde_json::to_string(&original).unwrap();
        let restored: APIToken = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.id, original.id);
        assert_eq!(restored.user_id, original.user_id);
        assert_eq!(restored.name, original.name);
        assert_eq!(restored.token, original.token);
        assert_eq!(restored.expired, original.expired);
    }
}
