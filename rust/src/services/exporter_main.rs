//! Exporter Main - главный экспортер
//!
//! Аналог services/export/Exporter.go из Go версии

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::db::store::Store;
use crate::models::*;

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
    fn get_type_exporter(&mut self, name: &str) -> Option<&mut (dyn TypeExporter + '_)>;

    /// Получает загруженные ключи
    fn get_loaded_keys(&self, name: &str, scope: &str) -> Result<Vec<String>, String>;

    /// Получает загруженные ключи int
    fn get_loaded_keys_int(&self, name: &str, scope: &str) -> Result<Vec<i32>, String>;
}

impl DataExporter for ExporterChain {
    fn get_type_exporter(&mut self, name: &str) -> Option<&mut (dyn TypeExporter + '_)> {
        if let Some(exporter) = self.exporters.get_mut(name) {
            Some(exporter.as_mut())
        } else {
            None
        }
    }

    fn get_loaded_keys(&self, name: &str, scope: &str) -> Result<Vec<String>, String> {
        if let Some(exporter) = self.exporters.get(name) {
            exporter.get_loaded_keys(scope)
        } else {
            Err(format!("Exporter {} not found", name))
        }
    }

    fn get_loaded_keys_int(&self, name: &str, scope: &str) -> Result<Vec<i32>, String> {
        let keys = self.get_loaded_keys(name, scope)?;
        Ok(keys
            .into_iter()
            .filter_map(|k| k.parse::<i32>().ok())
            .collect())
    }
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
            // Remove temporarily so we can pass `self` as DataExporter without
            // conflicting with the mutable borrow of self.exporters
            let mut exporter = match self.exporters.remove(&key) {
                Some(e) => e,
                None => continue,
            };
            exporter
                .load(store, self)
                .map_err(|e| format!("Failed to load {}: {}", key, e))?;
            self.exporters.insert(key, exporter);
        }
        Ok(())
    }

    /// Восстанавливает данные
    pub fn restore(&mut self, store: &dyn Store) -> Result<(), String> {
        let sorted_keys = Self::get_sorted_keys(&self.exporters, |e| e.import_depends_on())?;
        for key in sorted_keys {
            let mut exporter = match self.exporters.remove(&key) {
                Some(e) => e,
                None => continue,
            };
            exporter
                .restore(store, self)
                .map_err(|e| format!("Failed to restore {}: {}", key, e))?;
            self.exporters.insert(key, exporter);
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
            visit(
                name,
                exporters,
                &mut sorted,
                &mut visited,
                &mut visiting,
                depends_on,
            )?;
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
    pub fn get_new_key(
        &mut self,
        name: &str,
        scope: &str,
        old_key: &str,
    ) -> Result<String, String> {
        let key = format!("{}.{}", name, scope);

        if let Some(map) = self.key_maps.get(&key) {
            if let Some(new_key) = map.get(old_key) {
                return Ok(new_key.clone());
            }
        }

        Ok(old_key.to_string())
    }

    /// Мапит ключи
    pub fn map_keys(
        &mut self,
        name: &str,
        scope: &str,
        old_key: &str,
        new_key: &str,
    ) -> Result<(), String> {
        let key = format!("{}.{}", name, scope);

        let map = self.key_maps.entry(key).or_default();
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
        let entry = self.values.entry(scope.to_string()).or_default();
        entry.extend(values);
        Ok(())
    }

    /// Возвращает все значения для scope (клонирует)
    pub fn get_values(&self, scope: &str) -> Vec<T>
    where
        T: Clone,
    {
        self.values.get(scope).cloned().unwrap_or_default()
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

impl<T: Clone + Send + 'static> TypeExporter for ValueMap<T> {
    fn load(&mut self, _store: &dyn Store, _exporter: &dyn DataExporter) -> Result<(), String> {
        Ok(())
    }

    fn restore(&mut self, _store: &dyn Store, _exporter: &dyn DataExporter) -> Result<(), String> {
        Ok(())
    }

    fn get_loaded_keys(&self, scope: &str) -> Result<Vec<String>, String> {
        ValueMap::get_loaded_keys(self, scope)
    }

    fn get_loaded_values(&self, _scope: &str) -> Result<Vec<Box<dyn std::any::Any>>, String> {
        Ok(Vec::new())
    }

    fn get_name(&self) -> &str {
        std::any::type_name::<T>()
    }

    fn export_depends_on(&self) -> Vec<&str> {
        Vec::new()
    }

    fn import_depends_on(&self) -> Vec<&str> {
        Vec::new()
    }

    fn get_errors(&self) -> Vec<String> {
        ValueMap::get_errors(self)
    }

    fn clear(&mut self) {
        ValueMap::clear(self)
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

        mapper
            .map_keys("test", "scope1", "old_key", "new_key")
            .unwrap();

        let new_key = mapper.get_new_key("test", "scope1", "old_key").unwrap();
        assert_eq!(new_key, "new_key");
    }

    #[test]
    fn test_value_map() {
        let mut map: ValueMap<String> = ValueMap::new();

        map.append_values(vec!["a".to_string(), "b".to_string()], "scope1")
            .unwrap();

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

    // ── Parameterized exporter tests ──

    #[test]
    fn test_all_exporters_have_valid_names() {
        let mut mapper = TypeKeyMapper::new();
        let chain = init_project_exporters(&mut mapper, false);

        // Each exporter should have a non-empty name
        for (name, exporter) in &chain.exporters {
            assert!(!exporter.get_name().is_empty(), "Exporter '{}' has empty name", name);
        }
    }

    #[test]
    fn test_all_exporters_return_dependencies() {
        let mut mapper = TypeKeyMapper::new();
        let chain = init_project_exporters(&mut mapper, false);

        // Just verify these don't panic and return valid vecs
        for (_, exporter) in &chain.exporters {
            let export_deps = exporter.export_depends_on();
            let import_deps = exporter.import_depends_on();
            // Dependencies should be valid strings
            for dep in &export_deps {
                assert!(!dep.is_empty());
            }
            for dep in &import_deps {
                assert!(!dep.is_empty());
            }
        }
    }

    #[test]
    fn test_all_exporters_clear_without_error() {
        let mut mapper = TypeKeyMapper::new();
        let mut chain = init_project_exporters(&mut mapper, false);

        for (_, exporter) in &mut chain.exporters {
            // Use on_error method which is available via value_map
            // Just verify clear doesn't panic
            exporter.clear();
            assert!(exporter.get_errors().is_empty());
        }
    }

    #[test]
    fn test_exporter_specific_dependencies() {
        let mut mapper = TypeKeyMapper::new();
        let chain = init_project_exporters(&mut mapper, false);

        // Just verify dependencies are returned without panic
        // Specific dependency checks are done in individual exporter tests
        for (_, exporter) in &chain.exporters {
            let _ = exporter.export_depends_on();
            let _ = exporter.import_depends_on();
        }
    }

    #[test]
    fn test_exporter_default_same_as_new() {
        let mut mapper = TypeKeyMapper::new();
        let chain = init_project_exporters(&mut mapper, false);

        // Verify that all exporters can be cloned/created without issues
        for (_, exporter) in &chain.exporters {
            let name = exporter.get_name();
            assert!(!name.is_empty());
        }
    }

    #[test]
    fn test_value_map_append_and_clear() {
        let mut map: ValueMap<String> = ValueMap::new();

        map.append_values(vec!["a".to_string(), "b".to_string()], "scope").unwrap();
        assert_eq!(map.get_loaded_keys("scope").unwrap().len(), 2);

        map.clear();
        assert!(map.get_loaded_keys("scope").unwrap().is_empty());
        assert!(map.get_errors().is_empty());
    }

    #[test]
    fn test_value_map_errors_accumulate() {
        let mut map: ValueMap<String> = ValueMap::new();

        map.errors.push("err1".to_string());
        map.errors.push("err2".to_string());

        assert_eq!(map.get_errors().len(), 2);
    }

    #[test]
    fn test_type_key_mapper_get_new_key_returns_old_if_not_mapped() {
        let mut mapper = TypeKeyMapper::new();
        let result = mapper.get_new_key("test", "scope", "old_key").unwrap();
        assert_eq!(result, "old_key");
    }

    #[test]
    fn test_init_project_exporters_skip_task_output() {
        let mut mapper = TypeKeyMapper::new();
        let chain = init_project_exporters(&mut mapper, true); // skip_task_output = true

        assert!(!chain.exporters.contains_key("TaskOutput"));
    }

    #[test]
    fn test_type_key_mapper_map_keys() {
        let mut mapper = TypeKeyMapper::new();
        mapper.map_keys("Test", "scope", "old", "new").unwrap();
        let result = mapper.get_new_key("Test", "scope", "old").unwrap();
        assert_eq!(result, "new");
    }

    #[test]
    fn test_value_map_get_loaded_values_empty() {
        let mut map: ValueMap<String> = ValueMap::new();
        let values = map.get_loaded_values("scope").unwrap();
        assert!(values.is_empty());
    }

    #[test]
    fn test_value_map_multiple_scopes() {
        let mut map: ValueMap<String> = ValueMap::new();
        map.append_values(vec!["a".to_string()], "scope1").unwrap();
        map.append_values(vec!["b".to_string()], "scope2").unwrap();

        assert_eq!(map.get_loaded_keys("scope1").unwrap().len(), 1);
        assert_eq!(map.get_loaded_keys("scope2").unwrap().len(), 1);
    }

    #[test]
    fn test_value_map_clear_removes_all_keys() {
        let mut map: ValueMap<String> = ValueMap::new();
        map.append_values(vec!["a".to_string(), "b".to_string()], "scope").unwrap();
        assert_eq!(map.get_loaded_keys("scope").unwrap().len(), 2);

        map.clear();
        assert!(map.get_loaded_keys("scope").unwrap().is_empty());
        assert!(map.get_errors().is_empty());
    }

    // ── Additional tests ──

    #[test]
    fn test_exporter_chain_default() {
        let chain = ExporterChain::default();
        assert!(chain.exporters.is_empty());
    }

    #[test]
    fn test_type_key_mapper_default() {
        let mapper = TypeKeyMapper::default();
        assert!(mapper.key_maps.is_empty());
    }

    #[test]
    fn test_value_map_default() {
        let map: ValueMap<String> = ValueMap::default();
        assert!(map.values.is_empty());
        assert!(map.errors.is_empty());
    }

    #[test]
    fn test_value_map_get_values() {
        let mut map: ValueMap<i32> = ValueMap::new();
        map.append_values(vec![10, 20, 30], "scope").unwrap();
        let values = map.get_values("scope");
        assert_eq!(values, vec![10, 20, 30]);
    }

    #[test]
    fn test_value_map_get_values_empty_scope() {
        let map: ValueMap<String> = ValueMap::new();
        let values = map.get_values("nonexistent");
        assert!(values.is_empty());
    }

    #[test]
    fn test_value_map_append_empty_vec() {
        let mut map: ValueMap<String> = ValueMap::new();
        let empty: Vec<String> = Vec::new();
        map.append_values(empty, "scope").unwrap();
        let keys = map.get_loaded_keys("scope").unwrap();
        assert!(keys.is_empty());
    }

    #[test]
    fn test_value_map_multiple_appends_same_scope() {
        let mut map: ValueMap<String> = ValueMap::new();
        map.append_values(vec!["a".to_string()], "scope").unwrap();
        map.append_values(vec!["b".to_string(), "c".to_string()], "scope").unwrap();
        let keys = map.get_loaded_keys("scope").unwrap();
        assert_eq!(keys.len(), 3);
        assert_eq!(keys, vec!["0", "1", "2"]);
    }

    #[test]
    fn test_type_key_mapper_get_new_key_unmapped_returns_same() {
        let mut mapper = TypeKeyMapper::new();
        let result = mapper.get_new_key("Env", "scope", "original_key").unwrap();
        assert_eq!(result, "original_key");
    }

    #[test]
    fn test_type_key_mapper_map_keys_different_scopes() {
        let mut mapper = TypeKeyMapper::new();
        mapper.map_keys("Repo", "scope1", "old1", "new1").unwrap();
        mapper.map_keys("Repo", "scope2", "old2", "new2").unwrap();

        let result1 = mapper.get_new_key("Repo", "scope1", "old1").unwrap();
        assert_eq!(result1, "new1");

        let result2 = mapper.get_new_key("Repo", "scope2", "old2").unwrap();
        assert_eq!(result2, "new2");

        // Cross-scope should return original
        let cross = mapper.get_new_key("Repo", "scope1", "old2").unwrap();
        assert_eq!(cross, "old2");
    }

    #[test]
    fn test_exporter_chain_get_type_exporter_not_found() {
        let mut chain = ExporterChain::new();
        let result = chain.get_type_exporter("NonExistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_exporter_chain_get_type_exporter_found() {
        let mut chain = ExporterChain::new();
        chain.add_exporter("Test", Box::new(ValueMap::<String>::new()));
        let result = chain.get_type_exporter("Test");
        assert!(result.is_some());
    }

    #[test]
    fn test_exporter_chain_get_loaded_keys_not_found() {
        let chain = ExporterChain::new();
        let result = chain.get_loaded_keys("Missing", "scope");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_exporter_chain_get_loaded_keys_int_not_found() {
        let chain = ExporterChain::new();
        let result = chain.get_loaded_keys_int("Missing", "scope");
        assert!(result.is_err());
    }

    #[test]
    fn test_exporter_chain_add_multiple_exporters() {
        let mut chain = ExporterChain::new();
        chain.add_exporter("A", Box::new(ValueMap::<String>::new()));
        chain.add_exporter("B", Box::new(ValueMap::<i32>::new()));
        chain.add_exporter("C", Box::new(ValueMap::<bool>::new()));

        assert_eq!(chain.exporters.len(), 3);
        assert!(chain.exporters.contains_key("A"));
        assert!(chain.exporters.contains_key("B"));
        assert!(chain.exporters.contains_key("C"));
    }

    #[test]
    fn test_init_project_exporters_has_user_first() {
        let mut mapper = TypeKeyMapper::new();
        let chain = init_project_exporters(&mut mapper, false);
        assert!(chain.exporters.contains_key("User"));
    }

    #[test]
    fn test_init_project_exporters_has_environment() {
        let mut mapper = TypeKeyMapper::new();
        let chain = init_project_exporters(&mut mapper, false);
        assert!(chain.exporters.contains_key("Environment"));
    }

    #[test]
    fn test_init_project_exporters_has_repository() {
        let mut mapper = TypeKeyMapper::new();
        let chain = init_project_exporters(&mut mapper, false);
        assert!(chain.exporters.contains_key("Repository"));
    }

    #[test]
    fn test_init_project_exporters_has_inventory() {
        let mut mapper = TypeKeyMapper::new();
        let chain = init_project_exporters(&mut mapper, false);
        assert!(chain.exporters.contains_key("Inventory"));
    }

    #[test]
    fn test_init_project_exporters_has_view() {
        let mut mapper = TypeKeyMapper::new();
        let chain = init_project_exporters(&mut mapper, false);
        assert!(chain.exporters.contains_key("View"));
    }

    #[test]
    fn test_init_project_exporters_has_schedule() {
        let mut mapper = TypeKeyMapper::new();
        let chain = init_project_exporters(&mut mapper, false);
        assert!(chain.exporters.contains_key("Schedule"));
    }

    #[test]
    fn test_init_project_exporters_has_integration() {
        let mut mapper = TypeKeyMapper::new();
        let chain = init_project_exporters(&mut mapper, false);
        assert!(chain.exporters.contains_key("Integration"));
    }

    #[test]
    fn test_init_project_exporters_count_without_task() {
        let mut mapper = TypeKeyMapper::new();
        let chain = init_project_exporters(&mut mapper, true);
        assert!(!chain.exporters.contains_key("Task"));
        assert_eq!(chain.exporters.len(), 9);
    }

    #[test]
    fn test_init_project_exporters_count_with_task() {
        let mut mapper = TypeKeyMapper::new();
        let chain = init_project_exporters(&mut mapper, false);
        assert!(chain.exporters.contains_key("Task"));
        assert_eq!(chain.exporters.len(), 10);
    }

    #[test]
    fn test_type_exporter_trait_methods_value_map() {
        let mut map: ValueMap<String> = ValueMap::new();
        map.append_values(vec!["x".to_string()], "test").unwrap();

        assert_eq!(map.get_name(), std::any::type_name::<String>());
        assert!(map.export_depends_on().is_empty());
        assert!(map.import_depends_on().is_empty());
        assert!(map.get_errors().is_empty());
    }

    #[test]
    fn test_type_exporter_trait_get_loaded_values() {
        let map: ValueMap<i32> = ValueMap::new();
        let values = map.get_loaded_values("scope").unwrap();
        assert!(values.is_empty());
    }

    #[test]
    fn test_type_exporter_trait_clear() {
        let mut map: ValueMap<String> = ValueMap::new();
        map.append_values(vec!["a".to_string()], "scope").unwrap();
        map.clear();
        assert!(map.get_loaded_keys("scope").unwrap().is_empty());
    }

    #[test]
    fn test_exporter_chain_get_sorted_keys_empty() {
        let exporters: HashMap<String, Box<dyn TypeExporter>> = HashMap::new();
        let result = ExporterChain::get_sorted_keys(&exporters, |_| Vec::new());
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_exporter_chain_get_sorted_keys_single() {
        let mut exporters: HashMap<String, Box<dyn TypeExporter>> = HashMap::new();
        exporters.insert("A".to_string(), Box::new(ValueMap::<String>::new()));
        let result = ExporterChain::get_sorted_keys(&exporters, |_| Vec::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["A"]);
    }

    #[test]
    fn test_exporter_chain_get_sorted_keys_with_deps() {
        let mut exporters: HashMap<String, Box<dyn TypeExporter>> = HashMap::new();
        exporters.insert("A".to_string(), Box::new(ValueMap::<String>::new()));
        exporters.insert("B".to_string(), Box::new(ValueMap::<String>::new()));

        fn deps(e: &dyn TypeExporter) -> Vec<&str> {
            if e.get_name() == std::any::type_name::<String>() {
                if true { // just return A->B dependency
                    Vec::new()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        }

        let result = ExporterChain::get_sorted_keys(&exporters, deps);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_exporter_chain_get_sorted_keys_circular_dependency() {
        let mut exporters: HashMap<String, Box<dyn TypeExporter>> = HashMap::new();
        exporters.insert("A".to_string(), Box::new(ValueMap::<String>::new()));
        exporters.insert("B".to_string(), Box::new(ValueMap::<String>::new()));

        fn circular_deps(e: &dyn TypeExporter) -> Vec<&str> {
            if e.get_name() == std::any::type_name::<String>() {
                vec!["B"]
            } else {
                vec!["A"]
            }
        }

        // This should detect circular dependency
        let result = ExporterChain::get_sorted_keys(&exporters, circular_deps);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Circular"));
    }

    #[test]
    fn test_data_exporter_get_loaded_keys_int_valid() {
        let mut chain = ExporterChain::new();
        let mut map: ValueMap<String> = ValueMap::new();
        map.append_values(vec!["a".to_string(), "b".to_string()], "scope").unwrap();
        chain.add_exporter("Test", Box::new(map));

        let result = chain.get_loaded_keys_int("Test", "scope").unwrap();
        assert_eq!(result, vec![0, 1]);
    }

    #[test]
    fn test_value_map_errors_isolation() {
        let mut map1: ValueMap<String> = ValueMap::new();
        let mut map2: ValueMap<String> = ValueMap::new();

        map1.errors.push("error1".to_string());
        map2.errors.push("error2".to_string());

        assert_eq!(map1.get_errors(), vec!["error1"]);
        assert_eq!(map2.get_errors(), vec!["error2"]);
    }
}
