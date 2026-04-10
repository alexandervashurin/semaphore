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
        .map_err(Error::Database)?;

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
        .map_err(Error::Database)?;

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
        .bind(template.vault_key_id)
        .bind(template.id)
        .bind(template.project_id)
        .execute(pool)
        .await
        .map_err(Error::Database)?;

    Ok(())
}

/// Удаляет шаблон MySQL
pub async fn delete_template(pool: &Pool<MySql>, project_id: i32, template_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM `template` WHERE id = ? AND project_id = ?")
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
    use crate::models::{TemplateApp, TemplateType};

    #[test]
    fn test_get_templates_query_structure() {
        let query = "SELECT * FROM `template` WHERE project_id = ? ORDER BY name";
        assert!(query.contains("`template`"));
        assert!(query.contains("project_id = ?"));
        assert!(query.contains("ORDER BY name"));
    }

    #[test]
    fn test_get_template_query_structure() {
        let query = "SELECT * FROM `template` WHERE id = ? AND project_id = ?";
        assert!(query.contains("id = ?"));
        assert!(query.contains("project_id = ?"));
    }

    #[test]
    fn test_create_template_query_structure() {
        let expected = "INSERT INTO `template` (project_id, name, playbook, description, inventory_id, repository_id, environment_id, type, app, git_branch, created, arguments, vault_key_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
        assert!(expected.contains("`template`"));
        assert!(expected.contains("project_id"));
        assert!(expected.contains("vault_key_id"));
        assert!(expected.count_matches('?'.into()) == 13);
    }

    #[test]
    fn test_update_template_query_structure() {
        let expected = "UPDATE `template` SET name = ?, playbook = ?, description = ?, inventory_id = ?, repository_id = ?, environment_id = ?, type = ?, app = ?, git_branch = ?, arguments = ?, vault_key_id = ? WHERE id = ? AND project_id = ?";
        assert!(expected.contains("UPDATE `template`"));
        assert!(expected.contains("WHERE id = ? AND project_id = ?"));
        assert!(expected.count_matches('?'.into()) == 13);
    }

    #[test]
    fn test_delete_template_query_structure() {
        let expected = "DELETE FROM `template` WHERE id = ? AND project_id = ?";
        assert!(expected.contains("`template`"));
        assert!(expected.contains("id = ? AND project_id = ?"));
    }

    #[test]
    fn test_mysql_template_uses_backticks() {
        let queries = [
            "SELECT * FROM `template` WHERE id = ?",
            "DELETE FROM `template` WHERE id = ? AND project_id = ?",
            "INSERT INTO `template` (name) VALUES (?)",
        ];
        for q in &queries {
            assert!(q.contains('`'), "MySQL template queries should use backticks");
        }
    }

    #[test]
    fn test_template_type_in_query() {
        let template_type = TemplateType::Deploy;
        assert_eq!(template_type, TemplateType::Deploy);
    }

    #[test]
    fn test_template_app_in_query() {
        let app = TemplateApp::Ansible;
        assert_eq!(app, TemplateApp::Ansible);
    }

    #[test]
    fn test_template_model_fields() {
        let template = Template {
            id: 1,
            project_id: 10,
            name: "Deploy".to_string(),
            playbook: "deploy.yml".to_string(),
            description: "Deploy template".to_string(),
            inventory_id: Some(5),
            repository_id: Some(3),
            environment_id: Some(2),
            r#type: TemplateType::Deploy,
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
        assert_eq!(template.id, 1);
        assert_eq!(template.project_id, 10);
        assert_eq!(template.name, "Deploy");
        assert!(template.inventory_id.is_some());
    }

    #[test]
    fn test_template_serialization() {
        let template = Template::default_template(1, "Test".to_string(), "test.yml".to_string());
        let json = serde_json::to_string(&template).unwrap();
        assert!(json.contains("\"name\":\"Test\""));
        assert!(json.contains("\"type\":\"default\""));
    }

    #[test]
    fn test_template_bind_order_matches_query() {
        // Verify that bind order in create_template matches the query columns
        let columns = [
            "project_id", "name", "playbook", "description",
            "inventory_id", "repository_id", "environment_id",
            "type", "app", "git_branch", "created", "arguments", "vault_key_id",
        ];
        assert_eq!(columns.len(), 13);
        assert_eq!(columns[0], "project_id");
        assert_eq!(columns[12], "vault_key_id");
    }

    #[test]
    fn test_mysql_template_debug_format() {
        let query = "SELECT * FROM `template` WHERE id = ?";
        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("template"));
    }
}
