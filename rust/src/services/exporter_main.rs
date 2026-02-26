//! Exporter Main - главный экспортер
//!
//! Аналог services/export/Exporter.go из Go версии

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

use crate::models::*;
use crate::db::store::Store;

/// Цепочка экспортеров
pub struct ExporterChain {
    /// Маппер ключей
    pub mapper: TypeKeyMapper,
    
    /// Экспортеры по типам
    pub exporters: HashMap<String, Box<dyn TypeExporter>>,
}

/// Маппер ключей
pub struct TypeKeyMapper {
    /// Мапа ключей
    key_maps: HashMap<String, HashMap<String, String>>,
}

/// Мапа значений
pub struct ValueMap<T> {
    /// Значения
    values: HashMap<String, Vec<T>>,
    /// Ошибки
    errors: Vec<String>,
}

/// Трейт для экспортера типа
pub trait TypeExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, exporter: &dyn DataExporter) -> Result<(), String>;
    
    /// Восстанавливает данные
    fn restore(&mut self, store: &dyn Store, exporter: &dyn DataExporter) -> Result<(), String>;
    
    /// Получает загруженные ключи
    fn get_loaded_keys(&self, scope: &str) -> Result<Vec<String>, String>;
    
    /// Получает загруженные значения
    fn get_loaded_values(&self, scope: &str) -> Result<Vec<Box<dyn std::any::Any>>, String>;
    
    /// Получает имя
    fn get_name(&self) -> &str;
    
    /// Получает зависимости экспорта
    fn export_depends_on(&self) -> Vec<&str>;
    
    /// Получает зависимости импорта
    fn import_depends_on(&self) -> Vec<&str>;
    
    /// Получает ошибки
    fn get_errors(&self) -> Vec<String>;
    
    /// Очищает
    fn clear(&mut self);
}

/// Трейт для экспортера данных
pub trait DataExporter {
    /// Получает экспортер типа
    fn get_type_exporter(&mut self, name: &str) -> Option<&mut dyn TypeExporter>;
    
    /// Получает загруженные ключи
    fn get_loaded_keys(&self, name: &str, scope: &str) -> Result<Vec<String>, String>;
    
    /// Получает загруженные ключи int
    fn get_loaded_keys_int(&self, name: &str, scope: &str) -> Result<Vec<i32>, String>;
}

impl ExporterChain {
    /// Создаёт новую цепочку экспортеров
    pub fn new() -> Self {
        Self {
            mapper: TypeKeyMapper::new(),
            exporters: HashMap::new(),
        }
    }
    
    /// Добавляет экспортер
    pub fn add_exporter(&mut self, name: &str, exporter: Box<dyn TypeExporter>) {
        self.exporters.insert(name.to_string(), exporter);
    }
    
    /// Получает экспортер
    pub fn get_type_exporter(&mut self, name: &str) -> Option<&mut Box<dyn TypeExporter>> {
        self.exporters.get_mut(name)
    }
    
    /// Получает загруженные ключи
    pub fn get_loaded_keys(&self, name: &str, scope: &str) -> Result<Vec<String>, String> {
        if let Some(exporter) = self.exporters.get(name) {
            exporter.get_loaded_keys(scope)
        } else {
            Err(format!("Exporter {} not found", name))
        }
    }
    
    /// Загружает данные
    pub fn load(&mut self, store: &dyn Store) -> Result<(), String> {
        let sorted_keys = Self::get_sorted_keys(&self.exporters, |e| e.export_depends_on())?;
        
        for key in sorted_keys {
            if let Some(exporter) = self.exporters.get_mut(&key) {
                exporter.load(store, self)
                    .map_err(|e| format!("Failed to load {}: {}", key, e))?;
            }
        }
        
        Ok(())
    }
    
    /// Восстанавливает данные
    pub fn restore(&mut self, store: &dyn Store) -> Result<(), String> {
        let sorted_keys = Self::get_sorted_keys(&self.exporters, |e| e.import_depends_on())?;
        
        for key in sorted_keys {
            if let Some(exporter) = self.exporters.get_mut(&key) {
                exporter.restore(store, self)
                    .map_err(|e| format!("Failed to restore {}: {}", key, e))?;
            }
        }
        
        Ok(())
    }
    
    /// Сортирует ключи по зависимостям
    pub fn get_sorted_keys(
        exporters: &HashMap<String, Box<dyn TypeExporter>>,
        depends_on: fn(&dyn TypeExporter) -> Vec<&str>,
    ) -> Result<Vec<String>, String> {
        let mut sorted = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut visiting = std::collections::HashSet::new();
        
        fn visit(
            name: &str,
            exporters: &HashMap<String, Box<dyn TypeExporter>>,
            sorted: &mut Vec<String>,
            visited: &mut std::collections::HashSet<String>,
            visiting: &mut std::collections::HashSet<String>,
            depends_on: fn(&dyn TypeExporter) -> Vec<&str>,
        ) -> Result<(), String> {
            if visiting.contains(name) {
                return Err(format!("Circular dependency detected: {}", name));
            }
            
            if visited.contains(name) {
                return Ok(());
            }
            
            visiting.insert(name.to_string());
            
            if let Some(exporter) = exporters.get(name) {
                for dep in depends_on(exporter.as_ref()) {
                    visit(dep, exporters, sorted, visited, visiting, depends_on)?;
                }
            }
            
            visiting.remove(name);
            visited.insert(name.to_string());
            sorted.push(name.to_string());
            
            Ok(())
        }
        
        for name in exporters.keys() {
            visit(name, exporters, &mut sorted, &mut visited, &mut visiting, depends_on)?;
        }
        
        Ok(sorted)
    }
}

