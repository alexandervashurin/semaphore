//! Template CRUD - операции с шаблонами
//!
//! Аналог db/sql/template.go из Go версии (часть 1: CRUD)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    /// Получает все шаблоны проекта
    pub async fn get_templates(&self, project_id: i32) -> Result<Vec<Template>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let templates = sqlx::query_as::<_, Template>(
                    "SELECT * FROM template WHERE project_id = ?"
                )
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(templates)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Получает шаблон по ID
    pub async fn get_template(&self, project_id: i32, template_id: i32) -> Result<Template> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let template = sqlx::query_as::<_, Template>(
                    "SELECT * FROM template WHERE project_id = ? AND id = ?"
                )
                .bind(project_id)
                .bind(template_id)
                .fetch_one(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(template)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Создаёт новый шаблон
    pub async fn create_template(&self, mut template: Template) -> Result<Template> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO template (
                        project_id, name, playbook, arguments, type, 
                        inventory_id, repository_id, environment_id,
                        start_version, build_version, description,
                        survey_vars, vaults, tasks, created
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
                )
                .bind(template.project_id)
                .bind(&template.name)
                .bind(&template.playbook)
                .bind(&template.arguments)
                .bind(template.template_type.to_string())
                .bind(template.inventory_id)
                .bind(template.repository_id)
                .bind(template.environment_id)
                .bind(&template.start_version)
                .bind(&template.build_version)
                .bind(&template.description)
                .bind(&template.survey_vars)
                .bind(&template.vaults)
                .bind(template.tasks)
                .bind(template.created)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                template.id = result.last_insert_rowid() as i32;
                Ok(template)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Обновляет шаблон
    pub async fn update_template(&self, template: Template) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE template SET 
                        name = ?, playbook = ?, arguments = ?, type = ?,
                        inventory_id = ?, repository_id = ?, environment_id = ?,
                        start_version = ?, build_version = ?, description = ?,
                        survey_vars = ?, vaults = ?, tasks = ?
                    WHERE id = ? AND project_id = ?"
                )
                .bind(&template.name)
                .bind(&template.playbook)
                .bind(&template.arguments)
                .bind(template.template_type.to_string())
                .bind(template.inventory_id)
                .bind(template.repository_id)
                .bind(template.environment_id)
                .bind(&template.start_version)
                .bind(&template.build_version)
                .bind(&template.description)
                .bind(&template.survey_vars)
                .bind(&template.vaults)
                .bind(template.tasks)
                .bind(template.id)
                .bind(template.project_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Удаляет шаблон
    pub async fn delete_template(&self, project_id: i32, template_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query("DELETE FROM template WHERE id = ? AND project_id = ?")
                    .bind(template_id)
                    .bind(project_id)
                    .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                    .await
                    .map_err(|e| Error::Database(e))?;
                
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use chrono::Utc;

    async fn create_test_db() -> SqlDb {
        let temp_db = env::temp_dir().join("test_template.db");
        let db_path = temp_db.to_string_lossy().to_string();
        
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
                inventory_id INTEGER,
                repository_id INTEGER,
                environment_id INTEGER,
                start_version TEXT,
                build_version TEXT,
                description TEXT,
                survey_vars TEXT,
                vaults TEXT,
                tasks INTEGER NOT NULL,
                created DATETIME NOT NULL
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();
        
        db
    }

    #[tokio::test]
    async fn test_create_and_get_template() {
        let db = create_test_db().await;
        
        let template = Template {
            id: 0,
            project_id: 1,
            name: "Test Template".to_string(),
            playbook: "test.yml".to_string(),
            arguments: None,
            template_type: TemplateType::Task,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            start_version: None,
            build_version: None,
            description: None,
            survey_vars: None,
            vaults: Vec::new(),
            tasks: 0,
            created: Utc::now(),
        };
        
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
        let db = create_test_db().await;
        
        // Создаём несколько шаблонов
        for i in 0..5 {
            let template = Template {
                id: 0,
                project_id: 1,
                name: format!("Template {}", i),
                playbook: format!("test{}.yml", i),
                arguments: None,
                template_type: TemplateType::Task,
                inventory_id: None,
                repository_id: None,
                environment_id: None,
                start_version: None,
                build_version: None,
                description: None,
                survey_vars: None,
                vaults: Vec::new(),
                tasks: 0,
                created: Utc::now(),
            };
            db.create_template(template).await.unwrap();
        }
        
        let templates = db.get_templates(1).await.unwrap();
        assert!(templates.len() >= 5);
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_update_template() {
        let db = create_test_db().await;
        
        let template = Template {
            id: 0,
            project_id: 1,
            name: "Test Template".to_string(),
            playbook: "test.yml".to_string(),
            arguments: None,
            template_type: TemplateType::Task,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            start_version: None,
            build_version: None,
            description: None,
            survey_vars: None,
            vaults: Vec::new(),
            tasks: 0,
            created: Utc::now(),
        };
        
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
        let db = create_test_db().await;
        
        let template = Template {
            id: 0,
            project_id: 1,
            name: "Test Template".to_string(),
            playbook: "test.yml".to_string(),
            arguments: None,
            template_type: TemplateType::Task,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            start_version: None,
            build_version: None,
            description: None,
            survey_vars: None,
            vaults: Vec::new(),
            tasks: 0,
            created: Utc::now(),
        };
        
        let created = db.create_template(template).await.unwrap();
        
        db.delete_template(1, created.id).await.unwrap();
        
        let result = db.get_template(1, created.id).await;
        assert!(result.is_err());
        
        // Cleanup
        let _ = db.close().await;
    }
}
