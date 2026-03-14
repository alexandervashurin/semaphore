//! TemplateManager - управление шаблонами
//!
//! Реализация трейта TemplateManager для SqlStore

use crate::db::sql::SqlStore;
use crate::db::sql::types::SqlDialect;
use crate::db::store::*;
use crate::models::Template;
use crate::error::{Error, Result};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl TemplateManager for SqlStore {
    async fn get_templates(&self, project_id: i32) -> Result<Vec<Template>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM template WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Template {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    playbook: row.get("playbook"),
                    description: row.get("description"),
                    inventory_id: row.try_get("inventory_id").ok().flatten(),
                    repository_id: row.try_get("repository_id").ok().flatten(),
                    environment_id: row.try_get("environment_id").ok().flatten(),
                    r#type: row.get("type"),
                    app: row.get("app"),
                    git_branch: row.try_get("git_branch").ok().flatten(),
                    created: row.get("created"),
                    arguments: row.get("arguments"),
                    vault_key_id: row.get("vault_key_id"),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM template WHERE project_id = $1 ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Template {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    playbook: row.get("playbook"),
                    description: row.get("description"),
                    inventory_id: row.try_get("inventory_id").ok().flatten(),
                    repository_id: row.try_get("repository_id").ok().flatten(),
                    environment_id: row.try_get("environment_id").ok().flatten(),
                    r#type: row.get("type"),
                    app: row.get("app"),
                    git_branch: row.try_get("git_branch").ok().flatten(),
                    created: row.get("created"),
                    arguments: row.get("arguments"),
                    vault_key_id: row.get("vault_key_id"),
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `template` WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query)
                    .bind(project_id)
                    .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Template {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    playbook: row.get("playbook"),
                    description: row.get("description"),
                    inventory_id: row.try_get("inventory_id").ok().flatten(),
                    repository_id: row.try_get("repository_id").ok().flatten(),
                    environment_id: row.try_get("environment_id").ok().flatten(),
                    r#type: row.get("type"),
                    app: row.get("app"),
                    git_branch: row.try_get("git_branch").ok().flatten(),
                    created: row.get("created"),
                    arguments: row.get("arguments"),
                    vault_key_id: row.get("vault_key_id"),
                }).collect())
            }
        }
    }

    async fn get_template(&self, project_id: i32, template_id: i32) -> Result<Template> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM template WHERE id = ? AND project_id = ?";
                let row = sqlx::query(query)
                    .bind(template_id)
                    .bind(project_id)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Шаблон не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Template {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    playbook: row.get("playbook"),
                    description: row.get("description"),
                    inventory_id: row.get("inventory_id"),
                    repository_id: row.get("repository_id"),
                    environment_id: row.get("environment_id"),
                    r#type: row.get("type"),
                    app: row.get("app"),
                    git_branch: row.get("git_branch"),
                    created: row.get("created"),
                    arguments: row.get("arguments"),
                    vault_key_id: row.get("vault_key_id"),
                })
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM template WHERE id = $1 AND project_id = $2";
                let row = sqlx::query(query)
                    .bind(template_id)
                    .bind(project_id)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Шаблон не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Template {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    playbook: row.get("playbook"),
                    description: row.get("description"),
                    inventory_id: row.get("inventory_id"),
                    repository_id: row.get("repository_id"),
                    environment_id: row.get("environment_id"),
                    r#type: row.get("type"),
                    app: row.get("app"),
                    git_branch: row.get("git_branch"),
                    created: row.get("created"),
                    arguments: row.get("arguments"),
                    vault_key_id: row.get("vault_key_id"),
                })
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `template` WHERE id = ? AND project_id = ?";
                let row = sqlx::query(query)
                    .bind(template_id)
                    .bind(project_id)
                    .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Шаблон не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Template {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    name: row.get("name"),
                    playbook: row.get("playbook"),
                    description: row.get("description"),
                    inventory_id: row.get("inventory_id"),
                    repository_id: row.get("repository_id"),
                    environment_id: row.get("environment_id"),
                    r#type: row.get("type"),
                    app: row.get("app"),
                    git_branch: row.get("git_branch"),
                    created: row.get("created"),
                    arguments: row.get("arguments"),
                    vault_key_id: row.get("vault_key_id"),
                })
            }
        }
    }

    async fn create_template(&self, mut template: Template) -> Result<Template> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO template (project_id, name, playbook, description, inventory_id, repository_id, environment_id, type, app, git_branch, created, arguments, vault_key_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(template.project_id)
                    .bind(&template.name)
                    .bind(&template.playbook)
                    .bind(&template.description)
                    .bind(template.inventory_id)
                    .bind(template.repository_id)
                    .bind(template.environment_id)
                    .bind(&template.r#type)
                    .bind(&template.app)
                    .bind(&template.git_branch)
                    .bind(template.created)
                    .bind(&template.arguments)
                    .bind(template.vault_key_id)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                template.id = id;
                Ok(template)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO template (project_id, name, playbook, description, inventory_id, repository_id, environment_id, type, app, git_branch, created, arguments, vault_key_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(template.project_id)
                    .bind(&template.name)
                    .bind(&template.playbook)
                    .bind(&template.description)
                    .bind(template.inventory_id)
                    .bind(template.repository_id)
                    .bind(template.environment_id)
                    .bind(&template.r#type)
                    .bind(&template.app)
                    .bind(&template.git_branch)
                    .bind(template.created)
                    .bind(&template.arguments)
                    .bind(template.vault_key_id)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                template.id = id;
                Ok(template)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `template` (project_id, name, playbook, description, inventory_id, repository_id, environment_id, type, app, git_branch, created, arguments, vault_key_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
                sqlx::query(query)
                    .bind(template.project_id)
                    .bind(&template.name)
                    .bind(&template.playbook)
                    .bind(&template.description)
                    .bind(template.inventory_id)
                    .bind(template.repository_id)
                    .bind(template.environment_id)
                    .bind(&template.r#type)
                    .bind(&template.app)
                    .bind(&template.git_branch)
                    .bind(template.created)
                    .bind(&template.arguments)
                    .bind(template.vault_key_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                let id: i32 = sqlx::query_scalar("SELECT LAST_INSERT_ID()")
                    .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                template.id = id;
                Ok(template)
            }
        }
    }

    async fn update_template(&self, template: Template) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE template SET name = ?, playbook = ?, description = ?, inventory_id = ?, repository_id = ?, environment_id = ?, type = ?, app = ?, git_branch = ?, arguments = ?, vault_key_id = ? WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(&template.name)
                    .bind(&template.playbook)
                    .bind(&template.description)
                    .bind(template.inventory_id)
                    .bind(template.repository_id)
                    .bind(template.environment_id)
                    .bind(&template.r#type)
                    .bind(&template.app)
                    .bind(&template.git_branch)
                    .bind(&template.arguments)
                    .bind(&template.vault_key_id)
                    .bind(template.id)
                    .bind(template.project_id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE template SET name = $1, playbook = $2, description = $3, inventory_id = $4, repository_id = $5, environment_id = $6, type = $7, app = $8, git_branch = $9, arguments = $10, vault_key_id = $11 WHERE id = $12 AND project_id = $13";
                sqlx::query(query)
                    .bind(&template.name)
                    .bind(&template.playbook)
                    .bind(&template.description)
                    .bind(template.inventory_id)
                    .bind(template.repository_id)
                    .bind(template.environment_id)
                    .bind(&template.r#type)
                    .bind(&template.app)
                    .bind(&template.git_branch)
                    .bind(&template.arguments)
                    .bind(&template.vault_key_id)
                    .bind(template.id)
                    .bind(template.project_id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `template` SET name = ?, playbook = ?, description = ?, inventory_id = ?, repository_id = ?, environment_id = ?, type = ?, app = ?, git_branch = ?, arguments = ?, vault_key_id = ? WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(&template.name)
                    .bind(&template.playbook)
                    .bind(&template.description)
                    .bind(template.inventory_id)
                    .bind(template.repository_id)
                    .bind(template.environment_id)
                    .bind(&template.r#type)
                    .bind(&template.app)
                    .bind(&template.git_branch)
                    .bind(&template.arguments)
                    .bind(&template.vault_key_id)
                    .bind(template.id)
                    .bind(template.project_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_template(&self, project_id: i32, template_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                // Мягкое удаление - устанавливаем deleted = 1
                let query = "UPDATE template SET deleted = 1 WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(template_id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE template SET deleted = true WHERE id = $1 AND project_id = $2";
                sqlx::query(query)
                    .bind(template_id)
                    .bind(project_id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `template` SET deleted = 1 WHERE id = ? AND project_id = ?";
                sqlx::query(query)
                    .bind(template_id)
                    .bind(project_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

// ============================================================================
// InventoryManager - CRUD операции для инвентарей
// ============================================================================
