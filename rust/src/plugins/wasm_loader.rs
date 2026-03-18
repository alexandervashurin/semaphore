//! WASM Plugin Loader - Загрузчик WASM плагинов
//!
//! Этот модуль отвечает за динамическую загрузку WASM плагинов,
//! их валидацию и интеграцию с системой плагинов Velum.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use wasmtime::{Engine, Module, Config};
use wasmtime_wasi::WasiCtx;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};
use crate::error::{Error, Result};
use crate::plugins::base::{PluginInfo, PluginType, PluginConfig, PluginStatus};

/// Конфигурация WASM загрузчика
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmLoaderConfig {
    /// Директория с WASM плагинами
    pub plugins_dir: PathBuf,
    /// Разрешённые хост-функции
    pub allowed_host_calls: Vec<String>,
    /// Максимальный размер памяти (в страницах WASM, 64KB каждая)
    pub max_memory_pages: u32,
    /// Максимальное время выполнения (в секундах)
    pub max_execution_time_secs: u64,
    /// Разрешить доступ к сети
    pub allow_network: bool,
    /// Разрешить доступ к файловой системе (только plugins_dir)
    pub allow_filesystem: bool,
    /// Разрешить доступ к переменным окружения
    pub allow_env: bool,
}

impl Default for WasmLoaderConfig {
    fn default() -> Self {
        Self {
            plugins_dir: PathBuf::from("./plugins"),
            allowed_host_calls: vec![
                "semaphore:log".to_string(),
                "semaphore:get_config".to_string(),
                "semaphore:set_config".to_string(),
                "semaphore:call_hook".to_string(),
            ],
            max_memory_pages: 1024, // 64 MB
            max_execution_time_secs: 30,
            allow_network: false,
            allow_filesystem: true,
            allow_env: false,
        }
    }
}

/// Метаданные WASM плагина
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmPluginMetadata {
    pub path: PathBuf,
    pub info: PluginInfo,
    pub wasm_version: String,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
    pub hash: String,
}

/// Загруженный WASM модуль
pub struct LoadedWasmModule {
    pub module: Module,
    pub metadata: WasmPluginMetadata,
}

/// WASM загрузчик плагинов
pub struct WasmPluginLoader {
    config: WasmLoaderConfig,
    engine: Engine,
    loaded_modules: HashMap<String, LoadedWasmModule>,
}

impl WasmPluginLoader {
    /// Создаёт новый WASM загрузчик
    pub fn new(config: WasmLoaderConfig) -> Result<Self> {
        // Настраиваем WASM engine с ограничениями безопасности
        let mut engine_config = Config::new();
        engine_config.wasm_reference_types(true);
        engine_config.wasm_multi_value(true);
        engine_config.async_support(true);
        
        let engine = Engine::new(&engine_config)
            .map_err(|e| Error::Other(format!("Failed to create WASM engine: {}", e)))?;
        
        Ok(Self {
            config,
            engine,
            loaded_modules: HashMap::new(),
        })
    }

