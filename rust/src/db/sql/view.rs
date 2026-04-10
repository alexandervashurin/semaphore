//! View CRUD Operations
//!
//! Операции с представлениями в SQL

use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::View;
use sqlx::Row;

impl SqlDb {
    fn pg_pool_view(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает представления проекта
    pub async fn get_views(&self, project_id: i32) -> Result<Vec<View>> {
        let rows =
            sqlx::query("SELECT * FROM view WHERE project_id = $1 ORDER BY position ASC, id ASC")
                .bind(project_id)
                .fetch_all(self.pg_pool_view()?)
                .await
                .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| View {
                id: row.get("id"),
                project_id: row.get("project_id"),
                title: row.get("title"),
                position: row.try_get("position").ok().unwrap_or(0),
            })
            .collect())
    }

    /// Получает представление по ID
    pub async fn get_view(&self, project_id: i32, view_id: i32) -> Result<View> {
        let row = sqlx::query("SELECT * FROM view WHERE id = $1 AND project_id = $2")
            .bind(view_id)
            .bind(project_id)
            .fetch_one(self.pg_pool_view()?)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound("Представление не найдено".to_string()),
                _ => Error::Database(e),
            })?;

        Ok(View {
            id: row.get("id"),
            project_id: row.get("project_id"),
            title: row.get("title"),
            position: row.try_get("position").ok().unwrap_or(0),
        })
    }

    /// Создаёт представление
    pub async fn create_view(&self, mut view: View) -> Result<View> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO view (project_id, title, position) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(view.project_id)
        .bind(&view.title)
        .bind(view.position)
        .fetch_one(self.pg_pool_view()?)
        .await
        .map_err(Error::Database)?;

        view.id = id;
        Ok(view)
    }

    /// Обновляет представление
    pub async fn update_view(&self, view: View) -> Result<()> {
        sqlx::query("UPDATE view SET title = $1, position = $2 WHERE id = $3 AND project_id = $4")
            .bind(&view.title)
            .bind(view.position)
            .bind(view.id)
            .bind(view.project_id)
            .execute(self.pg_pool_view()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    /// Удаляет представление
    pub async fn delete_view(&self, project_id: i32, view_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM view WHERE id = $1 AND project_id = $2")
            .bind(view_id)
            .bind(project_id)
            .execute(self.pg_pool_view()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_name_method() {
        let view = View {
            id: 1,
            project_id: 10,
            title: "My View".to_string(),
            position: 0,
        };
        assert_eq!(view.name(), "My View");
    }

    #[test]
    fn test_view_serialization() {
        let view = View {
            id: 5,
            project_id: 20,
            title: "Production".to_string(),
            position: 1,
        };
        let json = serde_json::to_string(&view).unwrap();
        assert!(json.contains("\"title\":\"Production\""));
        assert!(json.contains("\"id\":5"));
        assert!(json.contains("\"position\":1"));
    }

    #[test]
    fn test_view_deserialization() {
        let json = r#"{"id":3,"project_id":15,"title":"Dev View","position":2}"#;
        let view: View = serde_json::from_str(json).unwrap();
        assert_eq!(view.id, 3);
        assert_eq!(view.title, "Dev View");
        assert_eq!(view.position, 2);
    }

    #[test]
    fn test_view_deserialization_with_name_alias() {
        let json = r#"{"id":1,"project_id":1,"name":"Alias Name","position":0}"#;
        let view: View = serde_json::from_str(json).unwrap();
        assert_eq!(view.title, "Alias Name");
    }

    #[test]
    fn test_view_clone() {
        let view = View {
            id: 10,
            project_id: 5,
            title: "Clone View".to_string(),
            position: 3,
        };
        let cloned = view.clone();
        assert_eq!(cloned.title, view.title);
        assert_eq!(cloned.position, view.position);
    }

    #[test]
    fn test_view_debug_format() {
        let view = View {
            id: 1,
            project_id: 1,
            title: "Debug".to_string(),
            position: 0,
        };
        let debug_str = format!("{:?}", view);
        assert!(debug_str.contains("View"));
        assert!(debug_str.contains("Debug"));
    }

    #[test]
    fn test_view_default_values() {
        let view = View {
            id: 0,
            project_id: 0,
            title: String::new(),
            position: 0,
        };
        assert_eq!(view.id, 0);
        assert_eq!(view.title, "");
        assert_eq!(view.name(), "");
    }

    #[test]
    fn test_view_position_ordering() {
        let views = vec![
            View { id: 3, project_id: 1, title: "Third".to_string(), position: 2 },
            View { id: 1, project_id: 1, title: "First".to_string(), position: 0 },
            View { id: 2, project_id: 1, title: "Second".to_string(), position: 1 },
        ];
        let mut sorted = views.clone();
        sorted.sort_by_key(|v| v.position);
        assert_eq!(sorted[0].title, "First");
        assert_eq!(sorted[1].title, "Second");
        assert_eq!(sorted[2].title, "Third");
    }

    #[test]
    fn test_view_serialization_roundtrip() {
        let original = View {
            id: 99,
            project_id: 42,
            title: "Roundtrip".to_string(),
            position: 10,
        };
        let json = serde_json::to_string(&original).unwrap();
        let decoded: View = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, original.id);
        assert_eq!(decoded.title, original.title);
        assert_eq!(decoded.position, original.position);
    }

    #[test]
    fn test_view_with_special_characters() {
        let view = View {
            id: 1,
            project_id: 1,
            title: "View with \"quotes\" & <tags>".to_string(),
            position: 0,
        };
        let json = serde_json::to_string(&view).unwrap();
        let decoded: View = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.title, "View with \"quotes\" & <tags>");
    }
}
