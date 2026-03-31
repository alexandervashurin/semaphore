//! Модели для запуска Playbook

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Запрос на запуск playbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookRunRequest {
    /// ID inventory для запуска (опционально)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_id: Option<i32>,

    /// ID environment с переменными (опционально)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<i32>,

    /// Дополнительные переменные (extra vars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_vars: Option<serde_json::Value>,

    /// Ограничение по хостам (ansible --limit)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<String>,

    /// Теги для запуска (ansible --tags)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Пропускаемые теги (ansible --skip-tags)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_tags: Option<Vec<String>>,

    /// Пользователь для запуска (опционально)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i32>,
}

/// Результат запуска playbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookRunResult {
    /// ID созданной задачи
    pub task_id: i32,

    /// ID шаблона
    pub template_id: i32,

    /// Статус задачи
    pub status: String,

    /// Сообщение
    pub message: String,
}

/// Параметры Ansible для задачи
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnsiblePlaybookParams {
    /// Путь к playbook
    pub playbook: String,

    /// Inventory ID
    pub inventory_id: Option<i32>,

    /// Environment ID
    pub environment_id: Option<i32>,

    /// Extra vars (JSON)
    pub extra_vars: Option<String>,

    /// Limit hosts
    pub limit: Option<String>,

    /// Tags
    pub tags: Option<String>,

    /// Skip tags
    pub skip_tags: Option<String>,
}

impl PlaybookRunRequest {
    /// Создает новый запрос на запуск playbook
    pub fn new() -> Self {
        Self {
            inventory_id: None,
            environment_id: None,
            extra_vars: None,
            limit: None,
            tags: None,
            skip_tags: None,
            user_id: None,
        }
    }

    /// Устанавливает inventory
    pub fn with_inventory(mut self, inventory_id: i32) -> Self {
        self.inventory_id = Some(inventory_id);
        self
    }

    /// Устанавливает environment
    pub fn with_environment(mut self, environment_id: i32) -> Self {
        self.environment_id = Some(environment_id);
        self
    }

    /// Устанавливает extra vars
    pub fn with_extra_vars(mut self, extra_vars: serde_json::Value) -> Self {
        self.extra_vars = Some(extra_vars);
        self
    }

    /// Устанавливает limit
    pub fn with_limit(mut self, limit: String) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Устанавливает tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Проверяет корректность запроса
    pub fn validate(&self) -> Result<(), String> {
        // Проверка extra_vars на валидный JSON
        if let Some(ref extra_vars) = self.extra_vars {
            if !extra_vars.is_object() && !extra_vars.is_null() {
                return Err("extra_vars должен быть JSON объектом".to_string());
            }
        }

        Ok(())
    }
}

impl Default for PlaybookRunRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_playbook_run_request_new() {
        let request = PlaybookRunRequest::new();
        assert!(request.inventory_id.is_none());
        assert!(request.environment_id.is_none());
        assert!(request.extra_vars.is_none());
    }

    #[test]
    fn test_playbook_run_request_builder() {
        let request = PlaybookRunRequest::new()
            .with_inventory(1)
            .with_environment(2)
            .with_limit("localhost".to_string())
            .with_tags(vec!["deploy".to_string(), "web".to_string()]);

        assert_eq!(request.inventory_id, Some(1));
        assert_eq!(request.environment_id, Some(2));
        assert_eq!(request.limit, Some("localhost".to_string()));
        assert_eq!(
            request.tags,
            Some(vec!["deploy".to_string(), "web".to_string()])
        );
    }

    #[test]
    fn test_validate_success() {
        let request = PlaybookRunRequest::new().with_extra_vars(json!({"key": "value"}));

        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_extra_vars() {
        let request = PlaybookRunRequest::new().with_extra_vars(json!(["array"]));

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_serialize_deserialize() {
        let request = PlaybookRunRequest::new()
            .with_inventory(1)
            .with_extra_vars(json!({"app": "myapp", "version": "1.0"}));

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: PlaybookRunRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.inventory_id, deserialized.inventory_id);
        assert_eq!(request.extra_vars, deserialized.extra_vars);
    }
}
