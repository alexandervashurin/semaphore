//! ProjectManager - управление проектами

use crate::db::sql::SqlStore;
use crate::db::sql::types::SqlDialect;
use crate::db::store::*;
use crate::models::{Project, ProjectUser};
use crate::error::{Error, Result};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl ProjectStore for SqlStore {
    async fn get_projects(&self, user_id: Option<i32>) -> Result<Vec<Project>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let (query, bind_user_id) = if let Some(uid) = user_id {
                    ("SELECT p.* FROM project p JOIN project__user pu ON p.id = pu.project_id WHERE pu.user_id = ?", Some(uid))
                } else {
                    ("SELECT * FROM project", None)
                };

                let mut q = sqlx::query(query);
                if let Some(uid) = bind_user_id {
                    q = q.bind(uid);
                }

                let rows = q
                    .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Project {
                    id: row.get("id"),
                    created: row.get("created"),
                    name: row.get("name"),
                    alert: row.get("alert"),
                    alert_chat: row.get("alert_chat"),
                    max_parallel_tasks: row.get("max_parallel_tasks"),
                    r#type: row.get("type"),
                    default_secret_storage_id: row.get("default_secret_storage_id"),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let (query, bind_user_id) = if let Some(uid) = user_id {
                    ("SELECT p.* FROM project p JOIN project__user pu ON p.id = pu.project_id WHERE pu.user_id = $1", Some(uid))
                } else {
                    ("SELECT * FROM project", None)
                };

                let mut q = sqlx::query(query);
                if let Some(uid) = bind_user_id {
                    q = q.bind(uid);
                }

                let rows = q
                    .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Project {
                    id: row.get("id"),
                    created: row.get("created"),
                    name: row.get("name"),
                    alert: row.get("alert"),
                    alert_chat: row.get("alert_chat"),
                    max_parallel_tasks: row.get("max_parallel_tasks"),
                    r#type: row.get("type"),
                    default_secret_storage_id: row.get("default_secret_storage_id"),
                }).collect())
            }
            SqlDialect::MySQL => {
                let (query, bind_user_id) = if let Some(uid) = user_id {
                    ("SELECT p.* FROM project p JOIN project__user pu ON p.id = pu.project_id WHERE pu.user_id = ?", Some(uid))
                } else {
                    ("SELECT * FROM project", None)
                };

                let mut q = sqlx::query(query);
                if let Some(uid) = bind_user_id {
                    q = q.bind(uid);
                }

                let rows = q
                    .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                Ok(rows.into_iter().map(|row| Project {
                    id: row.get("id"),
                    created: row.get("created"),
                    name: row.get("name"),
                    alert: row.get("alert"),
                    alert_chat: row.get("alert_chat"),
                    max_parallel_tasks: row.get("max_parallel_tasks"),
                    r#type: row.get("type"),
                    default_secret_storage_id: row.get("default_secret_storage_id"),
                }).collect())
            }
        }
    }

    async fn get_project(&self, project_id: i32) -> Result<Project> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM project WHERE id = ?";
                let row = sqlx::query(query)
                    .bind(project_id)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Проект не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Project {
                    id: row.get("id"),
                    created: row.get("created"),
                    name: row.get("name"),
                    alert: row.get("alert"),
                    alert_chat: row.get("alert_chat"),
                    max_parallel_tasks: row.get("max_parallel_tasks"),
                    r#type: row.get("type"),
                    default_secret_storage_id: row.get("default_secret_storage_id"),
                })
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM project WHERE id = $1";
                let row = sqlx::query(query)
                    .bind(project_id)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Проект не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Project {
                    id: row.get("id"),
                    created: row.get("created"),
                    name: row.get("name"),
                    alert: row.get("alert"),
                    alert_chat: row.get("alert_chat"),
                    max_parallel_tasks: row.get("max_parallel_tasks"),
                    r#type: row.get("type"),
                    default_secret_storage_id: row.get("default_secret_storage_id"),
                })
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM project WHERE id = ?";
                let row = sqlx::query(query)
                    .bind(project_id)
                    .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| match e {
                        sqlx::Error::RowNotFound => Error::NotFound("Проект не найден".to_string()),
                        _ => Error::Database(e),
                    })?;

                Ok(Project {
                    id: row.get("id"),
                    created: row.get("created"),
                    name: row.get("name"),
                    alert: row.get("alert"),
                    alert_chat: row.get("alert_chat"),
                    max_parallel_tasks: row.get("max_parallel_tasks"),
                    r#type: row.get("type"),
                    default_secret_storage_id: row.get("default_secret_storage_id"),
                })
            }
        }
    }

    async fn create_project(&self, mut project: Project) -> Result<Project> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO project (name, created, alert, alert_chat, max_parallel_tasks, type, default_secret_storage_id) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(&project.name)
                    .bind(project.created)
                    .bind(project.alert)
                    .bind(&project.alert_chat)
                    .bind(project.max_parallel_tasks)
                    .bind(&project.r#type)
                    .bind(&project.default_secret_storage_id)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                project.id = id;
                Ok(project)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO project (name, created, alert, alert_chat, max_parallel_tasks, type, default_secret_storage_id) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(&project.name)
                    .bind(project.created)
                    .bind(project.alert)
                    .bind(&project.alert_chat)
                    .bind(project.max_parallel_tasks)
                    .bind(&project.r#type)
                    .bind(&project.default_secret_storage_id)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                project.id = id;
                Ok(project)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO project (name, created, alert, alert_chat, max_parallel_tasks, type, default_secret_storage_id) VALUES (?, ?, ?, ?, ?, ?, ?)";
                sqlx::query(query)
                    .bind(&project.name)
                    .bind(project.created)
                    .bind(project.alert)
                    .bind(&project.alert_chat)
                    .bind(project.max_parallel_tasks)
                    .bind(&project.r#type)
                    .bind(&project.default_secret_storage_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                // Для MySQL нужно получить последний вставленный ID отдельно
                let last_id: i32 = sqlx::query_scalar("SELECT LAST_INSERT_ID()")
                    .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;

                project.id = last_id;
                Ok(project)
            }
        }
    }

    async fn update_project(&self, project: Project) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE project SET name = ?, alert = ?, alert_chat = ?, max_parallel_tasks = ?, type = ?, default_secret_storage_id = ? WHERE id = ?";
                sqlx::query(query)
                    .bind(&project.name)
                    .bind(project.alert)
                    .bind(&project.alert_chat)
                    .bind(project.max_parallel_tasks)
                    .bind(&project.r#type)
                    .bind(&project.default_secret_storage_id)
                    .bind(project.id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE project SET name = $1, alert = $2, alert_chat = $3, max_parallel_tasks = $4, type = $5, default_secret_storage_id = $6 WHERE id = $7";
                sqlx::query(query)
                    .bind(&project.name)
                    .bind(project.alert)
                    .bind(&project.alert_chat)
                    .bind(project.max_parallel_tasks)
                    .bind(&project.r#type)
                    .bind(&project.default_secret_storage_id)
                    .bind(project.id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE project SET name = ?, alert = ?, alert_chat = ?, max_parallel_tasks = ?, type = ?, default_secret_storage_id = ? WHERE id = ?";
                sqlx::query(query)
                    .bind(&project.name)
                    .bind(project.alert)
                    .bind(&project.alert_chat)
                    .bind(project.max_parallel_tasks)
                    .bind(&project.r#type)
                    .bind(&project.default_secret_storage_id)
                    .bind(project.id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_project(&self, project_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "DELETE FROM project WHERE id = ?";
                sqlx::query(query)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "DELETE FROM project WHERE id = $1";
                sqlx::query(query)
                    .bind(project_id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "DELETE FROM project WHERE id = ?";
                sqlx::query(query)
                    .bind(project_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn create_project_user(&self, project_user: ProjectUser) -> Result<()> {
        let role_str = project_user.role.to_string();
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query("INSERT INTO project__user (project_id, user_id, role, created) VALUES (?, ?, ?, ?)")
                    .bind(project_user.project_id)
                    .bind(project_user.user_id)
                    .bind(&role_str)
                    .bind(project_user.created)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query("INSERT INTO project__user (project_id, user_id, role, created) VALUES ($1, $2, $3, $4)")
                    .bind(project_user.project_id)
                    .bind(project_user.user_id)
                    .bind(&role_str)
                    .bind(project_user.created)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                sqlx::query("INSERT INTO project__user (project_id, user_id, `role`, created) VALUES (?, ?, ?, ?)")
                    .bind(project_user.project_id)
                    .bind(project_user.user_id)
                    .bind(&role_str)
                    .bind(project_user.created)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

// ============================================================================
// TemplateManager - CRUD операции для шаблонов
// ============================================================================
