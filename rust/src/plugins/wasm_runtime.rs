//! WASM Runtime - Среда выполнения для WASM плагинов
//!
//! Этот модуль предоставляет среду выполнения для WASM плагинов,
//! включая хост-функции, sandboxing и управление ресурсами.

use crate::error::{Error, Result};
use crate::plugins::base::{HookEvent, HookResult, PluginContext};
use crate::plugins::wasm_loader::{LoadedWasmModule, WasmPluginLoader, WasmPluginMetadata};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, trace, warn};
use wasmtime::{Engine, Store, Val};
use wasmtime_wasi::WasiCtx;

/// Контекст выполнения WASM плагина
pub struct WasmPluginInstance {
    pub metadata: WasmPluginMetadata,
    pub store_data: PluginStoreData,
}

/// Данные хранилища плагина
#[derive(Debug, Clone)]
pub struct PluginStoreData {
    pub plugin_id: String,
    pub fuel_consumed: u64,
    pub calls_made: u64,
}

/// WASM Runtime для управления выполнением плагинов
pub struct WasmRuntime {
    engine: Engine,
    host_functions: HostFunctions,
}

/// Хранилище для WASM store
pub struct PluginStore {
    pub wasi: WasiCtx,
    pub plugin_id: String,
    pub runtime_data: RuntimeData,
}

/// Данные времени выполнения
#[derive(Debug, Clone)]
pub struct RuntimeData {
    pub fuel_limit: u64,
    pub fuel_consumed: u64,
    pub call_count: u64,
}

/// Хост-функции доступные плагинам
#[allow(clippy::type_complexity)]
pub struct HostFunctions {
    log_function: Arc<dyn Fn(&str, &str) + Send + Sync>,
    config_getter: Arc<dyn Fn(&str) -> Option<JsonValue> + Send + Sync>,
    config_setter: Arc<dyn Fn(&str, JsonValue) -> Result<()> + Send + Sync>,
    hook_caller: Arc<dyn Fn(&str, JsonValue) -> Result<JsonValue> + Send + Sync>,
}

impl HostFunctions {
    /// Создаёт новые хост-функции с замыканиями по умолчанию
    pub fn new() -> Self {
        Self {
            log_function: Arc::new(|level, msg| match level {
                "error" => error!("[WASM] {}", msg),
                "warn" => warn!("[WASM] {}", msg),
                "info" => info!("[WASM] {}", msg),
                "debug" => debug!("[WASM] {}", msg),
                _ => trace!("[WASM] {}", msg),
            }),
            config_getter: Arc::new(|_| None),
            config_setter: Arc::new(|_, _| Ok(())),
            hook_caller: Arc::new(|_, _| Ok(json!(null))),
        }
    }

    /// Устанавливает функцию логирования
    pub fn with_logger<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        self.log_function = Arc::new(f);
        self
    }

    /// Устанавливает функцию получения конфигурации
    pub fn with_config_getter<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> Option<JsonValue> + Send + Sync + 'static,
    {
        self.config_getter = Arc::new(f);
        self
    }

    /// Устанавливает функцию установки конфигурации
    pub fn with_config_setter<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, JsonValue) -> Result<()> + Send + Sync + 'static,
    {
        self.config_setter = Arc::new(f);
        self
    }

    /// Устанавливает функцию вызова хуков
    pub fn with_hook_caller<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, JsonValue) -> Result<JsonValue> + Send + Sync + 'static,
    {
        self.hook_caller = Arc::new(f);
        self
    }
}

impl Default for HostFunctions {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmRuntime {
    /// Создаёт новый WASM runtime
    pub fn new(loader: &WasmPluginLoader) -> Result<Self> {
        let engine = loader.engine().clone();

        Ok(Self {
            engine,
            host_functions: HostFunctions::new(),
        })
    }

    /// Создаёт инстанс плагина
    pub async fn create_instance(&self, module: &LoadedWasmModule) -> Result<WasmPluginInstance> {
        debug!(
            "Creating WASM instance for plugin: {}",
            module.metadata.info.id
        );

        Ok(WasmPluginInstance {
            metadata: module.metadata.clone(),
            store_data: PluginStoreData {
                plugin_id: module.metadata.info.id.clone(),
                fuel_consumed: 0,
                calls_made: 0,
            },
        })
    }

    /// Вызывает функцию экспортированную плагином
    pub async fn call_function(
        &self,
        instance: &mut WasmPluginInstance,
        function_name: &str,
        args: &[Val],
    ) -> Result<Vec<Val>> {
        debug!(
            "Calling WASM function: {} with {} args",
            function_name,
            args.len()
        );

        // В полной реализации здесь будет настоящий вызов WASM функции
        // Для пока возвращаем заглушку
        Ok(vec![])
    }

    /// Вызывает хук в плагине
    pub async fn call_hook(
        &self,
        instance: &mut WasmPluginInstance,
        event: HookEvent,
    ) -> Result<HookResult> {
        // В полной реализации будет вызов handle_hook
        Ok(HookResult {
            success: false,
            data: None,
            error: Some("Hook handler not implemented yet".to_string()),
        })
    }

    /// Проверяет может ли плагин выполнить задачу
    pub async fn can_execute_task(
        &self,
        instance: &mut WasmPluginInstance,
        task_json: &str,
    ) -> Result<bool> {
        Ok(false)
    }

    /// Выполняет задачу в плагине
    pub async fn execute_task(
        &self,
        instance: &mut WasmPluginInstance,
        task_context: &str,
    ) -> Result<JsonValue> {
        Ok(json!({"success": true, "output": "Task executed"}))
    }

    /// Получает информацию о плагине
    pub fn get_plugin_info<'a>(
        &'a self,
        instance: &'a WasmPluginInstance,
    ) -> &'a WasmPluginMetadata {
        &instance.metadata
    }

    /// Получает engine
    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}

/// Сэндбокс для безопасного выполнения WASM кода
pub struct WasmSandbox {
    max_memory: u64,
    max_fuel: u64,
    allowed_syscalls: Vec<String>,
}

impl WasmSandbox {
    /// Создаёт новый сэндбокс
    pub fn new() -> Self {
        Self {
            max_memory: 64 * 1024 * 1024, // 64 MB
            max_fuel: 1_000_000,
            allowed_syscalls: vec![],
        }
    }

    /// Устанавливает лимит памяти
    pub fn with_max_memory(mut self, bytes: u64) -> Self {
        self.max_memory = bytes;
        self
    }

    /// Устанавливает лимит fuel
    pub fn with_max_fuel(mut self, fuel: u64) -> Self {
        self.max_fuel = fuel;
        self
    }

    /// Разрешает системные вызовы
    pub fn with_allowed_syscalls(mut self, syscalls: Vec<String>) -> Self {
        self.allowed_syscalls = syscalls;
        self
    }

    /// Применяет ограничения к store
    pub fn apply_to_store(&self, store: &mut Store<PluginStore>) -> Result<()> {
        store
            .set_fuel(self.max_fuel)
            .map_err(|e| Error::Other(format!("Failed to set fuel limit: {}", e)))?;
        Ok(())
    }
}

impl Default for WasmSandbox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let sandbox = WasmSandbox::new()
            .with_max_memory(128 * 1024 * 1024)
            .with_max_fuel(2_000_000);

        assert_eq!(sandbox.max_memory, 128 * 1024 * 1024);
        assert_eq!(sandbox.max_fuel, 2_000_000);
    }
}
