//! PlaybookRunManager - управление историей запусков playbook

use crate::db::sql::SqlStore;
use crate::db::sql::types::SqlDialect;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::playbook_run_history::*;
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl PlaybookRunManager for SqlStore {
    async fn get_playbook_runs(&self, filter: PlaybookRunFilter) -> Result<Vec<PlaybookRun>> {
        // TODO: Реализовать фильтрацию
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM playbook_run ORDER BY created DESC LIMIT 100";
                let runs = sqlx::query_as::<_, PlaybookRun>(query)
                    .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
                Ok(runs)
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM playbook_run ORDER BY created DESC LIMIT 100";
                let runs = sqlx::query_as::<_, PlaybookRun>(query)
                    .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
                Ok(runs)
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM playbook_run ORDER BY created DESC LIMIT 100";
                let runs = sqlx::query_as::<_, PlaybookRun>(query)
                    .fetch_all(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
                Ok(runs)
            }
        }
    }

    async fn get_playbook_run(&self, id: i32, project_id: i32) -> Result<PlaybookRun> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = "SELECT * FROM playbook_run WHERE id = ? AND project_id = ?";
                let run = sqlx::query_as::<_, PlaybookRun>(query)
                    .bind(id)
                    .bind(project_id)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
                Ok(run)
            }
            SqlDialect::PostgreSQL => {
                let query = "SELECT * FROM playbook_run WHERE id = $1 AND project_id = $2";
                let run = sqlx::query_as::<_, PlaybookRun>(query)
                    .bind(id)
                    .bind(project_id)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
                Ok(run)
            }
            SqlDialect::MySQL => {
                let query = "SELECT * FROM playbook_run WHERE id = ? AND project_id = ?";
                let run = sqlx::query_as::<_, PlaybookRun>(query)
                    .bind(id)
                    .bind(project_id)
                    .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
                Ok(run)
            }
        }
    }

    async fn create_playbook_run(&self, run: PlaybookRunCreate) -> Result<PlaybookRun> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = r#"
                    INSERT INTO playbook_run (
                        project_id, playbook_id, task_id, template_id,
                        inventory_id, environment_id, extra_vars, limit_hosts, tags, skip_tags,
                        user_id, status, created, updated
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'waiting', datetime('now'), datetime('now'))
                    RETURNING *
                "#;
                let created = sqlx::query_as::<_, PlaybookRun>(query)
                    .bind(run.project_id)
                    .bind(run.playbook_id)
                    .bind(run.task_id)
                    .bind(run.template_id)
                    .bind(run.inventory_id)
                    .bind(run.environment_id)
                    .bind(run.extra_vars)
                    .bind(run.limit_hosts)
                    .bind(run.tags)
                    .bind(run.skip_tags)
                    .bind(run.user_id)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
                Ok(created)
            }
            SqlDialect::PostgreSQL => {
                let query = r#"
                    INSERT INTO playbook_run (
                        project_id, playbook_id, task_id, template_id,
                        inventory_id, environment_id, extra_vars, limit_hosts, tags, skip_tags,
                        user_id, status, created, updated
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'waiting', NOW(), NOW())
                    RETURNING *
                "#;
                let created = sqlx::query_as::<_, PlaybookRun>(query)
                    .bind(run.project_id)
                    .bind(run.playbook_id)
                    .bind(run.task_id)
                    .bind(run.template_id)
                    .bind(run.inventory_id)
                    .bind(run.environment_id)
                    .bind(run.extra_vars)
                    .bind(run.limit_hosts)
                    .bind(run.tags)
                    .bind(run.skip_tags)
                    .bind(run.user_id)
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
                Ok(created)
            }
            SqlDialect::MySQL => {
                let query = r#"
                    INSERT INTO playbook_run (
                        project_id, playbook_id, task_id, template_id,
                        inventory_id, environment_id, extra_vars, limit_hosts, tags, skip_tags,
                        user_id, status, created, updated
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'waiting', NOW(), NOW())
                    RETURNING *
                "#;
                let created = sqlx::query_as::<_, PlaybookRun>(query)
                    .bind(run.project_id)
                    .bind(run.playbook_id)
                    .bind(run.task_id)
                    .bind(run.template_id)
                    .bind(run.inventory_id)
                    .bind(run.environment_id)
                    .bind(run.extra_vars)
                    .bind(run.limit_hosts)
                    .bind(run.tags)
                    .bind(run.skip_tags)
                    .bind(run.user_id)
                    .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
                Ok(created)
            }
        }
    }

    async fn update_playbook_run(&self, id: i32, project_id: i32, update: PlaybookRunUpdate) -> Result<PlaybookRun> {
        // TODO: Реализовать обновление
        self.get_playbook_run(id, project_id).await
    }

    async fn delete_playbook_run(&self, _id: i32, _project_id: i32) -> Result<()> {
        // TODO: Реализовать удаление
        Ok(())
    }

    async fn get_playbook_run_stats(&self, _playbook_id: i32) -> Result<PlaybookRunStats> {
        // TODO: Реализовать статистику
        Ok(PlaybookRunStats {
            total_runs: 0,
            success_runs: 0,
            failed_runs: 0,
            avg_duration_seconds: None,
            last_run: None,
        })
    }
}
