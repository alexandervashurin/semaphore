//! GraphQL Query корень - минимальная версия

use async_graphql::{Context, Object, Result};
use crate::api::state::AppState;
use crate::db::store::{UserManager, ProjectStore, TemplateManager, TaskManager};

use super::types::{User, Project, Template, Task};

/// Корневой тип для Query
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Получить всех пользователей
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        let state = ctx.data::<AppState>()?;
        let store = &state.store;
        
        let users = store.get_users(Default::default()).await?;
        Ok(users.into_iter().map(|u| User {
            id: u.id,
            username: u.username,
            name: u.name,
            email: u.email,
            admin: u.admin,
        }).collect())
    }

    /// Получить все проекты
    async fn projects(&self, ctx: &Context<'_>) -> Result<Vec<Project>> {
        let state = ctx.data::<AppState>()?;
        let store = &state.store;
        
        let projects = store.get_projects(None).await?;
        Ok(projects.into_iter().map(|p| Project {
            id: p.id,
            name: p.name,
        }).collect())
    }

    /// Получить шаблоны проекта
    async fn templates(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<Template>> {
        let state = ctx.data::<AppState>()?;
        let store = &state.store;
        
        let templates = store.get_templates(project_id).await?;
        Ok(templates.into_iter().map(|t| Template {
            id: t.id,
            project_id: t.project_id,
            name: t.name,
            playbook: t.playbook,
        }).collect())
    }

    /// Получить задачи проекта
    async fn tasks(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<Task>> {
        let state = ctx.data::<AppState>()?;
        let store = &state.store;

        let tasks = store.get_tasks(project_id, None).await?;
        Ok(tasks.into_iter().map(|t| Task {
            id: t.task.id,
            template_id: t.task.template_id,
            project_id: t.task.project_id,
            status: t.task.status.to_string(),
        }).collect())
    }

    /// Ping для проверки
    async fn ping(&self) -> Result<String> {
        Ok("pong".to_string())
    }
}
