//! RepositoryManager - управление репозиториями
//!
//! Реализация трейта RepositoryManager для SqlStore

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::Repository;
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl RepositoryManager for SqlStore {
    async fn get_repositories(&self, project_id: i32) -> Result<Vec<Repository>> {
        let query = "SELECT * FROM repository WHERE project_id = $1 ORDER BY name";
        let rows = sqlx::query(query)
            .bind(project_id)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| Repository {
                id: row.get("id"),
                project_id: row.get("project_id"),
                name: row.get("name"),
                git_url: row.get("git_url"),
                git_type: row.get("git_type"),
                git_branch: row.try_get("git_branch").ok().flatten(),
                key_id: row.try_get("key_id").ok().flatten(),
                git_path: row.try_get("git_path").ok().flatten(),
                created: row.get("created"),
            })
            .collect())
    }

    async fn get_repository(&self, project_id: i32, repository_id: i32) -> Result<Repository> {
        let query = "SELECT * FROM repository WHERE id = $1 AND project_id = $2";
        let row = sqlx::query(query)
            .bind(repository_id)
            .bind(project_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound("Репозиторий не найден".to_string()),
                _ => Error::Database(e),
            })?;

        Ok(Repository {
            id: row.get("id"),
            project_id: row.get("project_id"),
            name: row.get("name"),
            git_url: row.get("git_url"),
            git_type: row.get("git_type"),
            git_branch: row.get("git_branch"),
            key_id: row.get("key_id"),
            git_path: row.get("git_path"),
            created: row.get("created"),
        })
    }

    async fn create_repository(&self, mut repository: Repository) -> Result<Repository> {
        let query = "INSERT INTO repository (project_id, name, git_url, git_type, git_branch, key_id, git_path) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id";
        let id: i32 = sqlx::query_scalar(query)
            .bind(repository.project_id)
            .bind(&repository.name)
            .bind(&repository.git_url)
            .bind(&repository.git_type)
            .bind(&repository.git_branch)
            .bind(repository.key_id)
            .bind(&repository.git_path)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

        repository.id = id;
        Ok(repository)
    }

    async fn update_repository(&self, repository: Repository) -> Result<()> {
        let query = "UPDATE repository SET name = $1, git_url = $2, git_type = $3, git_branch = $4, key_id = $5, git_path = $6 WHERE id = $6 AND project_id = $8";
        sqlx::query(query)
            .bind(&repository.name)
            .bind(&repository.git_url)
            .bind(&repository.git_type)
            .bind(&repository.git_branch)
            .bind(repository.key_id)
            .bind(&repository.git_path)
            .bind(repository.id)
            .bind(repository.project_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_repository(&self, project_id: i32, repository_id: i32) -> Result<()> {
        let query = "DELETE FROM repository WHERE id = $1 AND project_id = $2";
        sqlx::query(query)
            .bind(repository_id)
            .bind(project_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::models::repository::{Repository, RepositoryType};
    use chrono::Utc;

    #[test]
    fn test_repository_type_serialization() {
        assert_eq!(
            serde_json::to_string(&RepositoryType::Git).unwrap(),
            "\"git\""
        );
        assert_eq!(
            serde_json::to_string(&RepositoryType::Http).unwrap(),
            "\"http\""
        );
        assert_eq!(
            serde_json::to_string(&RepositoryType::Https).unwrap(),
            "\"https\""
        );
        assert_eq!(
            serde_json::to_string(&RepositoryType::File).unwrap(),
            "\"file\""
        );
    }

    #[test]
    fn test_repository_new() {
        let repo = Repository::new(
            10,
            "my-repo".to_string(),
            "git@github.com:user/repo.git".to_string(),
        );
        assert_eq!(repo.project_id, 10);
        assert_eq!(repo.name, "my-repo");
        assert_eq!(repo.git_type, RepositoryType::Git);
        assert!(repo.git_branch.is_none());
        assert!(repo.key_id.is_none());
    }

    #[test]
    fn test_repository_default() {
        let repo = Repository::default();
        assert_eq!(repo.id, 0);
        assert!(repo.name.is_empty());
        assert!(repo.git_url.is_empty());
        assert_eq!(repo.git_type, RepositoryType::Git);
    }

    #[test]
    fn test_repository_serialization() {
        let repo = Repository {
            id: 1,
            project_id: 5,
            name: "deploy-repo".to_string(),
            git_url: "git@github.com:org/deploy.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: Some("main".to_string()),
            key_id: Some(3),
            git_path: None,
            created: Some(Utc::now()),
        };
        let json = serde_json::to_string(&repo).unwrap();
        assert!(json.contains("\"name\":\"deploy-repo\""));
        assert!(json.contains("\"git_branch\":\"main\""));
        assert!(json.contains("\"key_id\":3"));
    }

    #[test]
    fn test_repository_skip_nulls() {
        let repo = Repository {
            id: 1,
            project_id: 5,
            name: "simple".to_string(),
            git_url: "https://example.com/repo.git".to_string(),
            git_type: RepositoryType::Https,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let json = serde_json::to_string(&repo).unwrap();
        assert!(!json.contains("\"git_branch\""));
        assert!(!json.contains("\"key_id\""));
        assert!(!json.contains("\"git_path\""));
    }

    #[test]
    fn test_repository_get_clone_url() {
        let repo = Repository::new(
            1,
            "repo".to_string(),
            "https://github.com/user/repo.git".to_string(),
        );
        assert_eq!(repo.get_clone_url(), "https://github.com/user/repo.git");
    }

    #[test]
    fn test_repository_get_full_path() {
        let repo = Repository::new(
            1,
            "repo".to_string(),
            "https://example.com/repo.git".to_string(),
        );
        assert_eq!(repo.get_full_path(), "https://example.com/repo.git");

        let mut repo2 = repo.clone();
        repo2.git_path = Some("/path/to/repo".to_string());
        assert_eq!(repo2.get_full_path(), "/path/to/repo");
    }

    #[test]
    fn test_repository_clone() {
        let repo = Repository::new(
            1,
            "clone-repo".to_string(),
            "https://github.com/user/repo.git".to_string(),
        );
        let cloned = repo.clone();
        assert_eq!(cloned.name, repo.name);
        assert_eq!(cloned.git_url, repo.git_url);
    }

    #[test]
    fn test_repository_with_file_type() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "local-repo".to_string(),
            git_url: String::new(),
            git_type: RepositoryType::File,
            git_branch: None,
            key_id: None,
            git_path: Some("/var/repo".to_string()),
            created: None,
        };
        assert_eq!(repo.git_type, RepositoryType::File);
    }

    #[test]
    fn test_repository_decode_git_type() {
        let git: RepositoryType = serde_json::from_value(serde_json::json!("git")).unwrap();
        assert_eq!(git, RepositoryType::Git);

        let https: RepositoryType = serde_json::from_value(serde_json::json!("https")).unwrap();
        assert_eq!(https, RepositoryType::Https);

        let file: RepositoryType = serde_json::from_value(serde_json::json!("file")).unwrap();
        assert_eq!(file, RepositoryType::File);
    }
}
