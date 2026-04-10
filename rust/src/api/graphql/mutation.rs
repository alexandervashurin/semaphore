//! GraphQL Mutation корень — CRUD операции

use crate::api::state::AppState;
use crate::db::store::{ProjectStore, TaskManager, TemplateManager, UserManager};
use crate::models::template::{TemplateApp, TemplateType};
use crate::models::{Project as DbProject, Task as DbTask, Template as DbTemplate, User as DbUser};
use crate::services::task_logger::TaskStatus;
use async_graphql::{Context, InputObject, Object, Result};
use chrono::Utc;

use super::types::{Project, Task, Template, User};

/// Input для создания пользователя
#[derive(InputObject, Debug)]
pub struct CreateUserInput {
    pub username: String,
    pub email: String,
    pub name: Option<String>,
    pub password: String,
    pub admin: Option<bool>,
}

/// Input для создания проекта
#[derive(InputObject, Debug)]
pub struct CreateProjectInput {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

/// Input для создания шаблона
#[derive(InputObject, Debug)]
pub struct CreateTemplateInput {
    pub project_id: i32,
    pub name: String,
    pub playbook: String,
    pub description: Option<String>,
    pub inventory_id: Option<i32>,
    pub repository_id: Option<i32>,
    pub environment_id: Option<i32>,
}

/// Input для запуска задачи
#[derive(InputObject, Debug)]
pub struct CreateTaskInput {
    pub template_id: i32,
    pub project_id: i32,
    pub debug: Option<bool>,
    pub dry_run: Option<bool>,
    pub diff: Option<bool>,
}

/// Корневой тип для Mutation
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Создать пользователя
    async fn create_user(&self, ctx: &Context<'_>, input: CreateUserInput) -> Result<User> {
        let state = ctx.data::<AppState>()?;
        let store = &state.store;

        let admin = input.admin.unwrap_or(false);

        let new_user = DbUser {
            id: 0,
            created: Utc::now(),
            username: input.username.clone(),
            email: input.email.clone(),
            name: input.name.unwrap_or_default(),
            password: input.password.clone(),
            admin,
            external: false,
            pro: false,
            alert: false,
            totp: None,
            email_otp: None,
        };

        let created = store.create_user(new_user, &input.password).await?;

        Ok(User {
            id: created.id,
            username: created.username,
            name: created.name,
            email: created.email,
            admin: created.admin,
        })
    }

    /// Создать проект
    async fn create_project(
        &self,
        ctx: &Context<'_>,
        input: CreateProjectInput,
    ) -> Result<Project> {
        let state = ctx.data::<AppState>()?;
        let store = &state.store;

        let new_project = DbProject {
            id: 0,
            created: Utc::now(),
            name: input.name.clone(),
            alert: false,
            alert_chat: None,
            max_parallel_tasks: 0,
            r#type: String::new(),
            default_secret_storage_id: None,
        };

        let created = store.create_project(new_project).await?;

        Ok(Project {
            id: created.id,
            name: created.name,
        })
    }

    /// Создать шаблон
    async fn create_template(
        &self,
        ctx: &Context<'_>,
        input: CreateTemplateInput,
    ) -> Result<Template> {
        let state = ctx.data::<AppState>()?;
        let store = &state.store;

        let new_template = DbTemplate {
            id: 0,
            project_id: input.project_id,
            name: input.name.clone(),
            playbook: input.playbook.clone(),
            description: input.description.unwrap_or_default(),
            inventory_id: input.inventory_id,
            repository_id: input.repository_id,
            environment_id: input.environment_id,
            vault_key_id: None,
            arguments: None,
            git_branch: None,
            app: TemplateApp::Default,
            r#type: TemplateType::Default,
            created: Utc::now(),
            view_id: None,
            build_template_id: None,
            autorun: false,
            allow_override_args_in_task: false,
            allow_override_branch_in_task: false,
            allow_inventory_in_task: false,
            allow_parallel_tasks: false,
            suppress_success_alerts: false,
            require_approval: false,
            task_params: None,
            survey_vars: None,
            vaults: None,
            parent_template_id: None,
            execution_image: None,
            pre_template_id: None,
            post_template_id: None,
            fail_template_id: None,
            deploy_environment_id: None,
        };

        let created = store.create_template(new_template).await?;

        Ok(Template {
            id: created.id,
            project_id: created.project_id,
            name: created.name,
            playbook: created.playbook,
        })
    }

