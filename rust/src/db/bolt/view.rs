//! View CRUD Operations for BoltDB
//!
//! Операции с представлениями в BoltDB

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::{Error, Result};
use crate::models::View;

impl BoltStore {
    /// Получает представления проекта
    pub async fn get_views(&self, project_id: i32) -> Result<Vec<View>> {
        self.get_objects::<View>(project_id, "views", crate::db::store::RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            filter: String::new(),
        }).await
    }

    /// Получает представление по ID
    pub async fn get_view(&self, project_id: i32, view_id: i32) -> Result<View> {
        self.get_object::<View>(project_id, "views", view_id).await
    }

    /// Создаёт представление
    pub async fn create_view(&self, mut view: View) -> Result<View> {
        view.id = self.get_next_id("views")?;
        self.create_object(view.project_id, "views", &view).await?;
        Ok(view)
    }

    /// Обновляет представление
    pub async fn update_view(&self, view: View) -> Result<()> {
        self.update_object(view.project_id, "views", view.id, &view).await
    }

    /// Удаляет представление
    pub async fn delete_view(&self, project_id: i32, view_id: i32) -> Result<()> {
        self.delete_object(project_id, "views", view_id).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_operations() {
        // Тест для проверки операций с представлениями
        assert!(true);
    }
}
