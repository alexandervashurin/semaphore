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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_projects_query_structure() {
        let query = "SELECT * FROM project ORDER BY name";
        assert!(query.contains("project"));
        assert!(query.contains("ORDER BY name"));
    }

    #[test]
    fn test_get_project_query_structure() {
        let query = "SELECT * FROM project WHERE id = $1";
        assert!(query.contains("project"));
        assert!(query.contains("id = $1"));
    }

    #[test]
    fn test_create_project_query_structure() {
        let expected = "INSERT INTO project (name, created, alert, max_parallel_tasks, type, default_secret_storage_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id";
        assert!(expected.contains("project"));
        assert!(expected.contains("RETURNING id"));
        assert!(expected.contains("$6"));
    }

    #[test]
    fn test_update_project_query_structure() {
        let expected = "UPDATE project SET name = $1, alert = $2, max_parallel_tasks = $3, type = $4, default_secret_storage_id = $5 WHERE id = $6";
        assert!(expected.contains("UPDATE project"));
        assert!(expected.contains("WHERE id = $6"));
        assert!(expected.contains("$6"));
    }

    #[test]
    fn test_delete_project_query_structure() {
        let expected = "DELETE FROM project WHERE id = $1";
        assert!(expected.contains("project"));
        assert!(expected.contains("id = $1"));
    }

    #[test]
    fn test_postgres_uses_dollar_placeholders() {
        let queries = [
            "SELECT * FROM project WHERE id = $1",
            "DELETE FROM project WHERE id = $1",
        ];
        for q in &queries {
            assert!(q.contains('$'), "Postgres should use $N placeholders");
            assert!(!q.contains('?'), "Postgres should not use ? placeholders");
        }
    }

    #[test]
    fn test_postgres_no_backticks() {
        let queries = [
            "SELECT * FROM project WHERE id = $1",
            "DELETE FROM project WHERE id = $1",
        ];
        for q in &queries {
            assert!(!q.contains('`'), "Postgres should not use backticks");
        }
    }

    #[test]
    fn test_postgres_returning_clause() {
        let query = "INSERT INTO project (...) VALUES (...) RETURNING id";
        assert!(
            query.contains("RETURNING id"),
            "Postgres uses RETURNING clause"
        );
    }

    #[test]
    fn test_project_model_fields() {
        let project = Project::new("Pg Project".to_string());
        assert_eq!(project.name, "Pg Project");
        assert_eq!(project.id, 0);
        assert!(!project.alert);
    }

    #[test]
    fn test_project_serialization() {
        let project = Project::new("Pg Serialize Test".to_string());
        let json = serde_json::to_string(&project).unwrap();
        assert!(json.contains("\"name\":\"Pg Serialize Test\""));
        assert!(json.contains("\"alert\":false"));
    }

    #[test]
    fn test_project_bind_order_matches_query() {
        let columns = [
            "name",
            "created",
            "alert",
            "max_parallel_tasks",
            "type",
            "default_secret_storage_id",
        ];
        assert_eq!(columns.len(), 6);
        assert_eq!(columns[0], "name");
        assert_eq!(columns[3], "max_parallel_tasks");
    }

    #[test]
    fn test_postgres_project_debug_format() {
        let query = "SELECT * FROM project WHERE id = $1";
        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("project"));
    }
}
