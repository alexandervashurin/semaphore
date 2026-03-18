//! Plugin System - Базовая архитектура плагинов
//!
//! Система плагинов позволяет расширять функциональность Velum UI
//! без изменения основного кода приложения.
//!
//! Поддерживаемые типы плагинов:
//! - Task Executors - кастомные исполнители задач
//! - Notification Providers - провайдеры уведомлений
//! - Storage Providers - провайдеры хранилищ
//! - Auth Providers - провайдеры аутентификации
//! - API Extensions - расширения API

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use crate::error::{Error, Result};
use crate::models::Task;

/// Информация о плагине
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub r#type: PluginType,
    pub enabled: bool,
    pub dependencies: Vec<String>,
    pub config_schema: Option<JsonValue>,
}

/// Тип плагина
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    TaskExecutor,
    NotificationProvider,
    StorageProvider,
    AuthProvider,
    ApiExtension,
    Hook,
    Custom,
}

/// Статус плагина
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginStatus {
    Loaded,
    Unloaded,
    Error(String),
    Disabled,
}

/// Конфигурация плагина
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginConfig {
    pub enabled: bool,
    pub settings: HashMap<String, JsonValue>,
    pub secrets: HashMap<String, String>,
}

/// Метаданные плагина
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub r#type: PluginType,
    pub min_semaphore_version: Option<String>,
    pub dependencies: Vec<PluginDependency>,
    pub config: Option<PluginConfigSchema>,
}

/// Зависимость плагина
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub id: String,
    pub version: String,
    pub required: bool,
}

/// Схема конфигурации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfigSchema {
    pub fields: Vec<ConfigField>,
}

/// Поле конфигурации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigField {
    pub name: String,
    pub r#type: String,
    pub label: String,
    pub description: Option<String>,
    pub required: bool,
    pub default: Option<JsonValue>,
    pub options: Option<Vec<JsonValue>>,
}

/// Контекст выполнения плагина
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginContext {
    pub plugin_id: String,
    pub project_id: Option<i64>,
    pub user_id: Option<i64>,
    pub task_id: Option<i64>,
    pub metadata: HashMap<String, JsonValue>,
}

/// Событие хука
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEvent {
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub data: JsonValue,
    pub context: PluginContext,
}

/// Результат выполнения хука
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    pub success: bool,
    pub data: Option<JsonValue>,
    pub error: Option<String>,
}

// ============================================================================
// Трейты плагинов
// ============================================================================

/// Базовый трейт плагина
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Получает информацию о плагине
    fn info(&self) -> PluginInfo;
    
    /// Инициализирует плагин
    async fn initialize(&mut self, config: PluginConfig) -> Result<()>;
    
    /// Загружает плагин
    async fn load(&mut self) -> Result<()>;
    
    /// Выгружает плагин
    async fn unload(&mut self) -> Result<()>;
    
    /// Проверяет статус плагина
    fn status(&self) -> PluginStatus;
    
    /// Получает конфигурацию
    fn get_config(&self) -> PluginConfig;
    
    /// Обновляет конфигурацию
    async fn update_config(&mut self, config: PluginConfig) -> Result<()>;
}

/// Трейт для плагинов-исполнителей задач
#[async_trait]
pub trait TaskExecutorPlugin: Plugin {
    /// Проверяет возможность выполнения задачи
    async fn can_execute(&self, task: &Task) -> bool;
    
    /// Выполняет задачу
    async fn execute(&self, context: PluginContext, task: &Task) -> Result<TaskResult>;
    
    /// Останавливает выполнение задачи
    async fn stop(&self, context: PluginContext, task_id: i64) -> Result<()>;
}

/// Результат выполнения задачи
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub output: String,
    pub exit_code: i32,
    pub duration_secs: f64,
    pub metadata: HashMap<String, JsonValue>,
}

/// Трейт для плагинов уведомлений
#[async_trait]
pub trait NotificationPlugin: Plugin {
    /// Отправляет уведомление
    async fn send(&self, context: PluginContext, notification: Notification) -> Result<()>;
    
    /// Получает доступные каналы уведомлений
    fn get_channels(&self) -> Vec<NotificationChannel>;
}

