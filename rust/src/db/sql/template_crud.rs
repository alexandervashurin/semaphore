//! Template CRUD - операции с шаблонами
//!
//! Аналог db/sql/template.go из Go версии (часть 1: CRUD)
//!
//! DEPRECATED: Используйте модули sqlite::template, postgres::template, mysql::template

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    /// Получает все шаблоны проекта
    pub async fn get_templates(&self, project_id: i32) -> Result<Vec<Template>> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::template::get_templates(pool, project_id).await
    }

    /// Получает шаблон по ID
    pub async fn get_template(&self, project_id: i32, template_id: i32) -> Result<Template> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::template::get_template(pool, project_id, template_id).await
    }

    /// Создаёт новый шаблон
    pub async fn create_template(&self, mut template: Template) -> Result<Template> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::template::create_template(pool, template).await
    }

    /// Обновляет шаблон
    pub async fn update_template(&self, template: Template) -> Result<()> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::template::update_template(pool, template).await
    }

    /// Удаляет шаблон
    pub async fn delete_template(&self, project_id: i32, template_id: i32) -> Result<()> {
        let pool = self
            .get_postgres_pool()
            .ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
        crate::db::sql::postgres::template::delete_template(pool, project_id, template_id).await
    }
}

// Legacy code removed — now uses decomposed modules (postgres::template).

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_type_display() {
        assert_eq!(TemplateType::Default.to_string(), "default");
        assert_eq!(TemplateType::Ansible.to_string(), "ansible");
        assert_eq!(TemplateType::Terraform.to_string(), "terraform");
        assert_eq!(TemplateType::Shell.to_string(), "shell");
    }

    #[test]
    fn test_template_type_serialization() {
        let json = serde_json::to_string(&TemplateType::Build).unwrap();
        assert_eq!(json, "\"build\"");
    }

    #[test]
    fn test_template_type_deserialization() {
        let t: TemplateType = serde_json::from_str("\"deploy\"").unwrap();
        assert_eq!(t, TemplateType::Deploy);
    }

    #[test]
    fn test_template_app_display() {
        assert_eq!(TemplateApp::Ansible.to_string(), "ansible");
        assert_eq!(TemplateApp::Terraform.to_string(), "terraform");
        assert_eq!(TemplateApp::Bash.to_string(), "bash");
        assert_eq!(TemplateApp::Python.to_string(), "python");
    }

    #[test]
    fn test_template_app_serialization() {
        let json = serde_json::to_string(&TemplateApp::Tofu).unwrap();
        assert_eq!(json, "\"tofu\"");
    }

    #[test]
    fn test_template_app_deserialization() {
        let app: TemplateApp = serde_json::from_str("\"terragrunt\"").unwrap();
        assert_eq!(app, TemplateApp::Terragrunt);
    }

    #[test]
    fn test_template_default_constructor() {
        let tpl = Template::default_template(10, "Test".to_string(), "test.yml".to_string());
        assert_eq!(tpl.project_id, 10);
        assert_eq!(tpl.name, "Test");
        assert_eq!(tpl.playbook, "test.yml");
        assert!(!tpl.autorun);
        assert!(!tpl.require_approval);
    }

    #[test]
    fn test_template_serialization() {
        let tpl = Template::default_template(1, "deploy".to_string(), "deploy.yml".to_string());
        let json = serde_json::to_string(&tpl).unwrap();
        assert!(json.contains("\"name\":\"deploy\""));
        assert!(json.contains("\"playbook\":\"deploy.yml\""));
    }

    #[test]
    fn test_template_clone() {
        let tpl = Template::default_template(1, "clone".to_string(), "clone.yml".to_string());
        let cloned = tpl.clone();
        assert_eq!(cloned.name, tpl.name);
        assert_eq!(cloned.app, tpl.app);
    }

    #[test]
    fn test_template_role_perm_creation() {
        let perm = TemplateRolePerm {
            id: 1,
            project_id: 1,
            template_id: 5,
            role_id: 10,
            role_slug: "admin".to_string(),
        };
        assert_eq!(perm.template_id, 5);
        assert_eq!(perm.role_slug, "admin");
    }

    #[test]
    fn test_template_filter_default() {
        let filter = TemplateFilter::default();
        assert!(filter.project_id.is_none());
        assert!(filter.r#type.is_none());
        assert!(filter.app.is_none());
    }

    #[test]
    fn test_template_type_equality() {
        assert_eq!(TemplateType::Ansible, TemplateType::Ansible);
        assert_ne!(TemplateType::Ansible, TemplateType::Terraform);
    }

    #[test]
    fn test_template_app_equality() {
        assert_eq!(TemplateApp::Bash, TemplateApp::Bash);
        assert_ne!(TemplateApp::Bash, TemplateApp::Python);
    }
}
