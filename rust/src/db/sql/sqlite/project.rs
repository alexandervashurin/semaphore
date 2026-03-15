//! SQLite Project CRUD operations

use crate::error::{Error, Result};
use crate::models::*;
use chrono::{DateTime, Utc};
use sqlx::{Row, Sqlite, Pool};

/// Получает все проекты SQLite
pub async fn get_projects(pool: &Pool<Sqlite>, user_id: Option<i32>) -> Result<Vec<Project>> {
    let query = "SELECT * FROM project ORDER BY name";
    
    let projects = sqlx::query_as::<_, Project>(query)
        .fetch_all(pool)
        .await
        .map_err(Error::Database)?;

    Ok(projects)
}

/// Получает проект по ID SQLite
pub async fn get_project(pool: &Pool<Sqlite>, project_id: i32) -> Result<Project> {
    let query = "SELECT * FROM project WHERE id = ?";
    
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

/// Создаёт проект SQLite
pub async fn create_project(pool: &Pool<Sqlite>, mut project: Project) -> Result<Project> {
    let query = "INSERT INTO project (name, created, alert, max_parallel_tasks, type, default_secret_storage_id) VALUES (?, ?, ?, ?, ?, ?) RETURNING id";
    
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

/// Обновляет проект SQLite
pub async fn update_project(pool: &Pool<Sqlite>, project: Project) -> Result<()> {
    let query = "UPDATE project SET name = ?, alert = ?, max_parallel_tasks = ?, type = ?, default_secret_storage_id = ? WHERE id = ?";
    
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

/// Удаляет проект SQLite
pub async fn delete_project(pool: &Pool<Sqlite>, project_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM project WHERE id = ?")
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_project_crud() {
        let (pool, _temp) = crate::db::sql::test_helpers::create_test_pool().await.unwrap();
        crate::db::sql::test_helpers::init_project_table(&pool).await.unwrap();

        // Create
        let project = Project::new("Test Project".to_string());
        let created = create_project(&pool, project).await.unwrap();
        assert!(created.id > 0);
        
        // Read
        let retrieved = get_project(&pool, created.id).await.unwrap();
        assert_eq!(retrieved.name, "Test Project");
        
        // Update
        let mut updated = retrieved;
        updated.name = "Updated Project".to_string();
        update_project(&pool, updated).await.unwrap();
        
        // Delete
        delete_project(&pool, created.id).await.unwrap();
    }
}
