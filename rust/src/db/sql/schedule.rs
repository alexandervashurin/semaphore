//! Schedule CRUD Operations
//!
//! Операции с расписаниями в SQL

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::{Schedule, ScheduleWithTpl};

impl SqlDb {
    /// Получает расписания проекта
    pub async fn get_schedules(&self, project_id: i32) -> Result<Vec<Schedule>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let schedules = sqlx::query_as::<_, Schedule>(
                    "SELECT * FROM schedule WHERE project_id = ? ORDER BY name"
                )
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(schedules)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Получает расписание по ID
    pub async fn get_schedule(&self, project_id: i32, schedule_id: i32) -> Result<Schedule> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let schedule = sqlx::query_as::<_, Schedule>(
                    "SELECT * FROM schedule WHERE id = ? AND project_id = ?"
                )
                .bind(schedule_id)
                .bind(project_id)
                .fetch_optional(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                schedule.ok_or(Error::NotFound("Schedule not found".to_string()))
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Создаёт расписание
    pub async fn create_schedule(&self, mut schedule: Schedule) -> Result<Schedule> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO schedule (project_id, template_id, name, cron, active)
                     VALUES (?, ?, ?, ?, ?)"
                )
                .bind(schedule.project_id)
                .bind(schedule.template_id)
                .bind(&schedule.name)
                .bind(&schedule.cron)
                .bind(schedule.active)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                schedule.id = result.last_insert_rowid() as i32;
                Ok(schedule)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Обновляет расписание
    pub async fn update_schedule(&self, schedule: Schedule) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE schedule SET template_id = ?, name = ?, cron = ?, active = ?
                     WHERE id = ? AND project_id = ?"
                )
                .bind(schedule.template_id)
                .bind(&schedule.name)
                .bind(&schedule.cron)
                .bind(schedule.active)
                .bind(schedule.id)
                .bind(schedule.project_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;

                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }

    /// Удаляет расписание
    pub async fn delete_schedule(&self, project_id: i32, schedule_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("DELETE FROM schedule WHERE id = ? AND project_id = ?")
                    .bind(schedule_id)
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
