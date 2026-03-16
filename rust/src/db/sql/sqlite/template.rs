//! SQLite Template CRUD operations

use crate::error::{Error, Result};
use crate::models::*;
use chrono::{DateTime, Utc};
use sqlx::{Row, Sqlite, Pool};

/// Получает все шаблоны проекта SQLite
pub async fn get_templates(pool: &Pool<Sqlite>, project_id: i32) -> Result<Vec<Template>> {
    let query = "SELECT * FROM template WHERE project_id = ? ORDER BY name";
    
    let templates = sqlx::query_as::<_, Template>(query)
        .bind(project_id)
        .fetch_all(pool)
        .await
        .map_err(Error::Database)?;

    Ok(templates)
}

/// Получает шаблон по ID SQLite
pub async fn get_template(pool: &Pool<Sqlite>, project_id: i32, template_id: i32) -> Result<Template> {
    let query = "SELECT * FROM template WHERE id = ? AND project_id = ?";
    
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

/// Создаёт шаблон SQLite
pub async fn create_template(pool: &Pool<Sqlite>, mut template: Template) -> Result<Template> {
    let query = "INSERT INTO template (project_id, name, playbook, description, inventory_id, repository_id, environment_id, type, app, git_branch, created, arguments, vault_key_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id";
    
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

/// Обновляет шаблон SQLite
pub async fn update_template(pool: &Pool<Sqlite>, template: Template) -> Result<()> {
    let query = "UPDATE template SET name = ?, playbook = ?, description = ?, inventory_id = ?, repository_id = ?, environment_id = ?, type = ?, app = ?, git_branch = ?, arguments = ?, vault_key_id = ? WHERE id = ? AND project_id = ?";
    
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

/// Удаляет шаблон SQLite
pub async fn delete_template(pool: &Pool<Sqlite>, project_id: i32, template_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM template WHERE id = ? AND project_id = ?")
        .bind(template_id)
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::sql::test_helpers::create_test_pool;

    #[tokio::test]
    async fn test_sqlite_template_crud() {
        let (pool, _temp) = create_test_pool().await.unwrap();
        crate::db::sql::test_helpers::init_project_table(&pool).await.unwrap();
        crate::db::sql::test_helpers::init_template_table(&pool).await.unwrap();

        // Сначала создаём проект
        let project_query = "INSERT INTO project (name, created, alert, max_parallel_tasks, type) VALUES (?, ?, ?, ?, ?)";
        let result = sqlx::query(project_query)
            .bind("Test Project")
            .bind(Utc::now())
            .bind(false)
            .bind(0i32)
            .bind("default")
            .execute(&pool)
            .await
            .unwrap();
        let project_id = result.last_insert_rowid() as i32;
        
        // Create
        let template = Template {
            id: 0,
            project_id,
            name: "Test Template".to_string(),
            playbook: "test.yml".to_string(),
            description: "Test Description".to_string(),
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            r#type: TemplateType::Default,
            app: TemplateApp::Ansible,
            git_branch: Some("main".to_string()),
            created: Utc::now(),
            arguments: None,
            vault_key_id: None,
            view_id: None,
            build_template_id: None,
            autorun: false,
            allow_override_args_in_task: false,
            allow_override_branch_in_task: false,
            allow_inventory_in_task: false,
            allow_parallel_tasks: false,
            suppress_success_alerts: false,
            task_params: None,
            survey_vars: None,
            vaults: None,
        };
        
        let created = create_template(&pool, template).await.unwrap();
        assert!(created.id > 0);
        
        // Read
        let retrieved = get_template(&pool, project_id, created.id).await.unwrap();
        assert_eq!(retrieved.name, "Test Template");
        
        // Update
        let mut updated = retrieved;
        updated.name = "Updated Template".to_string();
        update_template(&pool, updated).await.unwrap();
        
        // Delete
        delete_template(&pool, project_id, created.id).await.unwrap();
    }
}