    /// Запустить задачу
    async fn create_task(&self, ctx: &Context<'_>, input: CreateTaskInput) -> Result<Task> {
        let state = ctx.data::<AppState>()?;
        let store = &state.store;

        let new_task = DbTask {
            id: 0,
            template_id: input.template_id,
            project_id: input.project_id,
            status: TaskStatus::Waiting,
            playbook: None,
            environment: None,
            secret: None,
            arguments: None,
            git_branch: None,
            user_id: None,
            integration_id: None,
            schedule_id: None,
            created: Utc::now(),
            start: None,
            end: None,
            message: None,
            commit_hash: None,
            commit_message: None,
            build_task_id: None,
            version: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            params: None,
        };

        let created = store.create_task(new_task).await?;

        Ok(Task {
            id: created.id,
            template_id: created.template_id,
            project_id: created.project_id,
            status: created.status.to_string(),
            created: created.created.to_rfc3339(),
        })
    }

    /// Обновить шаблон
    async fn update_template(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        id: i32,
        name: String,
        playbook: String,
    ) -> Result<Template> {
        let state = ctx.data::<AppState>()?;
        let store = &state.store;

        let template = store
            .get_template(project_id, id)
            .await
            .map_err(|_| async_graphql::Error::new("Template not found"))?;

        let updated = DbTemplate {
            name,
            playbook,
            ..template.clone()
        };

        store.update_template(updated).await?;

        Ok(Template {
            id,
            project_id: template.project_id,
            name: template.name.clone(),
            playbook: template.playbook.clone(),
        })
    }

    /// Удалить шаблон
    async fn delete_template(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let state = ctx.data::<AppState>()?;
        let store = &state.store;

        // Получаем project_id из шаблона
        let templates = store.get_templates(1).await?;
        let template = templates
            .iter()
            .find(|t| t.id == id)
            .ok_or_else(|| async_graphql::Error::new("Template not found"))?;
        let project_id = template.project_id;

        store.delete_template(project_id, id).await?;
        Ok(true)
    }

    /// Удалить задачу
    async fn delete_task(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let state = ctx.data::<AppState>()?;
        let store = &state.store;

        // Получаем project_id из задачи
        let tasks = store.get_tasks(1, None).await?;
        let task = tasks
            .iter()
            .find(|t| t.task.id == id)
            .ok_or_else(|| async_graphql::Error::new("Task not found"))?;
        let project_id = task.task.project_id;

        store.delete_task(project_id, id).await?;
        Ok(true)
    }

    /// Остановить задачу (перевести в статус stopped)
    async fn stop_task(&self, ctx: &Context<'_>, project_id: i32, task_id: i32) -> Result<bool> {
        use crate::db::store::TaskManager;
        let state = ctx.data::<AppState>()?;
        let store = &state.store;

        let mut task = store.get_task(project_id, task_id).await
            .map_err(|_| async_graphql::Error::new("Task not found"))?;

        if matches!(task.status,
            crate::services::task_logger::TaskStatus::Running |
            crate::services::task_logger::TaskStatus::Waiting)
        {
            task.status = crate::services::task_logger::TaskStatus::Stopped;
            store.update_task(task).await
                .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        }

        Ok(true)
    }

