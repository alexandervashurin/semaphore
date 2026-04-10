//! OptionsManager - управление опциями

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use async_trait::async_trait;
use sqlx::Row;
use std::collections::HashMap;

#[async_trait]
impl OptionsManager for SqlStore {
    async fn get_options(&self) -> Result<HashMap<String, String>> {
        let query = "SELECT key, value FROM option";
        let rows = sqlx::query(query)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let key: String = row.get("key");
                let value: String = row.get("value");
                (key, value)
            })
            .collect())
    }

    async fn get_option(&self, key: &str) -> Result<Option<String>> {
        let query = "SELECT value FROM option WHERE key = $1";
        let result = sqlx::query_scalar::<_, String>(query)
            .bind(key)
            .fetch_optional(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(result)
    }

    async fn set_option(&self, key: &str, value: &str) -> Result<()> {
        let query = "INSERT INTO option (key, value) VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value";
        sqlx::query(query)
            .bind(key)
            .bind(value)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_option(&self, key: &str) -> Result<()> {
        let query = "DELETE FROM option WHERE key = $1";
        sqlx::query(query)
            .bind(key)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn test_options_hashmap_serialization() {
        let mut options = HashMap::new();
        options.insert("app_name".to_string(), "Semaphore".to_string());
        options.insert("version".to_string(), "1.0.0".to_string());
        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("\"app_name\""));
        assert!(json.contains("\"Semaphore\""));
        assert!(json.contains("\"version\""));
    }

    #[test]
    fn test_options_hashmap_deserialization() {
        let json = r#"{"key1":"val1","key2":"val2"}"#;
        let options: HashMap<String, String> = serde_json::from_str(json).unwrap();
        assert_eq!(options.get("key1"), Some(&"val1".to_string()));
        assert_eq!(options.get("key2"), Some(&"val2".to_string()));
    }

    #[test]
    fn test_options_empty_hashmap() {
        let options: HashMap<String, String> = HashMap::new();
        let json = serde_json::to_string(&options).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_options_clone() {
        let mut options = HashMap::new();
        options.insert("max_tasks".to_string(), "100".to_string());
        let cloned = options.clone();
        assert_eq!(cloned.get("max_tasks"), options.get("max_tasks"));
    }

    #[test]
    fn test_option_key_value_pair() {
        let key = "feature_flag".to_string();
        let value = "enabled".to_string();
        assert_eq!(key, "feature_flag");
        assert_eq!(value, "enabled");
    }

    #[test]
    fn test_option_empty_value() {
        let mut options = HashMap::new();
        options.insert("empty_opt".to_string(), String::new());
        assert_eq!(options.get("empty_opt"), Some(&String::new()));
    }

    #[test]
    fn test_option_special_characters() {
        let mut options = HashMap::new();
        options.insert("url".to_string(), "https://example.com/path?q=test".to_string());
        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("https://example.com/path?q=test"));
    }

    #[test]
    fn test_option_multiple_entries() {
        let mut options = HashMap::new();
        for i in 0..10 {
            options.insert(format!("opt_{}", i), format!("value_{}", i));
        }
        assert_eq!(options.len(), 10);
        assert_eq!(options.get("opt_5"), Some(&"value_5".to_string()));
    }
}
