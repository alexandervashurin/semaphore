//! Repository CRUD Operations
//!
//! Адаптер для декомпозированных модулей
//!
//! Новые модули: sqlite::repository, postgres::repository, mysql::repository

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::{Repository, RepositoryType};

impl SqlDb {
    /// Получает репозитории проекта
    pub async fn get_repositories(&self, project_id: i32) -> Result<Vec<Repository>> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::repository::get_repositories(pool, project_id).await
    }

    /// Получает репозиторий по ID
    pub async fn get_repository(&self, project_id: i32, repository_id: i32) -> Result<Repository> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::repository::get_repository(pool, project_id, repository_id).await
    }

    /// Создаёт репозиторий
    pub async fn create_repository(&self, repository: Repository) -> Result<Repository> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::repository::create_repository(pool, repository).await
    }

    /// Обновляет репозиторий
    pub async fn update_repository(&self, repository: Repository) -> Result<()> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::repository::update_repository(pool, repository).await
    }

    /// Удаляет репозиторий
    pub async fn delete_repository(&self, project_id: i32, repository_id: i32) -> Result<()> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::repository::delete_repository(pool, project_id, repository_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_new() {
        let repo = Repository::new(10, "my-repo".to_string(), "git@github.com:user/repo.git".to_string());
        assert_eq!(repo.project_id, 10);
        assert_eq!(repo.name, "my-repo");
        assert_eq!(repo.git_type, RepositoryType::Git);
        assert!(repo.git_branch.is_none());
    }

    #[test]
    fn test_repository_default() {
        let repo = Repository::default();
        assert_eq!(repo.id, 0);
        assert!(repo.name.is_empty());
        assert_eq!(repo.git_type, RepositoryType::Git);
    }

    #[test]
    fn test_repository_type_serialization() {
        assert_eq!(serde_json::to_string(&RepositoryType::Git).unwrap(), "\"git\"");
        assert_eq!(serde_json::to_string(&RepositoryType::Http).unwrap(), "\"http\"");
        assert_eq!(serde_json::to_string(&RepositoryType::Https).unwrap(), "\"https\"");
        assert_eq!(serde_json::to_string(&RepositoryType::File).unwrap(), "\"file\"");
    }

    #[test]
    fn test_repository_type_deserialization() {
        let git: RepositoryType = serde_json::from_str("\"git\"").unwrap();
        assert_eq!(git, RepositoryType::Git);
    }

    #[test]
    fn test_repository_serialization() {
        let repo = Repository {
            id: 1,
            project_id: 5,
            name: "deploy".to_string(),
            git_url: "https://github.com/org/deploy.git".to_string(),
            git_type: RepositoryType::Https,
            git_branch: Some("main".to_string()),
            key_id: Some(3),
            git_path: None,
            created: None,
        };
        let json = serde_json::to_string(&repo).unwrap();
        assert!(json.contains("\"name\":\"deploy\""));
        assert!(json.contains("\"git_branch\":\"main\""));
    }

    #[test]
    fn test_repository_skip_nulls() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "simple".to_string(),
            git_url: "https://example.com/repo.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let json = serde_json::to_string(&repo).unwrap();
        assert!(!json.contains("git_branch"));
        assert!(!json.contains("key_id"));
    }

    #[test]
    fn test_repository_get_clone_url() {
        let repo = Repository::new(1, "repo".to_string(), "https://github.com/user/repo.git".to_string());
        assert_eq!(repo.get_clone_url(), "https://github.com/user/repo.git");
    }

    #[test]
    fn test_repository_get_full_path_without_git_path() {
        let repo = Repository::new(1, "repo".to_string(), "https://example.com/repo.git".to_string());
        assert_eq!(repo.get_full_path(), "https://example.com/repo.git");
    }

    #[test]
    fn test_repository_get_full_path_with_git_path() {
        let mut repo = Repository::new(1, "repo".to_string(), "https://example.com/repo.git".to_string());
        repo.git_path = Some("/path/to/repo".to_string());
        assert_eq!(repo.get_full_path(), "/path/to/repo");
    }

    #[test]
    fn test_repository_clone() {
        let repo = Repository::new(1, "clone".to_string(), "https://github.com/user/repo.git".to_string());
        let cloned = repo.clone();
        assert_eq!(cloned.git_url, repo.git_url);
        assert_eq!(cloned.git_type, repo.git_type);
    }

    #[test]
    fn test_repository_debug_format() {
        let repo = Repository::new(1, "debug".to_string(), "https://example.com/debug.git".to_string());
        let debug_str = format!("{:?}", repo);
        assert!(debug_str.contains("Repository"));
        assert!(debug_str.contains("debug"));
    }

    #[test]
    fn test_repository_type_equality() {
        assert_eq!(RepositoryType::Git, RepositoryType::Git);
        assert_ne!(RepositoryType::Git, RepositoryType::Http);
    }
}
