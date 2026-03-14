//! PlaybookRunManager - управление историей запусков playbook

use crate::db::sql::SqlStore;
use crate::db::sql::types::SqlDialect;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::playbook_run_history::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;

#[async_trait]
impl PlaybookRunManager for SqlStore {
    async fn get_playbook_runs(&self, filter: PlaybookRunFilter) -> Result<Vec<PlaybookRun>> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let mut query = String::from("SELECT * FROM playbook_run WHERE 1=1");
                
                if filter.project_id.is_some() {
                    query.push_str(" AND project_id = ?");
                }
                if filter.playbook_id.is_some() {
                    query.push_str(" AND playbook_id = ?");
                }
                if filter.status.is_some() {
                    query.push_str(" AND status = ?");
                }
                if filter.user_id.is_some() {
                    query.push_str(" AND user_id = ?");
                }
                if filter.date_from.is_some() {
                    query.push_str(" AND created >= ?");
                }
                if filter.date_to.is_some() {
                    query.push_str(" AND created <= ?");
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
                    .fetch_all(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
                Ok(runs)
            }
            SqlDialect::PostgreSQL => {
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
                    param_idx += 1;
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
                    .fetch_all(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
                Ok(runs)
            }
            SqlDialect::MySQL => {
                let mut query = String::from("SELECT * FROM playbook_run WHERE 1=1");
                
                if filter.project_id.is_some() {
                    query.push_str(" AND project_id = ?");
                }
                if filter.playbook_id.is_some() {
                    query.push_str(" AND playbook_id = ?");
                }
                if filter.status.is_some() {
                    query.push_str(" AND status = ?");
                }
                if filter.user_id.is_some() {
                    query.push_str(" AND user_id = ?");
                }
                if filter.date_from.is_some() {
                    query.push_str(" AND created >= ?");
                }
                if filter.date_to.is_some() {
                    query.push_str(" AND created <= ?");
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
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let mut query = String::from("UPDATE playbook_run SET updated = datetime('now')");
                let mut binds = 0;

                if let Some(ref status) = update.status {
                    binds += 1;
                    query.push_str(&format!(", status = ?{}", binds));
                }
                if let Some(ref start_time) = update.start_time {
                    binds += 1;
                    query.push_str(&format!(", start_time = ?{}", binds));
                }
                if let Some(ref end_time) = update.end_time {
                    binds += 1;
                    query.push_str(&format!(", end_time = ?{}", binds));
                }
                if let Some(ref duration) = update.duration_seconds {
                    binds += 1;
                    query.push_str(&format!(", duration_seconds = ?{}", binds));
                }
                if let Some(ref hosts_total) = update.hosts_total {
                    binds += 1;
                    query.push_str(&format!(", hosts_total = ?{}", binds));
                }
                if let Some(ref hosts_changed) = update.hosts_changed {
                    binds += 1;
                    query.push_str(&format!(", hosts_changed = ?{}", binds));
                }
                if let Some(ref hosts_unreachable) = update.hosts_unreachable {
                    binds += 1;
                    query.push_str(&format!(", hosts_unreachable = ?{}", binds));
                }
                if let Some(ref hosts_failed) = update.hosts_failed {
                    binds += 1;
                    query.push_str(&format!(", hosts_failed = ?{}", binds));
                }
                if let Some(ref output) = update.output {
                    binds += 1;
                    query.push_str(&format!(", output = ?{}", binds));
                }
                if let Some(ref error_message) = update.error_message {
                    binds += 1;
                    query.push_str(&format!(", error_message = ?{}", binds));
                }

                query.push_str(" WHERE id = ? AND project_id = ? RETURNING *");

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
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
                Ok(updated)
            }
            SqlDialect::PostgreSQL => {
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
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
                Ok(updated)
            }
            SqlDialect::MySQL => {
                let mut query = String::from("UPDATE playbook_run SET updated = NOW()");
                let mut binds = 0;

                if let Some(ref status) = update.status {
                    binds += 1;
                    query.push_str(&format!(", status = ?{}", binds));
                }
                if let Some(ref start_time) = update.start_time {
                    binds += 1;
                    query.push_str(&format!(", start_time = ?{}", binds));
                }
                if let Some(ref end_time) = update.end_time {
                    binds += 1;
                    query.push_str(&format!(", end_time = ?{}", binds));
                }
                if let Some(ref duration) = update.duration_seconds {
                    binds += 1;
                    query.push_str(&format!(", duration_seconds = ?{}", binds));
                }
                if let Some(ref hosts_total) = update.hosts_total {
                    binds += 1;
                    query.push_str(&format!(", hosts_total = ?{}", binds));
                }
                if let Some(ref hosts_changed) = update.hosts_changed {
                    binds += 1;
                    query.push_str(&format!(", hosts_changed = ?{}", binds));
                }
                if let Some(ref hosts_unreachable) = update.hosts_unreachable {
                    binds += 1;
                    query.push_str(&format!(", hosts_unreachable = ?{}", binds));
                }
                if let Some(ref hosts_failed) = update.hosts_failed {
                    binds += 1;
                    query.push_str(&format!(", hosts_failed = ?{}", binds));
                }
                if let Some(ref output) = update.output {
                    binds += 1;
                    query.push_str(&format!(", output = ?{}", binds));
                }
                if let Some(ref error_message) = update.error_message {
                    binds += 1;
                    query.push_str(&format!(", error_message = ?{}", binds));
                }

                query.push_str(" WHERE id = ? AND project_id = ?");

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
                    .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;
                Ok(updated)
            }
        }
    }

    async fn delete_playbook_run(&self, _id: i32, _project_id: i32) -> Result<()> {
        // TODO: Реализовать удаление
        Ok(())
    }

    async fn get_playbook_run_stats(&self, playbook_id: i32) -> Result<PlaybookRunStats> {
        match self.get_dialect() {
            SqlDialect::SQLite => {
                let query = r#"
                    SELECT 
                        COUNT(*) as total_runs,
                        SUM(CASE WHEN status = 'success' THEN 1 ELSE 0 END) as success_runs,
                        SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) as failed_runs,
                        AVG(duration_seconds) as avg_duration_seconds,
                        MAX(created) as last_run
                    FROM playbook_run
                    WHERE playbook_id = ?
                "#;
                let row = sqlx::query(query)
                    .bind(playbook_id)
                    .fetch_one(self.get_sqlite_pool().ok_or_else(|| Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;

                let total_runs: i64 = row.get("total_runs");
                let success_runs: i64 = row.get("success_runs");
                let failed_runs: i64 = row.get("failed_runs");
                let avg_duration_seconds: Option<f64> = row.get("avg_duration_seconds");
                let last_run: Option<String> = row.get("last_run");

                let last_run_dt = last_run.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d: chrono::DateTime<chrono::FixedOffset>| d.with_timezone(&Utc)));

                Ok(PlaybookRunStats {
                    total_runs,
                    success_runs,
                    failed_runs,
                    avg_duration_seconds,
                    last_run: last_run_dt,
                })
            }
            SqlDialect::PostgreSQL => {
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
                    .fetch_one(self.get_postgres_pool().ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))?)
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
            SqlDialect::MySQL => {
                let query = r#"
                    SELECT 
                        COUNT(*) as total_runs,
                        SUM(CASE WHEN status = 'success' THEN 1 ELSE 0 END) as success_runs,
                        SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) as failed_runs,
                        AVG(duration_seconds) as avg_duration_seconds,
                        MAX(created) as last_run
                    FROM playbook_run
                    WHERE playbook_id = ?
                "#;
                let row = sqlx::query(query)
                    .bind(playbook_id)
                    .fetch_one(self.get_mysql_pool().ok_or_else(|| Error::Other("MySQL pool not found".to_string()))?)
                    .await
                    .map_err(Error::Database)?;

                let total_runs: i64 = row.get("total_runs");
                let success_runs: i64 = row.get("success_runs");
                let failed_runs: i64 = row.get("failed_runs");
                let avg_duration_seconds: Option<f64> = row.get("avg_duration_seconds");
                let last_run: Option<String> = row.get("last_run");

                let last_run_dt = last_run.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d: chrono::DateTime<chrono::FixedOffset>| d.with_timezone(&Utc)));

                Ok(PlaybookRunStats {
                    total_runs,
                    success_runs,
                    failed_runs,
                    avg_duration_seconds,
                    last_run: last_run_dt,
                })
            }
        }
    }
}
