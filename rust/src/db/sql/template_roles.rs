//! Template Roles - операции с TemplateRole
//!
//! Аналог db/sql/template.go из Go версии (часть 3: TemplateRole)

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::*;
use sqlx::Row;

impl SqlDb {
    /// Получает все роли для шаблона
    pub async fn get_template_roles(&self, project_id: i32, template_id: i32) -> Result<Vec<TemplateRolePerm>> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let roles = sqlx::query_as::<_, TemplateRolePerm>(
                    "SELECT * FROM template_role WHERE template_id = ? AND project_id = ?"
                )
                .bind(template_id)
                .bind(project_id)
                .fetch_all(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(roles)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Создаёт TemplateRole
    pub async fn create_template_role(&self, mut role: TemplateRolePerm) -> Result<TemplateRolePerm> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                let result = sqlx::query(
                    "INSERT INTO template_role (template_id, project_id, role_id) VALUES (?, ?, ?)"
                )
                .bind(role.template_id)
                .bind(role.project_id)
                .bind(role.role_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                role.id = result.last_insert_rowid() as i32;
                Ok(role)
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Обновляет TemplateRole
    pub async fn update_template_role(&self, role: TemplateRolePerm) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "UPDATE template_role SET role_id = ? WHERE id = ? AND template_id = ? AND project_id = ?"
                )
                .bind(role.role_id)
                .bind(role.id)
                .bind(role.template_id)
                .bind(role.project_id)
                .execute(self.get_sqlite_pool().ok_or(Error::Other("SQLite pool not found".to_string()))?)
                .await
                .map_err(|e| Error::Database(e))?;
                
                Ok(())
            }
            _ => Err(Error::Other("Only SQLite supported for now".to_string()))
        }
    }
    
    /// Удаляет TemplateRole
    pub async fn delete_template_role(&self, project_id: i32, template_id: i32, role_id: i32) -> Result<()> {
        match self.get_dialect() {
            crate::db::sql::types::SqlDialect::SQLite => {
                sqlx::query(
                    "DELETE FROM template_role WHERE id = ? AND template_id = ? AND project_id = ?"
                )
                .bind(role_id)
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

    async fn create_test_db() -> SqlDb {
        let temp_db = env::temp_dir().join("test_template_role.db");
        let db_path = temp_db.to_string_lossy().to_string();
        
        let db = SqlDb::connect_sqlite(&db_path).await.unwrap();
        
        // Создаём таблицу template_role
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS template_role (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                template_id INTEGER NOT NULL,
                project_id INTEGER NOT NULL,
                role_id INTEGER NOT NULL
            )"
        )
        .execute(db.get_sqlite_pool().unwrap())
        .await
        .unwrap();
        
        db
    }

    #[tokio::test]
    async fn test_create_and_get_template_role() {
        let db = create_test_db().await;
        
        let role = TemplateRolePerm {
            id: 0,
            template_id: 1,
            project_id: 1,
            role_id: 2,
        };
        
        let created = db.create_template_role(role.clone()).await.unwrap();
        assert!(created.id > 0);
        
        let roles = db.get_template_roles(1, 1).await.unwrap();
        assert!(roles.len() >= 1);
        assert_eq!(roles[0].role_id, 2);
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_update_template_role() {
        let db = create_test_db().await;
        
        let role = TemplateRolePerm {
            id: 0,
            template_id: 1,
            project_id: 1,
            role_id: 2,
        };
        
        let created = db.create_template_role(role).await.unwrap();
        
        let mut updated = created.clone();
        updated.role_id = 3;
        
        db.update_template_role(updated).await.unwrap();
        
        let roles = db.get_template_roles(1, 1).await.unwrap();
        assert_eq!(roles[0].role_id, 3);
        
        // Cleanup
        let _ = db.close().await;
    }

    #[tokio::test]
    async fn test_delete_template_role() {
        let db = create_test_db().await;
        
        let role = TemplateRolePerm {
            id: 0,
            template_id: 1,
            project_id: 1,
            role_id: 2,
        };
        
        let created = db.create_template_role(role).await.unwrap();
        
        db.delete_template_role(1, 1, created.id).await.unwrap();
        
        let roles = db.get_template_roles(1, 1).await.unwrap();
        assert!(roles.is_empty());
        
        // Cleanup
        let _ = db.close().await;
    }
}
