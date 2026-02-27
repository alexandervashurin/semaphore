//! Repository CRUD Operations
//!
//! Операции с репозиториями в SQL

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::Repository;

impl SqlDb {
    /// Получает репозитории проекта
    pub async fn get_repositories(&self, project_id: i32) -> Result<Vec<Repository>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let repos = sqlx::query_as::<_, Repository>(
                    "SELECT * FROM repository WHERE project_id = ? ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(repos)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Получает репозиторий по ID
    pub async fn get_repository(&self, project_id: i32, repo_id: i32) -> Result<Repository> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let repo = sqlx::query_as::<_, Repository>(
                    "SELECT * FROM repository WHERE id = ? AND project_id = ?"
                )
                .bind(repo_id)
                .bind(project_id)
                .fetch_optional(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                repo.ok_or(Error::NotFound("Repository not found".to_string()))
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Создаёт репозиторий
    pub async fn create_repository(&self, mut repo: Repository) -> Result<Repository> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO repository (project_id, name, git_url, ssh_key_id)
                     VALUES (?, ?, ?, ?)"
                )
                .bind(repo.project_id)
                .bind(&repo.name)
                .bind(&repo.git_url)
                .bind(repo.ssh_key_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                repo.id = result.last_insert_rowid() as i32;
                Ok(repo)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Обновляет репозиторий
    pub async fn update_repository(&self, repo: Repository) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE repository SET name = ?, git_url = ?, ssh_key_id = ?
                     WHERE id = ? AND project_id = ?"
                )
                .bind(&repo.name)
                .bind(&repo.git_url)
                .bind(repo.ssh_key_id)
                .bind(repo.id)
                .bind(repo.project_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Удаляет репозиторий
    pub async fn delete_repository(&self, project_id: i32, repo_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("DELETE FROM repository WHERE id = ? AND project_id = ?")
                    .bind(repo_id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
}
