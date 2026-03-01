//! View, Option - операции в BoltDB
//!
//! Аналог db/bolt/view.go, option.go из Go версии

use std::sync::Arc;
use crate::db::bolt::BoltStore;
use crate::error::Result;
use crate::models::{View, RetrieveQueryParams};

// ============================================================================
// View Operations (positions only - main CRUD in view.rs)
// ============================================================================

impl BoltStore {
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
// Option Operations (delete_options only - main CRUD in option.rs)
// ============================================================================

impl BoltStore {
    /// Удаляет опции по фильтру
    pub async fn delete_options(&self, filter: &str) -> Result<()> {
        let options = self.get_options(RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            filter: filter.to_string(),
            sort_by: None,
            sort_inverted: false,
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

    #[test]
    fn test_placeholder() {
        assert!(true);
    }
}
