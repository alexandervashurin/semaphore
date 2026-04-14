//! Environment CRUD Operations
//!
//! Адаптер для декомпозированных модулей
//!
//! Новые модули: sqlite::environment, postgres::environment, mysql::environment

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::{Environment, EnvironmentSecret, EnvironmentSecretType};

impl SqlDb {
    /// Получает окружения проекта
    pub async fn get_environments(&self, project_id: i32) -> Result<Vec<Environment>> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::environment::get_environments(pool, project_id).await
    }

    /// Получает окружение по ID
    pub async fn get_environment(
        &self,
        project_id: i32,
        environment_id: i32,
    ) -> Result<Environment> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::environment::get_environment(pool, project_id, environment_id)
            .await
    }

    /// Создаёт окружение
    pub async fn create_environment(&self, environment: Environment) -> Result<Environment> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::environment::create_environment(pool, environment).await
    }

    /// Обновляет окружение
    pub async fn update_environment(&self, environment: Environment) -> Result<()> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::environment::update_environment(pool, environment).await
    }

    /// Удаляет окружение
    pub async fn delete_environment(&self, project_id: i32, environment_id: i32) -> Result<()> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::environment::delete_environment(pool, project_id, environment_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_new() {
        let env = Environment::new(
            10,
            "production".to_string(),
            r#"{"KEY":"value"}"#.to_string(),
        );
        assert_eq!(env.id, 0);
        assert_eq!(env.project_id, 10);
        assert_eq!(env.name, "production");
        assert!(env.secret_storage_id.is_none());
    }

    #[test]
    fn test_environment_default() {
        let env = Environment::default();
        assert_eq!(env.id, 0);
        assert!(env.name.is_empty());
        assert!(env.json.is_empty());
    }

    #[test]
    fn test_environment_serialization() {
        let env = Environment::new(1, "test".to_string(), "{}".to_string());
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"project_id\":1"));
    }

    #[test]
    fn test_environment_parse_json() {
        let env = Environment {
            id: 1,
            project_id: 1,
            name: "test".to_string(),
            json: r#"{"KEY1":"val1"}"#.to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: None,
            created: None,
        };
        let parsed = env.parse_json().unwrap();
        assert_eq!(parsed["KEY1"], "val1");
    }

    #[test]
    fn test_environment_parse_invalid_json() {
        let env = Environment {
            id: 1,
            project_id: 1,
            name: "test".to_string(),
            json: "not valid".to_string(),
            secret_storage_id: None,
            secret_storage_key_prefix: None,
            secrets: None,
            created: None,
        };
        assert!(env.parse_json().is_err());
    }

    #[test]
    fn test_environment_clone() {
        let env = Environment::new(5, "clone".to_string(), "{}".to_string());
        let cloned = env.clone();
        assert_eq!(cloned.name, env.name);
        assert_eq!(cloned.json, env.json);
    }

    #[test]
    fn test_environment_serialization_skip_nulls() {
        let env = Environment::default();
        let json = serde_json::to_string(&env).unwrap();
        assert!(!json.contains("secret_storage_id"));
        assert!(!json.contains("secret_storage_key_prefix"));
    }

    #[test]
    fn test_environment_with_secret_storage() {
        let env = Environment {
            id: 1,
            project_id: 1,
            name: "vault".to_string(),
            json: "{}".to_string(),
            secret_storage_id: Some(42),
            secret_storage_key_prefix: Some("app/prod".to_string()),
            secrets: None,
            created: None,
        };
        assert_eq!(env.secret_storage_id, Some(42));
        assert_eq!(env.secret_storage_key_prefix, Some("app/prod".to_string()));
    }

    #[test]
    fn test_environment_deserialization() {
        let json = r#"{"id":10,"project_id":5,"name":"dev","json":"{}"}"#;
        let env: Environment = serde_json::from_str(json).unwrap();
        assert_eq!(env.id, 10);
        assert_eq!(env.name, "dev");
        assert_eq!(env.project_id, 5);
    }

    #[test]
    fn test_environment_debug_format() {
        let env = Environment::new(1, "debug".to_string(), "{}".to_string());
        let debug_str = format!("{:?}", env);
        assert!(debug_str.contains("Environment"));
        assert!(debug_str.contains("debug"));
    }

    #[test]
    fn test_environment_secret_type_display() {
        let env_type = EnvironmentSecretType::Env;
        let json = serde_json::to_string(&env_type).unwrap();
        assert_eq!(json, "\"env\"");
    }

    #[test]
    fn test_environment_secret_serialization() {
        let secret = EnvironmentSecret {
            id: 1,
            environment_id: 10,
            secret_id: 5,
            secret_type: EnvironmentSecretType::Var,
        };
        let json = serde_json::to_string(&secret).unwrap();
        assert!(json.contains("\"environment_id\":10"));
        assert!(json.contains("\"secret_type\":\"var\""));
    }
}
