//! PostgreSQL Project CRUD operations

use crate::error::{Error, Result};
use crate::models::*;
use sqlx::{Pool, Postgres};

/// Получает все проекты PostgreSQL
pub async fn get_projects(pool: &Pool<Postgres>, user_id: Option<i32>) -> Result<Vec<Project>> {
    let query = "SELECT * FROM project ORDER BY name";

    let projects = sqlx::query_as::<_, Project>(query)
        .fetch_all(pool)
        .await
        .map_err(Error::Database)?;

    Ok(projects)
}

/// Получает проект по ID PostgreSQL
pub async fn get_project(pool: &Pool<Postgres>, project_id: i32) -> Result<Project> {
    let query = "SELECT * FROM project WHERE id = $1";

    let project = sqlx::query_as::<_, Project>(query)
        .bind(project_id)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound("Project not found".to_string()),
            _ => Error::Database(e),
        })?;

    Ok(project)
}

/// Создаёт проект PostgreSQL
pub async fn create_project(pool: &Pool<Postgres>, mut project: Project) -> Result<Project> {
    let query = "INSERT INTO project (name, created, alert, max_parallel_tasks, type, default_secret_storage_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id";

    let id: i32 = sqlx::query_scalar(query)
        .bind(&project.name)
        .bind(project.created)
        .bind(project.alert)
        .bind(project.max_parallel_tasks)
        .bind(&project.r#type)
        .bind(project.default_secret_storage_id)
        .fetch_one(pool)
        .await
        .map_err(Error::Database)?;

    project.id = id;
    Ok(project)
}

/// Обновляет проект PostgreSQL
pub async fn update_project(pool: &Pool<Postgres>, project: Project) -> Result<()> {
    let query = "UPDATE project SET name = $1, alert = $2, max_parallel_tasks = $3, type = $4, default_secret_storage_id = $5 WHERE id = $6";

    sqlx::query(query)
        .bind(&project.name)
        .bind(project.alert)
        .bind(project.max_parallel_tasks)
        .bind(&project.r#type)
        .bind(project.default_secret_storage_id)
        .bind(project.id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}

/// Удаляет проект PostgreSQL
pub async fn delete_project(pool: &Pool<Postgres>, project_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM project WHERE id = $1")
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}
