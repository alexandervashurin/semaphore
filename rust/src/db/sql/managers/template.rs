//! TemplateManager - управление шаблонами
//!
//! Реализация трейта TemplateManager для SqlStore

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::models::Template;
use crate::error::{Error, Result};
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl TemplateManager for SqlStore {
    async fn get_templates(&self, project_id: i32) -> Result<Vec<Template>> {
        let query = "SELECT * FROM template WHERE project_id = $1 ORDER BY name";
            let rows = sqlx::query(query)
                .bind(project_id)
                .fetch_all(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;

            Ok(rows.into_iter().map(|row| Template {
                id: row.get("id"),
                project_id: row.get("project_id"),
                name: row.get("name"),
                playbook: row.get("playbook"),
                description: row.get("description"),
                inventory_id: row.try_get("inventory_id").ok().flatten(),
                repository_id: row.try_get("repository_id").ok().flatten(),
                environment_id: row.try_get("environment_id").ok().flatten(),
                r#type: row.get("type"),
                app: row.get("app"),
                git_branch: row.try_get("git_branch").ok().flatten(),
                created: row.get("created"),
                arguments: row.get("arguments"),
                vault_key_id: row.get("vault_key_id"),
                view_id: row.try_get("view_id").ok().flatten(),
                build_template_id: row.try_get("build_template_id").ok().flatten(),
                autorun: row.try_get("autorun").ok().unwrap_or(false),
                allow_override_args_in_task: row.try_get("allow_override_args_vars").ok().unwrap_or(false),
                allow_override_branch_in_task: row.try_get("allow_override_branch_in_task").ok().unwrap_or(false),
                allow_inventory_in_task: row.try_get("allow_inventory_in_task").ok().unwrap_or(false),
                allow_parallel_tasks: row.try_get("allow_parallel_tasks").ok().unwrap_or(false),
                suppress_success_alerts: row.try_get("suppress_success_alerts").ok().unwrap_or(false),
                task_params: row.try_get("task_params").ok().flatten(),
                survey_vars: row.try_get("survey_vars").ok().flatten(),
                vaults: row.try_get("vaults").ok().flatten(),
            }).collect())
    }

    async fn get_template(&self, project_id: i32, template_id: i32) -> Result<Template> {
        let query = "SELECT * FROM template WHERE id = $1 AND project_id = $2";
            let row = sqlx::query(query)
                .bind(template_id)
                .bind(project_id)
                .fetch_one(self.get_postgres_pool()?)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound("Шаблон не найден".to_string()),
                    _ => Error::Database(e),
                })?;

            Ok(Template {
                id: row.get("id"),
                project_id: row.get("project_id"),
                name: row.get("name"),
                playbook: row.get("playbook"),
                description: row.get("description"),
                inventory_id: row.get("inventory_id"),
                repository_id: row.get("repository_id"),
                environment_id: row.get("environment_id"),
                r#type: row.get("type"),
                app: row.get("app"),
                git_branch: row.get("git_branch"),
                created: row.get("created"),
                arguments: row.get("arguments"),
                vault_key_id: row.get("vault_key_id"),
                view_id: row.try_get("view_id").ok().flatten(),
                build_template_id: row.try_get("build_template_id").ok().flatten(),
                autorun: row.try_get("autorun").ok().unwrap_or(false),
                allow_override_args_in_task: row.try_get("allow_override_args_vars").ok().unwrap_or(false),
                allow_override_branch_in_task: row.try_get("allow_override_branch_in_task").ok().unwrap_or(false),
                allow_inventory_in_task: row.try_get("allow_inventory_in_task").ok().unwrap_or(false),
                allow_parallel_tasks: row.try_get("allow_parallel_tasks").ok().unwrap_or(false),
                suppress_success_alerts: row.try_get("suppress_success_alerts").ok().unwrap_or(false),
                task_params: row.try_get("task_params").ok().flatten(),
                survey_vars: row.try_get("survey_vars").ok().flatten(),
                vaults: row.try_get("vaults").ok().flatten(),
            })
    }

    async fn create_template(&self, mut template: Template) -> Result<Template> {
        let query = "INSERT INTO template (project_id, name, playbook, description, inventory_id, repository_id, environment_id, type, app, git_branch, created, arguments, vault_key_id, view_id, build_template_id, autorun, allow_override_args_vars, allow_override_branch_in_task, allow_inventory_in_task, allow_parallel_tasks, suppress_success_alerts, task_params, survey_vars, vaults) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24) RETURNING id";
            let id: i32 = sqlx::query_scalar(query)
                .bind(template.project_id)
                .bind(&template.name)
                .bind(&template.playbook)
                .bind(&template.description)
                .bind(template.inventory_id)
                .bind(template.repository_id)
                .bind(template.environment_id)
                .bind(&template.r#type)
                .bind(&template.app)
                .bind(&template.git_branch)
                .bind(template.created)
                .bind(&template.arguments)
                .bind(template.vault_key_id)
                .bind(template.view_id)
                .bind(template.build_template_id)
                .bind(template.autorun)
                .bind(template.allow_override_args_in_task)
                .bind(template.allow_override_branch_in_task)
                .bind(template.allow_inventory_in_task)
                .bind(template.allow_parallel_tasks)
                .bind(template.suppress_success_alerts)
                .bind(&template.task_params)
                .bind(&template.survey_vars)
                .bind(&template.vaults)
                .fetch_one(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;

            template.id = id;
            Ok(template)
    }

    async fn update_template(&self, template: Template) -> Result<()> {
        let query = "UPDATE template SET name = $1, playbook = $2, description = $3, inventory_id = $4, repository_id = $5, environment_id = $6, type = $7, app = $8, git_branch = $9, arguments = $10, vault_key_id = $11, view_id = $12, build_template_id = $13, autorun = $14, allow_override_args_vars = $15, allow_override_branch_in_task = $16, allow_inventory_in_task = $17, allow_parallel_tasks = $18, suppress_success_alerts = $19, task_params = $20, survey_vars = $21, vaults = $22 WHERE id = $23 AND project_id = $24";
            sqlx::query(query)
                .bind(&template.name)
                .bind(&template.playbook)
                .bind(&template.description)
                .bind(template.inventory_id)
                .bind(template.repository_id)
                .bind(template.environment_id)
                .bind(&template.r#type)
                .bind(&template.app)
                .bind(&template.git_branch)
                .bind(&template.arguments)
                .bind(template.vault_key_id)
                .bind(template.view_id)
                .bind(template.build_template_id)
                .bind(template.autorun)
                .bind(template.allow_override_args_in_task)
                .bind(template.allow_override_branch_in_task)
                .bind(template.allow_inventory_in_task)
                .bind(template.allow_parallel_tasks)
                .bind(template.suppress_success_alerts)
                .bind(&template.task_params)
                .bind(&template.survey_vars)
                .bind(&template.vaults)
                .bind(template.id)
                .bind(template.project_id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_template(&self, project_id: i32, template_id: i32) -> Result<()> {
        let query = "UPDATE template SET deleted = true WHERE id = $1 AND project_id = $2";
            sqlx::query(query)
                .bind(template_id)
                .bind(project_id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        Ok(())
    }
}

// ============================================================================
// InventoryManager - CRUD операции для инвентарей
// ============================================================================
