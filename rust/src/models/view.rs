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

    #[test]
    fn test_view_clone() {
        let view = View {
            id: 5, project_id: 20, title: "Clone View".to_string(), position: 3,
        };
        let cloned = view.clone();
        assert_eq!(cloned.title, view.title);
        assert_eq!(cloned.position, view.position);
    }

    #[test]
    fn test_view_debug() {
        let view = View {
            id: 1, project_id: 1, title: "Debug View".to_string(), position: 0,
        };
        let debug_str = format!("{:?}", view);
        assert!(debug_str.contains("View"));
        assert!(debug_str.contains("Debug View"));
    }

    #[test]
    fn test_view_name_alias() {
        let view = View {
            id: 1, project_id: 1, title: "Title Test".to_string(), position: 0,
        };
        assert_eq!(view.name(), "Title Test");
        assert_eq!(view.title, view.name().to_string());
    }

    #[test]
    fn test_view_deserialization_with_name_alias() {
        let json = r#"{"id":1,"project_id":5,"name":"Aliased View","position":2}"#;
        let view: View = serde_json::from_str(json).unwrap();
        assert_eq!(view.title, "Aliased View");
    }

    #[test]
    fn test_view_zero_position() {
        let view = View {
            id: 1, project_id: 1, title: "Zero Pos".to_string(), position: 0,
        };
        assert_eq!(view.position, 0);
        let json = serde_json::to_string(&view).unwrap();
        assert!(json.contains("\"position\":0"));
    }

    #[test]
    fn test_view_negative_position() {
        let view = View {
            id: 1, project_id: 1, title: "Negative Pos".to_string(), position: -1,
        };
        let json = serde_json::to_string(&view).unwrap();
        assert!(json.contains("\"position\":-1"));
    }

    #[test]
    fn test_view_large_position() {
        let view = View {
            id: 1, project_id: 1, title: "Large Pos".to_string(), position: i32::MAX,
        };
        let json = serde_json::to_string(&view).unwrap();
        let restored: View = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.position, i32::MAX);
    }

    #[test]
    fn test_view_special_chars_title() {
        let view = View {
            id: 1, project_id: 1, title: "View & <special> \"chars\"".to_string(), position: 0,
        };
        let json = serde_json::to_string(&view).unwrap();
        let restored: View = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.title, "View & <special> \"chars\"");
    }
}
