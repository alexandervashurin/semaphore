//! Access Key CRUD Operations
//!
//! Операции с ключами доступа в SQL

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::{AccessKey, RetrieveQueryParams};

impl SqlDb {
    /// Получает ключи доступа проекта
    pub async fn get_access_keys(&self, project_id: i32, _params: RetrieveQueryParams) -> Result<Vec<AccessKey>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let keys = sqlx::query_as::<_, AccessKey>(
                    "SELECT * FROM access_key WHERE project_id = ? ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(keys)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Получает ключ доступа по ID
    pub async fn get_access_key(&self, project_id: i32, key_id: i32) -> Result<AccessKey> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let key = sqlx::query_as::<_, AccessKey>(
                    "SELECT * FROM access_key WHERE id = ? AND project_id = ?"
                )
                .bind(key_id)
                .bind(project_id)
                .fetch_optional(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                key.ok_or(Error::NotFound("Access key not found".to_string()))
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Создаёт ключ доступа
    pub async fn create_access_key(&self, mut key: AccessKey) -> Result<AccessKey> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO access_key (project_id, name, type, user_id, login_password, ssh_key, access_key, environment_id)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
                )
                .bind(key.project_id)
                .bind(&key.name)
                .bind(&key.key_type)
                .bind(key.user_id)
                .bind(&key.login_password)
                .bind(&key.ssh_key)
                .bind(&key.access_key)
                .bind(key.environment_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                key.id = result.last_insert_rowid() as i32;
                Ok(key)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Обновляет ключ доступа
    pub async fn update_access_key(&self, key: AccessKey) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE access_key SET name = ?, type = ?, user_id = ?, login_password = ?, ssh_key = ?, access_key = ?, environment_id = ?
                     WHERE id = ? AND project_id = ?"
                )
                .bind(&key.name)
                .bind(&key.key_type)
                .bind(key.user_id)
                .bind(&key.login_password)
                .bind(&key.ssh_key)
                .bind(&key.access_key)
                .bind(key.environment_id)
                .bind(key.id)
                .bind(key.project_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Удаляет ключ доступа
    pub async fn delete_access_key(&self, project_id: i32, key_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("DELETE FROM access_key WHERE id = ? AND project_id = ?")
                    .bind(key_id)
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
