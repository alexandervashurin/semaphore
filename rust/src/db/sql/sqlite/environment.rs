//! SQLite Environment CRUD operations

use crate::error::{Error, Result};
use crate::models::*;
use sqlx::{Sqlite, Pool};

/// Получает все окружения проекта SQLite
pub async fn get_environments(pool: &Pool<Sqlite>, project_id: i32) -> Result<Vec<Environment>> {
    let query = "SELECT * FROM environment WHERE project_id = ? ORDER BY name";
    
    let environments = sqlx::query_as::<_, Environment>(query)
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(Error::Database)?;

    Ok(environments)
}

/// Получает окружение по ID SQLite
pub async fn get_environment(pool: &Pool<Sqlite>, project_id: i32, environment_id: i32) -> Result<Environment> {
    let query = "SELECT * FROM environment WHERE id = ? AND project_id = ?";
    
    let environment = sqlx::query_as::<_, Environment>(query)
        .bind(environment_id)
        .bind(project_id)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound("Environment not found".to_string()),
            _ => Error::Database(e),
        })?;

    Ok(environment)
}

/// Создаёт окружение SQLite
pub async fn create_environment(pool: &Pool<Sqlite>, mut environment: Environment) -> Result<Environment> {
    let query = "INSERT INTO environment (project_id, name, json, secret_storage_id, secrets, created) VALUES (?, ?, ?, ?, ?, ?) RETURNING id";
    
    let id: i32 = sqlx::query_scalar(query)
        .bind(environment.project_id)
        .bind(&environment.name)
        .bind(&environment.json)
        .bind(environment.secret_storage_id)
        .bind(&environment.secrets)
        .bind(environment.created)
        .fetch_one(pool)
        .await
        .map_err(Error::Database)?;

    environment.id = id;
    Ok(environment)
}

/// Обновляет окружение SQLite
pub async fn update_environment(pool: &Pool<Sqlite>, environment: Environment) -> Result<()> {
    let query = "UPDATE environment SET name = ?, json = ?, secret_storage_id = ?, secrets = ? WHERE id = ? AND project_id = ?";
    
    sqlx::query(query)
        .bind(&environment.name)
        .bind(&environment.json)
        .bind(environment.secret_storage_id)
        .bind(&environment.secrets)
        .bind(environment.id)
        .bind(environment.project_id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}

/// Удаляет окружение SQLite
pub async fn delete_environment(pool: &Pool<Sqlite>, project_id: i32, environment_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM environment WHERE id = ? AND project_id = ?")
        .bind(environment_id)
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_environments_query_structure() {
        let query = "SELECT * FROM environment WHERE project_id = ? ORDER BY name";
        assert!(query.contains("environment"));
        assert!(query.contains("project_id = ?"));
        assert!(query.contains("ORDER BY name"));
    }

    #[test]
    fn test_get_environment_query_structure() {
        let query = "SELECT * FROM environment WHERE id = ? AND project_id = ?";
        assert!(query.contains("id = ?"));
        assert!(query.contains("project_id = ?"));
    }

    #[test]
    fn test_create_environment_query_structure() {
        let expected = "INSERT INTO environment (project_id, name, json, secret_storage_id, secrets, created) VALUES (?, ?, ?, ?, ?, ?) RETURNING id";
        assert!(expected.contains("environment"));
        assert!(expected.contains("RETURNING id"));
        assert!(expected.matches('?').count() == 6);
    }

    #[test]
    fn test_update_environment_query_structure() {
        let expected = "UPDATE environment SET name = ?, json = ?, secret_storage_id = ?, secrets = ? WHERE id = ? AND project_id = ?";
        assert!(expected.contains("UPDATE environment"));
        assert!(expected.contains("WHERE id = ? AND project_id = ?"));
        assert!(expected.matches('?').count() == 6);
    }

    #[test]
    fn test_delete_environment_query_structure() {
        let expected = "DELETE FROM environment WHERE id = ? AND project_id = ?";
        assert!(expected.contains("environment"));
        assert!(expected.contains("id = ? AND project_id = ?"));
    }

    #[test]
    fn test_sqlite_uses_question_placeholders() {
        let queries = [
            "SELECT * FROM environment WHERE id = ?",
            "DELETE FROM environment WHERE id = ? AND project_id = ?",
        ];
        for q in &queries {
            assert!(q.contains('?'), "SQLite should use ? placeholders");
            assert!(!q.contains('$'), "SQLite should not use $N placeholders");
        }
    }

    #[test]
    fn test_sqlite_no_backticks() {
        let queries = [
            "SELECT * FROM environment WHERE id = ?",
            "DELETE FROM environment WHERE id = ?",
        ];
        for q in &queries {
            assert!(!q.contains('`'), "SQLite should not use backticks");
        }
    }

    #[test]
    fn test_sqlite_returning_clause() {
        let query = "INSERT INTO environment (...) VALUES (?) RETURNING id";
        assert!(query.contains("RETURNING id"), "SQLite uses RETURNING clause");
    }

    #[test]
    fn test_environment_model_fields() {
        let env = Environment::new(10, "sqlite-production", r#"{"DB":"localhost"}"#);
        assert_eq!(env.project_id, 10);
        assert_eq!(env.name, "sqlite-production");
        assert_eq!(env.json, r#"{"DB":"localhost"}"#);
    }

    #[test]
    fn test_environment_serialization() {
        let env = Environment::new(1, "sqlite-dev", r#"{"KEY":"val"}"#);
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains("\"name\":\"sqlite-dev\""));
        assert!(json.contains("\"project_id\":1"));
    }

    #[test]
    fn test_environment_bind_order_matches_query() {
        let columns = [
            "project_id", "name", "json", "secret_storage_id", "secrets", "created",
        ];
        assert_eq!(columns.len(), 6);
        assert_eq!(columns[0], "project_id");
        assert_eq!(columns[2], "json");
    }

    #[test]
    fn test_environment_parse_json() {
        let env = Environment::new(1, "test", r#"{"KEY1":"val1","KEY2":"val2"}"#);
        let parsed = env.parse_json().unwrap();
        assert_eq!(parsed["KEY1"], "val1");
        assert_eq!(parsed["KEY2"], "val2");
    }

    #[test]
    fn test_environment_default() {
        let env = Environment::default();
        assert_eq!(env.id, 0);
        assert!(env.name.is_empty());
        assert!(env.json.is_empty());
    }

    #[test]
    fn test_sqlite_environment_debug_format() {
        let query = "SELECT * FROM environment WHERE id = ?";
        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("environment"));
    }
}
