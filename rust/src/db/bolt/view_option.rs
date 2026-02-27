//! View, Option - операции в BoltDB
//!
//! Аналог db/bolt/view.go, option.go из Go версии

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::Result;
use crate::models::{View, RetrieveQueryParams};

// ============================================================================
// View Operations
// ============================================================================

impl BoltStore {
    /// Получает представление по ID
    pub async fn get_view(&self, project_id: i32, view_id: i32) -> Result<View> {
        self.get_object(project_id, "views", view_id).await
    }

    /// Получает представления проекта
    pub async fn get_views(&self, project_id: i32) -> Result<Vec<View>> {
        self.get_objects::<View>(project_id, "views", RetrieveQueryParams {
            offset: 0,
            count: 1000,
            filter: String::new(),
        }).await
    }

    /// Обновляет представление
    pub async fn update_view(&self, view: View) -> Result<()> {
        self.update_object(view.project_id, "views", view).await
    }

    /// Создаёт представление
    pub async fn create_view(&self, mut view: View) -> Result<View> {
        view.created = chrono::Utc::now();
        
        let view_clone = view.clone();
        
        let new_view = self.db.update(|tx| {
            let bucket = tx.create_bucket_if_not_exists(b"views")?;
            
            let str = serde_json::to_vec(&view_clone)?;
            
            let id = bucket.next_sequence()?;
            let id = i64::MAX - id as i64;
            
            let mut view_with_id = view_clone;
            view_with_id.id = id as i32;
            
            let str = serde_json::to_vec(&view_with_id)?;
            bucket.put(id.to_be_bytes(), str)?;
            
            Ok(view_with_id)
        }).await?;
        
        Ok(new_view)
    }

    /// Удаляет представление
    pub async fn delete_view(&self, project_id: i32, view_id: i32) -> Result<()> {
        self.delete_object(project_id, "views", view_id).await
    }

    /// Устанавливает позиции представлений
    pub async fn set_view_positions(&self, project_id: i32, positions: std::collections::HashMap<i32, i32>) -> Result<()> {
        for (view_id, position) in positions {
            let mut view = self.get_view(project_id, view_id).await?;
            view.position = position;
            self.update_view(view).await?;
        }
        Ok(())
    }
}

// ============================================================================
// Option Operations
// ============================================================================

impl BoltStore {
    /// Получает все опции
    pub async fn get_options(&self, params: RetrieveQueryParams) -> Result<std::collections::HashMap<String, String>> {
        let mut res = std::collections::HashMap::new();

        let all_options = self.get_objects::<crate::models::OptionItem>(0, "options", params).await?;

        for opt in all_options {
            if params.filter.is_empty() {
                res.insert(opt.key, opt.value);
            } else if opt.key == params.filter || opt.key.starts_with(&format!("{}.", params.filter)) {
                res.insert(opt.key, opt.value);
            }
        }

        Ok(res)
    }

    /// Устанавливает опцию
    pub async fn set_option(&self, key: &str, value: &str) -> Result<()> {
        // Проверяем существует ли опция
        match self.get_option(key).await {
            Ok(_) => {
                // Обновляем существующую
                let opt = crate::models::OptionItem {
                    key: key.to_string(),
                    value: value.to_string(),
                };
                self.update_object(-1, "options", opt).await
            }
            Err(_) => {
                // Создаём новую
                let opt = crate::models::OptionItem {
                    key: key.to_string(),
                    value: value.to_string(),
                };
                self.create_object(-1, "options", opt).await?;
                Ok(())
            }
        }
    }

    /// Получает опцию по ключу
    pub async fn get_option(&self, key: &str) -> Result<String> {
        let options = self.get_objects::<crate::models::OptionItem>(0, "options", RetrieveQueryParams {
            offset: 0,
            count: 1000,
            filter: String::new(),
        }).await?;
        
        for opt in options {
            if opt.key == key {
                return Ok(opt.value);
            }
        }
        
        Err(crate::error::Error::NotFound("Опция не найдена".to_string()))
    }

    /// Удаляет опцию
    pub async fn delete_option(&self, key: &str) -> Result<()> {
        self.delete_object(-1, "options", key).await
    }

    /// Удаляет опции по фильтру
    pub async fn delete_options(&self, filter: &str) -> Result<()> {
        let options = self.get_options(RetrieveQueryParams {
            offset: 0,
            count: 1000,
            filter: filter.to_string(),
        }).await?;
        
        for (key, _) in options {
            if key == filter || key.starts_with(&format!("{}.", filter)) {
                self.delete_option(&key).await?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;

    fn create_test_bolt_db() -> BoltStore {
        let path = PathBuf::from("/tmp/test_view_option.db");
        BoltStore::new(path).unwrap()
    }

    fn create_test_view(project_id: i32, name: &str) -> View {
        View {
            id: 0,
            project_id,
            name: name.to_string(),
            position: 0,
            created: Utc::now(),
        }
    }

    // View Tests
    #[tokio::test]
    async fn test_create_view() {
        let db = create_test_bolt_db();
        let view = create_test_view(1, "Test View");
        
        let result = db.create_view(view).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_view() {
        let db = create_test_bolt_db();
        let view = create_test_view(1, "Test View");
        let created = db.create_view(view).await.unwrap();
        
        let retrieved = db.get_view(1, created.id).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().name, "Test View");
    }

    #[tokio::test]
    async fn test_get_views() {
        let db = create_test_bolt_db();
        
        // Создаём несколько представлений
        for i in 0..5 {
            let view = create_test_view(1, &format!("View {}", i));
            db.create_view(view).await.unwrap();
        }
        
        let views = db.get_views(1).await;
        assert!(views.is_ok());
        assert!(views.unwrap().len() >= 5);
    }

    #[tokio::test]
    async fn test_update_view() {
        let db = create_test_bolt_db();
        let view = create_test_view(1, "Test View");
        let mut created = db.create_view(view).await.unwrap();
        
        created.name = "Updated View".to_string();
        let result = db.update_view(created).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_view_positions() {
        let db = create_test_bolt_db();
        let view = create_test_view(1, "Test View");
        let created = db.create_view(view).await.unwrap();
        
        let mut positions = std::collections::HashMap::new();
        positions.insert(created.id, 10);
        
        let result = db.set_view_positions(1, positions).await;
        assert!(result.is_ok());
    }

    // Option Tests
    #[tokio::test]
    async fn test_set_option() {
        let db = create_test_bolt_db();
        
        let result = db.set_option("test.key", "test_value").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_option() {
        let db = create_test_bolt_db();
        db.set_option("test.key", "test_value").await.unwrap();
        
        let value = db.get_option("test.key").await;
        assert!(value.is_ok());
        assert_eq!(value.unwrap(), "test_value");
    }

    #[tokio::test]
    async fn test_get_options() {
        let db = create_test_bolt_db();
        
        // Устанавливаем несколько опций
        db.set_option("app.name", "Test App").await.unwrap();
        db.set_option("app.version", "1.0.0").await.unwrap();
        
        let params = RetrieveQueryParams {
            offset: 0,
            count: 100,
            filter: "app".to_string(),
        };
        
        let options = db.get_options(params).await;
        assert!(options.is_ok());
        assert!(options.unwrap().len() >= 2);
    }

    #[tokio::test]
    async fn test_delete_option() {
        let db = create_test_bolt_db();
        db.set_option("test.delete", "value").await.unwrap();
        
        let result = db.delete_option("test.delete").await;
        assert!(result.is_ok());
        
        let value = db.get_option("test.delete").await;
        assert!(value.is_err());
    }
}
