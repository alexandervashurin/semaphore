//! Модель API-токена

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// API-токен для аутентификации
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct APIToken {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub token: String,
    pub created: DateTime<Utc>,
    pub expired: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_api_token_deserialization() {
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
            name: "Clone Test".to_string(),
            token: "token_value".to_string(),
            created: Utc::now(),
            expired: false,
        };
        let cloned = token.clone();
        assert_eq!(cloned.name, token.name);
        assert_eq!(cloned.user_id, token.user_id);
    }
}
