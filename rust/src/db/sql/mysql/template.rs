//! MySQL Template CRUD operations

use crate::error::{Error, Result};
use crate::models::*;
use sqlx::{Pool, MySql};

/// Получает все шаблоны проекта MySQL
pub async fn get_templates(pool: &Pool<MySql>, project_id: i32) -> Result<Vec<Template>> {
    let query = "SELECT * FROM `template` WHERE project_id = ? ORDER BY name";
    
    let templates = sqlx::query_as::<_, Template>(query)
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(|e| Error::Database(e))?;

    Ok(templates)
}

/// Получает шаблон по ID MySQL
pub async fn get_template(pool: &Pool<MySql>, project_id: i32, template_id: i32) -> Result<Template> {
    let query = "SELECT * FROM `template` WHERE id = ? AND project_id = ?";
    
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

/// Создаёт шаблон MySQL
pub async fn create_template(pool: &Pool<MySql>, mut template: Template) -> Result<Template> {
    let query = "INSERT INTO `template` (project_id, name, playbook, description, inventory_id, repository_id, environment_id, type, app, git_branch, created, arguments, vault_key_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
    
    let result = sqlx::query(query)
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
        .execute(pool)
        .await
        .map_err(|e| Error::Database(e))?;

    template.id = result.last_insert_id() as i32;
    Ok(template)
}

/// Обновляет шаблон MySQL
pub async fn update_template(pool: &Pool<MySql>, template: Template) -> Result<()> {
    let query = "UPDATE `template` SET name = ?, playbook = ?, description = ?, inventory_id = ?, repository_id = ?, environment_id = ?, type = ?, app = ?, git_branch = ?, arguments = ?, vault_key_id = ? WHERE id = ? AND project_id = ?";
    
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
        .bind(&template.vault_key_id)
        .bind(template.id)
        .bind(template.project_id)
        .execute(pool)
        .await
        .map_err(|e| Error::Database(e))?;

    Ok(())
}

/// Удаляет шаблон MySQL
pub async fn delete_template(pool: &Pool<MySql>, project_id: i32, template_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM `template` WHERE id = ? AND project_id = ?")
        .bind(template_id)
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(|e| Error::Database(e))?;

    Ok(())
}
