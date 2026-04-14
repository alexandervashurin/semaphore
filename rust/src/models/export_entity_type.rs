//! Export Entity Type
//!
//! Типы экспортируемых сущностей

use serde::{Deserialize, Serialize};

/// Тип экспортируемой сущности
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExportEntityType {
    /// Проект
    Project,

    /// Шаблон
    Template,

    /// Задача
    Task,

    /// Пользователь
    User,

    /// Инвентарь
    Inventory,

    /// Репозиторий
    Repository,

    /// Окружение
    Environment,

    /// Ключ доступа
    AccessKey,

    /// Интеграция
    Integration,

    /// Расписание
    Schedule,

    /// Другое
    Other,
}

impl ExportEntityType {
    /// Получает строковое представление
    pub fn as_str(&self) -> &'static str {
        match self {
            ExportEntityType::Project => "project",
            ExportEntityType::Template => "template",
            ExportEntityType::Task => "task",
            ExportEntityType::User => "user",
            ExportEntityType::Inventory => "inventory",
            ExportEntityType::Repository => "repository",
            ExportEntityType::Environment => "environment",
            ExportEntityType::AccessKey => "access_key",
            ExportEntityType::Integration => "integration",
            ExportEntityType::Schedule => "schedule",
            ExportEntityType::Other => "other",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_entity_type_as_str() {
        assert_eq!(ExportEntityType::Project.as_str(), "project");
        assert_eq!(ExportEntityType::Template.as_str(), "template");
        assert_eq!(ExportEntityType::Task.as_str(), "task");
        assert_eq!(ExportEntityType::User.as_str(), "user");
    }

    #[test]
    fn test_export_entity_type_serialization() {
        assert_eq!(
            serde_json::to_string(&ExportEntityType::Project).unwrap(),
            "\"project\""
        );
        assert_eq!(
            serde_json::to_string(&ExportEntityType::AccessKey).unwrap(),
            "\"access_key\""
        );
    }

    #[test]
    fn test_export_entity_type_all_variants() {
        let types = vec![
            ExportEntityType::Project,
            ExportEntityType::Template,
            ExportEntityType::Task,
            ExportEntityType::User,
            ExportEntityType::Inventory,
            ExportEntityType::Repository,
            ExportEntityType::Environment,
            ExportEntityType::AccessKey,
            ExportEntityType::Integration,
            ExportEntityType::Schedule,
            ExportEntityType::Other,
        ];

        for t in &types {
            let json = serde_json::to_string(t).unwrap();
            assert!(json.starts_with('"'));
            assert!(json.ends_with('"'));
        }
    }

    #[test]
    fn test_export_entity_type_clone() {
        let t = ExportEntityType::Integration;
        let cloned = t.clone();
        assert_eq!(cloned, t);
    }

    #[test]
    fn test_export_entity_type_debug() {
        let t = ExportEntityType::Project;
        let debug_str = format!("{:?}", t);
        assert!(debug_str.contains("Project"));
    }

    #[test]
    fn test_export_entity_type_deserialization() {
        let project: ExportEntityType = serde_json::from_str("\"project\"").unwrap();
        let template: ExportEntityType = serde_json::from_str("\"template\"").unwrap();
        let task: ExportEntityType = serde_json::from_str("\"task\"").unwrap();
        assert_eq!(project, ExportEntityType::Project);
        assert_eq!(template, ExportEntityType::Template);
        assert_eq!(task, ExportEntityType::Task);
    }

    #[test]
    fn test_export_entity_type_all_variants_as_str() {
        assert_eq!(ExportEntityType::Inventory.as_str(), "inventory");
        assert_eq!(ExportEntityType::Repository.as_str(), "repository");
        assert_eq!(ExportEntityType::Environment.as_str(), "environment");
        assert_eq!(ExportEntityType::AccessKey.as_str(), "access_key");
        assert_eq!(ExportEntityType::Schedule.as_str(), "schedule");
        assert_eq!(ExportEntityType::Other.as_str(), "other");
    }

    #[test]
    fn test_export_entity_type_equality() {
        let a = ExportEntityType::Project;
        let b = ExportEntityType::Project;
        let c = ExportEntityType::Template;
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_export_entity_type_serialization_roundtrip() {
        for t in vec![
            ExportEntityType::Project,
            ExportEntityType::Template,
            ExportEntityType::Task,
            ExportEntityType::User,
            ExportEntityType::Inventory,
            ExportEntityType::Repository,
            ExportEntityType::Environment,
            ExportEntityType::AccessKey,
            ExportEntityType::Integration,
            ExportEntityType::Schedule,
            ExportEntityType::Other,
        ] {
            let json = serde_json::to_string(&t).unwrap();
            let deserialized: ExportEntityType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, t);
        }
    }

    #[test]
    fn test_export_entity_type_default_deserialization() {
        // Test deserializing user and other variants
        let user: ExportEntityType = serde_json::from_str("\"user\"").unwrap();
        assert_eq!(user, ExportEntityType::User);
        let other: ExportEntityType = serde_json::from_str("\"other\"").unwrap();
        assert_eq!(other, ExportEntityType::Other);
    }

    #[test]
    fn test_export_entity_type_invalid_deserialization() {
        let result: Result<ExportEntityType, _> = serde_json::from_str("\"invalid_type\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_export_entity_type_count_variants() {
        // Ensure we have exactly 11 variants
        let variants = vec![
            ExportEntityType::Project,
            ExportEntityType::Template,
            ExportEntityType::Task,
            ExportEntityType::User,
            ExportEntityType::Inventory,
            ExportEntityType::Repository,
            ExportEntityType::Environment,
            ExportEntityType::AccessKey,
            ExportEntityType::Integration,
            ExportEntityType::Schedule,
            ExportEntityType::Other,
        ];
        assert_eq!(variants.len(), 11);
    }

    #[test]
    fn test_export_entity_type_as_str_matches_serialization() {
        for t in vec![
            ExportEntityType::Project,
            ExportEntityType::Template,
            ExportEntityType::Task,
            ExportEntityType::User,
            ExportEntityType::Inventory,
            ExportEntityType::Repository,
            ExportEntityType::Environment,
            ExportEntityType::AccessKey,
            ExportEntityType::Integration,
            ExportEntityType::Schedule,
            ExportEntityType::Other,
        ] {
            let as_str = t.as_str();
            let serialized = serde_json::to_string(&t).unwrap();
            assert!(serialized.contains(as_str));
        }
    }

    #[test]
    fn test_export_entity_type_debug_format() {
        for (variant, name) in vec![
            (ExportEntityType::Project, "Project"),
            (ExportEntityType::Template, "Template"),
            (ExportEntityType::Task, "Task"),
            (ExportEntityType::Other, "Other"),
        ] {
            let debug_str = format!("{:?}", variant);
            assert!(debug_str.contains(name));
        }
    }

    #[test]
    fn test_export_entity_type_all_roundtrip() {
        for t in vec![
            ExportEntityType::Project,
            ExportEntityType::Template,
            ExportEntityType::Task,
            ExportEntityType::User,
            ExportEntityType::Inventory,
            ExportEntityType::Repository,
            ExportEntityType::Environment,
            ExportEntityType::AccessKey,
            ExportEntityType::Integration,
            ExportEntityType::Schedule,
            ExportEntityType::Other,
        ] {
            let json = serde_json::to_string(&t).unwrap();
            let restored: ExportEntityType = serde_json::from_str(&json).unwrap();
            assert_eq!(t, restored);
        }
    }
}
