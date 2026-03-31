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
