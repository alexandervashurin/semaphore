//! Schedule CRUD Operations
//!
//! Операции с расписаниями в SQL

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::{Schedule, ScheduleWithTpl};
use chrono::Utc;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_struct_fields() {
        let schedule = Schedule {
            id: 1,
            template_id: 10,
            project_id: 5,
            cron: "0 * * * *".to_string(),
            cron_format: Some("standard".to_string()),
            name: "Hourly Build".to_string(),
            active: true,
            last_commit_hash: Some("abc123".to_string()),
            repository_id: Some(3),
            created: Some("2024-01-01T00:00:00Z".to_string()),
            run_at: None,
            delete_after_run: false,
        };
        assert_eq!(schedule.id, 1);
        assert_eq!(schedule.cron, "0 * * * *");
        assert!(schedule.active);
        assert!(!schedule.delete_after_run);
    }

    #[test]
    fn test_schedule_clone() {
        let schedule = Schedule {
            id: 1,
            template_id: 10,
            project_id: 5,
            cron: "*/5 * * * *".to_string(),
            cron_format: None,
            name: "Frequent".to_string(),
            active: false,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        let cloned = schedule.clone();
        assert_eq!(cloned.id, schedule.id);
        assert_eq!(cloned.name, schedule.name);
        assert_eq!(cloned.active, schedule.active);
    }

    #[test]
    fn test_schedule_serialization() {
        let schedule = Schedule {
            id: 1,
            template_id: 10,
            project_id: 5,
            cron: "0 0 * * *".to_string(),
            cron_format: Some("daily".to_string()),
            name: "Daily Build".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        let json = serde_json::to_string(&schedule).unwrap();
        assert!(json.contains("\"name\":\"Daily Build\""));
        assert!(json.contains("\"cron\":\"0 0 * * *\""));
        assert!(json.contains("\"active\":true"));
    }

    #[test]
    fn test_schedule_deserialization() {
        let json = r#"{"id":5,"template_id":20,"project_id":10,"cron":"*/10 * * * *","cron_format":"frequent","name":"Every 10 min","active":true,"last_commit_hash":null,"repository_id":null,"created":null,"delete_after_run":false}"#;
        let schedule: Schedule = serde_json::from_str(json).unwrap();
        assert_eq!(schedule.id, 5);
        assert_eq!(schedule.name, "Every 10 min");
        assert!(schedule.active);
    }

    #[test]
    fn test_schedule_with_tpl_struct() {
        let schedule = Schedule {
            id: 1,
            template_id: 10,
            project_id: 5,
            cron: "0 * * * *".to_string(),
            cron_format: None,
            name: "Test".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        let with_tpl = ScheduleWithTpl {
            schedule,
            tpl_playbook: Some("deploy.yml".to_string()),
        };
        assert_eq!(with_tpl.tpl_playbook, Some("deploy.yml".to_string()));
        assert_eq!(with_tpl.schedule.name, "Test");
    }

    #[test]
    fn test_schedule_with_tpl_clone() {
        let schedule = Schedule {
            id: 0,
            template_id: 0,
            project_id: 0,
            cron: "".to_string(),
            cron_format: None,
            name: "".to_string(),
            active: false,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        let with_tpl = ScheduleWithTpl {
            schedule,
            tpl_playbook: None,
        };
        let cloned = with_tpl.clone();
        assert_eq!(cloned.schedule.id, with_tpl.schedule.id);
        assert_eq!(cloned.tpl_playbook, with_tpl.tpl_playbook);
    }

    #[test]
    fn test_schedule_run_at_field() {
        let schedule = Schedule {
            id: 1,
            template_id: 10,
            project_id: 5,
            cron: String::new(),
            cron_format: Some("run_at".to_string()),
            name: "One-time".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: Some("2024-12-31T23:59:59Z".to_string()),
            delete_after_run: true,
        };
        let json = serde_json::to_string(&schedule).unwrap();
        assert!(json.contains("\"run_at\":\"2024-12-31T23:59:59Z\""));
        assert!(json.contains("\"delete_after_run\":true"));
    }

    #[test]
    fn test_schedule_default_values() {
        let schedule = Schedule {
            id: 0,
            template_id: 0,
            project_id: 0,
            cron: "".to_string(),
            cron_format: None,
            name: "".to_string(),
            active: false,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        assert_eq!(schedule.id, 0);
        assert_eq!(schedule.template_id, 0);
        assert_eq!(schedule.project_id, 0);
        assert_eq!(schedule.cron, "");
        assert!(schedule.cron_format.is_none());
        assert_eq!(schedule.name, "");
        assert!(!schedule.active);
        assert!(schedule.last_commit_hash.is_none());
        assert!(schedule.repository_id.is_none());
        assert!(schedule.created.is_none());
        assert!(schedule.run_at.is_none());
        assert!(!schedule.delete_after_run);
    }

    #[test]
    fn test_schedule_serialization_skip_nulls() {
        let schedule = Schedule {
            id: 1,
            template_id: 10,
            project_id: 5,
            cron: "0 * * * *".to_string(),
            cron_format: None,
            name: "Test".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        let json = serde_json::to_string(&schedule).unwrap();
        // run_at uses skip_serializing_if
        assert!(!json.contains("\"run_at\":"));
    }

    #[test]
    fn test_schedule_all_fields_set() {
        let schedule = Schedule {
            id: 100,
            template_id: 200,
            project_id: 300,
            cron: "0 12 * * 1".to_string(),
            cron_format: Some("weekly".to_string()),
            name: "Weekly Monday Noon".to_string(),
            active: true,
            last_commit_hash: Some("def456".to_string()),
            repository_id: Some(7),
            created: Some("2024-06-01T00:00:00Z".to_string()),
            run_at: Some("2024-06-03T12:00:00Z".to_string()),
            delete_after_run: false,
        };
        assert_eq!(schedule.id, 100);
        assert_eq!(schedule.template_id, 200);
        assert_eq!(schedule.project_id, 300);
        assert_eq!(schedule.cron, "0 12 * * 1");
        assert_eq!(schedule.repository_id, Some(7));
    }
}
