//! Environment CRUD Operations
//!
//! Операции с окружениями в SQL

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::Environment;

impl SqlDb {
    /// Получает окружения проекта
    pub async fn get_environments(&self, project_id: i32) -> Result<Vec<Environment>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let envs = sqlx::query_as::<_, Environment>(
                    "SELECT * FROM environment WHERE project_id = ? ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(envs)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Получает окружение по ID
    pub async fn get_environment(&self, project_id: i32, env_id: i32) -> Result<Environment> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let env = sqlx::query_as::<_, Environment>(
                    "SELECT * FROM environment WHERE id = ? AND project_id = ?"
                )
                .bind(env_id)
                .bind(project_id)
                .fetch_optional(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                env.ok_or(Error::NotFound("Environment not found".to_string()))
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Создаёт окружение
    pub async fn create_environment(&self, mut env: Environment) -> Result<Environment> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO environment (project_id, name, json, secret_storage_id)
                     VALUES (?, ?, ?, ?)"
                )
                .bind(env.project_id)
                .bind(&env.name)
                .bind(&env.json)
                .bind(env.secret_storage_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                env.id = result.last_insert_rowid() as i32;
                Ok(env)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Обновляет окружение
    pub async fn update_environment(&self, env: Environment) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE environment SET name = ?, json = ?, secret_storage_id = ?
                     WHERE id = ? AND project_id = ?"
                )
                .bind(&env.name)
                .bind(&env.json)
                .bind(env.secret_storage_id)
                .bind(env.id)
                .bind(env.project_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Удаляет окружение
    pub async fn delete_environment(&self, project_id: i32, env_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("DELETE FROM environment WHERE id = ? AND project_id = ?")
                    .bind(env_id)
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
