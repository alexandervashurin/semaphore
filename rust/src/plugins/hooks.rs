//! Plugin Hooks - Система хуков для плагинов
//!
//! Хуки позволяют плагинам реагировать на события системы
//! и модифицировать поведение приложения.

use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use crate::error::{Error, Result};
use crate::plugins::base::{HookEvent, HookResult, PluginContext};

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
    pub async fn trigger(&self, hook_type: HookType, data: JsonValue, context: PluginContext) -> Result<Vec<HookResult>> {
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

/// Helper функции для создания событий

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
