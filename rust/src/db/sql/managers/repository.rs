//! RepositoryManager - управление репозиториями
//!
//! Реализация трейта RepositoryManager для SqlStore

use crate::db::sql::SqlStore;
use crate::db::sql::types::SqlDialect;
use crate::db::store::*;
use crate::models::Repository;
use crate::error::{Error, Result};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl RepositoryManager for SqlStore {
    async fn get_repositories(&self, project_id: i32) -> Result<Vec<Repository>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM repository WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;

                Ok(rows.into_iter().map(|row| Repository {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    git_url: row.get("git_url"),
                    git_type: row.get("git_type"),
                    git_branch: row.try_get("git_branch").ok().flatten(),
                    key_id: row.try_get("key_id").ok().flatten(),
                    git_path: row.try_get("git_path").ok().flatten(),
                    created: row.get("created"),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM repository WHERE project_id = $1 ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;

                Ok(rows.into_iter().map(|row| Repository {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    git_url: row.get("git_url"),
                    git_type: row.get("git_type"),
                    git_branch: row.try_get("git_branch").ok().flatten(),
                    key_id: row.try_get("key_id").ok().flatten(),
                    git_path: row.try_get("git_path").ok().flatten(),
                    created: row.get("created"),
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `repository` WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;

                Ok(rows.into_iter().map(|row| Repository {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    git_url: row.get("git_url"),
                    git_type: row.get("git_type"),
                    git_branch: row.try_get("git_branch").ok().flatten(),
                    key_id: row.try_get("key_id").ok().flatten(),
                    git_path: row.try_get("git_path").ok().flatten(),
                    created: row.get("created"),
                }).collect())
            }
        }
    }

    async fn get_repository(&self, project_id: i32, repository_id: i32) -> Result<Repository> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM repository WHERE id = ? AND project_id = ?";
                let row = sqlx::query(query)
                    .bind(repository_id)
                    .bind(project_id)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
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
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM repository WHERE id = $1 AND project_id = $2";
                let row = sqlx::query(query)
                    .bind(repository_id)
                    .bind(project_id)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
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
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `repository` WHERE id = ? AND project_id = ?";
                let row = sqlx::query(query)
                    .bind(repository_id)
                    .bind(project_id)
                    .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
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
        }
    }

    async fn create_repository(&self, mut repository: Repository) -> Result<Repository> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO repository (project_id, name, git_url, git_type, git_branch, key_id, git_path) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(repository.project_id)
                    .bind(&repository.name)
                    .bind(&repository.git_url)
                    .bind(&repository.git_type)
                    .bind(&repository.git_branch)
                    .bind(repository.key_id)
                    .bind(&repository.git_path)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;

                repository.id = id;
                Ok(repository)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO repository (project_id, name, git_url, git_type, git_branch, key_id, git_path) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(repository.project_id)
                    .bind(&repository.name)
                    .bind(&repository.git_url)
                    .bind(&repository.git_type)
                    .bind(&repository.git_branch)
                    .bind(repository.key_id)
                    .bind(&repository.git_path)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;

                repository.id = id;
                Ok(repository)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `repository` (project_id, name, git_url, git_type, git_branch, key_id, git_path) VALUES (?, ?, ?, ?, ?, ?, ?)";
                sqlx::query(query)
                    .bind(repository.project_id)
                    .bind(&repository.name)
                    .bind(&repository.git_url)
                    .bind(&repository.git_type)
                    .bind(&repository.git_branch)
                    .bind(repository.key_id)
                    .bind(&repository.git_path)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;

                let id: i32 = sqlx::query_scalar("SELECT LAST_INSERT_ID()")
                    .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;

                repository.id = id;
                Ok(repository)
            }
        }
    }

    async fn update_repository(&self, repository: Repository) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE repository SET name = ?, git_url = ?, git_type = ?, git_branch = ?, key_id = ?, git_path = ? WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(&repository.name)
                    .bind(&repository.git_url)
                    .bind(&repository.git_type)
                    .bind(&repository.git_branch)
                    .bind(repository.key_id)
                    .bind(&repository.git_path)
                    .bind(repository.id)
                    .bind(repository.project_id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
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
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `repository` SET name = ?, git_url = ?, git_type = ?, git_branch = ?, key_id = ?, git_path = ? WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(&repository.name)
                    .bind(&repository.git_url)
                    .bind(&repository.git_type)
                    .bind(&repository.git_branch)
                    .bind(repository.key_id)
                    .bind(&repository.git_path)
                    .bind(repository.id)
                    .bind(repository.project_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
        }
        Ok(())
    }

    async fn delete_repository(&self, project_id: i32, repository_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "DELETE FROM repository WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(repository_id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
                let query = "DELETE FROM repository WHERE id = $1 AND project_id = $2";
                sqlx::query(query)
                    .bind(repository_id)
                    .bind(project_id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                let query = "DELETE FROM `repository` WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(repository_id)
                    .bind(project_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
        }
        Ok(())
    }
}