    /// Запустить шаблон с дополнительными параметрами
    async fn run_template(
        &self,
        ctx: &Context<'_>,
        project_id: i32,
        template_id: i32,
        extra_vars: Option<String>,
        debug: Option<bool>,
        dry_run: Option<bool>,
    ) -> Result<Task> {
        let state = ctx.data::<AppState>()?;
        let store = &state.store;

        let new_task = DbTask {
            id: 0,
            template_id,
            project_id,
            status: crate::services::task_logger::TaskStatus::Waiting,
            playbook: None,
            environment: extra_vars,
            secret: None,
            arguments: None,
            git_branch: None,
            user_id: None,
            integration_id: None,
            schedule_id: None,
            created: Utc::now(),
            start: None,
            end: None,
            message: None,
            commit_hash: None,
            commit_message: None,
            build_task_id: None,
            version: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            params: debug.map(|d| serde_json::json!({"debug": d, "dry_run": dry_run.unwrap_or(false)})),
        };

        let created = store.create_task(new_task).await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let task_result = Task {
            id: created.id,
            template_id: created.template_id,
            project_id: created.project_id,
            status: created.status.to_string(),
            created: created.created.to_rfc3339(),
        };

        // Publish to subscription channel
        super::subscription::publish_task_created(task_result.clone());

        Ok(task_result)
    }

    /// Одобрить Terraform план
    async fn approve_plan(
        &self,
        ctx: &Context<'_>,
        plan_id: i32,
        reviewed_by: i32,
        comment: Option<String>,
    ) -> Result<bool> {
        use crate::db::store::PlanApprovalManager;
        let state = ctx.data::<AppState>()?;
        let store = &state.store;

        store.approve_plan(plan_id as i64, reviewed_by, comment)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(true)
    }

