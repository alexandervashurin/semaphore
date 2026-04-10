//! ViewManager - управление представлениями

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::View;
use async_trait::async_trait;

#[async_trait]
impl ViewManager for SqlStore {
    async fn get_views(&self, project_id: i32) -> Result<Vec<View>> {
        self.db.get_views(project_id).await
    }
    async fn get_view(&self, project_id: i32, view_id: i32) -> Result<View> {
        self.db.get_view(project_id, view_id).await
    }
    async fn create_view(&self, view: View) -> Result<View> {
        self.db.create_view(view).await
    }
    async fn update_view(&self, view: View) -> Result<()> {
        self.db.update_view(view).await
    }
    async fn delete_view(&self, project_id: i32, view_id: i32) -> Result<()> {
        self.db.delete_view(project_id, view_id).await
    }

    async fn set_view_positions(&self, project_id: i32, positions: Vec<(i32, i32)>) -> Result<()> {
        // positions: Vec<(view_id, position)>
        for (view_id, position) in positions {
            let query = "UPDATE view SET position = $1 WHERE id = $2 AND project_id = $3";
            sqlx::query(query)
                .bind(position)
                .bind(view_id)
                .bind(project_id)
                .execute(self.get_postgres_pool()?)
                .await
                .map_err(Error::Database)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::models::View;

    #[test]
    fn test_view_name_returns_title() {
        let view = View {
            id: 1, project_id: 10, title: "My View".to_string(), position: 0,
        };
        assert_eq!(view.name(), "My View");
    }

    #[test]
    fn test_view_default_values() {
        let view = View {
            id: 0, project_id: 0, title: String::new(), position: 0,
        };
        assert_eq!(view.id, 0);
        assert!(view.title.is_empty());
    }

    #[test]
    fn test_view_serialization() {
        let view = View {
            id: 5, project_id: 20, title: "Test View".to_string(), position: 2,
        };
        let json = serde_json::to_string(&view).unwrap();
        assert!(json.contains("\"title\":\"Test View\""));
        assert!(json.contains("\"id\":5"));
        assert!(json.contains("\"position\":2"));
    }

    #[test]
    fn test_view_deserialization() {
        let json = r#"{"id":3,"project_id":15,"title":"Deserialized View","position":1}"#;
        let view: View = serde_json::from_str(json).unwrap();
        assert_eq!(view.id, 3);
        assert_eq!(view.title, "Deserialized View");
        assert_eq!(view.project_id, 15);
    }

    #[test]
    fn test_view_clone() {
        let view = View {
            id: 1, project_id: 5, title: "Clone View".to_string(), position: 3,
        };
        let cloned = view.clone();
        assert_eq!(cloned.title, view.title);
        assert_eq!(cloned.position, view.position);
    }

    #[test]
    fn test_view_position() {
        let view = View {
            id: 1, project_id: 1, title: "Positioned".to_string(), position: 10,
        };
        assert_eq!(view.position, 10);
    }

    #[test]
    fn test_view_deserialize_with_name_alias() {
        // View has #[serde(alias = "name")] on title field
        let json = r#"{"id":1,"project_id":1,"name":"Aliased View","position":0}"#;
        let view: View = serde_json::from_str(json).unwrap();
        assert_eq!(view.title, "Aliased View");
    }

    #[test]
    fn test_view_zero_position() {
        let view = View {
            id: 1, project_id: 1, title: "First".to_string(), position: 0,
        };
        assert_eq!(view.position, 0);
    }
}
