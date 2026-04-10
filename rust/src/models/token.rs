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

    #[test]
    fn test_api_token_expired() {
        let token = APIToken {
            id: 1,
            user_id: 1,
            name: "Expired Token".to_string(),
            token: "expired_token".to_string(),
            created: Utc::now() - chrono::Duration::days(30),
            expired: true,
        };
        let json = serde_json::to_string(&token).unwrap();
        assert!(json.contains("\"expired\":true"));
        assert!(json.contains("\"name\":\"Expired Token\""));
    }

    #[test]
    fn test_api_token_deserialization_full() {
        let json = r#"{"id":10,"user_id":50,"name":"Full Token","token":"secret123","created":"2024-06-01T12:00:00Z","expired":false}"#;
        let token: APIToken = serde_json::from_str(json).unwrap();
        assert_eq!(token.id, 10);
        assert_eq!(token.user_id, 50);
        assert_eq!(token.name, "Full Token");
        assert!(!token.expired);
    }

    #[test]
    fn test_api_token_debug() {
        let token = APIToken {
            id: 1, user_id: 1, name: "Debug Token".to_string(),
            token: "debug_secret".to_string(), created: Utc::now(), expired: false,
        };
        let debug_str = format!("{:?}", token);
        assert!(debug_str.contains("APIToken"));
        assert!(debug_str.contains("Debug Token"));
    }

    #[test]
    fn test_api_token_empty_name() {
        let token = APIToken {
            id: 1, user_id: 1, name: String::new(),
            token: "token_val".to_string(), created: Utc::now(), expired: false,
        };
        let json = serde_json::to_string(&token).unwrap();
        assert!(json.contains("\"name\":\"\""));
    }

    #[test]
    fn test_api_token_future_created() {
        let future = Utc::now() + chrono::Duration::days(365);
        let token = APIToken {
            id: 1, user_id: 1, name: "Future Token".to_string(),
            token: "future".to_string(), created: future, expired: false,
        };
        let json = serde_json::to_string(&token).unwrap();
        assert!(json.contains("\"name\":\"Future Token\""));
    }

    #[test]
    fn test_api_token_roundtrip() {
        let original = APIToken {
            id: 42, user_id: 100, name: "Roundtrip".to_string(),
            token: "roundtrip_secret".to_string(), created: Utc::now(), expired: false,
        };
        let json = serde_json::to_string(&original).unwrap();
        let restored: APIToken = serde_json::from_str(&json).unwrap();
        assert_eq!(original.id, restored.id);
        assert_eq!(original.name, restored.name);
        assert_eq!(original.user_id, restored.user_id);
    }

    #[test]
    fn test_api_token_max_ids() {
        let token = APIToken {
            id: i32::MAX, user_id: i32::MAX, name: "Max IDs".to_string(),
            token: "max_token".to_string(), created: Utc::now(), expired: false,
        };
        let json = serde_json::to_string(&token).unwrap();
        let restored: APIToken = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, i32::MAX);
        assert_eq!(restored.user_id, i32::MAX);
    }

    #[test]
    fn test_api_token_special_chars_name() {
        let token = APIToken {
            id: 1, user_id: 1, name: "Token & <special> \"quotes\"".to_string(),
            token: "special_token".to_string(), created: Utc::now(), expired: false,
        };
        let json = serde_json::to_string(&token).unwrap();
        let restored: APIToken = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, "Token & <special> \"quotes\"");
    }
}
