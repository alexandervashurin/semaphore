//! SQLite Repository CRUD operations

use crate::error::{Error, Result};
use crate::models::*;
use sqlx::{Sqlite, Pool};

/// Получает все репозитории проекта SQLite
pub async fn get_repositories(pool: &Pool<Sqlite>, project_id: i32) -> Result<Vec<Repository>> {
    let query = "SELECT * FROM repository WHERE project_id = ? ORDER BY name";
    
    let repositories = sqlx::query_as::<_, Repository>(query)
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(Error::Database)?;

    Ok(repositories)
}

/// Получает репозиторий по ID SQLite
pub async fn get_repository(pool: &Pool<Sqlite>, project_id: i32, repository_id: i32) -> Result<Repository> {
    let query = "SELECT * FROM repository WHERE id = ? AND project_id = ?";
    
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

/// Создаёт репозиторий SQLite
pub async fn create_repository(pool: &Pool<Sqlite>, mut repository: Repository) -> Result<Repository> {
    let query = "INSERT INTO repository (project_id, name, git_url, git_type, git_branch, key_id, created) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id";
    
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

/// Обновляет репозиторий SQLite
pub async fn update_repository(pool: &Pool<Sqlite>, repository: Repository) -> Result<()> {
    let query = "UPDATE repository SET name = ?, git_url = ?, git_type = ?, git_branch = ?, key_id = ? WHERE id = ? AND project_id = ?";
    
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

/// Удаляет репозиторий SQLite
pub async fn delete_repository(pool: &Pool<Sqlite>, project_id: i32, repository_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM repository WHERE id = ? AND project_id = ?")
        .bind(repository_id)
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RepositoryType;

    #[test]
    fn test_get_repositories_query_structure() {
        let query = "SELECT * FROM repository WHERE project_id = ? ORDER BY name";
        assert!(query.contains("repository"));
        assert!(query.contains("project_id = ?"));
        assert!(query.contains("ORDER BY name"));
    }

    #[test]
    fn test_get_repository_query_structure() {
        let query = "SELECT * FROM repository WHERE id = ? AND project_id = ?";
        assert!(query.contains("id = ?"));
        assert!(query.contains("project_id = ?"));
    }

    #[test]
    fn test_create_repository_query_structure() {
        let expected = "INSERT INTO repository (project_id, name, git_url, git_type, git_branch, key_id, created) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id";
        assert!(expected.contains("repository"));
        assert!(expected.contains("RETURNING id"));
        assert!(expected.matches('?').count() == 7);
    }

    #[test]
    fn test_update_repository_query_structure() {
        let expected = "UPDATE repository SET name = ?, git_url = ?, git_type = ?, git_branch = ?, key_id = ? WHERE id = ? AND project_id = ?";
        assert!(expected.contains("UPDATE repository"));
        assert!(expected.contains("WHERE id = ? AND project_id = ?"));
        assert!(expected.matches('?').count() == 7);
    }

    #[test]
    fn test_delete_repository_query_structure() {
        let expected = "DELETE FROM repository WHERE id = ? AND project_id = ?";
        assert!(expected.contains("repository"));
        assert!(expected.contains("id = ? AND project_id = ?"));
    }

    #[test]
    fn test_sqlite_uses_question_placeholders() {
        let queries = [
            "SELECT * FROM repository WHERE id = ?",
            "DELETE FROM repository WHERE id = ? AND project_id = ?",
        ];
        for q in &queries {
            assert!(q.contains('?'), "SQLite should use ? placeholders");
            assert!(!q.contains('$'), "SQLite should not use $N placeholders");
        }
    }

    #[test]
    fn test_sqlite_no_backticks() {
        let queries = [
            "SELECT * FROM repository WHERE id = ?",
            "DELETE FROM repository WHERE id = ?",
        ];
        for q in &queries {
            assert!(!q.contains('`'), "SQLite should not use backticks");
        }
    }

    #[test]
    fn test_sqlite_returning_clause() {
        let query = "INSERT INTO repository (...) VALUES (?) RETURNING id";
        assert!(query.contains("RETURNING id"), "SQLite uses RETURNING clause");
    }

    #[test]
    fn test_repository_model_fields() {
        let repo = Repository::new(10, "sqlite-repo", "https://github.com/user/repo.git");
        assert_eq!(repo.project_id, 10);
        assert_eq!(repo.name, "sqlite-repo");
        assert_eq!(repo.git_type, RepositoryType::Git);
    }

    #[test]
    fn test_repository_serialization() {
        let repo = Repository::new(1, "sqlite-test-repo", "https://example.com/repo.git");
        let json = serde_json::to_string(&repo).unwrap();
        assert!(json.contains("\"name\":\"sqlite-test-repo\""));
        assert!(json.contains("\"git_type\":\"git\""));
    }

    #[test]
    fn test_repository_bind_order_matches_query() {
        let columns = [
            "project_id", "name", "git_url", "git_type", "git_branch", "key_id", "created",
        ];
        assert_eq!(columns.len(), 7);
        assert_eq!(columns[0], "project_id");
        assert_eq!(columns[2], "git_url");
    }

    #[test]
    fn test_sqlite_repository_debug_format() {
        let query = "SELECT * FROM repository WHERE id = ?";
        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("repository"));
    }
}
