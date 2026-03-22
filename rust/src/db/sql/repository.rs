//! Repository CRUD Operations
//!
//! Адаптер для декомпозированных модулей
//!
//! Новые модули: sqlite::repository, postgres::repository, mysql::repository

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::Repository;

impl SqlDb {
    /// Получает репозитории проекта
    pub async fn get_repositories(&self, project_id: i32) -> Result<Vec<Repository>> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::repository::get_repositories(pool, project_id).await
    }

    /// Получает репозиторий по ID
    pub async fn get_repository(&self, project_id: i32, repository_id: i32) -> Result<Repository> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::repository::get_repository(pool, project_id, repository_id).await
    }

    /// Создаёт репозиторий
    pub async fn create_repository(&self, repository: Repository) -> Result<Repository> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::repository::create_repository(pool, repository).await
    }

    /// Обновляет репозиторий
    pub async fn update_repository(&self, repository: Repository) -> Result<()> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::repository::update_repository(pool, repository).await
    }

    /// Удаляет репозиторий
    pub async fn delete_repository(&self, project_id: i32, repository_id: i32) -> Result<()> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::repository::delete_repository(pool, project_id, repository_id).await
    }
}