    /// Отклонить Terraform план
    async fn reject_plan(
        &self,
        ctx: &Context<'_>,
        plan_id: i32,
        reviewed_by: i32,
        reason: Option<String>,
    ) -> Result<bool> {
        use crate::db::store::PlanApprovalManager;
        let state = ctx.data::<AppState>()?;
        let store = &state.store;

        store.reject_plan(plan_id as i64, reviewed_by, reason)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user_input_fields() {
        let input = CreateUserInput {
            username: "admin".to_string(),
            email: "admin@example.com".to_string(),
            name: Some("Admin User".to_string()),
            password: "secret123".to_string(),
            admin: Some(true),
        };
        assert_eq!(input.username, "admin");
        assert_eq!(input.email, "admin@example.com");
        assert_eq!(input.name, Some("Admin User".to_string()));
        assert_eq!(input.admin, Some(true));
    }

    #[test]
    fn test_create_user_input_optional_fields_none() {
        let input = CreateUserInput {
            username: "user".to_string(),
            email: "user@example.com".to_string(),
            name: None,
            password: "pass".to_string(),
            admin: None,
        };
        assert_eq!(input.username, "user");
        assert!(input.name.is_none());
        assert!(input.admin.is_none());
    }

    #[test]
    fn test_create_user_input_field_values() {
        let input = CreateUserInput {
            username: "test".to_string(),
            email: "test@test.com".to_string(),
            name: Some("Test".to_string()),
            password: "pwd".to_string(),
            admin: Some(false),
        };
        assert_eq!(input.username, "test");
        assert_eq!(input.email, "test@test.com");
        assert_eq!(input.name, Some("Test".to_string()));
        assert_eq!(input.admin, Some(false));
    }

    #[test]
    fn test_create_project_input_fields() {
        let input = CreateProjectInput {
            name: "My Project".to_string(),
            description: Some("A test project".to_string()),
            color: Some("#FF0000".to_string()),
        };
        assert_eq!(input.name, "My Project");
        assert_eq!(input.description, Some("A test project".to_string()));
        assert_eq!(input.color, Some("#FF0000".to_string()));
    }

    #[test]
    fn test_create_project_input_values() {
        let input = CreateProjectInput {
            name: "TestProject".to_string(),
            description: None,
            color: None,
        };
        assert_eq!(input.name, "TestProject");
        assert!(input.description.is_none());
        assert!(input.color.is_none());
    }

    #[test]
    fn test_create_project_input_minimal() {
        let input = CreateProjectInput {
            name: "Minimal".to_string(),
            description: None,
            color: None,
        };
        assert_eq!(input.name, "Minimal");
    }

    #[test]
    fn test_create_template_input_fields() {
        let input = CreateTemplateInput {
            project_id: 1,
            name: "deploy-template".to_string(),
            playbook: "deploy.yml".to_string(),
            description: Some("Deployment template".to_string()),
            inventory_id: Some(2),
            repository_id: Some(3),
            environment_id: Some(4),
        };
        assert_eq!(input.project_id, 1);
        assert_eq!(input.name, "deploy-template");
        assert_eq!(input.playbook, "deploy.yml");
        assert_eq!(input.inventory_id, Some(2));
    }

    #[test]
    fn test_create_template_input_minimal() {
        let input = CreateTemplateInput {
            project_id: 1,
            name: "simple".to_string(),
            playbook: "simple.yml".to_string(),
            description: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
        };
        assert_eq!(input.project_id, 1);
        assert!(input.inventory_id.is_none());
        assert!(input.repository_id.is_none());
    }

    #[test]
    fn test_create_template_input_values() {
        let input = CreateTemplateInput {
            project_id: 5,
            name: "tpl".to_string(),
            playbook: "run.yml".to_string(),
            description: Some("desc".to_string()),
            inventory_id: Some(10),
            repository_id: None,
            environment_id: None,
        };
        assert_eq!(input.project_id, 5);
        assert_eq!(input.name, "tpl");
        assert_eq!(input.inventory_id, Some(10));
    }

    #[test]
    fn test_create_task_input_fields() {
        let input = CreateTaskInput {
            template_id: 1,
            project_id: 2,
            debug: Some(true),
            dry_run: Some(false),
            diff: Some(true),
        };
        assert_eq!(input.template_id, 1);
        assert_eq!(input.project_id, 2);
        assert_eq!(input.debug, Some(true));
        assert_eq!(input.dry_run, Some(false));
        assert_eq!(input.diff, Some(true));
    }

    #[test]
    fn test_create_task_input_all_none() {
        let input = CreateTaskInput {
            template_id: 1,
            project_id: 2,
            debug: None,
            dry_run: None,
            diff: None,
        };
        assert!(input.debug.is_none());
        assert!(input.dry_run.is_none());
        assert!(input.diff.is_none());
    }

    #[test]
    fn test_create_task_input_values() {
        let input = CreateTaskInput {
            template_id: 10,
            project_id: 20,
            debug: Some(true),
            dry_run: Some(true),
            diff: Some(false),
        };
        assert_eq!(input.template_id, 10);
        assert_eq!(input.project_id, 20);
        assert_eq!(input.debug, Some(true));
        assert_eq!(input.dry_run, Some(true));
        assert_eq!(input.diff, Some(false));
    }

    #[test]
    fn test_mutation_root_exists() {
        let root = MutationRoot;
        // Verify MutationRoot can be instantiated
        drop(root);
    }

    #[test]
    fn test_create_user_input_fields_equality() {
        let input = CreateUserInput {
            username: "clone_user".to_string(),
            email: "clone@test.com".to_string(),
            name: Some("Clone".to_string()),
            password: "pass".to_string(),
            admin: Some(false),
        };
        assert_eq!(input.username, "clone_user");
        assert_eq!(input.email, "clone@test.com");
        assert_eq!(input.name, Some("Clone".to_string()));
    }

    #[test]
    fn test_create_project_input_fields_equality() {
        let input = CreateProjectInput {
            name: "clone-project".to_string(),
            description: None,
            color: Some("#00FF00".to_string()),
        };
        assert_eq!(input.name, "clone-project");
        assert_eq!(input.color, Some("#00FF00".to_string()));
    }
}
