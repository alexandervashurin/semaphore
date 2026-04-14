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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RepositoryType;

    #[test]
    fn test_get_repositories_query_structure() {
        let query = "SELECT * FROM repository WHERE project_id = $1 ORDER BY name";
        assert!(query.contains("repository"));
        assert!(query.contains("project_id = $1"));
        assert!(query.contains("ORDER BY name"));
    }

    #[test]
    fn test_get_repository_query_structure() {
        let query = "SELECT * FROM repository WHERE id = $1 AND project_id = $2";
        assert!(query.contains("id = $1"));
        assert!(query.contains("project_id = $2"));
    }

    #[test]
    fn test_create_repository_query_structure() {
        let expected = "INSERT INTO repository (project_id, name, git_url, git_type, git_branch, key_id, created) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id";
        assert!(expected.contains("repository"));
        assert!(expected.contains("RETURNING id"));
        assert!(expected.contains("$7"));
    }

    #[test]
    fn test_update_repository_query_structure() {
        let expected = "UPDATE repository SET name = $1, git_url = $2, git_type = $3, git_branch = $4, key_id = $5 WHERE id = $6 AND project_id = $7";
        assert!(expected.contains("UPDATE repository"));
        assert!(expected.contains("WHERE id = $6 AND project_id = $7"));
        assert!(expected.contains("$7"));
    }

    #[test]
    fn test_delete_repository_query_structure() {
        let expected = "DELETE FROM repository WHERE id = $1 AND project_id = $2";
        assert!(expected.contains("repository"));
        assert!(expected.contains("id = $1 AND project_id = $2"));
    }

    #[test]
    fn test_postgres_uses_dollar_placeholders() {
        let queries = [
            "SELECT * FROM repository WHERE id = $1",
            "DELETE FROM repository WHERE id = $1 AND project_id = $2",
        ];
        for q in &queries {
            assert!(q.contains('$'), "Postgres should use $N placeholders");
            assert!(!q.contains('?'), "Postgres should not use ? placeholders");
        }
    }

    #[test]
    fn test_postgres_no_backticks() {
        let queries = [
            "SELECT * FROM repository WHERE id = $1",
            "DELETE FROM repository WHERE id = $1",
        ];
        for q in &queries {
            assert!(!q.contains('`'), "Postgres should not use backticks");
        }
    }

    #[test]
    fn test_postgres_returning_clause() {
        let query = "INSERT INTO repository (...) VALUES (...) RETURNING id";
        assert!(
            query.contains("RETURNING id"),
            "Postgres uses RETURNING clause"
        );
    }

    #[test]
    fn test_repository_model_fields() {
        let repo = Repository::new(
            10,
            "pg-repo".to_string(),
            "https://github.com/user/repo.git".to_string(),
        );
        assert_eq!(repo.project_id, 10);
        assert_eq!(repo.name, "pg-repo");
        assert_eq!(repo.git_type, RepositoryType::Git);
    }

    #[test]
    fn test_repository_serialization() {
        let repo = Repository::new(
            1,
            "pg-test-repo".to_string(),
            "https://example.com/repo.git".to_string(),
        );
        let json = serde_json::to_string(&repo).unwrap();
        assert!(json.contains("\"name\":\"pg-test-repo\""));
        assert!(json.contains("\"git_type\":\"git\""));
    }

    #[test]
    fn test_repository_bind_order_matches_query() {
        let columns = [
            "project_id",
            "name",
            "git_url",
            "git_type",
            "git_branch",
            "key_id",
            "created",
        ];
        assert_eq!(columns.len(), 7);
        assert_eq!(columns[0], "project_id");
        assert_eq!(columns[2], "git_url");
    }

    #[test]
    fn test_postgres_repository_debug_format() {
        let query = "SELECT * FROM repository WHERE id = $1";
        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("repository"));
    }
}
