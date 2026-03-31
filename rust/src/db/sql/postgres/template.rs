//! PostgreSQL Template CRUD operations

use crate::error::{Error, Result};
use crate::models::*;
use sqlx::{Pool, Postgres};

/// Получает все шаблоны проекта PostgreSQL
pub async fn get_templates(pool: &Pool<Postgres>, project_id: i32) -> Result<Vec<Template>> {
    let query = "SELECT * FROM template WHERE project_id = $1 ORDER BY name";

    let templates = sqlx::query_as::<_, Template>(query)
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(Error::Database)?;

    Ok(templates)
}

/// Получает шаблон по ID PostgreSQL
pub async fn get_template(
    pool: &Pool<Postgres>,
    project_id: i32,
    template_id: i32,
) -> Result<Template> {
    let query = "SELECT * FROM template WHERE id = $1 AND project_id = $2";

    let template = sqlx::query_as::<_, Template>(query)
        .bind(template_id)
        .bind(project_id)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound("Template not found".to_string()),
            _ => Error::Database(e),
        })?;

    Ok(template)
}

/// Создаёт шаблон PostgreSQL
pub async fn create_template(pool: &Pool<Postgres>, mut template: Template) -> Result<Template> {
    let query = "INSERT INTO template (project_id, name, playbook, description, inventory_id, repository_id, environment_id, type, app, git_branch, created, arguments, vault_key_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING id";

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
        .fetch_one(pool)
        .await
        .map_err(Error::Database)?;

    template.id = id;
    Ok(template)
}

/// Обновляет шаблон PostgreSQL
pub async fn update_template(pool: &Pool<Postgres>, template: Template) -> Result<()> {
    let query = "UPDATE template SET name = $1, playbook = $2, description = $3, inventory_id = $4, repository_id = $5, environment_id = $6, type = $7, app = $8, git_branch = $9, arguments = $10, vault_key_id = $11 WHERE id = $12 AND project_id = $13";

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
        .bind(template.id)
        .bind(template.project_id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}

/// Удаляет шаблон PostgreSQL
pub async fn delete_template(
    pool: &Pool<Postgres>,
    project_id: i32,
    template_id: i32,
) -> Result<()> {
    sqlx::query("DELETE FROM template WHERE id = $1 AND project_id = $2")
        .bind(template_id)
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}