/// Уведомление
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub title: String,
    pub message: String,
    pub level: NotificationLevel,
    pub channels: Vec<String>,
    pub data: JsonValue,
}

/// Уровень уведомления
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Канал уведомления
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub config_schema: Option<JsonValue>,
}

/// Трейт для плагинов хуков
#[async_trait]
pub trait HookPlugin: Plugin {
    /// Возвращает список поддерживаемых хуков
    fn get_hooks(&self) -> Vec<String>;
    
    /// Выполняет хук
    async fn execute_hook(&self, event: HookEvent) -> Result<HookResult>;
}

/// Трейт для плагинов хранилищ
#[async_trait]
pub trait StoragePlugin: Plugin {
    /// Сохраняет данные
    async fn save(&self, key: &str, data: JsonValue) -> Result<()>;
    
    /// Загружает данные
    async fn load(&self, key: &str) -> Result<Option<JsonValue>>;
    
    /// Удаляет данные
    async fn delete(&self, key: &str) -> Result<()>;
    
    /// Список всех ключей
    async fn list(&self, prefix: Option<&str>) -> Result<Vec<String>>;
}

/// Трейт для плагинов аутентификации
#[async_trait]
pub trait AuthPlugin: Plugin {
    /// Аутентифицирует пользователя
    async fn authenticate(&self, credentials: AuthCredentials) -> Result<AuthResult>;
    
    /// Проверяет токен
    async fn validate_token(&self, token: &str) -> Result<AuthResult>;
    
    /// Создаёт токен
    async fn create_token(&self, user_id: i64) -> Result<String>;
}

/// Учётные данные
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredentials {
    pub username: String,
    pub password: Option<String>,
    pub token: Option<String>,
    pub provider: String,
    pub metadata: HashMap<String, String>,
}

/// Результат аутентификации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub success: bool,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub metadata: HashMap<String, JsonValue>,
    pub error: Option<String>,
}

/// Трейт для расширений API
#[async_trait]
pub trait ApiExtensionPlugin: Plugin {
    /// Возвращает маршруты API
    fn get_routes(&self) -> Vec<ApiRoute>;
}

/// Маршрут API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRoute {
    pub method: String,
    pub path: String,
    pub handler: String,
    pub description: String,
    pub requires_auth: bool,
    pub requires_admin: bool,
}

// ============================================================================
// Менеджер плагинов (базовая структура)
// ============================================================================

use tokio::sync::RwLock;

pub struct PluginManager {
    plugins: HashMap<String, Arc<RwLock<dyn Plugin>>>,
    hooks: HashMap<String, Vec<String>>, // hook_name -> plugin_ids
    config: PluginManagerConfig,
    /// WASM загрузчик плагинов
    wasm_loader: Option<crate::plugins::wasm_loader::WasmPluginLoader>,
    /// WASM runtime для выполнения плагинов
    wasm_runtime: Option<crate::plugins::wasm_runtime::WasmRuntime>,
}

/// Конфигурация менеджера плагинов
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginManagerConfig {
    pub plugins_dir: String,
    pub enabled_plugins: Vec<String>,
    pub disabled_plugins: Vec<String>,
    pub auto_load: bool,
    /// Конфигурация WASM плагинов
    pub wasm_enabled: bool,
    pub wasm_plugins_dir: Option<String>,
    pub wasm_max_memory_mb: u32,
    pub wasm_max_execution_secs: u64,
    pub wasm_allow_network: bool,
    pub wasm_allow_filesystem: bool,
}

