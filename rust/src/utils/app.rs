//! App Types
//!
//! Типы приложений для Semaphore

use serde::{Deserialize, Serialize};

/// Приложение Semaphore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    /// Активно ли приложение
    pub active: bool,

    /// Приоритет приложения
    pub priority: i32,

    /// Заголовок приложения
    pub title: String,

    /// Иконка приложения
    pub icon: String,

    /// Цвет приложения
    pub color: String,

    /// Тёмный цвет приложения
    pub dark_color: String,

    /// Путь к приложению
    pub app_path: String,

    /// Аргументы приложения
    pub app_args: Vec<String>,
}

impl App {
    /// Создаёт новое приложение
    pub fn new(title: String, app_path: String) -> Self {
        Self {
            active: true,
            priority: 0,
            title,
            icon: String::new(),
            color: String::new(),
            dark_color: String::new(),
            app_path,
            app_args: Vec::new(),
        }
    }

    /// Создаёт приложение с параметрами
    pub fn with_params(
        title: String,
        app_path: String,
        icon: String,
        color: String,
    ) -> Self {
        Self {
            active: true,
            priority: 0,
            title,
            icon,
            color,
            dark_color: String::new(),
            app_path,
            app_args: Vec::new(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(String::new(), String::new())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new("Test App".to_string(), "/path/to/app".to_string());
        assert!(app.active);
        assert_eq!(app.title, "Test App");
        assert_eq!(app.app_path, "/path/to/app");
        assert_eq!(app.priority, 0);
    }

    #[test]
    fn test_app_with_params() {
        let app = App::with_params(
            "Test App".to_string(),
            "/path/to/app".to_string(),
            "icon.png".to_string(),
            "#FF0000".to_string(),
        );
        assert_eq!(app.title, "Test App");
        assert_eq!(app.icon, "icon.png");
        assert_eq!(app.color, "#FF0000");
    }

    #[test]
    fn test_app_default() {
        let app: App = App::default();
        assert!(!app.active); // default sets active=false via empty strings
        assert_eq!(app.priority, 0);
    }

    #[test]
    fn test_app_serialization() {
        let app = App::new("Test".to_string(), "/path".to_string());
        let json = serde_json::to_string(&app).unwrap();
        assert!(json.contains("Test"));
        assert!(json.contains("/path"));
    }
}
