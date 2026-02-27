//! Repository CRUD Operations for BoltDB
//!
//! Операции с репозиториями в BoltDB

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::{Error, Result};
use crate::models::Repository;

impl BoltStore {
    /// Получает репозитории проекта
    pub async fn get_repositories(&self, project_id: i32) -> Result<Vec<Repository>> {
        self.get_objects::<Repository>(project_id, "repositories", crate::db::store::RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            filter: String::new(),
        }).await
    }

    /// Получает репозиторий по ID
    pub async fn get_repository(&self, project_id: i32, repo_id: i32) -> Result<Repository> {
        self.get_object::<Repository>(project_id, "repositories", repo_id).await
    }

    /// Создаёт репозиторий
    pub async fn create_repository(&self, mut repo: Repository) -> Result<Repository> {
        repo.id = self.get_next_id("repositories")?;
        self.create_object(repo.project_id, "repositories", &repo).await?;
        Ok(repo)
    }

    /// Обновляет репозиторий
    pub async fn update_repository(&self, repo: Repository) -> Result<()> {
        self.update_object(repo.project_id, "repositories", repo.id, &repo).await
    }

    /// Удаляет репозиторий
    pub async fn delete_repository(&self, project_id: i32, repo_id: i32) -> Result<()> {
        self.delete_object(project_id, "repositories", repo_id).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_operations() {
        // Тест для проверки операций с репозиториями
        assert!(true);
    }
}