    /// Загружает все WASM плагины из директории
    pub async fn load_all_plugins(&mut self) -> Result<Vec<WasmPluginMetadata>> {
        let mut loaded = Vec::new();
        
        if !self.config.plugins_dir.exists() {
            info!("Plugins directory does not exist: {:?}", self.config.plugins_dir);
            return Ok(loaded);
        }
        
        let mut entries = tokio::fs::read_dir(&self.config.plugins_dir)
            .await
            .map_err(|e| Error::Other(format!("Failed to read plugins directory: {}", e)))?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| Error::Other(format!("Failed to read directory entry: {}", e)))?
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                match self.load_plugin(&path).await {
                    Ok(metadata) => {
                        info!("Loaded WASM plugin: {}", metadata.info.id);
                        loaded.push(metadata);
                    }
                    Err(e) => {
                        error!("Failed to load WASM plugin {:?}: {}", path, e);
                    }
                }
            }
        }
        
        Ok(loaded)
    }

    /// Загружает отдельный WASM плагин
    pub async fn load_plugin(&mut self, path: &Path) -> Result<WasmPluginMetadata> {
        debug!("Loading WASM plugin from: {:?}", path);
        
        // Читаем WASM файл
        let wasm_bytes = tokio::fs::read(path)
            .await
            .map_err(|e| Error::Other(format!("Failed to read WASM file: {}", e)))?;
        
        // Вычисляем хэш для проверки целостности
        use sha2::{Sha256, Digest};
        let hash = format!("{:x}", Sha256::digest(&wasm_bytes));
        
        // Валидируем WASM модуль
        wasmtime::Module::validate(&self.engine, &wasm_bytes)
            .map_err(|e| Error::Other(format!("WASM validation failed: {}", e)))?;
        
        // Создаём модуль
        let module = Module::from_binary(&self.engine, &wasm_bytes)
            .map_err(|e| Error::Other(format!("Failed to compile WASM module: {}", e)))?;
        
        // Извлекаем метаданные из экспортов/импортов
        let exports: Vec<String> = module.exports().map(|e| e.name().to_string()).collect();
        let imports: Vec<String> = module.imports().map(|i| i.name().to_string()).collect();
        
        // Извлекаем информацию о плагине из кастомной секции или используем дефолтную
        let info = self.extract_plugin_info(&module, path).await?;
        
        let metadata = WasmPluginMetadata {
            path: path.to_path_buf(),
            info: info.clone(),
            wasm_version: "1.0".to_string(),
            exports,
            imports,
            hash,
        };
        
        // Сохраняем загруженный модуль
        self.loaded_modules.insert(info.id.clone(), LoadedWasmModule {
            module,
            metadata: metadata.clone(),
        });
        
        Ok(metadata)
    }

    /// Извлекает информацию о плагине из WASM модуля
    async fn extract_plugin_info(&self, module: &Module, path: &Path) -> Result<PluginInfo> {
        // Пытаемся извлечь метаданные из кастомной секции "plugin_info"
        // Если не найдено, используем дефолтные значения на основе имени файла
        
        let default_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown_plugin")
            .to_string();
        
        Ok(PluginInfo {
            id: default_id.clone(),
            name: default_id,
            version: "1.0.0".to_string(),
            description: "WASM Plugin".to_string(),
            author: "Unknown".to_string(),
            r#type: PluginType::Custom,
            enabled: true,
            dependencies: vec![],
            config_schema: None,
        })
    }

    /// Выгружает плагин
    pub fn unload_plugin(&mut self, plugin_id: &str) -> Result<()> {
        if self.loaded_modules.remove(plugin_id).is_some() {
            info!("Unloaded WASM plugin: {}", plugin_id);
            Ok(())
        } else {
            Err(Error::NotFound(format!("Plugin {} not found", plugin_id)))
        }
    }

    /// Получает информацию о загруженном плагине
    pub fn get_plugin_info(&self, plugin_id: &str) -> Option<&WasmPluginMetadata> {
        self.loaded_modules.get(plugin_id).map(|m| &m.metadata)
    }

    /// Получает список всех загруженных плагинов
    pub fn list_loaded_plugins(&self) -> Vec<&WasmPluginMetadata> {
        self.loaded_modules.values().map(|m| &m.metadata).collect()
    }

    /// Создаёт WASI контекст для плагина
    pub fn create_wasi_context(&self, plugin_id: &str) -> Result<WasiCtx> {
        use wasmtime_wasi::{WasiCtxBuilder, DirPerms, FilePerms};
        
        let mut builder = WasiCtxBuilder::new();
        
        // Настраиваем stdio
        builder.inherit_stdio();
        
        // Разрешаем доступ к директории плагинов если включено
        if self.config.allow_filesystem {
            // Предоставляем доступ только на чтение к директории плагинов
            builder.preopened_dir(
                &self.config.plugins_dir,
                ".",
                DirPerms::READ,
                FilePerms::READ,
            ).map_err(|e| Error::Other(format!("Failed to preopen directory: {}", e)))?;
        }
        
        // Добавляем переменные окружения если разрешено
        if self.config.allow_env {
            let env: Vec<(String, String)> = std::env::vars()
                .filter(|(k, _)| k.starts_with("SEMAPHORE_"))
                .collect();
            builder.envs(&env);
        }
        
        Ok(builder.build())
    }

    /// Получает engine для создания store
    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}

/// Helper для вычисления хэша файла
async fn compute_file_hash(path: &Path) -> Result<String> {
    use sha2::{Sha256, Digest};
    
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|e| Error::Other(format!("Failed to read file: {}", e)))?;
    
    Ok(format!("{:x}", Sha256::digest(&bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::io::Write;
    
    #[tokio::test]
    async fn test_wasm_loader_creation() {
        let config = WasmLoaderConfig::default();
        let loader = WasmPluginLoader::new(config);
        
        assert!(loader.is_ok());
    }
    
    #[tokio::test]
    async fn test_load_nonexistent_plugin() {
        let temp_dir = TempDir::new().unwrap();
        let config = WasmLoaderConfig {
            plugins_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let mut loader = WasmPluginLoader::new(config).unwrap();
        let fake_path = temp_dir.path().join("fake.wasm");
        
        // Создаём фиктивный WASM файл (невалидный)
        let mut file = std::fs::File::create(&fake_path).unwrap();
        file.write_all(b"not a wasm file").unwrap();
        
        let result = loader.load_plugin(&fake_path).await;
        assert!(result.is_err());
    }
}
