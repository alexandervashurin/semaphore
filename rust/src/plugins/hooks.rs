//! Plugin Hooks - Система хуков для плагинов
//!
//! Хуки позволяют плагинам реагировать на события системы
//! и модифицировать поведение приложения.

use crate::error::{Error, Result};
use crate::plugins::base::{HookEvent, HookResult, PluginContext};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Типы хуков
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum HookType {
    // Задачи
    TaskBeforeCreate,
    TaskAfterCreate,
    TaskBeforeStart,
    TaskAfterStart,
    TaskBeforeComplete,
    TaskAfterComplete,
    TaskBeforeFail,
    TaskAfterFail,
    TaskBeforeStop,
    TaskAfterStop,
    TaskBeforeDelete,
    TaskAfterDelete,

    // Проекты
    ProjectBeforeCreate,
    ProjectAfterCreate,
    ProjectBeforeUpdate,
    ProjectAfterUpdate,
    ProjectBeforeDelete,
    ProjectAfterDelete,

    // Пользователи
    UserBeforeLogin,
    UserAfterLogin,
    UserBeforeLogout,
    UserAfterLogout,
    UserBeforeCreate,
    UserAfterCreate,

    // Шаблоны
    TemplateBeforeCreate,
    TemplateAfterCreate,
    TemplateBeforeRun,
    TemplateAfterRun,

    // Уведомления
    NotificationBeforeSend,
    NotificationAfterSend,

    // Webhook
    WebhookBeforeSend,
    WebhookAfterSend,

    // Кастомные хуки
    Custom(String),
}

impl std::fmt::Display for HookType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookType::TaskBeforeCreate => write!(f, "task.before_create"),
            HookType::TaskAfterCreate => write!(f, "task.after_create"),
            HookType::TaskBeforeStart => write!(f, "task.before_start"),
            HookType::TaskAfterStart => write!(f, "task.after_start"),
            HookType::TaskBeforeComplete => write!(f, "task.before_complete"),
            HookType::TaskAfterComplete => write!(f, "task.after_complete"),
            HookType::TaskBeforeFail => write!(f, "task.before_fail"),
            HookType::TaskAfterFail => write!(f, "task.after_fail"),
            HookType::TaskBeforeStop => write!(f, "task.before_stop"),
            HookType::TaskAfterStop => write!(f, "task.after_stop"),
            HookType::TaskBeforeDelete => write!(f, "task.before_delete"),
            HookType::TaskAfterDelete => write!(f, "task.after_delete"),
            HookType::ProjectBeforeCreate => write!(f, "project.before_create"),
            HookType::ProjectAfterCreate => write!(f, "project.after_create"),
            HookType::ProjectBeforeUpdate => write!(f, "project.before_update"),
            HookType::ProjectAfterUpdate => write!(f, "project.after_update"),
            HookType::ProjectBeforeDelete => write!(f, "project.before_delete"),
            HookType::ProjectAfterDelete => write!(f, "project.after_delete"),
            HookType::UserBeforeLogin => write!(f, "user.before_login"),
            HookType::UserAfterLogin => write!(f, "user.after_login"),
            HookType::UserBeforeLogout => write!(f, "user.before_logout"),
            HookType::UserAfterLogout => write!(f, "user.after_logout"),
            HookType::UserBeforeCreate => write!(f, "user.before_create"),
            HookType::UserAfterCreate => write!(f, "user.after_create"),
            HookType::TemplateBeforeCreate => write!(f, "template.before_create"),
            HookType::TemplateAfterCreate => write!(f, "template.after_create"),
            HookType::TemplateBeforeRun => write!(f, "template.before_run"),
            HookType::TemplateAfterRun => write!(f, "template.after_run"),
            HookType::NotificationBeforeSend => write!(f, "notification.before_send"),
            HookType::NotificationAfterSend => write!(f, "notification.after_send"),
            HookType::WebhookBeforeSend => write!(f, "webhook.before_send"),
            HookType::WebhookAfterSend => write!(f, "webhook.after_send"),
            HookType::Custom(name) => write!(f, "custom.{}", name),
        }
    }
}

