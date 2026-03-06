//! MySQL Repository CRUD operations

use crate::error::{Error, Result};
use crate::models::*;
use sqlx::{Pool, MySql};

/// Получает все репозитории проекта MySQL
pub async fn get_repositories(pool: &Pool<MySql>, project_id: i32) -> Result<Vec<Repository>> {
    let query = "SELECT * FROM `repository` WHERE project_id = ? ORDER BY name";
    
    let repositories = sqlx::query_as::<_, Repository>(query)
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(|e| Error::Database(e))?;

    Ok(repositories)
}

/// Получает репозиторий по ID MySQL
pub async fn get_repository(pool: &Pool<MySql>, project_id: i32, repository_id: i32) -> Result<Repository> {
    let query = "SELECT * FROM `repository` WHERE id = ? AND project_id = ?";
    
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

/// Создаёт репозиторий MySQL
pub async fn create_repository(pool: &Pool<MySql>, mut repository: Repository) -> Result<Repository> {
    let query = "INSERT INTO `repository` (project_id, name, git_url, git_type, git_branch, key_id, created) VALUES (?, ?, ?, ?, ?, ?, ?)";
    
    let result = sqlx::query(query)
        .bind(repository.project_id)
        .bind(&repository.name)
        .bind(&repository.git_url)
        .bind(&repository.git_type)
        .bind(&repository.git_branch)
        .bind(repository.key_id)
        .bind(repository.created)
        .execute(pool)
        .await
        .map_err(|e| Error::Database(e))?;

    repository.id = result.last_insert_id() as i32;
    Ok(repository)
}

/// Обновляет репозиторий MySQL
pub async fn update_repository(pool: &Pool<MySql>, repository: Repository) -> Result<()> {
    let query = "UPDATE `repository` SET name = ?, git_url = ?, git_type = ?, git_branch = ?, key_id = ? WHERE id = ? AND project_id = ?";
    
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
        .map_err(|e| Error::Database(e))?;

    Ok(())
}

/// Удаляет репозиторий MySQL
pub async fn delete_repository(pool: &Pool<MySql>, project_id: i32, repository_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM `repository` WHERE id = ? AND project_id = ?")
        .bind(repository_id)
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(|e| Error::Database(e))?;

    Ok(())
}
