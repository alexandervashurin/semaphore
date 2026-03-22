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
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::template::get_templates(pool, project_id).await
    }

    /// Получает шаблон по ID
    pub async fn get_template(&self, project_id: i32, template_id: i32) -> Result<Template> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::template::get_template(pool, project_id, template_id).await
    }

    /// Создаёт новый шаблон
    pub async fn create_template(&self, mut template: Template) -> Result<Template> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::template::create_template(pool, template).await
    }

    /// Обновляет шаблон
    pub async fn update_template(&self, template: Template) -> Result<()> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::template::update_template(pool, template).await
    }

    /// Удаляет шаблон
    pub async fn delete_template(&self, project_id: i32, template_id: i32) -> Result<()> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::template::delete_template(pool, project_id, template_id).await
    }
}

// Legacy code removed - now uses decomposed modules

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    struct TestDb {
        db: SqlDb,
        _temp: tempfile::NamedTempFile,
    }

    async fn create_test_db() -> TestDb {
        let (db_path, temp) = crate::db::sql::init::test_sqlite_url();

        let db = SqlDb::connect_sqlite(&db_path).await.unwrap();

        // Создаём таблицу template
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS template (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                playbook TEXT NOT NULL,
                arguments TEXT,
                type TEXT NOT NULL,
                app TEXT NOT NULL DEFAULT 'ansible',
                git_branch TEXT DEFAULT '',
                deleted INTEGER NOT NULL DEFAULT 0,
                inventory_id INTEGER,
                repository_id INTEGER,
                environment_id INTEGER,
                start_version TEXT,
                build_version TEXT,
                description TEXT,
                survey_vars TEXT,
                vaults TEXT,
                tasks INTEGER NOT NULL DEFAULT 0,
                vault_key_id INTEGER,
                become_key_id INTEGER,
                created DATETIME NOT NULL,
                view_id INTEGER,
                build_template_id INTEGER,
                autorun INTEGER NOT NULL DEFAULT 0,
                allow_override_args_in_task INTEGER NOT NULL DEFAULT 0,
                allow_override_branch_in_task INTEGER NOT NULL DEFAULT 0,
                allow_inventory_in_task INTEGER NOT NULL DEFAULT 0,
                allow_parallel_tasks INTEGER NOT NULL DEFAULT 0,
                suppress_success_alerts INTEGER NOT NULL DEFAULT 0,
                task_params TEXT
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();

        TestDb { db, _temp: temp }
    }

    #[tokio::test]
    async fn test_create_and_get_template() {
        let TestDb { db, _temp } = create_test_db().await;
        
        let mut template = Template::default();
        template.project_id = 1;
        template.name = "Test Template".to_string();
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.created = Utc::now();
        
        let created = db.create_template(template.clone()).await.unwrap();
        assert!(created.id > 0);
        
        let retrieved = db.get_template(1, created.id).await.unwrap();
        assert_eq!(retrieved.name, "Test Template");
        assert_eq!(retrieved.playbook, "test.yml");
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_get_templates() {
        let TestDb { db, _temp } = create_test_db().await;
        
        // Создаём несколько шаблонов
        for i in 0..5 {
            let mut template = Template::default();
            template.project_id = 1;
            template.name = format!("Template {}", i);
            template.playbook = format!("test{}.yml", i);
            template.r#type = TemplateType::Task;
            template.created = Utc::now();
            db.create_template(template).await.unwrap();
        }
        
        let templates = db.get_templates(1).await.unwrap();
        assert!(templates.len() >= 5);
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_update_template() {
        let TestDb { db, _temp } = create_test_db().await;
        
        let mut template = Template::default();
        template.project_id = 1;
        template.name = "Test Template".to_string();
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.created = Utc::now();
        
        let created = db.create_template(template).await.unwrap();
        
        let mut updated = created.clone();
        updated.name = "Updated Template".to_string();
        updated.playbook = "updated.yml".to_string();
        
        db.update_template(updated).await.unwrap();
        
        let retrieved = db.get_template(1, created.id).await.unwrap();
        assert_eq!(retrieved.name, "Updated Template");
        assert_eq!(retrieved.playbook, "updated.yml");
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_delete_template() {
        let TestDb { db, _temp } = create_test_db().await;
        
        let mut template = Template::default();
        template.project_id = 1;
        template.name = "Test Template".to_string();
        template.playbook = "test.yml".to_string();
        template.r#type = TemplateType::Task;
        template.created = Utc::now();
        
        let created = db.create_template(template).await.unwrap();
        
        db.delete_template(1, created.id).await.unwrap();
        
        let result = db.get_template(1, created.id).await;
        assert!(result.is_err());
        
        // Cleanup
        let _ = db.close().await;
    }
}
