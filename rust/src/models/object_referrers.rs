//! Модель ObjectReferrers - ссылки на объекты

use serde::{Deserialize, Serialize};

/// Ссылки на объект
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObjectReferrers {
    /// Ссылки из шаблонов
    #[serde(default)]
    pub templates: Vec<i32>,

    /// Ссылки из задач
    #[serde(default)]
    pub tasks: Vec<i32>,

    /// Ссылки из расписаний
    #[serde(default)]
    pub schedules: Vec<i32>,

    /// Ссылки из интеграций
    #[serde(default)]
    pub integrations: Vec<i32>,
}

impl ObjectReferrers {
    /// Создаёт новые пустые ссылки
    pub fn new() -> Self {
        Self::default()
    }

    /// Проверяет, есть ли ссылки
    pub fn is_empty(&self) -> bool {
        self.templates.is_empty()
            && self.tasks.is_empty()
            && self.schedules.is_empty()
            && self.integrations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_referrers_default() {
        let referrers = ObjectReferrers::default();
        assert!(referrers.is_empty());
        assert!(referrers.templates.is_empty());
    }

    #[test]
    fn test_object_referrers_new() {
        let referrers = ObjectReferrers::new();
        assert!(referrers.is_empty());
    }

    #[test]
    fn test_object_referrers_not_empty() {
        let referrers = ObjectReferrers {
            templates: vec![1, 2],
            tasks: vec![],
            schedules: vec![],
            integrations: vec![],
        };
        assert!(!referrers.is_empty());
    }

    #[test]
    fn test_object_referrers_serialization() {
        let referrers = ObjectReferrers {
            templates: vec![1, 2, 3],
            tasks: vec![10, 20],
            schedules: vec![],
            integrations: vec![5],
        };
        let json = serde_json::to_string(&referrers).unwrap();
        assert!(json.contains("\"templates\":[1,2,3]"));
        assert!(json.contains("\"tasks\":[10,20]"));
    }
}
