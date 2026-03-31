//! Schedule CRUD Operations
//!
//! Операции с расписаниями в SQL

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::{Schedule, ScheduleWithTpl};
use sqlx::Row;

impl SqlDb {
    fn pg_pool_schedule(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает расписания проекта
    pub async fn get_schedules(&self, project_id: i32) -> Result<Vec<Schedule>> {
        let rows = sqlx::query("SELECT * FROM schedule WHERE project_id = $1 ORDER BY name")
            .bind(project_id)
            .fetch_all(self.pg_pool_schedule()?)
            .await
            .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| Schedule {
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
                run_at: row.try_get("run_at").ok().flatten(),
                delete_after_run: row
                    .try_get::<bool, _>("delete_after_run")
                    .ok()
                    .unwrap_or(false),
            })
            .collect())
    }

    /// Получает все активные расписания (без фильтра по проекту)
    pub async fn get_all_schedules(&self) -> Result<Vec<Schedule>> {
        let rows = sqlx::query("SELECT * FROM schedule ORDER BY id")
            .fetch_all(self.pg_pool_schedule()?)
            .await
            .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| Schedule {
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
                run_at: row.try_get("run_at").ok().flatten(),
                delete_after_run: row
                    .try_get::<bool, _>("delete_after_run")
                    .ok()
                    .unwrap_or(false),
            })
            .collect())
    }

    /// Получает расписание по ID
    pub async fn get_schedule(&self, project_id: i32, schedule_id: i32) -> Result<Schedule> {
        let row = sqlx::query("SELECT * FROM schedule WHERE id = $1 AND project_id = $2")
            .bind(schedule_id)
            .bind(project_id)
            .fetch_one(self.pg_pool_schedule()?)
            .await
            .map_err(|e| match e {
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
            run_at: row.try_get("run_at").ok().flatten(),
            delete_after_run: row
                .try_get::<bool, _>("delete_after_run")
                .ok()
                .unwrap_or(false),
        })
    }

    /// Создаёт расписание
    pub async fn create_schedule(&self, mut schedule: Schedule) -> Result<Schedule> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO schedule (project_id, template_id, cron, cron_format, name, active, \
             created, run_at, delete_after_run) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id",
        )
        .bind(schedule.project_id)
        .bind(schedule.template_id)
        .bind(&schedule.cron)
        .bind(&schedule.cron_format)
        .bind(&schedule.name)
        .bind(schedule.active)
        .bind(&schedule.created)
        .bind(&schedule.run_at)
        .bind(schedule.delete_after_run)
        .fetch_one(self.pg_pool_schedule()?)
        .await
        .map_err(Error::Database)?;

        schedule.id = id;
        Ok(schedule)
    }

    /// Обновляет расписание
    pub async fn update_schedule(&self, schedule: Schedule) -> Result<()> {
        sqlx::query(
            "UPDATE schedule SET cron = $1, cron_format = $2, name = $3, active = $4, \
             run_at = $5, delete_after_run = $6 WHERE id = $7 AND project_id = $8",
        )
        .bind(&schedule.cron)
        .bind(&schedule.cron_format)
        .bind(&schedule.name)
        .bind(schedule.active)
        .bind(&schedule.run_at)
        .bind(schedule.delete_after_run)
        .bind(schedule.id)
        .bind(schedule.project_id)
        .execute(self.pg_pool_schedule()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Удаляет расписание
    pub async fn delete_schedule(&self, project_id: i32, schedule_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM schedule WHERE id = $1 AND project_id = $2")
            .bind(schedule_id)
            .bind(project_id)
            .execute(self.pg_pool_schedule()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}
