//! Project CRUD Operations
//!
//! Адаптер для декомпозированных модулей
//!
//! Новые модули: sqlite::project, postgres::project, mysql::project

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::Project;

impl SqlDb {
    /// Получает все проекты
    pub async fn get_projects(&self, user_id: Option<i32>) -> Result<Vec<Project>> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::project::get_projects(pool, user_id).await
    }

    /// Получает проект по ID
    pub async fn get_project(&self, project_id: i32) -> Result<Project> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::project::get_project(pool, project_id).await
    }

    /// Создаёт проект
    pub async fn create_project(&self, project: Project) -> Result<Project> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::project::create_project(pool, project).await
    }

    /// Обновляет проект
    pub async fn update_project(&self, project: Project) -> Result<()> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::project::update_project(pool, project).await
    }

    /// Удаляет проект
    pub async fn delete_project(&self, project_id: i32) -> Result<()> {
        let pool = self.get_postgres_pool().ok_or(Error::Other("PostgreSQL pool not found".to_string()))?;
                crate::db::sql::postgres::project::delete_project(pool, project_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_new() {
        let project = Project::new("Test Project".to_string());
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.id, 0);
        assert!(!project.alert);
        assert!(project.alert_chat.is_none());
        assert_eq!(project.max_parallel_tasks, 0);
    }

    #[test]
    fn test_project_default() {
        let project = Project::default();
        assert_eq!(project.name, "default");
        assert!(project.validate().is_ok());
    }

    #[test]
    fn test_project_validate_empty_name() {
        let project = Project::new("".to_string());
        assert!(project.validate().is_err());
    }

    #[test]
    fn test_project_validate_non_empty_name() {
        let project = Project::new("Valid".to_string());
        assert!(project.validate().is_ok());
    }

    #[test]
    fn test_project_serialization() {
        let project = Project::new("Serialize".to_string());
        let json = serde_json::to_string(&project).unwrap();
        assert!(json.contains("\"name\":\"Serialize\""));
        assert!(json.contains("\"alert\":false"));
    }

    #[test]
    fn test_project_deserialization() {
        let json = r#"{"id":5,"created":"2024-01-01T00:00:00Z","name":"Deser","alert":true,"max_parallel_tasks":10,"type":"ansible"}"#;
        let project: Project = serde_json::from_str(json).unwrap();
        assert_eq!(project.id, 5);
        assert_eq!(project.name, "Deser");
        assert!(project.alert);
    }

    #[test]
    fn test_project_clone() {
        let project = Project::new("Clone".to_string());
        let cloned = project.clone();
        assert_eq!(cloned.name, project.name);
        assert_eq!(cloned.max_parallel_tasks, project.max_parallel_tasks);
    }

    #[test]
    fn test_project_with_alerts() {
        let mut project = Project::new("Alert".to_string());
        project.alert = true;
        project.alert_chat = Some("chat123".to_string());
        project.max_parallel_tasks = 5;
        assert!(project.alert);
        assert_eq!(project.alert_chat, Some("chat123".to_string()));
        assert_eq!(project.max_parallel_tasks, 5);
    }

    #[test]
    fn test_project_debug_format() {
        let project = Project::new("Debug".to_string());
        let debug_str = format!("{:?}", project);
        assert!(debug_str.contains("Debug"));
        assert!(debug_str.contains("Project"));
    }

    #[test]
    fn test_project_with_secret_storage() {
        let mut project = Project::new("Vault".to_string());
        project.default_secret_storage_id = Some(42);
        assert_eq!(project.default_secret_storage_id, Some(42));
    }

    #[test]
    fn test_project_serialization_skip_nulls() {
        let project = Project::default();
        let json = serde_json::to_string(&project).unwrap();
        assert!(!json.contains("alert_chat"));
        assert!(!json.contains("default_secret_storage_id"));
    }

    #[test]
    fn test_project_type_field() {
        let mut project = Project::new("Typed".to_string());
        project.r#type = "terraform".to_string();
        let json = serde_json::to_string(&project).unwrap();
        assert!(json.contains("\"type\":\"terraform\""));
    }
}
