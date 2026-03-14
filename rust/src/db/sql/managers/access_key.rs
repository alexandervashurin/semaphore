//! AccessKeyManager - управление ключами доступа
//!
//! Реализация трейта AccessKeyManager для SqlStore

use crate::db::sql::SqlStore;
use crate::db::sql::types::SqlDialect;
use crate::db::store::*;
use crate::models::AccessKey;
use crate::error::{Error, Result};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl AccessKeyManager for SqlStore {
    async fn get_access_keys(&self, project_id: i32) -> Result<Vec<AccessKey>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM access_key WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| AccessKey {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    r#type: row.get("type"),
                    user_id: row.get("user_id"),
                    login_password_login: row.get("login_password_login"),
                    login_password_password: row.get("login_password_password"),
                    ssh_key: row.get("ssh_key"),
                    ssh_passphrase: row.get("ssh_passphrase"),
                    access_key_access_key: row.get("access_key_access_key"),
                    access_key_secret_key: row.get("access_key_secret_key"),
                    secret_storage_id: row.get("secret_storage_id"),
                    owner: row.get("owner"),
                    environment_id: row.get("environment_id"),
                    created: row.get("created"),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM access_key WHERE project_id = $1 ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| AccessKey {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    r#type: row.get("type"),
                    user_id: row.get("user_id"),
                    login_password_login: row.get("login_password_login"),
                    login_password_password: row.get("login_password_password"),
                    ssh_key: row.get("ssh_key"),
                    ssh_passphrase: row.get("ssh_passphrase"),
                    access_key_access_key: row.get("access_key_access_key"),
                    access_key_secret_key: row.get("access_key_secret_key"),
                    secret_storage_id: row.get("secret_storage_id"),
                    owner: row.get("owner"),
                    environment_id: row.get("environment_id"),
                    created: row.get("created"),
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `access_key` WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| AccessKey {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    r#type: row.get("type"),
                    user_id: row.get("user_id"),
                    login_password_login: row.get("login_password_login"),
                    login_password_password: row.get("login_password_password"),
                    ssh_key: row.get("ssh_key"),
                    ssh_passphrase: row.get("ssh_passphrase"),
                    access_key_access_key: row.get("access_key_access_key"),
                    access_key_secret_key: row.get("access_key_secret_key"),
                    secret_storage_id: row.get("secret_storage_id"),
                    owner: row.get("owner"),
                    environment_id: row.get("environment_id"),
                    created: row.get("created"),
                }).collect())
            }
        }
    }

    async fn get_access_key(&self, project_id: i32, key_id: i32) -> Result<AccessKey> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM access_key WHERE id = ? AND project_id = ?";
                let row = sqlx::query(query)
                    .bind(key_id)
                    .bind(project_id)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Ключ доступа не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(AccessKey {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    r#type: row.get("type"),
                    user_id: row.get("user_id"),
                    login_password_login: row.get("login_password_login"),
                    login_password_password: row.get("login_password_password"),
                    ssh_key: row.get("ssh_key"),
                    ssh_passphrase: row.get("ssh_passphrase"),
                    access_key_access_key: row.get("access_key_access_key"),
                    access_key_secret_key: row.get("access_key_secret_key"),
                    secret_storage_id: row.get("secret_storage_id"),
                    owner: row.get("owner"),
                    environment_id: row.get("environment_id"),
                    created: row.get("created"),
                })
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM access_key WHERE id = $1 AND project_id = $2";
                let row = sqlx::query(query)
                    .bind(key_id)
                    .bind(project_id)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Ключ доступа не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(AccessKey {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    r#type: row.get("type"),
                    user_id: row.get("user_id"),
                    login_password_login: row.get("login_password_login"),
                    login_password_password: row.get("login_password_password"),
                    ssh_key: row.get("ssh_key"),
                    ssh_passphrase: row.get("ssh_passphrase"),
                    access_key_access_key: row.get("access_key_access_key"),
                    access_key_secret_key: row.get("access_key_secret_key"),
                    secret_storage_id: row.get("secret_storage_id"),
                    owner: row.get("owner"),
                    environment_id: row.get("environment_id"),
                    created: row.get("created"),
                })
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `access_key` WHERE id = ? AND project_id = ?";
                let row = sqlx::query(query)
                    .bind(key_id)
                    .bind(project_id)
                    .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Ключ доступа не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(AccessKey {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    r#type: row.get("type"),
                    user_id: row.get("user_id"),
                    login_password_login: row.get("login_password_login"),
                    login_password_password: row.get("login_password_password"),
                    ssh_key: row.get("ssh_key"),
                    ssh_passphrase: row.get("ssh_passphrase"),
                    access_key_access_key: row.get("access_key_access_key"),
                    access_key_secret_key: row.get("access_key_secret_key"),
                    secret_storage_id: row.get("secret_storage_id"),
                    owner: row.get("owner"),
                    environment_id: row.get("environment_id"),
                    created: row.get("created"),
                })
            }
        }
    }

    async fn create_access_key(&self, mut key: AccessKey) -> Result<AccessKey> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO access_key (project_id, name, type, user_id, login_password_login, login_password_password, ssh_key, ssh_passphrase, access_key_access_key, access_key_secret_key, secret_storage_id, owner, environment_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(key.project_id)
                    .bind(&key.name)
                    .bind(&key.r#type)
                    .bind(&key.user_id)
                    .bind(&key.login_password_login)
                    .bind(&key.login_password_password)
                    .bind(&key.ssh_key)
                    .bind(&key.ssh_passphrase)
                    .bind(&key.access_key_access_key)
                    .bind(&key.access_key_secret_key)
                    .bind(&key.secret_storage_id)
                    .bind(key.owner.as_ref().map(|o| o.to_string()))
                    .bind(&key.environment_id)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                key.id = id;
                Ok(key)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO access_key (project_id, name, type, user_id, login_password_login, login_password_password, ssh_key, ssh_passphrase, access_key_access_key, access_key_secret_key, secret_storage_id, owner, environment_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(key.project_id)
                    .bind(&key.name)
                    .bind(&key.r#type)
                    .bind(&key.user_id)
                    .bind(&key.login_password_login)
                    .bind(&key.login_password_password)
                    .bind(&key.ssh_key)
                    .bind(&key.ssh_passphrase)
                    .bind(&key.access_key_access_key)
                    .bind(&key.access_key_secret_key)
                    .bind(&key.secret_storage_id)
                    .bind(key.owner.as_ref().map(|o| o.to_string()))
                    .bind(&key.environment_id)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                key.id = id;
                Ok(key)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `access_key` (project_id, name, type, user_id, login_password_login, login_password_password, ssh_key, ssh_passphrase, access_key_access_key, access_key_secret_key, secret_storage_id, owner, environment_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
                sqlx::query(query)
                    .bind(key.project_id)
                    .bind(&key.name)
                    .bind(&key.r#type)
                    .bind(&key.user_id)
                    .bind(&key.login_password_login)
                    .bind(&key.login_password_password)
                    .bind(&key.ssh_key)
                    .bind(&key.ssh_passphrase)
                    .bind(&key.access_key_access_key)
                    .bind(&key.access_key_secret_key)
                    .bind(&key.secret_storage_id)
                    .bind(key.owner.as_ref().map(|o| o.to_string()))
                    .bind(&key.environment_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                let id: i32 = sqlx::query_scalar("SELECT LAST_INSERT_ID()")
                    .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                key.id = id;
                Ok(key)
            }
        }
    }

    async fn update_access_key(&self, key: AccessKey) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE access_key SET name = ?, type = ?, user_id = ?, login_password_login = ?, login_password_password = ?, ssh_key = ?, ssh_passphrase = ?, access_key_access_key = ?, access_key_secret_key = ?, secret_storage_id = ?, owner = ?, environment_id = ? WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(&key.name)
                    .bind(&key.r#type)
                    .bind(&key.user_id)
                    .bind(&key.login_password_login)
                    .bind(&key.login_password_password)
                    .bind(&key.ssh_key)
                    .bind(&key.ssh_passphrase)
                    .bind(&key.access_key_access_key)
                    .bind(&key.access_key_secret_key)
                    .bind(&key.secret_storage_id)
                    .bind(key.owner.as_ref().map(|o| o.to_string()))
                    .bind(&key.environment_id)
                    .bind(key.id)
                    .bind(key.project_id.unwrap_or(0))
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE access_key SET name = $1, type = $2, user_id = $3, login_password_login = $4, login_password_password = $5, ssh_key = $6, ssh_passphrase = $7, access_key_access_key = $8, access_key_secret_key = $9, secret_storage_id = $10, owner = $11, environment_id = $12 WHERE id = $13 AND project_id = $14";
                sqlx::query(query)
                    .bind(&key.name)
                    .bind(&key.r#type)
                    .bind(&key.user_id)
                    .bind(&key.login_password_login)
                    .bind(&key.login_password_password)
                    .bind(&key.ssh_key)
                    .bind(&key.ssh_passphrase)
                    .bind(&key.access_key_access_key)
                    .bind(&key.access_key_secret_key)
                    .bind(&key.secret_storage_id)
                    .bind(key.owner.as_ref().map(|o| o.to_string()))
                    .bind(&key.environment_id)
                    .bind(key.id)
                    .bind(key.project_id.unwrap_or(0))
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `access_key` SET name = ?, type = ?, user_id = ?, login_password_login = ?, login_password_password = ?, ssh_key = ?, ssh_passphrase = ?, access_key_access_key = ?, access_key_secret_key = ?, secret_storage_id = ?, owner = ?, environment_id = ? WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(&key.name)
                    .bind(&key.r#type)
                    .bind(&key.user_id)
                    .bind(&key.login_password_login)
                    .bind(&key.login_password_password)
                    .bind(&key.ssh_key)
                    .bind(&key.ssh_passphrase)
                    .bind(&key.access_key_access_key)
                    .bind(&key.access_key_secret_key)
                    .bind(&key.secret_storage_id)
                    .bind(key.owner.as_ref().map(|o| o.to_string()))
                    .bind(&key.environment_id)
                    .bind(key.id)
                    .bind(key.project_id.unwrap_or(0))
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_access_key(&self, project_id: i32, key_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "DELETE FROM access_key WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(key_id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "DELETE FROM access_key WHERE id = $1 AND project_id = $2";
                sqlx::query(query)
                    .bind(key_id)
                    .bind(project_id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "DELETE FROM `access_key` WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(key_id)
                    .bind(project_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

