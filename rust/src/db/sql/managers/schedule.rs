//! ScheduleManager - управление расписанием
//!
//! Реализация трейта ScheduleManager для SqlStore

use crate::db::sql::SqlStore;
use crate::db::sql::types::SqlDialect;
use crate::db::store::*;
use crate::models::{Schedule, ScheduleWithTpl};
use crate::error::{Error, Result};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl ScheduleManager for SqlStore {
    async fn get_schedules(&self, project_id: i32) -> Result<Vec<Schedule>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM schedule WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query).bind(project_id).fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| Schedule {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    template_id: row.get("template_id"),
                    cron: row.get("cron"),
                    cron_format: row.try_get("cron_format").ok().flatten(),
                    name: row.get("name"),
                    active: row.get("active"),
                    last_commit_hash: row.try_get("last_commit_hash").ok().flatten(),
                    repository_id: row.try_get("repository_id").ok(),
                    created: row.try_get("created").ok(),
                }).collect())
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM schedule WHERE project_id = $1 ORDER BY name";
                let rows = sqlx::query(query).bind(project_id).fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| Schedule {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    template_id: row.get("template_id"),
                    cron: row.get("cron"),
                    cron_format: row.try_get("cron_format").ok().flatten(),
                    name: row.get("name"),
                    active: row.get("active"),
                    last_commit_hash: row.try_get("last_commit_hash").ok().flatten(),
                    repository_id: row.try_get("repository_id").ok(),
                    created: row.try_get("created").ok(),
                }).collect())
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `schedule` WHERE project_id = ? ORDER BY name";
                let rows = sqlx::query(query).bind(project_id).fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                Ok(rows.into_iter().map(|row| Schedule {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    template_id: row.get("template_id"),
                    cron: row.get("cron"),
                    cron_format: row.try_get("cron_format").ok().flatten(),
                    name: row.get("name"),
                    active: row.get("active"),
                    last_commit_hash: row.try_get("last_commit_hash").ok().flatten(),
                    repository_id: row.try_get("repository_id").ok(),
                    created: row.try_get("created").ok(),
                }).collect())
            }
        }
    }

    async fn get_schedule(&self, _project_id: i32, schedule_id: i32) -> Result<Schedule> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM schedule WHERE id = ?";
                let row = sqlx::query(query).bind(schedule_id).fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?).await.map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Расписание не найдено".to_string()),
                    _ => Error::Database(e),
                })?;
                Ok(Schedule {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    template_id: row.get("template_id"),
                    cron: row.get("cron"),
                    cron_format: row.try_get("cron_format").ok().flatten(),
                    name: row.get("name"),
                    active: row.get("active"),
                    last_commit_hash: row.try_get("last_commit_hash").ok().flatten(),
                    repository_id: row.try_get("repository_id").ok(),
                    created: row.try_get("created").ok(),
                })
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM schedule WHERE id = $1";
                let row = sqlx::query(query).bind(schedule_id).fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?).await.map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Расписание не найдено".to_string()),
                    _ => Error::Database(e),
                })?;
                Ok(Schedule {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    template_id: row.get("template_id"),
                    cron: row.get("cron"),
                    cron_format: row.try_get("cron_format").ok().flatten(),
                    name: row.get("name"),
                    active: row.get("active"),
                    last_commit_hash: row.try_get("last_commit_hash").ok().flatten(),
                    repository_id: row.try_get("repository_id").ok(),
                    created: row.try_get("created").ok(),
                })
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM `schedule` WHERE id = ?";
                let row = sqlx::query(query).bind(schedule_id).fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?).await.map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Расписание не найдено".to_string()),
                    _ => Error::Database(e),
                })?;
                Ok(Schedule {
                    id: row.get("id"),
                    project_id: row.get("project_id"),
                    template_id: row.get("template_id"),
                    cron: row.get("cron"),
                    cron_format: row.try_get("cron_format").ok().flatten(),
                    name: row.get("name"),
                    active: row.get("active"),
                    last_commit_hash: row.try_get("last_commit_hash").ok().flatten(),
                    repository_id: row.try_get("repository_id").ok(),
                    created: row.try_get("created").ok(),
                })
            }
        }
    }

    async fn create_schedule(&self, mut schedule: Schedule) -> Result<Schedule> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "INSERT INTO schedule (project_id, template_id, cron, name, active, created) VALUES (?, ?, ?, ?, ?, ?) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(schedule.project_id)
                    .bind(schedule.template_id)
                    .bind(&schedule.cron)
                    .bind(&schedule.name)
                    .bind(schedule.active)
                    .bind(&schedule.created)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                schedule.id = id;
                Ok(schedule)
            }
            SqlDialect::PostgreSQL => {
                let query = "INSERT INTO schedule (project_id, template_id, cron, name, active, created) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id";
                let id: i32 = sqlx::query_scalar(query)
                    .bind(schedule.project_id)
                    .bind(schedule.template_id)
                    .bind(&schedule.cron)
                    .bind(&schedule.name)
                    .bind(schedule.active)
                    .bind(&schedule.created)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                schedule.id = id;
                Ok(schedule)
            }
            SqlDialect::MySQL => {
                let query = "INSERT INTO `schedule` (project_id, template_id, cron, name, active, created) VALUES (?, ?, ?, ?, ?, ?)";
                let result = sqlx::query(query)
                    .bind(schedule.project_id)
                    .bind(schedule.template_id)
                    .bind(&schedule.cron)
                    .bind(&schedule.name)
                    .bind(schedule.active)
                    .bind(&schedule.created)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
                schedule.id = result.last_insert_id() as i32;
                Ok(schedule)
            }
        }
    }

    async fn update_schedule(&self, schedule: Schedule) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE schedule SET cron = ?, name = ?, active = ? WHERE id = ?";
                sqlx::query(query)
                    .bind(&schedule.cron)
                    .bind(&schedule.name)
                    .bind(schedule.active)
                    .bind(schedule.id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE schedule SET cron = $1, name = $2, active = $3 WHERE id = $4";
                sqlx::query(query)
                    .bind(&schedule.cron)
                    .bind(&schedule.name)
                    .bind(schedule.active)
                    .bind(schedule.id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `schedule` SET cron = ?, name = ?, active = ? WHERE id = ?";
                sqlx::query(query)
                    .bind(&schedule.cron)
                    .bind(&schedule.name)
                    .bind(schedule.active)
                    .bind(schedule.id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn delete_schedule(&self, _project_id: i32, schedule_id: i32) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "DELETE FROM schedule WHERE id = ?";
                sqlx::query(query).bind(schedule_id).execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "DELETE FROM schedule WHERE id = $1";
                sqlx::query(query).bind(schedule_id).execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "DELETE FROM `schedule` WHERE id = ?";
                sqlx::query(query).bind(schedule_id).execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn set_schedule_active(&self, _project_id: i32, schedule_id: i32, active: bool) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "UPDATE schedule SET active = ? WHERE id = ?";
                sqlx::query(query).bind(active).bind(schedule_id).execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                let query = "UPDATE schedule SET active = $1 WHERE id = $2";
                sqlx::query(query).bind(active).bind(schedule_id).execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                let query = "UPDATE `schedule` SET active = ? WHERE id = ?";
                sqlx::query(query).bind(active).bind(schedule_id).execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?).await.map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }

    async fn set_schedule_commit_hash(&self, _project_id: i32, schedule_id: i32, hash: &str) -> Result<()> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                sqlx::query("UPDATE schedule SET last_commit_hash = ? WHERE id = ?")
                    .bind(hash)
                    .bind(schedule_id)
                    .execute(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::PostgreSQL => {
                sqlx::query("UPDATE schedule SET last_commit_hash = $1 WHERE id = $2")
                    .bind(hash)
                    .bind(schedule_id)
                    .execute(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
            SqlDialect::MySQL => {
                sqlx::query("UPDATE `schedule` SET last_commit_hash = ? WHERE id = ?")
                    .bind(hash)
                    .bind(schedule_id)
                    .execute(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
            }
        }
        Ok(())
    }
}

