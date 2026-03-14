//! TaskManager - управление задачами
//!
//! Реализация трейта TaskManager для SqlStore

use crate::db::sql::SqlStore;
use crate::db::sql::types::SqlDialect;
use crate::db::store::*;
use crate::models::{Task, TaskWithTpl, TaskOutput};
use crate::services::task_logger::TaskStatus;
use crate::error::{Error, Result};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl TaskManager for SqlStore {
    async fn get_tasks(&self, project_id: i32, template_id: Option<i32>) -> Result<Vec<TaskWithTpl>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = if template_id.is_some() {
                    "SELECT t.*, tpl.playbook as tpl_playbook, tpl.type as tpl_type, tpl.app as tpl_app, u.name as user_name FROM task t LEFT JOIN template tpl ON t.template_id = tpl.id LEFT JOIN \"user\" u ON t.user_id = u.id WHERE t.project_id = ? AND t.template_id = ? ORDER BY t.created DESC"
                } else {
                    "SELECT t.*, tpl.playbook as tpl_playbook, tpl.type as tpl_type, tpl.app as tpl_app, u.name as user_name FROM task t LEFT JOIN template tpl ON t.template_id = tpl.id LEFT JOIN \"user\" u ON t.user_id = u.id WHERE t.project_id = ? ORDER BY t.created DESC"
                };
                let mut q = sqlx::query(query).bind(project_id);
                if let Some(tid) = template_id {
                    q = q.bind(tid);
                }
                let rows = q.fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| TaskWithTpl {
                    task: Task {
                        id: row.get("id"),
                        template_id: row.get("template_id"),
                        project_id: row.get("project_id"),
                        status: row.get("status"),
                        playbook: row.try_get("playbook").ok().flatten(),
                        environment: row.try_get("environment").ok().flatten(),
                        secret: None,
                        arguments: row.try_get("arguments").ok().flatten(),
                        git_branch: row.try_get("git_branch").ok().flatten(),
                        user_id: row.try_get("user_id").ok(),
                        integration_id: row.try_get("integration_id").ok(),
                        schedule_id: row.try_get("schedule_id").ok(),
                        created: row.get("created"),
                        start: row.try_get("start_time").ok(),
                        end: row.try_get("end_time").ok(),
                        message: row.try_get("message").ok().flatten(),
                        commit_hash: row.try_get("commit_hash").ok().flatten(),
                        commit_message: row.try_get("commit_message").ok().flatten(),
                        build_task_id: row.try_get("build_task_id").ok(),
                        version: row.try_get("version").ok().flatten(),
                        inventory_id: row.try_get("inventory_id").ok(),
                        repository_id: row.try_get("repository_id").ok(),
                        environment_id: row.try_get("environment_id").ok(),
                        params: None,
                    },
                    tpl_playbook: row.get("tpl_playbook"),
                    tpl_type: row.try_get("tpl_type").ok(),
                    tpl_app: row.try_get("tpl_app").ok(),
                    user_name: row.try_get("user_name").ok(),
                    build_task: None,
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = if template_id.is_some() {
                    "SELECT t.*, tpl.playbook as tpl_playbook, tpl.type as tpl_type, tpl.app as tpl_app, u.name as user_name FROM task t LEFT JOIN template tpl ON t.template_id = tpl.id LEFT JOIN \"user\" u ON t.user_id = u.id WHERE t.project_id = $1 AND t.template_id = $2 ORDER BY t.created DESC"
                } else {
                    "SELECT t.*, tpl.playbook as tpl_playbook, tpl.type as tpl_type, tpl.app as tpl_app, u.name as user_name FROM task t LEFT JOIN template tpl ON t.template_id = tpl.id LEFT JOIN \"user\" u ON t.user_id = u.id WHERE t.project_id = $1 ORDER BY t.created DESC"
                };
                let mut q = sqlx::query(query).bind(project_id);
                if let Some(tid) = template_id {
                    q = q.bind(tid);
                }
                let rows = q.fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| TaskWithTpl {
                    task: Task {
                        id: row.get("id"),
                        template_id: row.get("template_id"),
                        project_id: row.get("project_id"),
                        status: row.get("status"),
                        playbook: row.try_get("playbook").ok().flatten(),
                        environment: row.try_get("environment").ok().flatten(),
                        secret: None,
                        arguments: row.try_get("arguments").ok().flatten(),
                        git_branch: row.try_get("git_branch").ok().flatten(),
                        user_id: row.try_get("user_id").ok(),
                        integration_id: row.try_get("integration_id").ok(),
                        schedule_id: row.try_get("schedule_id").ok(),
                        created: row.get("created"),
                        start: row.try_get("start_time").ok(),
                        end: row.try_get("end_time").ok(),
                        message: row.try_get("message").ok().flatten(),
                        commit_hash: row.try_get("commit_hash").ok().flatten(),
                        commit_message: row.try_get("commit_message").ok().flatten(),
                        build_task_id: row.try_get("build_task_id").ok(),
                        version: row.try_get("version").ok().flatten(),
                        inventory_id: row.try_get("inventory_id").ok(),
                        repository_id: row.try_get("repository_id").ok(),
                        environment_id: row.try_get("environment_id").ok(),
                        params: None,
                    },
                    tpl_playbook: row.get("tpl_playbook"),
                    tpl_type: row.try_get("tpl_type").ok(),
                    tpl_app: row.try_get("tpl_app").ok(),
                    user_name: row.try_get("user_name").ok(),
                    build_task: None,
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = if template_id.is_some() {
                    "SELECT t.*, tpl.playbook as tpl_playbook, tpl.type as tpl_type, tpl.app as tpl_app, u.name as user_name FROM task t LEFT JOIN `template` tpl ON t.template_id = tpl.id LEFT JOIN `user` u ON t.user_id = u.id WHERE t.project_id = ? AND t.template_id = ? ORDER BY t.created DESC"
                } else {
                    "SELECT t.*, tpl.playbook as tpl_playbook, tpl.type as tpl_type, tpl.app as tpl_app, u.name as user_name FROM task t LEFT JOIN `template` tpl ON t.template_id = tpl.id LEFT JOIN `user` u ON t.user_id = u.id WHERE t.project_id = ? ORDER BY t.created DESC"
                };
                let mut q = sqlx::query(query).bind(project_id);
                if let Some(tid) = template_id {
                    q = q.bind(tid);
                }
                let rows = q.fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| TaskWithTpl {
                    task: Task {
                        id: row.get("id"),
                        template_id: row.get("template_id"),
                        project_id: row.get("project_id"),
                        status: row.get("status"),
                        playbook: row.try_get("playbook").ok().flatten(),
                        environment: row.try_get("environment").ok().flatten(),
                        secret: None,
                        arguments: row.try_get("arguments").ok().flatten(),
                        git_branch: row.try_get("git_branch").ok().flatten(),
                        user_id: row.try_get("user_id").ok(),
                        integration_id: row.try_get("integration_id").ok(),
                        schedule_id: row.try_get("schedule_id").ok(),
                        created: row.get("created"),
                        start: row.try_get("start_time").ok(),
                        end: row.try_get("end_time").ok(),
                        message: row.try_get("message").ok().flatten(),
                        commit_hash: row.try_get("commit_hash").ok().flatten(),
                        commit_message: row.try_get("commit_message").ok().flatten(),
                        build_task_id: row.try_get("build_task_id").ok(),
                        version: row.try_get("version").ok().flatten(),
                        inventory_id: row.try_get("inventory_id").ok(),
                        repository_id: row.try_get("repository_id").ok(),
                        environment_id: row.try_get("environment_id").ok(),
                        params: None,
                    },
                    tpl_playbook: row.try_get("tpl_playbook").ok().flatten(),
                    tpl_type: row.try_get("tpl_type").ok(),
                    tpl_app: row.try_get("tpl_app").ok(),
                    user_name: row.try_get("user_name").ok(),
                    build_task: None,
                }).collect())
            }
        }
    }

    async fn get_task(&self, _project_id: i32, task_id: i32) -> Result<Task> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM task WHERE id = ?";
                let row = sqlx::query(query).bind(task_id).fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?).await.map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Задача не найдена".to_string()),
                    _ => Error::Database(e),
                })?;
                Ok(Task {
                    id: row.get("id"),
                    template_id: row.get("template_id"),
                    project_id: row.get("project_id"),
                    status: row.get("status"),
                    playbook: row.try_get("playbook").ok().flatten(),
                    environment: row.try_get("environment").ok().flatten(),
                    secret: None,
                    arguments: row.try_get("arguments").ok().flatten(),
                    git_branch: row.try_get("git_branch").ok().flatten(),
                    user_id: row.try_get("user_id").ok(),
                    integration_id: row.try_get("integration_id").ok(),
                    schedule_id: row.try_get("schedule_id").ok(),
                    created: row.get("created"),
                    start: row.try_get("start_time").ok(),
                    end: row.try_get("end_time").ok(),
                    message: row.try_get("message").ok().flatten(),
                    commit_hash: row.try_get("commit_hash").ok().flatten(),
                    commit_message: row.try_get("commit_message").ok().flatten(),
                    build_task_id: row.try_get("build_task_id").ok(),
                    version: row.try_get("version").ok().flatten(),
                    inventory_id: row.try_get("inventory_id").ok(),
                    repository_id: row.try_get("repository_id").ok(),
                    environment_id: row.try_get("environment_id").ok(),
                    params: None,
                })
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM task WHERE id = $1";
                let row = sqlx::query(query).bind(task_id).fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?).await.map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Задача не найдена".to_string()),
                    _ => Error::Database(e),
                })?;
                Ok(Task {
                    id: row.get("id"),
                    template_id: row.get("template_id"),
                    project_id: row.get("project_id"),
                    status: row.get("status"),
                    playbook: row.try_get("playbook").ok().flatten(),
                    environment: row.try_get("environment").ok().flatten(),
                    secret: None,
                    arguments: row.try_get("arguments").ok().flatten(),
                    git_branch: row.try_get("git_branch").ok().flatten(),
                    user_id: row.try_get("user_id").ok(),
                    integration_id: row.try_get("integration_id").ok(),
                    schedule_id: row.try_get("schedule_id").ok(),
                    created: row.get("created"),
                    start: row.try_get("start_time").ok(),
                    end: row.try_get("end_time").ok(),
                    message: row.try_get("message").ok().flatten(),
                    commit_hash: row.try_get("commit_hash").ok().flatten(),
                    commit_message: row.try_get("commit_message").ok().flatten(),
                    build_task_id: row.try_get("build_task_id").ok(),
                    version: row.try_get("version").ok().flatten(),
                    inventory_id: row.try_get("inventory_id").ok(),
                    repository_id: row.try_get("repository_id").ok(),
                    environment_id: row.try_get("environment_id").ok(),
                    params: None,
                })
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `task` WHERE id = ?";
                let row = sqlx::query(query).bind(task_id).fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?).await.map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Задача не найдена".to_string()),
                    _ => Error::Database(e),
                })?;
                Ok(Task {
                    id: row.get("id"),
                    template_id: row.get("template_id"),
                    project_id: row.get("project_id"),
                    status: row.get("status"),
                    playbook: row.try_get("playbook").ok().flatten(),
                    environment: row.try_get("environment").ok().flatten(),
                    secret: None,
                    arguments: row.try_get("arguments").ok().flatten(),
                    git_branch: row.try_get("git_branch").ok().flatten(),
                    user_id: row.try_get("user_id").ok(),
                    integration_id: row.try_get("integration_id").ok(),
                    schedule_id: row.try_get("schedule_id").ok(),
                    created: row.get("created"),
                    start: row.try_get("start_time").ok(),
                    end: row.try_get("end_time").ok(),
                    message: row.try_get("message").ok().flatten(),
                    commit_hash: row.try_get("commit_hash").ok().flatten(),
                    commit_message: row.try_get("commit_message").ok().flatten(),
                    build_task_id: row.try_get("build_task_id").ok(),
                    version: row.try_get("version").ok().flatten(),
                    inventory_id: row.try_get("inventory_id").ok(),
                    repository_id: row.try_get("repository_id").ok(),
                    environment_id: row.try_get("environment_id").ok(),
                    params: None,
                })
            }
        }
    }

    async fn create_task(&self, mut task: Task) -> Result<Task> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO task (template_id, project_id, status, playbook, environment, arguments, git_branch, user_id, integration_id, schedule_id, created, start_time, end_time, message, commit_hash, commit_message, build_task_id, version, inventory_id, repository_id, environment_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(task.template_id)
                    .bind(task.project_id)
                    .bind(&task.status.to_string())
                    .bind(&task.playbook)
                    .bind(&task.environment)
                    .bind(&task.arguments)
                    .bind(&task.git_branch)
                    .bind(&task.user_id)
                    .bind(&task.integration_id)
                    .bind(&task.schedule_id)
                    .bind(task.created)
                    .bind(&task.start)
                    .bind(&task.end)
                    .bind(&task.message)
                    .bind(&task.commit_hash)
                    .bind(&task.commit_message)
                    .bind(&task.build_task_id)
                    .bind(&task.version)
                    .bind(&task.inventory_id)
                    .bind(&task.repository_id)
                    .bind(&task.environment_id)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                task.id = id;
                Ok(task)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO task (template_id, project_id, status, playbook, environment, arguments, git_branch, user_id, integration_id, schedule_id, created, start_time, end_time, message, commit_hash, commit_message, build_task_id, version, inventory_id, repository_id, environment_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(task.template_id)
                    .bind(task.project_id)
                    .bind(&task.status.to_string())
                    .bind(&task.playbook)
                    .bind(&task.environment)
                    .bind(&task.arguments)
                    .bind(&task.git_branch)
                    .bind(&task.user_id)
                    .bind(&task.integration_id)
                    .bind(&task.schedule_id)
                    .bind(task.created)
                    .bind(&task.start)
                    .bind(&task.end)
                    .bind(&task.message)
                    .bind(&task.commit_hash)
                    .bind(&task.commit_message)
                    .bind(&task.build_task_id)
                    .bind(&task.version)
                    .bind(&task.inventory_id)
                    .bind(&task.repository_id)
                    .bind(&task.environment_id)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                task.id = id;
                Ok(task)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `task` (template_id, project_id, status, playbook, environment, arguments, git_branch, user_id, integration_id, schedule_id, created, start_time, end_time, message, commit_hash, commit_message, build_task_id, version, inventory_id, repository_id, environment_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
                let result = sqlx::query(query)
                    .bind(task.template_id)
                    .bind(task.project_id)
                    .bind(&task.status.to_string())
                    .bind(&task.playbook)
                    .bind(&task.environment)
                    .bind(&task.arguments)
                    .bind(&task.git_branch)
                    .bind(&task.user_id)
                    .bind(&task.integration_id)
                    .bind(&task.schedule_id)
                    .bind(task.created)
                    .bind(&task.start)
                    .bind(&task.end)
                    .bind(&task.message)
                    .bind(&task.commit_hash)
                    .bind(&task.commit_message)
                    .bind(&task.build_task_id)
                    .bind(&task.version)
                    .bind(&task.inventory_id)
                    .bind(&task.repository_id)
                    .bind(&task.environment_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                task.id = result.last_insert_id() as i32;
                Ok(task)
            }
        }
    }

    async fn update_task(&self, task: Task) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE task SET status = ?, playbook = ?, environment = ?, arguments = ?, git_branch = ?, user_id = ?, integration_id = ?, schedule_id = ?, start_time = ?, end_time = ?, message = ?, commit_hash = ?, commit_message = ?, build_task_id = ?, version = ?, inventory_id = ?, repository_id = ?, environment_id = ? WHERE id = ?";
                sqlx::query(query)
                    .bind(&task.status.to_string())
                    .bind(&task.playbook)
                    .bind(&task.environment)
                    .bind(&task.arguments)
                    .bind(&task.git_branch)
                    .bind(&task.user_id)
                    .bind(&task.integration_id)
                    .bind(&task.schedule_id)
                    .bind(&task.start)
                    .bind(&task.end)
                    .bind(&task.message)
                    .bind(&task.commit_hash)
                    .bind(&task.commit_message)
                    .bind(&task.build_task_id)
                    .bind(&task.version)
                    .bind(&task.inventory_id)
                    .bind(&task.repository_id)
                    .bind(&task.environment_id)
                    .bind(task.id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE task SET status = $1, playbook = $2, environment = $3, arguments = $4, git_branch = $5, user_id = $6, integration_id = $7, schedule_id = $8, start_time = $9, end_time = $10, message = $11, commit_hash = $12, commit_message = $13, build_task_id = $14, version = $15, inventory_id = $16, repository_id = $17, environment_id = $18 WHERE id = $19";
                sqlx::query(query)
                    .bind(&task.status.to_string())
                    .bind(&task.playbook)
                    .bind(&task.environment)
                    .bind(&task.arguments)
                    .bind(&task.git_branch)
                    .bind(&task.user_id)
                    .bind(&task.integration_id)
                    .bind(&task.schedule_id)
                    .bind(&task.start)
                    .bind(&task.end)
                    .bind(&task.message)
                    .bind(&task.commit_hash)
                    .bind(&task.commit_message)
                    .bind(&task.build_task_id)
                    .bind(&task.version)
                    .bind(&task.inventory_id)
                    .bind(&task.repository_id)
                    .bind(&task.environment_id)
                    .bind(task.id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `task` SET status = ?, playbook = ?, environment = ?, arguments = ?, git_branch = ?, user_id = ?, integration_id = ?, schedule_id = ?, start_time = ?, end_time = ?, message = ?, commit_hash = ?, commit_message = ?, build_task_id = ?, version = ?, inventory_id = ?, repository_id = ?, environment_id = ? WHERE id = ?";
                sqlx::query(query)
                    .bind(&task.status.to_string())
                    .bind(&task.playbook)
                    .bind(&task.environment)
                    .bind(&task.arguments)
                    .bind(&task.git_branch)
                    .bind(&task.user_id)
                    .bind(&task.integration_id)
                    .bind(&task.schedule_id)
                    .bind(&task.start)
                    .bind(&task.end)
                    .bind(&task.message)
                    .bind(&task.commit_hash)
                    .bind(&task.commit_message)
                    .bind(&task.build_task_id)
                    .bind(&task.version)
                    .bind(&task.inventory_id)
                    .bind(&task.repository_id)
                    .bind(&task.environment_id)
                    .bind(task.id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_task(&self, _project_id: i32, task_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "DELETE FROM task WHERE id = ?";
                sqlx::query(query).bind(task_id).execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "DELETE FROM task WHERE id = $1";
                sqlx::query(query).bind(task_id).execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "DELETE FROM `task` WHERE id = ?";
                sqlx::query(query).bind(task_id).execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn get_task_outputs(&self, task_id: i32) -> Result<Vec<TaskOutput>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM task_output WHERE task_id = ? ORDER BY time";
                let rows = sqlx::query(query).bind(task_id).fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| TaskOutput {
                    id: row.get("id"),
                    task_id: row.get("task_id"),
                    project_id: row.get("project_id"),
                    stage_id: row.try_get("stage_id").ok(),
                    time: row.get("time"),
                    output: row.get("output"),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM task_output WHERE task_id = $1 ORDER BY time";
                let rows = sqlx::query(query).bind(task_id).fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| TaskOutput {
                    id: row.get("id"),
                    task_id: row.get("task_id"),
                    project_id: row.get("project_id"),
                    stage_id: row.try_get("stage_id").ok(),
                    time: row.get("time"),
                    output: row.get("output"),
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `task_output` WHERE task_id = ? ORDER BY time";
                let rows = sqlx::query(query).bind(task_id).fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| TaskOutput {
                    id: row.get("id"),
                    task_id: row.get("task_id"),
                    project_id: row.get("project_id"),
                    stage_id: row.try_get("stage_id").ok(),
                    time: row.get("time"),
                    output: row.get("output"),
                }).collect())
            }
        }
    }

    async fn create_task_output(&self, mut output: TaskOutput) -> Result<TaskOutput> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO task_output (task_id, project_id, time, output) VALUES (?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(output.task_id)
                    .bind(output.project_id)
                    .bind(output.time)
                    .bind(&output.output)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                output.id = id;
                Ok(output)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO task_output (task_id, project_id, time, output) VALUES ($1, $2, $3, $4) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(output.task_id)
                    .bind(output.project_id)
                    .bind(output.time)
                    .bind(&output.output)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                output.id = id;
                Ok(output)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `task_output` (task_id, project_id, time, output) VALUES (?, ?, ?, ?)";
                let result = sqlx::query(query)
                    .bind(output.task_id)
                    .bind(output.project_id)
                    .bind(output.time)
                    .bind(&output.output)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                output.id = result.last_insert_id() as i32;
                Ok(output)
            }
        }
    }

    async fn update_task_status(&self, _project_id: i32, task_id: i32, status: TaskStatus) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE task SET status = ? WHERE id = ?";
                sqlx::query(query).bind(&status.to_string()).bind(task_id).execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE task SET status = $1 WHERE id = $2";
                sqlx::query(query).bind(&status.to_string()).bind(task_id).execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `task` SET status = ? WHERE id = ?";
                sqlx::query(query).bind(&status.to_string()).bind(task_id).execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