impl PluginManager {
    /// Создаёт новый менеджер плагинов
    pub fn new(config: PluginManagerConfig) -> Self {
        // Инициализируем WASM загрузчик если включено
        let (wasm_loader, wasm_runtime) = if config.wasm_enabled {
            let wasm_config = crate::plugins::wasm_loader::WasmLoaderConfig {
                plugins_dir: config.wasm_plugins_dir.clone().map(std::path::PathBuf::from)
                    .unwrap_or_else(|| std::path::PathBuf::from("./plugins")),
                max_memory_pages: config.wasm_max_memory_mb * 16, // MB -> страницы (64KB)
                max_execution_time_secs: config.wasm_max_execution_secs,
                allow_network: config.wasm_allow_network,
                allow_filesystem: config.wasm_allow_filesystem,
                allow_env: false,
                allowed_host_calls: vec![
                    "semaphore:log".to_string(),
                    "semaphore:get_config".to_string(),
                    "semaphore:set_config".to_string(),
                    "semaphore:call_hook".to_string(),
                ],
            };
            
            match crate::plugins::wasm_loader::WasmPluginLoader::new(wasm_config) {
                Ok(loader) => {
                    tracing::info!("WASM plugin loader initialized");
                    (Some(loader), None) // wasm_runtime создаётся позже
                }
                Err(e) => {
                    tracing::error!("Failed to initialize WASM plugin loader: {}", e);
                    (None, None)
                }
            }
        } else {
            (None, None)
        };
        
        Self {
            plugins: HashMap::new(),
            hooks: HashMap::new(),
            config,
            wasm_loader,
            wasm_runtime,
        }
    }

    /// Инициализирует WASM runtime
    pub async fn initialize_wasm_runtime(&mut self) -> Result<()> {
        if let Some(loader) = &self.wasm_loader {
            match crate::plugins::wasm_runtime::WasmRuntime::new(loader) {
                Ok(runtime) => {
                    self.wasm_runtime = Some(runtime);
                    tracing::info!("WASM runtime initialized");
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Failed to initialize WASM runtime: {}", e);
                    Err(e)
                }
            }
        } else {
            Ok(()) // WASM не включён
        }
    }

    /// Загружает все WASM плагины
    pub async fn load_wasm_plugins(&mut self) -> Result<Vec<crate::plugins::wasm_loader::WasmPluginMetadata>> {
        if let Some(loader) = &mut self.wasm_loader {
            match loader.load_all_plugins().await {
                Ok(plugins) => {
                    tracing::info!("Loaded {} WASM plugin(s)", plugins.len());
                    Ok(plugins)
                }
                Err(e) => {
                    tracing::error!("Failed to load WASM plugins: {}", e);
                    Err(e)
                }
            }
        } else {
            Ok(Vec::new()) // WASM не включён
        }
    }

    /// Выгружает WASM плагин
    pub fn unload_wasm_plugin(&mut self, plugin_id: &str) -> Result<()> {
        if let Some(loader) = &mut self.wasm_loader {
            loader.unload_plugin(plugin_id)
        } else {
            Err(Error::NotFound("WASM loader not initialized".to_string()))
        }
    }

    /// Получает список загруженных WASM плагинов
    pub fn list_wasm_plugins(&self) -> Vec<&crate::plugins::wasm_loader::WasmPluginMetadata> {
        if let Some(loader) = &self.wasm_loader {
            loader.list_loaded_plugins()
        } else {
            Vec::new()
        }
    }

    /// Вызывает хук в WASM плагинах
    pub async fn trigger_wasm_hooks(
        &self,
        hook_type: crate::plugins::hooks::HookType,
        data: serde_json::Value,
        context: crate::plugins::base::PluginContext,
    ) -> Result<Vec<crate::plugins::base::HookResult>> {
        // В полной реализации здесь будет вызов WASM плагинов
        // Для пока возвращаем пустой результат
        tracing::debug!("WASM hook triggered: {:?}", hook_type);
        Ok(Vec::new())
    }
    
    /// Регистрирует плагин
    pub async fn register(&mut self, plugin: Arc<RwLock<dyn Plugin>>) -> Result<()> {
        let info = {
            let plugin_guard = plugin.read().await;
            plugin_guard.info()
        };
        
        if self.plugins.contains_key(&info.id) {
            return Err(Error::Other(format!("Plugin {} already registered", info.id)));
        }
        
        // Проверяем зависимости
        for dep in &info.dependencies {
            if !self.plugins.contains_key(dep) && !self.is_plugin_optional(dep) {
                return Err(Error::Other(format!(
                    "Missing required dependency: {}",
                    dep
                )));
            }
        }
        
        self.plugins.insert(info.id.clone(), plugin);
        
        Ok(())
    }
    
