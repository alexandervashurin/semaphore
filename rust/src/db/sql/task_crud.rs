//! Task CRUD - операции с задачами (PostgreSQL)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use crate::services::task_logger::TaskStatus;
use sqlx::Row;

impl SqlDb {
    fn pg_pool(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Конвертирует SQL row в Task
    fn row_to_task(row: &sqlx::postgres::PgRow) -> Result<Task> {
        let params_json: Option<String> = row.try_get("params").ok().flatten();
        let params = if let Some(json_str) = params_json {
            serde_json::from_str(&json_str).ok()
        } else {
            None
        };

        Ok(Task {
            id: row.get("id"),
            template_id: row.get("template_id"),
            project_id: row.get("project_id"),
            status: serde_json::from_str(&format!("\"{}\"", row.get::<String, _>("status")))
                .map_err(|e| Error::Other(format!("Failed to parse TaskStatus: {}", e)))?,
            playbook: row.get("playbook"),
            environment: row.get("environment"),
            secret: row.get("secret"),
            arguments: row.get("arguments"),
            git_branch: row.get("git_branch"),
            user_id: row.get("user_id"),
            integration_id: row.get("integration_id"),
            schedule_id: row.get("schedule_id"),
            created: row.get("created"),
            start: row.try_get("start_time").ok(),
            end: row.try_get("end_time").ok(),
            message: row.get("message"),
            commit_hash: row.get("commit_hash"),
            commit_message: row.get("commit_message"),
            build_task_id: row.get("build_task_id"),
            version: row.get("version"),
            inventory_id: row.get("inventory_id"),
            repository_id: row.get("repository_id"),
            environment_id: row.get("environment_id"),
            params,
        })
    }

    /// Получает задачи проекта
    pub async fn get_tasks(&self, project_id: i32, template_id: Option<i32>) -> Result<Vec<TaskWithTpl>> {
        let pool = self.pg_pool()?;
        let mut query = String::from(
            "SELECT t.*, tpl.playbook as tpl_playbook
             FROM task t
             LEFT JOIN template tpl ON t.template_id = tpl.id AND tpl.project_id = t.project_id
             WHERE t.project_id = $1"
        );

        let rows = if let Some(tpl_id) = template_id {
            query.push_str(" AND t.template_id = $2");
            sqlx::query(&query)
                .bind(project_id)
                .bind(tpl_id)
                .fetch_all(pool)
                .await
                .map_err(Error::Database)?
        } else {
            sqlx::query(&query)
                .bind(project_id)
                .fetch_all(pool)
                .await
                .map_err(Error::Database)?
        };

        let mut tasks = Vec::new();
        for row in rows {
            let task = Self::row_to_task(&row)?;
            let tpl_playbook: Option<String> = row.try_get("tpl_playbook").ok();
            tasks.push(TaskWithTpl {
                task,
                tpl_playbook,
                tpl_type: None,
                tpl_app: None,
                user_name: None,
                build_task: None,
            });
        }

        Ok(tasks)
    }

    /// Получает все задачи без фильтра проекта (глобальный список)
    pub async fn get_global_tasks(
        &self,
        status_filter: Option<Vec<String>>,
        limit: Option<i32>,
    ) -> Result<Vec<TaskWithTpl>> {
        let lim = limit.unwrap_or(100);
        let pool = self.pg_pool()?;

        let base = "SELECT t.*, tpl.playbook as tpl_playbook FROM task t \
                     LEFT JOIN template tpl ON t.template_id = tpl.id AND tpl.project_id = t.project_id";

        let (filter, rows) = if let Some(ref statuses) = status_filter {
            if statuses.is_empty() {
                let q = format!("{} ORDER BY t.created DESC LIMIT {}", base, lim);
                let r = sqlx::query(&q).fetch_all(pool).await.map_err(Error::Database)?;
                (String::new(), r)
            } else {
                // Build $1,$2,... placeholders
                let placeholders: Vec<String> = (1..=statuses.len()).map(|i| format!("${}", i)).collect();
                let filter_str = format!(" WHERE t.status IN ({})", placeholders.join(", "));
                let q = format!("{}{} ORDER BY t.created DESC LIMIT {}", base, filter_str, lim);
                let mut query = sqlx::query(&q);
                for s in statuses {
                    query = query.bind(s);
                }
                let r = query.fetch_all(pool).await.map_err(Error::Database)?;
                (filter_str, r)
            }
        } else {
            let q = format!("{} ORDER BY t.created DESC LIMIT {}", base, lim);
            let r = sqlx::query(&q).fetch_all(pool).await.map_err(Error::Database)?;
            (String::new(), r)
        };
        let _ = filter;

        let mut tasks = Vec::new();
        for row in rows {
            let task = Self::row_to_task(&row)?;
            let tpl_playbook: Option<String> = row.try_get("tpl_playbook").ok();
            tasks.push(TaskWithTpl { task, tpl_playbook, tpl_type: None, tpl_app: None, user_name: None, build_task: None });
        }
        Ok(tasks)
    }

    /// Получает задачу по ID
    pub async fn get_task(&self, project_id: i32, task_id: i32) -> Result<Task> {
        let row = sqlx::query(
            "SELECT * FROM task WHERE id = $1 AND project_id = $2"
        )
        .bind(task_id)
        .bind(project_id)
        .fetch_optional(self.pg_pool()?)
        .await
        .map_err(Error::Database)?
        .ok_or_else(|| Error::NotFound("Task not found".to_string()))?;

        Self::row_to_task(&row)
    }

    /// Создаёт новую задачу
    pub async fn create_task(&self, mut task: Task) -> Result<Task> {
        let params_json = task.params.as_ref()
            .map(|p| serde_json::to_string(p).ok())
            .flatten();

        let id: i32 = sqlx::query_scalar(
            "INSERT INTO task (project_id, template_id, status, message, commit_hash, commit_message,
             version, inventory_id, repository_id, environment_id, environment, secret, user_id,
             integration_id, schedule_id, build_task_id, git_branch, arguments, params, playbook, created)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21)
             RETURNING id"
        )
        .bind(task.project_id)
        .bind(task.template_id)
        .bind(task.status.to_string())
        .bind(&task.message)
        .bind(&task.commit_hash)
        .bind(&task.commit_message)
        .bind(&task.version)
        .bind(task.inventory_id)
        .bind(task.repository_id)
        .bind(task.environment_id)
        .bind(&task.environment)
        .bind(&task.secret)
        .bind(task.user_id)
        .bind(task.integration_id)
        .bind(task.schedule_id)
        .bind(task.build_task_id)
        .bind(&task.git_branch)
        .bind(&task.arguments)
        .bind(&params_json)
        .bind(&task.playbook)
        .bind(task.created)
        .fetch_one(self.pg_pool()?)
        .await
        .map_err(Error::Database)?;

        task.id = id;
        Ok(task)
    }

    /// Обновляет задачу
    pub async fn update_task(&self, task: Task) -> Result<()> {
        sqlx::query(
            "UPDATE task SET status = $1, message = $2, commit_hash = $3, commit_message = $4,
             version = $5, start_time = $6, end_time = $7 WHERE id = $8 AND project_id = $9"
        )
        .bind(task.status.to_string())
        .bind(&task.message)
        .bind(&task.commit_hash)
        .bind(&task.commit_message)
        .bind(&task.version)
        .bind(task.start)
        .bind(task.end)
        .bind(task.id)
        .bind(task.project_id)
        .execute(self.pg_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Обновляет статус задачи
    pub async fn update_task_status(&self, project_id: i32, task_id: i32, status: TaskStatus) -> Result<()> {
        sqlx::query(
            "UPDATE task SET status = $1 WHERE id = $2 AND project_id = $3"
        )
        .bind(status.to_string())
        .bind(task_id)
        .bind(project_id)
        .execute(self.pg_pool()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Удаляет задачу
    pub async fn delete_task(&self, project_id: i32, task_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM task WHERE id = $1 AND project_id = $2")
            .bind(task_id)
            .bind(project_id)
            .execute(self.pg_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}
