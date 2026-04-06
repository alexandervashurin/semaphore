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
        assert_eq!(serde_json::to_string(&ExportEntityType::Project).unwrap(), "\"project\"");
        assert_eq!(serde_json::to_string(&ExportEntityType::AccessKey).unwrap(), "\"access_key\"");
    }
}
