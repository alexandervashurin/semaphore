//! MySQL Project CRUD operations

use crate::error::{Error, Result};
use crate::models::*;
use sqlx::{Pool, MySql};

/// Получает все проекты MySQL
pub async fn get_projects(pool: &Pool<MySql>, user_id: Option<i32>) -> Result<Vec<Project>> {
    let query = "SELECT * FROM `project` ORDER BY name";
    
    let projects = sqlx::query_as::<_, Project>(query)
        .fetch_all(pool)
        .await
        .map_err(|e| Error::Database(e))?;

    Ok(projects)
}

/// Получает проект по ID MySQL
pub async fn get_project(pool: &Pool<MySql>, project_id: i32) -> Result<Project> {
    let query = "SELECT * FROM `project` WHERE id = ?";
    
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

/// Создаёт проект MySQL
pub async fn create_project(pool: &Pool<MySql>, mut project: Project) -> Result<Project> {
    let query = "INSERT INTO `project` (name, created, alert, max_parallel_tasks, type, default_secret_storage_id) VALUES (?, ?, ?, ?, ?, ?)";
    
    let result = sqlx::query(query)
        .bind(&project.name)
        .bind(project.created)
        .bind(project.alert)
        .bind(project.max_parallel_tasks)
        .bind(&project.r#type)
        .bind(project.default_secret_storage_id)
        .execute(pool)
        .await
        .map_err(|e| Error::Database(e))?;

    project.id = result.last_insert_id() as i32;
    Ok(project)
}

/// Обновляет проект MySQL
pub async fn update_project(pool: &Pool<MySql>, project: Project) -> Result<()> {
    let query = "UPDATE `project` SET name = ?, alert = ?, max_parallel_tasks = ?, type = ?, default_secret_storage_id = ? WHERE id = ?";
    
    sqlx::query(query)
        .bind(&project.name)
        .bind(project.alert)
        .bind(project.max_parallel_tasks)
        .bind(&project.r#type)
        .bind(project.default_secret_storage_id)
        .bind(project.id)
        .execute(pool)
        .await
        .map_err(|e| Error::Database(e))?;

    Ok(())
}

/// Удаляет проект MySQL
pub async fn delete_project(pool: &Pool<MySql>, project_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM `project` WHERE id = ?")
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(|e| Error::Database(e))?;

    Ok(())
}
