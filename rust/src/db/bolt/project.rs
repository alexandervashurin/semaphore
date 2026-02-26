//! Project - операции с проектами в BoltDB
//!
//! Аналог db/bolt/project.go из Go версии

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::Result;
use crate::models::Project;
use chrono::Utc;

impl BoltStore {
    /// Создаёт новый проект
    pub async fn create_project(&self, mut project: Project) -> Result<Project> {
        project.created = Utc::now();
        
        let project_clone = project.clone();
        
        let new_project = self.db.update(|tx| {
            let bucket = tx.create_bucket_if_not_exists(b"projects")?;
            
            let str = serde_json::to_vec(&project_clone)?;
            
            let id = bucket.next_sequence()?;
            let id = i64::MAX - id as i64;
            
            let mut project_with_id = project_clone;
            project_with_id.id = id as i32;
            
            let str = serde_json::to_vec(&project_with_id)?;
            bucket.put(id.to_be_bytes(), str)?;
            
            Ok(project_with_id)
        }).await?;
        
        Ok(new_project)
    }

    /// Получает все проекты
    pub async fn get_all_projects(&self) -> Result<Vec<Project>> {
        self.get_objects::<Project>(0, "projects", crate::models::RetrieveQueryParams {
            offset: 0,
            count: 1000,
            filter: String::new(),
        }).await
    }

    /// Получает проекты пользователя
    pub async fn get_projects(&self, user_id: i32) -> Result<Vec<Project>> {
        let all_projects = self.get_all_projects().await?;
        
        let mut projects = Vec::new();
        
        for project in all_projects {
            // Проверяем права пользователя
            match self.get_project_user(project.id, user_id).await {
                Ok(_) => projects.push(project),
                Err(crate::error::Error::NotFound(_)) => continue,
                Err(e) => return Err(e),
            }
        }
        
        Ok(projects)
    }

    /// Получает проект по ID
    pub async fn get_project(&self, project_id: i32) -> Result<Project> {
        self.get_object(0, "projects", project_id).await
    }

    /// Удаляет проект
    pub async fn delete_project(&self, project_id: i32) -> Result<()> {
        self.delete_object(0, "projects", project_id).await
    }

    /// Обновляет проект
    pub async fn update_project(&self, project: Project) -> Result<()> {
        self.update_object(0, "projects", project).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_bolt_db() -> BoltStore {
        let path = PathBuf::from("/tmp/test_projects.db");
        BoltStore::new(path).unwrap()
    }

    fn create_test_project(name: &str) -> Project {
        Project {
            id: 0,
            created: Utc::now(),
            name: name.to_string(),
            alert: false,
            alert_chat: None,
            max_parallel_tasks: 0,
            project_type: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_create_project() {
        let db = create_test_bolt_db();
        let project = create_test_project("Test Project");
        
        let result = db.create_project(project).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_project() {
        let db = create_test_bolt_db();
        let project = create_test_project("Test Project");
        let created = db.create_project(project).await.unwrap();
        
        let retrieved = db.get_project(created.id).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().name, "Test Project");
    }

    #[tokio::test]
    async fn test_get_all_projects() {
        let db = create_test_bolt_db();
        
        // Создаём несколько проектов
        for i in 0..5 {
            let project = create_test_project(&format!("Project {}", i));
            db.create_project(project).await.unwrap();
        }
        
        let projects = db.get_all_projects().await;
        assert!(projects.is_ok());
        assert!(projects.unwrap().len() >= 5);
    }

    #[tokio::test]
    async fn test_update_project() {
        let db = create_test_bolt_db();
        let project = create_test_project("Test Project");
        let mut created = db.create_project(project).await.unwrap();
        
        created.name = "Updated Project".to_string();
        let result = db.update_project(created).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_project() {
        let db = create_test_bolt_db();
        let project = create_test_project("Test Project");
        let created = db.create_project(project).await.unwrap();
        
        let result = db.delete_project(created.id).await;
        assert!(result.is_ok());
        
        let retrieved = db.get_project(created.id).await;
        assert!(retrieved.is_err());
    }
}
