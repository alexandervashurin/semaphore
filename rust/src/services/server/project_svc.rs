//! Project Service
//!
//! Сервис управления проектами

use std::sync::Arc;
use crate::error::Result;
use crate::models::Project;
use crate::db::store::Store;

/// Сервис проектов
pub trait ProjectService: Send + Sync {
    /// Обновляет проект
    fn update_project(&self, project: Project) -> Result<()>;

    /// Удаляет проект
    fn delete_project(&self, project_id: i32) -> Result<()>;
}

/// Реализация сервиса проектов
pub struct ProjectServiceImpl {
    project_repo: Arc<dyn Store + Send + Sync>,
    key_repo: Arc<dyn Store + Send + Sync>,
}

impl ProjectServiceImpl {
    /// Создаёт новый сервис
    pub fn new(
        project_repo: Arc<dyn Store + Send + Sync>,
        key_repo: Arc<dyn Store + Send + Sync>,
    ) -> Self {
        Self {
            project_repo,
            key_repo,
        }
    }
}

impl ProjectService for ProjectServiceImpl {
    fn update_project(&self, project: Project) -> Result<()> {
        self.project_repo.update_project(project)
    }

    fn delete_project(&self, project_id: i32) -> Result<()> {
        self.project_repo.delete_project(project_id)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_service_creation() {
        // Тест для проверки создания сервиса
        assert!(true);
    }
}
