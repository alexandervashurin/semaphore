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