/// Обработчик хука
#[async_trait]
pub trait HookHandler: Send + Sync {
    /// Имя обработчика
    fn name(&self) -> &str;

    /// Приоритет выполнения (меньше = раньше)
    fn priority(&self) -> i32 {
        0
    }

    /// Выполнение обработчика
    async fn handle(&self, event: HookEvent) -> Result<HookResult>;
}

/// Реестр хуков
pub struct HookRegistry {
    handlers: RwLock<Vec<Arc<dyn HookHandler>>>,
}

impl HookRegistry {
    /// Создаёт новый реестр
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(Vec::new()),
        }
    }

    /// Регистрирует обработчик
    pub async fn register(&self, handler: Arc<dyn HookHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.push(handler);
        handlers.sort_by_key(|h| h.priority());
    }

    /// Вызывает хук
    pub async fn trigger(
        &self,
        hook_type: HookType,
        data: JsonValue,
        context: PluginContext,
    ) -> Result<Vec<HookResult>> {
        let handlers = self.handlers.read().await;
        let mut results = Vec::new();

        let event = HookEvent {
            name: hook_type.to_string(),
            timestamp: Utc::now(),
            data,
            context,
        };

        for handler in handlers.iter() {
            match handler.handle(event.clone()).await {
                Ok(result) => {
                    if !result.success {
                        warn!("Hook handler {} failed: {:?}", handler.name(), result.error);
                    }
                    results.push(result);
                }
                Err(e) => {
                    error!("Hook handler {} error: {}", handler.name(), e);
                    results.push(HookResult {
                        success: false,
                        data: None,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Вызывает хук и останавливается при первой ошибке
    pub async fn trigger_until_failure(
        &self,
        hook_type: HookType,
        data: JsonValue,
        context: PluginContext,
    ) -> Result<Option<HookResult>> {
        let handlers = self.handlers.read().await;

        let event = HookEvent {
            name: hook_type.to_string(),
            timestamp: Utc::now(),
            data,
            context,
        };

        for handler in handlers.iter() {
            match handler.handle(event.clone()).await {
                Ok(result) => {
                    if !result.success {
                        return Ok(Some(result));
                    }
                }
                Err(e) => {
                    return Ok(Some(HookResult {
                        success: false,
                        data: None,
                        error: Some(e.to_string()),
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Получает количество зарегистрированных обработчиков
    pub async fn count(&self) -> usize {
        self.handlers.read().await.len()
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Создаёт событие для задачи
pub fn create_task_event(
    hook_type: HookType,
    task_id: i64,
    task_name: &str,
    project_id: Option<i64>,
    user_id: Option<i64>,
    extra_data: Option<JsonValue>,
) -> (HookEvent, JsonValue) {
    let mut data = json!({
        "task_id": task_id,
        "task_name": task_name,
    });

    if let Some(extra) = extra_data {
        if let Some(obj) = extra.as_object() {
            if let Some(data_obj) = data.as_object_mut() {
                data_obj.extend(obj.clone());
            }
        }
    }

    let event = HookEvent {
        name: hook_type.to_string(),
        timestamp: Utc::now(),
        data: data.clone(),
        context: PluginContext {
            plugin_id: "core".to_string(),
            project_id,
            user_id,
            task_id: Some(task_id),
            metadata: HashMap::new(),
        },
    };

    (event, data)
}

/// Создаёт событие для проекта
pub fn create_project_event(
    hook_type: HookType,
    project_id: i64,
    project_name: &str,
    user_id: Option<i64>,
    extra_data: Option<JsonValue>,
) -> (HookEvent, JsonValue) {
    let mut data = json!({
        "project_id": project_id,
        "project_name": project_name,
    });

    if let Some(extra) = extra_data {
        if let Some(obj) = extra.as_object() {
            if let Some(data_obj) = data.as_object_mut() {
                data_obj.extend(obj.clone());
            }
        }
    }

    let event = HookEvent {
        name: hook_type.to_string(),
        timestamp: Utc::now(),
        data: data.clone(),
        context: PluginContext {
            plugin_id: "core".to_string(),
            project_id: Some(project_id),
            user_id,
            task_id: None,
            metadata: HashMap::new(),
        },
    };

    (event, data)
}

use std::collections::HashMap;

// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ========================================================================
    // Тесты для HookType
    // ========================================================================

    #[test]
    fn test_hook_type_display_task() {
        assert_eq!(HookType::TaskBeforeCreate.to_string(), "task.before_create");
        assert_eq!(HookType::TaskAfterCreate.to_string(), "task.after_create");
        assert_eq!(HookType::TaskBeforeStart.to_string(), "task.before_start");
        assert_eq!(HookType::TaskAfterStart.to_string(), "task.after_start");
        assert_eq!(
            HookType::TaskBeforeComplete.to_string(),
            "task.before_complete"
        );
        assert_eq!(
            HookType::TaskAfterComplete.to_string(),
            "task.after_complete"
        );
        assert_eq!(HookType::TaskBeforeFail.to_string(), "task.before_fail");
        assert_eq!(HookType::TaskAfterFail.to_string(), "task.after_fail");
        assert_eq!(HookType::TaskBeforeStop.to_string(), "task.before_stop");
        assert_eq!(HookType::TaskAfterStop.to_string(), "task.after_stop");
        assert_eq!(HookType::TaskBeforeDelete.to_string(), "task.before_delete");
        assert_eq!(HookType::TaskAfterDelete.to_string(), "task.after_delete");
    }

    #[test]
    fn test_hook_type_display_project() {
        assert_eq!(
            HookType::ProjectBeforeCreate.to_string(),
            "project.before_create"
        );
        assert_eq!(
            HookType::ProjectAfterCreate.to_string(),
            "project.after_create"
        );
        assert_eq!(
            HookType::ProjectBeforeUpdate.to_string(),
            "project.before_update"
        );
        assert_eq!(
            HookType::ProjectAfterUpdate.to_string(),
            "project.after_update"
        );
        assert_eq!(
            HookType::ProjectBeforeDelete.to_string(),
            "project.before_delete"
        );
        assert_eq!(
            HookType::ProjectAfterDelete.to_string(),
            "project.after_delete"
        );
    }

    #[test]
    fn test_hook_type_display_user() {
        assert_eq!(HookType::UserBeforeLogin.to_string(), "user.before_login");
        assert_eq!(HookType::UserAfterLogin.to_string(), "user.after_login");
        assert_eq!(HookType::UserBeforeLogout.to_string(), "user.before_logout");
        assert_eq!(HookType::UserAfterLogout.to_string(), "user.after_logout");
        assert_eq!(HookType::UserBeforeCreate.to_string(), "user.before_create");
        assert_eq!(HookType::UserAfterCreate.to_string(), "user.after_create");
    }

    #[test]
    fn test_hook_type_display_template() {
        assert_eq!(
            HookType::TemplateBeforeCreate.to_string(),
            "template.before_create"
        );
        assert_eq!(
            HookType::TemplateAfterCreate.to_string(),
            "template.after_create"
        );
        assert_eq!(
            HookType::TemplateBeforeRun.to_string(),
            "template.before_run"
        );
        assert_eq!(HookType::TemplateAfterRun.to_string(), "template.after_run");
    }

    #[test]
    fn test_hook_type_display_notification() {
        assert_eq!(
            HookType::NotificationBeforeSend.to_string(),
            "notification.before_send"
        );
        assert_eq!(
            HookType::NotificationAfterSend.to_string(),
            "notification.after_send"
        );
        assert_eq!(
            HookType::WebhookBeforeSend.to_string(),
            "webhook.before_send"
        );
        assert_eq!(HookType::WebhookAfterSend.to_string(), "webhook.after_send");
    }

    #[test]
    fn test_hook_type_display_custom() {
        let custom_hook = HookType::Custom("my_custom_hook".to_string());
        assert_eq!(custom_hook.to_string(), "custom.my_custom_hook");
    }

    #[test]
    fn test_hook_type_serialization() {
        let hooks = vec![
            HookType::TaskBeforeCreate,
            HookType::ProjectAfterCreate,
            HookType::UserAfterLogin,
            HookType::Custom("test".to_string()),
        ];

        for hook_type in hooks {
            let json = serde_json::to_string(&hook_type).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_hook_type_equality() {
        assert_eq!(HookType::TaskBeforeCreate, HookType::TaskBeforeCreate);
        assert_eq!(HookType::TaskAfterCreate, HookType::TaskAfterCreate);
        assert_ne!(HookType::TaskBeforeCreate, HookType::TaskAfterCreate);

        let custom1 = HookType::Custom("test".to_string());
        let custom2 = HookType::Custom("test".to_string());
        let custom3 = HookType::Custom("other".to_string());

        assert_eq!(custom1, custom2);
        assert_ne!(custom1, custom3);
    }

    // ========================================================================
    // Тесты для HookRegistry
    // ========================================================================

    #[tokio::test]
    async fn test_hook_registry_creation() {
        let registry = HookRegistry::new();
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_hook_registry_register_handler() {
        let registry = HookRegistry::new();

        // Создаём простой тестовый обработчик
        struct TestHandler;

        #[async_trait::async_trait]
        impl HookHandler for TestHandler {
            fn name(&self) -> &str {
                "test_handler"
            }

            fn priority(&self) -> i32 {
                0
            }

            async fn handle(&self, _event: HookEvent) -> Result<HookResult> {
                Ok(HookResult {
                    success: true,
                    data: Some(json!({"handled": true})),
                    error: None,
                })
            }
        }

        registry.register(Arc::new(TestHandler)).await;
        assert_eq!(registry.count().await, 1);
    }

    #[tokio::test]
    async fn test_hook_registry_trigger() {
        let registry = HookRegistry::new();

        struct TestHandler;

        #[async_trait::async_trait]
        impl HookHandler for TestHandler {
            fn name(&self) -> &str {
                "test_handler"
            }

            fn priority(&self) -> i32 {
                0
            }

            async fn handle(&self, _event: HookEvent) -> Result<HookResult> {
                Ok(HookResult {
                    success: true,
                    data: Some(json!({"result": "ok"})),
                    error: None,
                })
            }
        }

        registry.register(Arc::new(TestHandler)).await;

        let context = PluginContext {
            plugin_id: "test".to_string(),
            project_id: None,
            user_id: None,
            task_id: None,
            metadata: HashMap::new(),
        };

        let results = registry
            .trigger(HookType::TaskBeforeCreate, json!({"task_id": 1}), context)
            .await;

        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
    }

    #[tokio::test]
    async fn test_hook_registry_trigger_until_failure() {
        let registry = HookRegistry::new();

        struct SuccessHandler;

        #[async_trait::async_trait]
        impl HookHandler for SuccessHandler {
            fn name(&self) -> &str {
                "success_handler"
            }

            fn priority(&self) -> i32 {
                0
            }

            async fn handle(&self, _event: HookEvent) -> Result<HookResult> {
                Ok(HookResult {
                    success: true,
                    data: None,
                    error: None,
                })
            }
        }

        registry.register(Arc::new(SuccessHandler)).await;

        let context = PluginContext {
            plugin_id: "test".to_string(),
            project_id: None,
            user_id: None,
            task_id: None,
            metadata: HashMap::new(),
        };

        let result = registry
            .trigger_until_failure(HookType::TaskAfterCreate, json!({}), context)
            .await;

        assert!(result.is_ok());
        // Все обработчики успешны, поэтому результат None
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_hook_registry_multiple_handlers() {
        let registry = HookRegistry::new();

        struct Handler1;
        struct Handler2;

        #[async_trait::async_trait]
        impl HookHandler for Handler1 {
            fn name(&self) -> &str {
                "handler1"
            }

            fn priority(&self) -> i32 {
                1
            }

            async fn handle(&self, _event: HookEvent) -> Result<HookResult> {
                Ok(HookResult {
                    success: true,
                    data: Some(json!({"handler": 1})),
                    error: None,
                })
            }
        }

        #[async_trait::async_trait]
        impl HookHandler for Handler2 {
            fn name(&self) -> &str {
                "handler2"
            }

            fn priority(&self) -> i32 {
                2
            }

            async fn handle(&self, _event: HookEvent) -> Result<HookResult> {
                Ok(HookResult {
                    success: true,
                    data: Some(json!({"handler": 2})),
                    error: None,
                })
            }
        }

        registry.register(Arc::new(Handler1)).await;
        registry.register(Arc::new(Handler2)).await;

        assert_eq!(registry.count().await, 2);
    }

    // ========================================================================
    // Тесты для create_task_event
    // ========================================================================

    #[test]
    fn test_create_task_event() {
        let (event, data) = create_task_event(
            HookType::TaskBeforeCreate,
            100,
            "Test Task",
            Some(1),
            Some(42),
            None,
        );

        assert_eq!(event.name, "task.before_create");
        assert_eq!(event.context.task_id, Some(100));
        assert_eq!(event.context.project_id, Some(1));
        assert_eq!(event.context.user_id, Some(42));

        let data_obj = data.as_object().unwrap();
        assert_eq!(data_obj.get("task_id").unwrap(), &100);
        assert_eq!(data_obj.get("task_name").unwrap(), &"Test Task");
    }

    #[test]
    fn test_create_task_event_with_extra_data() {
        let extra_data = json!({"extra_field": "extra_value", "number": 42});

        let (event, data) = create_task_event(
            HookType::TaskAfterStart,
            200,
            "Task with Extra",
            Some(2),
            Some(10),
            Some(extra_data.clone()),
        );

        let data_obj = data.as_object().unwrap();
        assert_eq!(data_obj.get("task_id").unwrap(), &200);
        assert_eq!(data_obj.get("task_name").unwrap(), &"Task with Extra");
        assert_eq!(data_obj.get("extra_field").unwrap(), &"extra_value");
        assert_eq!(data_obj.get("number").unwrap(), &42);
    }

    // ========================================================================
    // Тесты для create_project_event
    // ========================================================================

    #[test]
    fn test_create_project_event() {
        let (event, data) = create_project_event(
            HookType::ProjectAfterCreate,
            5,
            "Test Project",
            Some(1),
            None,
        );

        assert_eq!(event.name, "project.after_create");
        assert_eq!(event.context.project_id, Some(5));
        assert_eq!(event.context.user_id, Some(1));
        assert_eq!(event.context.task_id, None);

        let data_obj = data.as_object().unwrap();
        assert_eq!(data_obj.get("project_id").unwrap(), &5);
        assert_eq!(data_obj.get("project_name").unwrap(), &"Test Project");
    }

    #[test]
    fn test_create_project_event_with_extra_data() {
        let extra_data = json!({"description": "Test description", "active": true});

        let (event, data) = create_project_event(
            HookType::ProjectBeforeUpdate,
            10,
            "Updated Project",
            Some(5),
            Some(extra_data.clone()),
        );

        let data_obj = data.as_object().unwrap();
        assert_eq!(data_obj.get("project_id").unwrap(), &10);
        assert_eq!(data_obj.get("project_name").unwrap(), &"Updated Project");
        assert_eq!(data_obj.get("description").unwrap(), &"Test description");
        assert_eq!(data_obj.get("active").unwrap(), &true);
    }
}
