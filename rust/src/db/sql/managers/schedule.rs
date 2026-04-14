//! ScheduleManager - управление расписанием
//!
//! Реализация трейта ScheduleManager для SqlStore

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::{Schedule, ScheduleWithTpl};
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

    async fn set_schedule_active(
        &self,
        _project_id: i32,
        schedule_id: i32,
        active: bool,
    ) -> Result<()> {
        sqlx::query("UPDATE schedule SET active = $1 WHERE id = $2")
            .bind(active)
            .bind(schedule_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn set_schedule_commit_hash(
        &self,
        _project_id: i32,
        schedule_id: i32,
        hash: &str,
    ) -> Result<()> {
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
        id: row.get("id"),
        project_id: row.get("project_id"),
        template_id: row.get("template_id"),
        cron: row.get("cron"),
        cron_format: row.try_get("cron_format").ok().flatten(),
        name: row.get("name"),
        active: row.get("active"),
        last_commit_hash: row.try_get("last_commit_hash").ok().flatten(),
        repository_id: row.try_get("repository_id").ok(),
        created: row.try_get("created").ok().flatten(),
        run_at: row.try_get("run_at").ok().flatten(),
        delete_after_run: row.get("delete_after_run"),
    }
}

// Helper to expose the ScheduleWithTpl type — used in the ScheduleManager trait.
#[allow(dead_code)]
fn _tpl_hint(_: ScheduleWithTpl) {}

#[cfg(test)]
mod tests {
    use crate::models::{Schedule, ScheduleWithTpl};

    #[test]
    fn test_schedule_serialization() {
        let schedule = Schedule {
            id: 1,
            template_id: 10,
            project_id: 5,
            cron: "0 * * * *".to_string(),
            cron_format: Some("standard".to_string()),
            name: "Hourly Deploy".to_string(),
            active: true,
            last_commit_hash: Some("abc123".to_string()),
            repository_id: Some(3),
            created: Some("2024-01-01T00:00:00Z".to_string()),
            run_at: None,
            delete_after_run: false,
        };
        let json = serde_json::to_string(&schedule).unwrap();
        assert!(json.contains("\"name\":\"Hourly Deploy\""));
        assert!(json.contains("\"cron\":\"0 * * * *\""));
        assert!(json.contains("\"active\":true"));
    }

    #[test]
    fn test_schedule_skip_nulls() {
        let schedule = Schedule {
            id: 1,
            template_id: 10,
            project_id: 5,
            cron: "0 0 * * *".to_string(),
            cron_format: None,
            name: "Daily".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        let json = serde_json::to_string(&schedule).unwrap();
        assert!(!json.contains("\"run_at\":"));
        assert!(json.contains("\"last_commit_hash\":null"));
        assert!(json.contains("\"repository_id\":null"));
    }

    #[test]
    fn test_schedule_run_at_serialization() {
        let schedule = Schedule {
            id: 1,
            template_id: 10,
            project_id: 5,
            cron: String::new(),
            cron_format: Some("run_at".to_string()),
            name: "One-time deploy".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: Some("2024-06-15T10:00:00Z".to_string()),
            delete_after_run: true,
        };
        let json = serde_json::to_string(&schedule).unwrap();
        assert!(json.contains("\"run_at\":\"2024-06-15T10:00:00Z\""));
        assert!(json.contains("\"delete_after_run\":true"));
    }

    #[test]
    fn test_schedule_with_tpl_serialization() {
        let schedule = Schedule {
            id: 1,
            template_id: 10,
            project_id: 5,
            cron: "*/5 * * * *".to_string(),
            cron_format: None,
            name: "Frequent".to_string(),
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
        let json = serde_json::to_string(&with_tpl).unwrap();
        assert!(json.contains("\"tpl_playbook\":\"deploy.yml\""));
        assert!(json.contains("\"name\":\"Frequent\""));
    }

    #[test]
    fn test_schedule_with_tpl_none() {
        let schedule = Schedule {
            id: 1,
            template_id: 10,
            project_id: 5,
            cron: "0 0 * * *".to_string(),
            cron_format: None,
            name: "No Tpl".to_string(),
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
        let json = serde_json::to_string(&with_tpl).unwrap();
        assert!(!json.contains("\"tpl_playbook\":"));
    }

    #[test]
    fn test_schedule_clone() {
        let schedule = Schedule {
            id: 1,
            template_id: 10,
            project_id: 5,
            cron: "0 * * * *".to_string(),
            cron_format: None,
            name: "Clone Test".to_string(),
            active: true,
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
    fn test_schedule_default_values() {
        let schedule = Schedule {
            id: 0,
            template_id: 0,
            project_id: 0,
            cron: String::new(),
            cron_format: None,
            name: String::new(),
            active: false,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        assert!(schedule.cron.is_empty());
        assert!(schedule.name.is_empty());
        assert!(!schedule.active);
        assert!(!schedule.delete_after_run);
    }

    #[test]
    fn test_schedule_active_flag() {
        let active_schedule = Schedule {
            id: 1,
            template_id: 1,
            project_id: 1,
            cron: "* * * * *".to_string(),
            cron_format: None,
            name: "Active".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        assert!(active_schedule.active);

        let inactive = Schedule {
            id: 2,
            template_id: 1,
            project_id: 1,
            cron: "* * * * *".to_string(),
            cron_format: None,
            name: "Inactive".to_string(),
            active: false,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        assert!(!inactive.active);
    }

    #[test]
    fn test_schedule_delete_after_run() {
        let schedule = Schedule {
            id: 1,
            template_id: 1,
            project_id: 1,
            cron: "".to_string(),
            cron_format: Some("run_at".to_string()),
            name: "OneShot".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: Some("2025-01-01T00:00:00Z".to_string()),
            delete_after_run: true,
        };
        assert!(schedule.delete_after_run);
    }

    #[test]
    fn test_schedule_cron_formats() {
        let formats = vec!["standard", "run_at", "cronstrue", "quartz"];
        for fmt in formats {
            let schedule = Schedule {
                id: 0,
                template_id: 0,
                project_id: 0,
                cron: "0 0 * * *".to_string(),
                cron_format: Some(fmt.to_string()),
                name: "fmt-test".to_string(),
                active: true,
                last_commit_hash: None,
                repository_id: None,
                created: None,
                run_at: None,
                delete_after_run: false,
            };
            let json = serde_json::to_string(&schedule).unwrap();
            assert!(json.contains(&format!("\"cron_format\":\"{}\"", fmt)));
        }
    }

    #[test]
    fn test_schedule_with_repository() {
        let schedule = Schedule {
            id: 1,
            template_id: 5,
            project_id: 10,
            cron: "0 12 * * *".to_string(),
            cron_format: None,
            name: "With Repo".to_string(),
            active: true,
            last_commit_hash: Some("deadbeef".to_string()),
            repository_id: Some(42),
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        let json = serde_json::to_string(&schedule).unwrap();
        assert!(json.contains("\"repository_id\":42"));
        assert!(json.contains("\"last_commit_hash\":\"deadbeef\""));
    }

    #[test]
    fn test_schedule_name_not_empty() {
        let schedule = Schedule {
            id: 0,
            template_id: 0,
            project_id: 0,
            cron: "".to_string(),
            cron_format: None,
            name: "Test Schedule".to_string(),
            active: false,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };
        assert!(!schedule.name.is_empty());
    }
}
