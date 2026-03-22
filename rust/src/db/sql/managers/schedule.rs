//! ScheduleManager - управление расписанием
//!
//! Реализация трейта ScheduleManager для SqlStore

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::models::{Schedule, ScheduleWithTpl};
use crate::error::{Error, Result};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl ScheduleManager for SqlStore {
    async fn get_schedules(&self, project_id: i32) -> Result<Vec<Schedule>> {
        let rows = sqlx::query(
            "SELECT id, project_id, template_id, cron, cron_format, name, active, \
             last_commit_hash, repository_id, created::text AS created, run_at, delete_after_run \
             FROM schedule WHERE project_id = $1 ORDER BY name",
        )
        .bind(project_id)
        .fetch_all(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;

        Ok(rows.into_iter().map(row_to_schedule).collect())
    }

    async fn get_schedule(&self, _project_id: i32, schedule_id: i32) -> Result<Schedule> {
        let row = sqlx::query(
            "SELECT id, project_id, template_id, cron, cron_format, name, active, \
             last_commit_hash, repository_id, created::text AS created, run_at, delete_after_run \
             FROM schedule WHERE id = $1",
        )
        .bind(schedule_id)
        .fetch_one(self.get_postgres_pool()?)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound("Расписание не найдено".to_string()),
            _ => Error::Database(e),
        })?;

        Ok(row_to_schedule(row))
    }

    async fn create_schedule(&self, mut schedule: Schedule) -> Result<Schedule> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO schedule (project_id, template_id, cron, cron_format, name, active, run_at, delete_after_run) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
        )
        .bind(schedule.project_id)
        .bind(schedule.template_id)
        .bind(&schedule.cron)
        .bind(&schedule.cron_format)
        .bind(&schedule.name)
        .bind(schedule.active)
        .bind(&schedule.run_at)
        .bind(schedule.delete_after_run)
        .fetch_one(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;

        schedule.id = id;
        Ok(schedule)
    }

    async fn update_schedule(&self, schedule: Schedule) -> Result<()> {
        sqlx::query(
            "UPDATE schedule SET cron = $1, cron_format = $2, name = $3, active = $4, \
             run_at = $5, delete_after_run = $6 WHERE id = $7",
        )
        .bind(&schedule.cron)
        .bind(&schedule.cron_format)
        .bind(&schedule.name)
        .bind(schedule.active)
        .bind(&schedule.run_at)
        .bind(schedule.delete_after_run)
        .bind(schedule.id)
        .execute(self.get_postgres_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_schedule(&self, _project_id: i32, schedule_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM schedule WHERE id = $1")
            .bind(schedule_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn set_schedule_active(&self, _project_id: i32, schedule_id: i32, active: bool) -> Result<()> {
        sqlx::query("UPDATE schedule SET active = $1 WHERE id = $2")
            .bind(active)
            .bind(schedule_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn set_schedule_commit_hash(&self, _project_id: i32, schedule_id: i32, hash: &str) -> Result<()> {
        sqlx::query("UPDATE schedule SET last_commit_hash = $1 WHERE id = $2")
            .bind(hash)
            .bind(schedule_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn get_all_schedules(&self) -> Result<Vec<Schedule>> {
        self.db.get_all_schedules().await
    }
}

fn row_to_schedule(row: sqlx::postgres::PgRow) -> Schedule {
    Schedule {
        id:               row.get("id"),
        project_id:       row.get("project_id"),
        template_id:      row.get("template_id"),
        cron:             row.get("cron"),
        cron_format:      row.try_get("cron_format").ok().flatten(),
        name:             row.get("name"),
        active:           row.get("active"),
        last_commit_hash: row.try_get("last_commit_hash").ok().flatten(),
        repository_id:    row.try_get("repository_id").ok(),
        created:          row.try_get("created").ok().flatten(),
        run_at:           row.try_get("run_at").ok().flatten(),
        delete_after_run: row.get("delete_after_run"),
    }
}

// Helper to expose the ScheduleWithTpl type — used in the ScheduleManager trait.
#[allow(dead_code)]
fn _tpl_hint(_: ScheduleWithTpl) {}
