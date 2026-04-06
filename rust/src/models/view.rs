//! Модель представления (View)

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Представление - группировка шаблонов
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct View {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub project_id: i32,
    #[serde(alias = "name")]
    pub title: String,
    #[serde(default)]
    pub position: i32,
}

impl View {
    /// Получает имя представления (алиас на title)
    pub fn name(&self) -> &str {
        &self.title
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_name_returns_title() {
        let view = View {
            id: 1,
            project_id: 10,
            title: "My View".to_string(),
            position: 0,
        };
        assert_eq!(view.name(), "My View");
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
        assert_eq!(view.name(), "");
    }

    #[test]
    fn test_view_serialization() {
        let view = View {
            id: 5,
            project_id: 20,
            title: "Test View".to_string(),
            position: 2,
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
}
