//! PostgreSQL Repository CRUD operations

use crate::error::{Error, Result};
use crate::models::*;
use sqlx::{Pool, Postgres};

/// Получает все репозитории проекта PostgreSQL
pub async fn get_repositories(pool: &Pool<Postgres>, project_id: i32) -> Result<Vec<Repository>> {
    let query = "SELECT * FROM repository WHERE project_id = $1 ORDER BY name";

    let repositories = sqlx::query_as::<_, Repository>(query)
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(Error::Database)?;

    Ok(repositories)
}

/// Получает репозиторий по ID PostgreSQL
pub async fn get_repository(
    pool: &Pool<Postgres>,
    project_id: i32,
    repository_id: i32,
) -> Result<Repository> {
    let query = "SELECT * FROM repository WHERE id = $1 AND project_id = $2";

    let repository = sqlx::query_as::<_, Repository>(query)
        .bind(repository_id)
        .bind(project_id)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound("Repository not found".to_string()),
            _ => Error::Database(e),
        })?;

    Ok(repository)
}

/// Создаёт репозиторий PostgreSQL
pub async fn create_repository(
    pool: &Pool<Postgres>,
    mut repository: Repository,
) -> Result<Repository> {
    let query = "INSERT INTO repository (project_id, name, git_url, git_type, git_branch, key_id, created) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id";

    let id: i32 = sqlx::query_scalar(query)
        .bind(repository.project_id)
        .bind(&repository.name)
        .bind(&repository.git_url)
        .bind(&repository.git_type)
        .bind(&repository.git_branch)
        .bind(repository.key_id)
        .bind(repository.created)
        .fetch_one(pool)
        .await
        .map_err(Error::Database)?;

    repository.id = id;
    Ok(repository)
}

/// Обновляет репозиторий PostgreSQL
pub async fn update_repository(pool: &Pool<Postgres>, repository: Repository) -> Result<()> {
    let query = "UPDATE repository SET name = $1, git_url = $2, git_type = $3, git_branch = $4, key_id = $5 WHERE id = $6 AND project_id = $7";

    sqlx::query(query)
        .bind(&repository.name)
        .bind(&repository.git_url)
        .bind(&repository.git_type)
        .bind(&repository.git_branch)
        .bind(repository.key_id)
        .bind(repository.id)
        .bind(repository.project_id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}

/// Удаляет репозиторий PostgreSQL
pub async fn delete_repository(
    pool: &Pool<Postgres>,
    project_id: i32,
    repository_id: i32,
) -> Result<()> {
    sqlx::query("DELETE FROM repository WHERE id = $1 AND project_id = $2")
        .bind(repository_id)
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}
