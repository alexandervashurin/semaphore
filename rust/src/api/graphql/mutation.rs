//! GraphQL Mutation корень — CRUD операции

use async_graphql::{Context, Object, Result, InputObject};
use chrono::Utc;
use crate::api::state::AppState;
use crate::db::store::{UserManager, ProjectStore, TemplateManager, TaskManager};
use crate::models::{User as DbUser, Project as DbProject, Template as DbTemplate, Task as DbTask};
use crate::models::template::{TemplateType, TemplateApp};
use crate::services::task_logger::TaskStatus;

use super::types::{User, Project, Template, Task};

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
    async fn create_project(&self, ctx: &Context<'_>, input: CreateProjectInput) -> Result<Project> {
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
    async fn create_template(&self, ctx: &Context<'_>, input: CreateTemplateInput) -> Result<Template> {
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
            r#type: TemplateType::Ansible,
            created: Utc::now(),
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
        })
    }

    /// Обновить шаблон
    async fn update_template(&self, ctx: &Context<'_>, id: i32, name: String, playbook: String) -> Result<Template> {
        let state = ctx.data::<AppState>()?;
        let store = &state.store;

        // Получаем текущий шаблон
        let templates = store.get_templates(1).await?; // TODO: получить project_id из шаблона
        let template = templates.iter().find(|t| t.id == id)
            .ok_or_else(|| async_graphql::Error::new("Template not found"))?;

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
        let template = templates.iter().find(|t| t.id == id)
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
        let task = tasks.iter().find(|t| t.task.id == id)
            .ok_or_else(|| async_graphql::Error::new("Task not found"))?;
        let project_id = task.task.project_id;

        store.delete_task(project_id, id).await?;
        Ok(true)
    }
}
