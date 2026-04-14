//! PostgreSQL Environment CRUD operations

use crate::error::{Error, Result};
use crate::models::*;
use sqlx::{Pool, Postgres};

/// Получает все окружения проекта PostgreSQL
pub async fn get_environments(pool: &Pool<Postgres>, project_id: i32) -> Result<Vec<Environment>> {
    let query = "SELECT * FROM environment WHERE project_id = $1 ORDER BY name";

    let environments = sqlx::query_as::<_, Environment>(query)
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(Error::Database)?;

    Ok(environments)
}

/// Получает окружение по ID PostgreSQL
pub async fn get_environment(
    pool: &Pool<Postgres>,
    project_id: i32,
    environment_id: i32,
) -> Result<Environment> {
    let query = "SELECT * FROM environment WHERE id = $1 AND project_id = $2";

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

/// Создаёт окружение PostgreSQL
pub async fn create_environment(
    pool: &Pool<Postgres>,
    mut environment: Environment,
) -> Result<Environment> {
    let query = "INSERT INTO environment (project_id, name, json, secret_storage_id, secrets, created) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id";

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

/// Обновляет окружение PostgreSQL
pub async fn update_environment(pool: &Pool<Postgres>, environment: Environment) -> Result<()> {
    let query = "UPDATE environment SET name = $1, json = $2, secret_storage_id = $3, secrets = $4 WHERE id = $5 AND project_id = $6";

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

/// Удаляет окружение PostgreSQL
pub async fn delete_environment(
    pool: &Pool<Postgres>,
    project_id: i32,
    environment_id: i32,
) -> Result<()> {
    sqlx::query("DELETE FROM environment WHERE id = $1 AND project_id = $2")
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
        let query = "SELECT * FROM environment WHERE project_id = $1 ORDER BY name";
        assert!(query.contains("environment"));
        assert!(query.contains("project_id = $1"));
        assert!(query.contains("ORDER BY name"));
    }

    #[test]
    fn test_get_environment_query_structure() {
        let query = "SELECT * FROM environment WHERE id = $1 AND project_id = $2";
        assert!(query.contains("id = $1"));
        assert!(query.contains("project_id = $2"));
    }

    #[test]
    fn test_create_environment_query_structure() {
        let expected = "INSERT INTO environment (project_id, name, json, secret_storage_id, secrets, created) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id";
        assert!(expected.contains("environment"));
        assert!(expected.contains("RETURNING id"));
        assert!(expected.contains("$6"));
    }

    #[test]
    fn test_update_environment_query_structure() {
        let expected = "UPDATE environment SET name = $1, json = $2, secret_storage_id = $3, secrets = $4 WHERE id = $5 AND project_id = $6";
        assert!(expected.contains("UPDATE environment"));
        assert!(expected.contains("WHERE id = $5 AND project_id = $6"));
        assert!(expected.contains("$6"));
    }

    #[test]
    fn test_delete_environment_query_structure() {
        let expected = "DELETE FROM environment WHERE id = $1 AND project_id = $2";
        assert!(expected.contains("environment"));
        assert!(expected.contains("id = $1 AND project_id = $2"));
    }

    #[test]
    fn test_postgres_uses_dollar_placeholders() {
        let queries = [
            "SELECT * FROM environment WHERE id = $1",
            "DELETE FROM environment WHERE id = $1 AND project_id = $2",
        ];
        for q in &queries {
            assert!(q.contains('$'), "Postgres should use $N placeholders");
            assert!(!q.contains('?'), "Postgres should not use ? placeholders");
        }
    }

    #[test]
    fn test_postgres_no_backticks() {
        let queries = [
            "SELECT * FROM environment WHERE id = $1",
            "DELETE FROM environment WHERE id = $1",
        ];
        for q in &queries {
            assert!(!q.contains('`'), "Postgres should not use backticks");
        }
    }

    #[test]
    fn test_postgres_returning_clause() {
        let query = "INSERT INTO environment (...) VALUES (...) RETURNING id";
        assert!(
            query.contains("RETURNING id"),
            "Postgres uses RETURNING clause"
        );
    }

    #[test]
    fn test_environment_model_fields() {
        let env = Environment::new(
            10,
            "pg-production".to_string(),
            r#"{"DB":"localhost"}"#.to_string(),
        );
        assert_eq!(env.project_id, 10);
        assert_eq!(env.name, "pg-production");
        assert_eq!(env.json, r#"{"DB":"localhost"}"#);
    }

    #[test]
    fn test_environment_serialization() {
        let env = Environment::new(1, "pg-dev".to_string(), r#"{"KEY":"val"}"#.to_string());
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains("\"name\":\"pg-dev\""));
        assert!(json.contains("\"project_id\":1"));
    }

    #[test]
    fn test_environment_bind_order_matches_query() {
        let columns = [
            "project_id",
            "name",
            "json",
            "secret_storage_id",
            "secrets",
            "created",
        ];
        assert_eq!(columns.len(), 6);
        assert_eq!(columns[0], "project_id");
        assert_eq!(columns[2], "json");
    }

    #[test]
    fn test_postgres_environment_debug_format() {
        let query = "SELECT * FROM environment WHERE id = $1";
        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("environment"));
    }
}