impl Default for ExporterChain {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeKeyMapper {
    /// Создаёт новый TypeKeyMapper
    pub fn new() -> Self {
        Self {
            key_maps: HashMap::new(),
        }
    }
    
    /// Получает новый ключ
    pub fn get_new_key(&mut self, name: &str, scope: &str, old_key: &str) -> Result<String, String> {
        let key = format!("{}.{}", name, scope);
        
        if let Some(map) = self.key_maps.get(&key) {
            if let Some(new_key) = map.get(old_key) {
                return Ok(new_key.clone());
            }
        }
        
        Ok(old_key.to_string())
    }
    
    /// Мапит ключи
    pub fn map_keys(&mut self, name: &str, scope: &str, old_key: &str, new_key: &str) -> Result<(), String> {
        let key = format!("{}.{}", name, scope);
        
        let map = self.key_maps.entry(key).or_insert_with(HashMap::new);
        map.insert(old_key.to_string(), new_key.to_string());
        
        Ok(())
    }
}

impl Default for TypeKeyMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ValueMap<T> {
    /// Создаёт новую ValueMap
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            errors: Vec::new(),
        }
    }
    
    /// Получает загруженные ключи
    pub fn get_loaded_keys(&self, scope: &str) -> Result<Vec<String>, String> {
        if let Some(values) = self.values.get(scope) {
            Ok((0..values.len()).map(|i| i.to_string()).collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Добавляет значения
    pub fn append_values(&mut self, values: Vec<T>, scope: &str) -> Result<(), String> {
        let entry = self.values.entry(scope.to_string()).or_insert_with(Vec::new);
        entry.extend(values);
        Ok(())
    }
    
    /// Получает ошибки
    pub fn get_errors(&self) -> Vec<String> {
        self.errors.clone()
    }
    
    /// Очищает
    pub fn clear(&mut self) {
        self.values.clear();
        self.errors.clear();
    }
}

impl<T> Default for ValueMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Инициализирует экспортеры проекта
pub fn init_project_exporters(mapper: &mut TypeKeyMapper, skip_task_output: bool) -> ExporterChain {
    let mut chain = ExporterChain::new();
    
    // Добавляем экспортеры в порядке зависимостей
    // User должен быть первым
    chain.add_exporter("User", Box::new(ValueMap::<User>::new()));
    
    // Затем AccessKey
    chain.add_exporter("AccessKey", Box::new(ValueMap::<AccessKey>::new()));
    
    // Environment
    chain.add_exporter("Environment", Box::new(ValueMap::<Environment>::new()));
    
    // Repository
    chain.add_exporter("Repository", Box::new(ValueMap::<Repository>::new()));
    
    // Inventory
    chain.add_exporter("Inventory", Box::new(ValueMap::<Inventory>::new()));
    
    // Template
    chain.add_exporter("Template", Box::new(ValueMap::<Template>::new()));
    
    // View
    chain.add_exporter("View", Box::new(ValueMap::<View>::new()));
    
    // Schedule
    chain.add_exporter("Schedule", Box::new(ValueMap::<Schedule>::new()));
    
    // Integration
    chain.add_exporter("Integration", Box::new(ValueMap::<Integration>::new()));
    
    // Task (опционально)
    if !skip_task_output {
        chain.add_exporter("Task", Box::new(ValueMap::<Task>::new()));
    }
    
    chain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exporter_chain_creation() {
        let chain = ExporterChain::new();
        assert!(chain.exporters.is_empty());
    }

    #[test]
    fn test_type_key_mapper() {
        let mut mapper = TypeKeyMapper::new();
        
        mapper.map_keys("test", "scope1", "old_key", "new_key").unwrap();
        
        let new_key = mapper.get_new_key("test", "scope1", "old_key").unwrap();
        assert_eq!(new_key, "new_key");
    }

    #[test]
    fn test_value_map() {
        let mut map: ValueMap<String> = ValueMap::new();
        
        map.append_values(vec!["a".to_string(), "b".to_string()], "scope1").unwrap();
        
        let keys = map.get_loaded_keys("scope1").unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_init_project_exporters() {
        let mut mapper = TypeKeyMapper::new();
        let chain = init_project_exporters(&mut mapper, false);
        
        assert!(chain.exporters.contains_key("User"));
        assert!(chain.exporters.contains_key("AccessKey"));
        assert!(chain.exporters.contains_key("Task"));
    }
}