    /// Загружает все плагины
    pub async fn load_all(&mut self) -> Result<()> {
        for plugin_id in self.config.enabled_plugins.clone() {
            if let Some(plugin) = self.plugins.get(&plugin_id) {
                let mut plugin_guard = plugin.write().await;
                if let Err(e) = plugin_guard.load().await {
                    tracing::error!("Failed to load plugin {}: {}", plugin_id, e);
                }
            }
        }
        Ok(())
    }
    
    /// Выгружает все плагины
    pub async fn unload_all(&mut self) -> Result<()> {
        let plugin_ids: Vec<String> = self.plugins.keys().cloned().collect();
        for plugin_id in plugin_ids {
            if let Some(plugin) = self.plugins.get(&plugin_id) {
                let mut plugin_guard = plugin.write().await;
                if let Err(e) = plugin_guard.unload().await {
                    tracing::error!("Failed to unload plugin {}: {}", plugin_id, e);
                }
            }
        }
        self.plugins.clear();
        Ok(())
    }
    
    /// Получает плагин по ID
    pub fn get_plugin(&self, plugin_id: &str) -> Option<Arc<RwLock<dyn Plugin>>> {
        self.plugins.get(plugin_id).cloned()
    }
    
    /// Получает список всех плагинов (включая WASM)
    pub async fn list_plugins(&self) -> Vec<PluginInfo> {
        let mut infos = Vec::new();
        
        // Добавляем нативные плагины
        for plugin in self.plugins.values() {
            let plugin_guard = plugin.read().await;
            infos.push(plugin_guard.info());
        }
        
        // Добавляем WASM плагины
        if let Some(loader) = &self.wasm_loader {
            for wasm_plugin in loader.list_loaded_plugins() {
                infos.push(wasm_plugin.info.clone());
            }
        }
        
        infos
    }
    
    /// Включает плагин
    pub fn enable_plugin(&mut self, plugin_id: &str) -> Result<()> {
        if !self.plugins.contains_key(plugin_id) {
            return Err(Error::NotFound(format!("Plugin {} not found", plugin_id)));
        }
        
        self.config.enabled_plugins.push(plugin_id.to_string());
        self.config.disabled_plugins.retain(|id| id != plugin_id);
        
        Ok(())
    }
    
    /// Отключает плагин
    pub fn disable_plugin(&mut self, plugin_id: &str) -> Result<()> {
        self.config.enabled_plugins.retain(|id| id != plugin_id);
        self.config.disabled_plugins.push(plugin_id.to_string());
        
        Ok(())
    }
    
    /// Проверяет, является ли плагин опциональным
    fn is_plugin_optional(&self, plugin_id: &str) -> bool {
        !self.config.enabled_plugins.contains(&plugin_id.to_string())
    }
}

// ============================================================================
// Макросы для упрощения создания плагинов
// ============================================================================

/// Макрос для объявления плагина
#[macro_export]
macro_rules! declare_plugin {
    ($name:ident, $type:expr, $version:expr, $description:expr, $author:expr) => {
        pub struct $name {
            config: PluginConfig,
            status: PluginStatus,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    config: PluginConfig::default(),
                    status: PluginStatus::Unloaded,
                }
            }
        }

        impl Plugin for $name {
            fn info(&self) -> PluginInfo {
                PluginInfo {
                    id: stringify!($name).to_string(),
                    name: stringify!($name).to_string(),
                    version: $version.to_string(),
                    description: $description.to_string(),
                    author: $author.to_string(),
                    r#type: $type,
                    enabled: self.config.enabled,
                    dependencies: vec![],
                    config_schema: None,
                }
            }

            async fn initialize(&mut self, config: PluginConfig) -> Result<()> {
                self.config = config;
                Ok(())
            }

            async fn load(&mut self) -> Result<()> {
                self.status = PluginStatus::Loaded;
                Ok(())
            }

            async fn unload(&mut self) -> Result<()> {
                self.status = PluginStatus::Unloaded;
                Ok(())
            }

            fn status(&self) -> PluginStatus {
                self.status.clone()
            }

            fn get_config(&self) -> PluginConfig {
                self.config.clone()
            }

            async fn update_config(&mut self, config: PluginConfig) -> Result<()> {
                self.config = config;
                Ok(())
            }
        }
    };
}

// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ========================================================================
    // Тесты для PluginType
    // ========================================================================

    #[test]
    fn test_plugin_type_serialization() {
        let types = vec![
            PluginType::TaskExecutor,
            PluginType::NotificationProvider,
            PluginType::StorageProvider,
            PluginType::AuthProvider,
            PluginType::ApiExtension,
            PluginType::Hook,
            PluginType::Custom,
        ];

        for plugin_type in types {
            let json = serde_json::to_string(&plugin_type).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_plugin_type_display() {
        assert_eq!(
            serde_json::to_string(&PluginType::TaskExecutor).unwrap(),
            "\"task_executor\""
        );
        assert_eq!(
            serde_json::to_string(&PluginType::NotificationProvider).unwrap(),
            "\"notification_provider\""
        );
        assert_eq!(
            serde_json::to_string(&PluginType::StorageProvider).unwrap(),
            "\"storage_provider\""
        );
    }

    // ========================================================================
    // Тесты для PluginStatus
    // ========================================================================

    #[test]
    fn test_plugin_status_display() {
        let status_loaded = PluginStatus::Loaded;
        let status_unloaded = PluginStatus::Unloaded;
        let status_disabled = PluginStatus::Disabled;
        let status_error = PluginStatus::Error("Test error".to_string());

        // Просто проверяем, что статусы создаются корректно
        assert!(format!("{:?}", status_loaded).contains("Loaded"));
        assert!(format!("{:?}", status_unloaded).contains("Unloaded"));
        assert!(format!("{:?}", status_disabled).contains("Disabled"));
        assert!(format!("{:?}", status_error).contains("Test error"));
    }

    // ========================================================================
    // Тесты для PluginConfig
    // ========================================================================

    #[test]
    fn test_plugin_config_default() {
        let config = PluginConfig::default();
        // derive(Default) устанавливает false для bool
        assert!(!config.enabled);
        assert!(config.settings.is_empty());
        assert!(config.secrets.is_empty());
    }

    #[test]
    fn test_plugin_config_with_settings() {
        let mut settings = HashMap::new();
        settings.insert("key".to_string(), json!("value"));

        let mut secrets = HashMap::new();
        secrets.insert("secret_key".to_string(), "secret_value".to_string());

        let config = PluginConfig {
            enabled: false,
            settings: settings.clone(),
            secrets: secrets.clone(),
        };

        assert!(!config.enabled);
        assert_eq!(config.settings.len(), 1);
        assert_eq!(config.secrets.len(), 1);
    }

    // ========================================================================
    // Тесты для PluginInfo
    // ========================================================================

    #[test]
    fn test_plugin_info_creation() {
        let info = PluginInfo {
            id: "test_plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            r#type: PluginType::TaskExecutor,
            enabled: true,
            dependencies: vec!["dep1".to_string(), "dep2".to_string()],
            config_schema: None,
        };

        assert_eq!(info.id, "test_plugin");
        assert_eq!(info.name, "Test Plugin");
        assert_eq!(info.version, "1.0.0");
        assert!(info.enabled);
        assert_eq!(info.dependencies.len(), 2);
    }

    #[test]
    fn test_plugin_info_serialization() {
        let info = PluginInfo {
            id: "my_plugin".to_string(),
            name: "My Plugin".to_string(),
            version: "2.0.0".to_string(),
            description: "Description".to_string(),
            author: "Author".to_string(),
            r#type: PluginType::Hook,
            enabled: false,
            dependencies: vec![],
            config_schema: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"id\":\"my_plugin\""));
        assert!(json.contains("\"enabled\":false"));
        assert!(json.contains("\"type\":\"hook\""));
    }

    // ========================================================================
    // Тесты для PluginManifest
    // ========================================================================

    #[test]
    fn test_plugin_manifest_creation() {
        let manifest = PluginManifest {
            id: "manifest_plugin".to_string(),
            name: "Manifest Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Plugin with manifest".to_string(),
            author: "Author".to_string(),
            homepage: Some("https://example.com".to_string()),
            repository: Some("https://github.com/example/plugin".to_string()),
            license: Some("MIT".to_string()),
            r#type: PluginType::NotificationProvider,
            min_semaphore_version: Some("2.0.0".to_string()),
            dependencies: vec![],
            config: None,
        };

        assert_eq!(manifest.id, "manifest_plugin");
        assert!(manifest.homepage.is_some());
        assert!(manifest.repository.is_some());
        assert_eq!(manifest.license, Some("MIT".to_string()));
    }

    // ========================================================================
    // Тесты для PluginDependency
    // ========================================================================

    #[test]
    fn test_plugin_dependency() {
        let dep = PluginDependency {
            id: "dependency_plugin".to_string(),
            version: "1.0.0".to_string(),
            required: true,
        };

        assert_eq!(dep.id, "dependency_plugin");
        assert!(dep.required);
    }

    // ========================================================================
    // Тесты для PluginContext
    // ========================================================================

    #[test]
    fn test_plugin_context_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), json!("value"));

        let context = PluginContext {
            plugin_id: "test_plugin".to_string(),
            project_id: Some(1),
            user_id: Some(42),
            task_id: Some(100),
            metadata: metadata.clone(),
        };

        assert_eq!(context.plugin_id, "test_plugin");
        assert_eq!(context.project_id, Some(1));
        assert_eq!(context.user_id, Some(42));
        assert_eq!(context.task_id, Some(100));
    }

    // ========================================================================
    // Тесты для HookEvent
    // ========================================================================

    #[test]
    fn test_hook_event_creation() {
        let context = PluginContext {
            plugin_id: "test".to_string(),
            project_id: None,
            user_id: None,
            task_id: None,
            metadata: HashMap::new(),
        };

        let event = HookEvent {
            name: "test_hook".to_string(),
            timestamp: Utc::now(),
            data: json!({"key": "value"}),
            context: context.clone(),
        };

        assert_eq!(event.name, "test_hook");
        assert_eq!(event.context.plugin_id, "test");
    }

    // ========================================================================
    // Тесты для HookResult
    // ========================================================================

    #[test]
    fn test_hook_result_success() {
        let result = HookResult {
            success: true,
            data: Some(json!({"result": "ok"})),
            error: None,
        };

        assert!(result.success);
        assert!(result.data.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_hook_result_error() {
        let result = HookResult {
            success: false,
            data: None,
            error: Some("Test error".to_string()),
        };

        assert!(!result.success);
        assert!(result.error.is_some());
    }

    // ========================================================================
    // Тесты для TaskResult
    // ========================================================================

    #[test]
    fn test_task_result() {
        let mut metadata = HashMap::new();
        metadata.insert("exit_status".to_string(), json!("success"));

        let result = TaskResult {
            success: true,
            output: "Task output".to_string(),
            exit_code: 0,
            duration_secs: 1.5,
            metadata: metadata.clone(),
        };

        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.duration_secs, 1.5);
    }

    // ========================================================================
    // Тесты для Notification
    // ========================================================================

    #[test]
    fn test_notification() {
        let notification = Notification {
            title: "Test Notification".to_string(),
            message: "Test message".to_string(),
            level: NotificationLevel::Info,
            channels: vec!["email".to_string()],
            data: json!({}),
        };

        assert_eq!(notification.title, "Test Notification");
        assert_eq!(notification.level, NotificationLevel::Info);
        assert_eq!(notification.channels.len(), 1);
    }

    #[test]
    fn test_notification_level_serialization() {
        assert_eq!(
            serde_json::to_string(&NotificationLevel::Info).unwrap(),
            "\"info\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationLevel::Warning).unwrap(),
            "\"warning\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationLevel::Error).unwrap(),
            "\"error\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationLevel::Critical).unwrap(),
            "\"critical\""
        );
    }

    // ========================================================================
    // Тесты для NotificationChannel
    // ========================================================================

    #[test]
    fn test_notification_channel() {
        let channel = NotificationChannel {
            id: "email_channel".to_string(),
            name: "Email".to_string(),
            description: "Email notifications".to_string(),
            config_schema: None,
        };

        assert_eq!(channel.id, "email_channel");
        assert_eq!(channel.name, "Email");
    }

    // ========================================================================
    // Тесты для AuthCredentials
    // ========================================================================

    #[test]
    fn test_auth_credentials() {
        let mut metadata = HashMap::new();
        metadata.insert("provider_key".to_string(), "value".to_string());

        let credentials = AuthCredentials {
            username: "user".to_string(),
            password: Some("pass".to_string()),
            token: None,
            provider: "test_provider".to_string(),
            metadata: metadata.clone(),
        };

        assert_eq!(credentials.username, "user");
        assert_eq!(credentials.password, Some("pass".to_string()));
        assert!(credentials.token.is_none());
    }

    // ========================================================================
    // Тесты для AuthResult
    // ========================================================================

    #[test]
    fn test_auth_result_success() {
        let mut metadata = HashMap::new();
        metadata.insert("auth_key".to_string(), json!("value"));

        let result = AuthResult {
            success: true,
            user_id: Some(1),
            username: Some("user".to_string()),
            email: Some("user@example.com".to_string()),
            metadata: metadata.clone(),
            error: None,
        };

        assert!(result.success);
        assert_eq!(result.user_id, Some(1));
        assert_eq!(result.username, Some("user".to_string()));
    }

    #[test]
    fn test_auth_result_error() {
        let result = AuthResult {
            success: false,
            user_id: None,
            username: None,
            email: None,
            metadata: HashMap::new(),
            error: Some("Invalid credentials".to_string()),
        };

        assert!(!result.success);
        assert!(result.error.is_some());
    }

    // ========================================================================
    // Тесты для ApiRoute
    // ========================================================================

    #[test]
    fn test_api_route() {
        let route = ApiRoute {
            method: "GET".to_string(),
            path: "/api/v1/test".to_string(),
            handler: "test_handler".to_string(),
            description: "Test API route".to_string(),
            requires_auth: true,
            requires_admin: false,
        };

        assert_eq!(route.method, "GET");
        assert_eq!(route.path, "/api/v1/test");
        assert!(route.requires_auth);
        assert!(!route.requires_admin);
    }

    // ========================================================================
    // Тесты для PluginManagerConfig
    // ========================================================================

    #[test]
    fn test_plugin_manager_config_default() {
        let config = PluginManagerConfig::default();
        assert!(config.plugins_dir.is_empty());
        assert!(config.enabled_plugins.is_empty());
        assert!(config.disabled_plugins.is_empty());
        assert!(!config.auto_load);
        assert!(!config.wasm_enabled);
    }

    #[test]
    fn test_plugin_manager_config_custom() {
        let config = PluginManagerConfig {
            plugins_dir: "/plugins".to_string(),
            enabled_plugins: vec!["plugin1".to_string()],
            disabled_plugins: vec!["plugin2".to_string()],
            auto_load: true,
            wasm_enabled: true,
            wasm_plugins_dir: Some("/wasm_plugins".to_string()),
            wasm_max_memory_mb: 64,
            wasm_max_execution_secs: 30,
            wasm_allow_network: false,
            wasm_allow_filesystem: true,
        };

        assert_eq!(config.plugins_dir, "/plugins");
        assert!(config.auto_load);
        assert!(config.wasm_enabled);
        assert_eq!(config.wasm_max_memory_mb, 64);
    }

    // ========================================================================
    // Тесты для PluginManager
    // ========================================================================

    #[test]
    fn test_plugin_manager_creation() {
        let config = PluginManagerConfig::default();
        let manager = PluginManager::new(config);
        
        // Проверяем, что менеджер создан
        assert!(manager.plugins.is_empty());
        assert!(manager.hooks.is_empty());
    }

    #[test]
    fn test_plugin_manager_enable_disable_plugin() {
        let mut config = PluginManagerConfig::default();
        config.enabled_plugins = vec!["test_plugin".to_string()];
        
        let mut manager = PluginManager::new(config);
        
        // Добавим тестовый плагин вручную для проверки
        manager.config.enabled_plugins.push("plugin_to_disable".to_string());
        
        // Отключаем плагин
        let result = manager.disable_plugin("plugin_to_disable");
        assert!(result.is_ok());
        assert!(manager.config.disabled_plugins.contains(&"plugin_to_disable".to_string()));
    }

    #[test]
    fn test_plugin_manager_enable_not_found() {
        let config = PluginManagerConfig::default();
        let mut manager = PluginManager::new(config);
        
        // Пытаемся включить несуществующий плагин
        let result = manager.enable_plugin("nonexistent_plugin");
        assert!(result.is_err());
    }
}
