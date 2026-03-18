//! ProjectManager - управление проектами

use crate::db::sql::SqlStore;
use crate::db::sql::types::SqlDialect;
use crate::db::store::*;
use crate::models::{Project, ProjectUser, Role};
use crate::error::{Error, Result};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl ProjectStore for SqlStore {
    async fn get_projects(&self, user_id: Option<i32>) -> Result<Vec<Project>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let (query, bind_user_id) = if let Some(uid) = user_id {
                    ("SELECT p.* FROM project p JOIN project_user pu ON p.id = pu.project_id WHERE pu.user_id = ?", Some(uid))
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
                    .map_err(Error::Database)?;

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
                    ("SELECT p.* FROM project p JOIN project_user pu ON p.id = pu.project_id WHERE pu.user_id = $1", Some(uid))
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
                    .map_err(Error::Database)?;

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
                    ("SELECT p.* FROM project p JOIN project_user pu ON p.id = pu.project_id WHERE pu.user_id = ?", Some(uid))
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
                    .map_err(Error::Database)?;

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
                    .bind(project.default_secret_storage_id)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;

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
                    .bind(project.default_secret_storage_id)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;

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
                    .bind(project.default_secret_storage_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;

                // Для MySQL нужно получить последний вставленный ID отдельно
                let last_id: i32 = sqlx::query_scalar("SELECT LAST_INSERT_ID()")
                    .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;

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
                    .bind(project.default_secret_storage_id)
                    .bind(project.id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE project SET name = $1, alert = $2, alert_chat = $3, max_parallel_tasks = $4, type = $5, default_secret_storage_id = $6 WHERE id = $7";
                sqlx::query(query)
                    .bind(&project.name)
                    .bind(project.alert)
                    .bind(&project.alert_chat)
                    .bind(project.max_parallel_tasks)
                    .bind(&project.r#type)
                    .bind(project.default_secret_storage_id)
                    .bind(project.id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE project SET name = ?, alert = ?, alert_chat = ?, max_parallel_tasks = ?, type = ?, default_secret_storage_id = ? WHERE id = ?";
                sqlx::query(query)
                    .bind(&project.name)
                    .bind(project.alert)
                    .bind(&project.alert_chat)
                    .bind(project.max_parallel_tasks)
                    .bind(&project.r#type)
                    .bind(project.default_secret_storage_id)
                    .bind(project.id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
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
                    .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
                let query = "DELETE FROM project WHERE id = $1";
                sqlx::query(query)
                    .bind(project_id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                let query = "DELETE FROM project WHERE id = ?";
                sqlx::query(query)
                    .bind(project_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
        }
        Ok(())
    }

    async fn create_project_user(&self, project_user: ProjectUser) -> Result<()> {
        let role_str = project_user.role.to_string();
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query("INSERT INTO project_user (project_id, user_id, role, created) VALUES (?, ?, ?, ?)")
                    .bind(project_user.project_id)
                    .bind(project_user.user_id)
                    .bind(&role_str)
                    .bind(project_user.created)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query("INSERT INTO project_user (project_id, user_id, role, created) VALUES ($1, $2, $3, $4)")
                    .bind(project_user.project_id)
                    .bind(project_user.user_id)
                    .bind(&role_str)
                    .bind(project_user.created)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                sqlx::query("INSERT INTO project_user (project_id, user_id, `role`, created) VALUES (?, ?, ?, ?)")
                    .bind(project_user.project_id)
                    .bind(project_user.user_id)
                    .bind(&role_str)
                    .bind(project_user.created)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
        }
        Ok(())
    }

    async fn delete_project_user(&self, project_id: i32, user_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query("DELETE FROM project_user WHERE project_id = ? AND user_id = ?")
                    .bind(project_id)
                    .bind(user_id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query("DELETE FROM project_user WHERE project_id = $1 AND user_id = $2")
                    .bind(project_id)
                    .bind(user_id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                sqlx::query("DELETE FROM project_user WHERE project_id = ? AND user_id = ?")
                    .bind(project_id)
                    .bind(user_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
        }
        Ok(())
    }
}

// ============================================================================
// ProjectRoleManager - CRUD для кастомных ролей проекта
// ============================================================================

#[async_trait]
impl ProjectRoleManager for SqlStore {
    async fn get_project_roles(&self, project_id: i32) -> Result<Vec<Role>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let roles = sqlx::query_as::<_, Role>(
                    "SELECT id, project_id, slug, name, description, permissions FROM project_role WHERE project_id = ? ORDER BY id"
                )
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(roles)
            }
            SqlDialect::PostgreSQL => {
                let roles = sqlx::query_as::<_, Role>(
                    "SELECT id, project_id, slug, name, description, permissions FROM project_role WHERE project_id = $1 ORDER BY id"
                )
                .bind(project_id)
                .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(roles)
            }
            SqlDialect::MySQL => {
                let roles = sqlx::query_as::<_, Role>(
                    "SELECT id, project_id, slug, name, description, permissions FROM project_role WHERE project_id = ? ORDER BY id"
                )
                .bind(project_id)
                .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
                Ok(roles)
            }
        }
    }

    async fn create_project_role(&self, role: Role) -> Result<Role> {
        let permissions = role.permissions.unwrap_or(0);
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let pool = self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?;
                let result = sqlx::query(
                    "INSERT INTO project_role (project_id, slug, name, description, permissions) VALUES (?, ?, ?, ?, ?)"
                )
                .bind(role.project_id)
                .bind(&role.slug)
                .bind(&role.name)
                .bind(&role.description)
                .bind(permissions)
                .execute(pool)
                .await
                .map_err(Error::Database)?;
                let id = result.last_insert_rowid() as i32;
                Ok(Role { id, ..role })
            }
            SqlDialect::PostgreSQL => {
                let pool = self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?;
                let row = sqlx::query_as::<_, Role>(
                    "INSERT INTO project_role (project_id, slug, name, description, permissions) VALUES ($1, $2, $3, $4, $5) RETURNING id, project_id, slug, name, description, permissions"
                )
                .bind(role.project_id)
                .bind(&role.slug)
                .bind(&role.name)
                .bind(&role.description)
                .bind(permissions)
                .fetch_one(pool)
                .await
                .map_err(Error::Database)?;
                Ok(row)
            }
            SqlDialect::MySQL => {
                let pool = self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?;
                let result = sqlx::query(
                    "INSERT INTO project_role (project_id, slug, name, description, permissions) VALUES (?, ?, ?, ?, ?)"
                )
                .bind(role.project_id)
                .bind(&role.slug)
                .bind(&role.name)
                .bind(&role.description)
                .bind(permissions)
                .execute(pool)
                .await
                .map_err(Error::Database)?;
                let id = result.last_insert_id() as i32;
                Ok(Role { id, ..role })
            }
        }
    }

    async fn update_project_role(&self, role: Role) -> Result<()> {
        let permissions = role.permissions.unwrap_or(0);
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE project_role SET slug = ?, name = ?, description = ?, permissions = ? WHERE id = ? AND project_id = ?"
                )
                .bind(&role.slug)
                .bind(&role.name)
                .bind(&role.description)
                .bind(permissions)
                .bind(role.id)
                .bind(role.project_id)
                .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query(
                    "UPDATE project_role SET slug = $1, name = $2, description = $3, permissions = $4 WHERE id = $5 AND project_id = $6"
                )
                .bind(&role.slug)
                .bind(&role.name)
                .bind(&role.description)
                .bind(permissions)
                .bind(role.id)
                .bind(role.project_id)
                .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                sqlx::query(
                    "UPDATE project_role SET slug = ?, name = ?, description = ?, permissions = ? WHERE id = ? AND project_id = ?"
                )
                .bind(&role.slug)
                .bind(&role.name)
                .bind(&role.description)
                .bind(permissions)
                .bind(role.id)
                .bind(role.project_id)
                .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                .await
                .map_err(Error::Database)?;
            }
        }
        Ok(())
    }

    async fn delete_project_role(&self, project_id: i32, role_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query("DELETE FROM project_role WHERE id = ? AND project_id = ?")
                    .bind(role_id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query("DELETE FROM project_role WHERE id = $1 AND project_id = $2")
                    .bind(role_id)
                    .bind(project_id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
            SqlDialect::MySQL => {
                sqlx::query("DELETE FROM project_role WHERE id = ? AND project_id = ?")
                    .bind(role_id)
                    .bind(project_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
            }
        }
        Ok(())
    }
}

// ============================================================================
// TemplateManager - CRUD операции для шаблонов
// ============================================================================
