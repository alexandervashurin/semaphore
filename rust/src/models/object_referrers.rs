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

    #[test]
    fn test_object_referrers_deserialization() {
        let json = r#"{"templates":[1],"tasks":[2,3],"schedules":[],"integrations":[]}"#;
        let referrers: ObjectReferrers = serde_json::from_str(json).unwrap();
        assert_eq!(referrers.templates, vec![1]);
        assert_eq!(referrers.tasks, vec![2, 3]);
    }

    #[test]
    fn test_object_referrers_with_all_fields_populated() {
        let referrers = ObjectReferrers {
            templates: vec![1],
            tasks: vec![2],
            schedules: vec![3],
            integrations: vec![4],
        };
        assert!(!referrers.is_empty());
        assert_eq!(referrers.templates.len(), 1);
        assert_eq!(referrers.integrations.len(), 1);
    }

    #[test]
    fn test_object_referrers_clone() {
        let referrers = ObjectReferrers {
            templates: vec![1, 2], tasks: vec![3], schedules: vec![4, 5, 6],
            integrations: vec![],
        };
        let cloned = referrers.clone();
        assert_eq!(cloned.templates, referrers.templates);
        assert_eq!(cloned.schedules, referrers.schedules);
    }

    #[test]
    fn test_object_referrers_debug() {
        let referrers = ObjectReferrers {
            templates: vec![1], tasks: vec![], schedules: vec![], integrations: vec![],
        };
        let debug_str = format!("{:?}", referrers);
        assert!(debug_str.contains("ObjectReferrers"));
    }

    #[test]
    fn test_object_referrers_deserialization_empty() {
        let json = r#"{"templates":[],"tasks":[],"schedules":[],"integrations":[]}"#;
        let referrers: ObjectReferrers = serde_json::from_str(json).unwrap();
        assert!(referrers.is_empty());
        assert!(referrers.templates.is_empty());
    }

    #[test]
    fn test_object_referrers_many_refs() {
        let referrers = ObjectReferrers {
            templates: (1..=100).collect(),
            tasks: (101..=200).collect(),
            schedules: vec![],
            integrations: vec![],
        };
        assert_eq!(referrers.templates.len(), 100);
        assert_eq!(referrers.tasks.len(), 100);
    }

    #[test]
    fn test_object_referrers_is_empty_individual() {
        let referrers = ObjectReferrers {
            templates: vec![],
            tasks: vec![],
            schedules: vec![],
            integrations: vec![],
        };
        assert!(referrers.templates.is_empty());
        assert!(referrers.tasks.is_empty());
        assert!(referrers.schedules.is_empty());
        assert!(referrers.integrations.is_empty());
    }

    #[test]
    fn test_object_referrers_serialization_all() {
        let referrers = ObjectReferrers {
            templates: vec![1, 2, 3],
            tasks: vec![4, 5],
            schedules: vec![6],
            integrations: vec![7, 8, 9, 10],
        };
        let json = serde_json::to_string(&referrers).unwrap();
        assert!(json.contains("\"templates\":[1,2,3]"));
        assert!(json.contains("\"tasks\":[4,5]"));
        assert!(json.contains("\"schedules\":[6]"));
        assert!(json.contains("\"integrations\":[7,8,9,10]"));
    }

    #[test]
    fn test_object_referrers_only_integrations() {
        let referrers = ObjectReferrers {
            templates: vec![],
            tasks: vec![],
            schedules: vec![],
            integrations: vec![1],
        };
        assert!(!referrers.is_empty());
        assert_eq!(referrers.integrations, vec![1]);
    }
}
