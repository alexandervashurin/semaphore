//! EnvironmentManager - управление окружениями
//!
//! Реализация трейта EnvironmentManager для SqlStore

use crate::db::sql::SqlStore;
use crate::db::sql::types::SqlDialect;
use crate::db::store::*;
use crate::models::Environment;
use crate::error::{Error, Result};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl EnvironmentManager for SqlStore {
    async fn get_environments(&self, project_id: i32) -> Result<Vec<Environment>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM environment WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Environment {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    json: row.get("json"),
                    secret_storage_id: row.get("secret_storage_id"),
                    secrets: row.get("secrets"),
                    created: row.get("created"),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM environment WHERE project_id = $1 ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Environment {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    json: row.get("json"),
                    secret_storage_id: row.get("secret_storage_id"),
                    secrets: row.get("secrets"),
                    created: row.get("created"),
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `environment` WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Environment {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    json: row.get("json"),
                    secret_storage_id: row.get("secret_storage_id"),
                    secrets: row.get("secrets"),
                    created: row.get("created"),
                }).collect())
            }
        }
    }

    async fn get_environment(&self, project_id: i32, environment_id: i32) -> Result<Environment> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM environment WHERE id = ? AND project_id = ?";
                let row = sqlx::query(query)
                    .bind(environment_id)
                    .bind(project_id)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Окружение не найдено".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Environment {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    json: row.get("json"),
                    secret_storage_id: row.get("secret_storage_id"),
                    secrets: row.get("secrets"),
                    created: row.get("created"),
                })
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM environment WHERE id = $1 AND project_id = $2";
                let row = sqlx::query(query)
                    .bind(environment_id)
                    .bind(project_id)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Окружение не найдено".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Environment {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    json: row.get("json"),
                    secret_storage_id: row.get("secret_storage_id"),
                    secrets: row.get("secrets"),
                    created: row.get("created"),
                })
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `environment` WHERE id = ? AND project_id = ?";
                let row = sqlx::query(query)
                    .bind(environment_id)
                    .bind(project_id)
                    .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Окружение не найдено".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Environment {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    json: row.get("json"),
                    secret_storage_id: row.get("secret_storage_id"),
                    secrets: row.get("secrets"),
                    created: row.get("created"),
                })
            }
        }
    }

    async fn create_environment(&self, mut environment: Environment) -> Result<Environment> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO environment (project_id, name, json, secret_storage_id, secrets) VALUES (?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(environment.project_id)
                    .bind(&environment.name)
                    .bind(&environment.json)
                    .bind(&environment.secret_storage_id)
                    .bind(&environment.secrets)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                environment.id = id;
                Ok(environment)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO environment (project_id, name, json, secret_storage_id, secrets) VALUES ($1, $2, $3, $4, $5) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(environment.project_id)
                    .bind(&environment.name)
                    .bind(&environment.json)
                    .bind(&environment.secret_storage_id)
                    .bind(&environment.secrets)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                environment.id = id;
                Ok(environment)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `environment` (project_id, name, json, secret_storage_id, secrets) VALUES (?, ?, ?, ?, ?)";
                sqlx::query(query)
                    .bind(environment.project_id)
                    .bind(&environment.name)
                    .bind(&environment.json)
                    .bind(&environment.secret_storage_id)
                    .bind(&environment.secrets)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                let id: i32 = sqlx::query_scalar("SELECT LAST_INSERT_ID()")
                    .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                environment.id = id;
                Ok(environment)
            }
        }
    }

    async fn update_environment(&self, environment: Environment) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE environment SET name = ?, json = ?, secret_storage_id = ?, secrets = ? WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(&environment.name)
                    .bind(&environment.json)
                    .bind(&environment.secret_storage_id)
                    .bind(&environment.secrets)
                    .bind(environment.id)
                    .bind(environment.project_id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE environment SET name = $1, json = $2, secret_storage_id = $3, secrets = $4 WHERE id = $5 AND project_id = $6";
                sqlx::query(query)
                    .bind(&environment.name)
                    .bind(&environment.json)
                    .bind(&environment.secret_storage_id)
                    .bind(&environment.secrets)
                    .bind(environment.id)
                    .bind(environment.project_id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `environment` SET name = ?, json = ?, secret_storage_id = ?, secrets = ? WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(&environment.name)
                    .bind(&environment.json)
                    .bind(&environment.secret_storage_id)
                    .bind(&environment.secrets)
                    .bind(environment.id)
                    .bind(environment.project_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_environment(&self, project_id: i32, environment_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "DELETE FROM environment WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(environment_id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "DELETE FROM environment WHERE id = $1 AND project_id = $2";
                sqlx::query(query)
                    .bind(environment_id)
                    .bind(project_id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "DELETE FROM `environment` WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(environment_id)
                    .bind(project_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

