//! PlaybookRunManager - управление историей запусков playbook

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::playbook_run_history::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

#[async_trait]
impl PlaybookRunManager for SqlStore {
    async fn get_playbook_runs(&self, filter: PlaybookRunFilter) -> Result<Vec<PlaybookRun>> {
        let mut query = String::from("SELECT * FROM playbook_run WHERE 1=1");
            let mut param_idx = 1;

            if filter.project_id.is_some() {
                query.push_str(&format!(" AND project_id = ${}", param_idx));
                param_idx += 1;
            }
            if filter.playbook_id.is_some() {
                query.push_str(&format!(" AND playbook_id = ${}", param_idx));
                param_idx += 1;
            }
            if filter.status.is_some() {
                query.push_str(&format!(" AND status = ${}", param_idx));
                param_idx += 1;
            }
            if filter.user_id.is_some() {
                query.push_str(&format!(" AND user_id = ${}", param_idx));
                param_idx += 1;
            }
            if filter.date_from.is_some() {
                query.push_str(&format!(" AND created >= ${}", param_idx));
                param_idx += 1;
            }
            if filter.date_to.is_some() {
                query.push_str(&format!(" AND created <= ${}", param_idx));
                let _ = param_idx;
            }

            query.push_str(" ORDER BY created DESC");

            let limit = filter.limit.unwrap_or(100);
            query.push_str(&format!(" LIMIT {}", limit));

            if let Some(offset) = filter.offset {
                query.push_str(&format!(" OFFSET {}", offset));
            }

            let mut sql_query = sqlx::query_as::<_, PlaybookRun>(&query);
            
            if let Some(project_id) = filter.project_id {
                sql_query = sql_query.bind(project_id);
            }
            if let Some(playbook_id) = filter.playbook_id {
                sql_query = sql_query.bind(playbook_id);
            }
            if let Some(status) = filter.status {
                sql_query = sql_query.bind(status.to_string());
            }
            if let Some(user_id) = filter.user_id {
                sql_query = sql_query.bind(user_id);
            }
            if let Some(date_from) = filter.date_from {
                sql_query = sql_query.bind(date_from.to_rfc3339());
            }
            if let Some(date_to) = filter.date_to {
                sql_query = sql_query.bind(date_to.to_rfc3339());
            }

            let runs = sql_query
                .fetch_all(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
            Ok(runs)
    }

    async fn get_playbook_run(&self, id: i32, project_id: i32) -> Result<PlaybookRun> {
        let query = "SELECT * FROM playbook_run WHERE id = $1 AND project_id = $2";
            let run = sqlx::query_as::<_, PlaybookRun>(query)
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
            Ok(run)
    }

    async fn get_playbook_run_by_task_id(&self, task_id: i32) -> Result<Option<PlaybookRun>> {
        let run = sqlx::query_as::<_, PlaybookRun>(
                "SELECT * FROM playbook_run WHERE task_id = $1 LIMIT 1"
            )
            .bind(task_id)
            .fetch_optional(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
            Ok(run)
    }

    async fn update_playbook_run_status(&self, id: i32, status: PlaybookRunStatus) -> Result<()> {
        sqlx::query("UPDATE playbook_run SET status = $1, updated = NOW() WHERE id = $2")
                .bind(status.to_string())
                .bind(id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }

    async fn create_playbook_run(&self, run: PlaybookRunCreate) -> Result<PlaybookRun> {
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
                .fetch_one(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
            Ok(created)
    }

    async fn update_playbook_run(&self, id: i32, project_id: i32, update: PlaybookRunUpdate) -> Result<PlaybookRun> {
        let mut query = String::from("UPDATE playbook_run SET updated = NOW()");
            let mut param_idx = 1;

            if update.status.is_some() {
                query.push_str(&format!(", status = ${}", param_idx));
                param_idx += 1;
            }
            if update.start_time.is_some() {
                query.push_str(&format!(", start_time = ${}", param_idx));
                param_idx += 1;
            }
            if update.end_time.is_some() {
                query.push_str(&format!(", end_time = ${}", param_idx));
                param_idx += 1;
            }
            if update.duration_seconds.is_some() {
                query.push_str(&format!(", duration_seconds = ${}", param_idx));
                param_idx += 1;
            }
            if update.hosts_total.is_some() {
                query.push_str(&format!(", hosts_total = ${}", param_idx));
                param_idx += 1;
            }
            if update.hosts_changed.is_some() {
                query.push_str(&format!(", hosts_changed = ${}", param_idx));
                param_idx += 1;
            }
            if update.hosts_unreachable.is_some() {
                query.push_str(&format!(", hosts_unreachable = ${}", param_idx));
                param_idx += 1;
            }
            if update.hosts_failed.is_some() {
                query.push_str(&format!(", hosts_failed = ${}", param_idx));
                param_idx += 1;
            }
            if update.output.is_some() {
                query.push_str(&format!(", output = ${}", param_idx));
                param_idx += 1;
            }
            if update.error_message.is_some() {
                query.push_str(&format!(", error_message = ${}", param_idx));
                param_idx += 1;
            }

            query.push_str(&format!(" WHERE id = ${} AND project_id = ${} RETURNING *", param_idx, param_idx + 1));

            let mut sql_query = sqlx::query_as::<_, PlaybookRun>(&query);

            if let Some(status) = update.status {
                sql_query = sql_query.bind(status.to_string());
            }
            if let Some(start_time) = update.start_time {
                sql_query = sql_query.bind(start_time.to_rfc3339());
            }
            if let Some(end_time) = update.end_time {
                sql_query = sql_query.bind(end_time.to_rfc3339());
            }
            if let Some(duration) = update.duration_seconds {
                sql_query = sql_query.bind(duration);
            }
            if let Some(hosts_total) = update.hosts_total {
                sql_query = sql_query.bind(hosts_total);
            }
            if let Some(hosts_changed) = update.hosts_changed {
                sql_query = sql_query.bind(hosts_changed);
            }
            if let Some(hosts_unreachable) = update.hosts_unreachable {
                sql_query = sql_query.bind(hosts_unreachable);
            }
            if let Some(hosts_failed) = update.hosts_failed {
                sql_query = sql_query.bind(hosts_failed);
            }
            if let Some(output) = update.output {
                sql_query = sql_query.bind(output);
            }
            if let Some(error_message) = update.error_message {
                sql_query = sql_query.bind(error_message);
            }

            let updated = sql_query
                .bind(id)
                .bind(project_id)
                .fetch_one(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
            Ok(updated)
    }

    async fn delete_playbook_run(&self, id: i32, project_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM playbook_run WHERE id = $1 AND project_id = $2")
                .bind(id)
                .bind(project_id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }

    async fn get_playbook_run_stats(&self, playbook_id: i32) -> Result<PlaybookRunStats> {
        let query = r#"
                SELECT 
                    COUNT(*) as total_runs,
                    SUM(CASE WHEN status = 'success' THEN 1 ELSE 0 END) as success_runs,
                    SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) as failed_runs,
                    AVG(duration_seconds) as avg_duration_seconds,
                    MAX(created) as last_run
                FROM playbook_run
                WHERE playbook_id = $1
            "#;
            let row = sqlx::query(query)
                .bind(playbook_id)
                .fetch_one(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;

            let total_runs: i64 = row.get("total_runs");
            let success_runs: i64 = row.get("success_runs");
            let failed_runs: i64 = row.get("failed_runs");
            let avg_duration_seconds: Option<f64> = row.get("avg_duration_seconds");
            let last_run: Option<DateTime<Utc>> = row.get("last_run");

            Ok(PlaybookRunStats {
                total_runs,
                success_runs,
                failed_runs,
                avg_duration_seconds,
                last_run,
            })
    }
}
