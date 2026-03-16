//! Exporter - экспорт данных проекта
//!
//! Аналог services/export/Exporter.go из Go версии

use std::collections::{HashMap, HashSet};
use tracing::{info, warn, error};

/// Константы для типов сущностей
pub const USER: &str = "User";
pub const PROJECT: &str = "Project";
pub const ACCESS_KEY: &str = "AccessKey";
pub const ENVIRONMENT: &str = "Environment";
pub const TEMPLATE: &str = "Template";
pub const INVENTORY: &str = "Inventory";
pub const REPOSITORY: &str = "Repository";
pub const VIEW: &str = "View";
pub const ROLE: &str = "Role";
pub const INTEGRATION: &str = "Integration";
pub const SCHEDULE: &str = "Schedule";
pub const TASK: &str = "Task";
pub const PROJECT_USER: &str = "ProjectUser";
pub const OPTION: &str = "Option";
pub const EVENT: &str = "Event";
pub const RUNNER: &str = "Runner";

/// EntityKey - ключ сущности
pub type EntityKey = String;

/// Создаёт ключ из int
pub fn new_key_from_int(key: i32) -> EntityKey {
    key.to_string()
}

/// Создаёт ключ из строки
pub fn new_key(key: &str) -> EntityKey {
    key.to_string()
}

/// ErrorHandler - обработчик ошибок
pub trait ErrorHandler {
    fn on_error(&self, err: &str);
}

/// Progress - интерфейс прогресса
pub trait Progress {
    fn update(&mut self, progress: f32);
}

/// KeyMapper - маппер ключей
pub trait KeyMapper {
    fn get_new_key(&mut self, name: &str, scope: &str, old_key: &EntityKey, err_handler: &dyn ErrorHandler) -> Result<EntityKey, String>;
    fn get_new_key_int(&mut self, name: &str, scope: &str, old_key: i32, err_handler: &dyn ErrorHandler) -> Result<i32, String>;
    fn get_new_key_int_ref(&mut self, name: &str, scope: &str, old_key: Option<i32>, err_handler: &dyn ErrorHandler) -> Result<Option<i32>, String>;
    fn map_keys(&mut self, name: &str, scope: &str, old_key: &EntityKey, new_key: &EntityKey) -> Result<(), String>;
    fn map_int_keys(&mut self, name: &str, scope: &str, old_key: i32, new_key: i32) -> Result<(), String>;
    fn ignore_key_not_found(&self) -> bool;
}

/// DataExporter - экспорт данных
pub trait DataExporter: KeyMapper {
    fn get_type_exporter(&mut self, name: &str) -> &mut dyn TypeExporter;
    fn get_loaded_keys(&self, name: &str, scope: &str) -> Result<Vec<EntityKey>, String>;
    fn get_loaded_keys_int(&self, name: &str, scope: &str) -> Result<Vec<i32>, String>;
}

/// TypeExporter - экспорт типа
pub trait TypeExporter {
    fn load(&mut self, store: &dyn crate::db::Store, exporter: &dyn DataExporter, progress: &mut dyn Progress) -> Result<(), String>;
    fn restore(&mut self, store: &dyn crate::db::Store, exporter: &dyn DataExporter, progress: &mut dyn Progress) -> Result<(), String>;
    fn get_loaded_keys(&self, scope: &str) -> Result<Vec<EntityKey>, String>;
    fn get_loaded_values(&self, scope: &str) -> Result<Vec<Box<dyn std::any::Any>>, String>;
    fn get_name(&self) -> &str;
    fn export_depends_on(&self) -> Vec<&str>;
    fn import_depends_on(&self) -> Vec<&str>;
    fn get_errors(&self) -> Vec<String>;
    fn clear(&mut self);
}

/// TypeKeyMapper - маппер ключей для типа
pub struct TypeKeyMapper {
    key_maps: HashMap<String, HashMap<EntityKey, EntityKey>>,
}

impl Default for TypeKeyMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeKeyMapper {
    pub fn new() -> Self {
        Self {
            key_maps: HashMap::new(),
        }
    }

    fn get_key_map(&mut self, name: &str, scope: &str) -> &mut HashMap<EntityKey, EntityKey> {
        let key = format!("{}.{}", name, scope);
        self.key_maps.entry(key).or_default()
    }
}

impl KeyMapper for TypeKeyMapper {
    fn get_new_key(&mut self, name: &str, scope: &str, old_key: &EntityKey, _err_handler: &dyn ErrorHandler) -> Result<EntityKey, String> {
        let key_map = self.get_key_map(name, scope);
        
        if let Some(new_key) = key_map.get(old_key) {
            Ok(new_key.clone())
        } else {
            Ok(old_key.clone())
        }
    }

    fn get_new_key_int(&mut self, name: &str, scope: &str, old_key: i32, err_handler: &dyn ErrorHandler) -> Result<i32, String> {
        let old_key_str = new_key_from_int(old_key);
        let new_key_str = self.get_new_key(name, scope, &old_key_str, err_handler)?;
        new_key_str.parse::<i32>().map_err(|e| e.to_string())
    }

    fn get_new_key_int_ref(&mut self, name: &str, scope: &str, old_key: Option<i32>, err_handler: &dyn ErrorHandler) -> Result<Option<i32>, String> {
        match old_key {
            Some(key) => {
                let new_key = self.get_new_key_int(name, scope, key, err_handler)?;
                Ok(Some(new_key))
            }
            None => Ok(None),
        }
    }

    fn map_keys(&mut self, name: &str, scope: &str, old_key: &EntityKey, new_key: &EntityKey) -> Result<(), String> {
        let key_map = self.get_key_map(name, scope);
        key_map.insert(old_key.clone(), new_key.clone());
        Ok(())
    }

    fn map_int_keys(&mut self, name: &str, scope: &str, old_key: i32, new_key: i32) -> Result<(), String> {
        let old_key_str = new_key_from_int(old_key);
        let new_key_str = new_key_from_int(new_key);
        self.map_keys(name, scope, &old_key_str, &new_key_str)
    }

    fn ignore_key_not_found(&self) -> bool {
        false
    }
}

/// ValueMap - мапа значений
pub struct ValueMap<T> {
    values: HashMap<String, Vec<T>>,
    errors: Vec<String>,
}

impl<T: Clone> Default for ValueMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> ValueMap<T> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            errors: Vec::new(),
        }
    }

    pub fn get_loaded_keys(&self, scope: &str) -> Result<Vec<EntityKey>, String> {
        let key = scope.to_string();
        Ok((0..self.values.get(&key).map(|v| v.len()).unwrap_or(0))
            .map(|i| new_key_from_int(i as i32))
            .collect())
    }

    pub fn get_loaded_values(&self, _scope: &str) -> Result<Vec<Box<dyn std::any::Any>>, String> {
        // Упрощённая реализация
        Ok(Vec::new())
    }

    pub fn append_values(&mut self, values: Vec<T>, scope: &str) -> Result<(), String> {
        let key = scope.to_string();
        self.values.entry(key).or_default().extend(values);
        Ok(())
    }

    pub fn export_depends_on(&self) -> Vec<&str> {
        Vec::new()
    }

    pub fn import_depends_on(&self) -> Vec<&str> {
        Vec::new()
    }

    pub fn on_error(&mut self, err: &str) {
        self.errors.push(err.to_string());
    }

    pub fn get_errors(&self) -> Vec<String> {
        self.errors.clone()
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.errors.clear();
    }
}

impl<T: Clone + Send + 'static> TypeExporter for ValueMap<T> {
    fn load(&mut self, _store: &dyn crate::db::Store, _exporter: &dyn DataExporter, _progress: &mut dyn Progress) -> Result<(), String> {
        Ok(())
    }

    fn restore(&mut self, _store: &dyn crate::db::Store, _exporter: &dyn DataExporter, _progress: &mut dyn Progress) -> Result<(), String> {
        Ok(())
    }

    fn get_loaded_keys(&self, scope: &str) -> Result<Vec<EntityKey>, String> {
        ValueMap::get_loaded_keys(self, scope)
    }

    fn get_loaded_values(&self, scope: &str) -> Result<Vec<Box<dyn std::any::Any>>, String> {
        ValueMap::get_loaded_values(self, scope)
    }

    fn get_name(&self) -> &str {
        std::any::type_name::<T>()
    }

    fn export_depends_on(&self) -> Vec<&str> {
        ValueMap::export_depends_on(self)
    }

    fn import_depends_on(&self) -> Vec<&str> {
        ValueMap::import_depends_on(self)
    }

    fn get_errors(&self) -> Vec<String> {
        ValueMap::get_errors(self)
    }

    fn clear(&mut self) {
        ValueMap::clear(self)
    }
}

/// ExporterChain - цепочка экспортеров
pub struct ExporterChain {
    exporters: HashMap<String, Box<dyn TypeExporter>>,
    /// Маппинг старых ключей в новые (для импорта)
    key_mapping: HashMap<String, HashMap<EntityKey, EntityKey>>,
    /// Маппинг старых integer ключей в новые
    int_key_mapping: HashMap<String, HashMap<i32, i32>>,
    /// Игнорировать отсутствующие ключи
    ignore_key_not_found: bool,
}

impl KeyMapper for ExporterChain {
    fn get_new_key(&mut self, name: &str, _scope: &str, old_key: &EntityKey, _err_handler: &dyn ErrorHandler) -> Result<EntityKey, String> {
        // Проверяем маппинг
        if let Some(mapping) = self.key_mapping.get(name) {
            if let Some(new_key) = mapping.get(old_key) {
                return Ok(new_key.clone());
            }
        }
        // Если маппинга нет, возвращаем старый ключ
        Ok(old_key.clone())
    }

    fn get_new_key_int(&mut self, name: &str, _scope: &str, old_key: i32, _err_handler: &dyn ErrorHandler) -> Result<i32, String> {
        // Проверяем маппинг
        if let Some(mapping) = self.int_key_mapping.get(name) {
            if let Some(new_key) = mapping.get(&old_key) {
                return Ok(*new_key);
            }
        }
        // Если маппинга нет, возвращаем старый ключ
        Ok(old_key)
    }

    fn get_new_key_int_ref(&mut self, name: &str, _scope: &str, old_key: Option<i32>, _err_handler: &dyn ErrorHandler) -> Result<Option<i32>, String> {
        match old_key {
            Some(key) => {
                let mapped = self.get_new_key_int(name, _scope, key, _err_handler)?;
                Ok(Some(mapped))
            }
            None => Ok(None),
        }
    }

    fn map_keys(&mut self, name: &str, _scope: &str, old_key: &EntityKey, new_key: &EntityKey) -> Result<(), String> {
        // Сохраняем маппинг
        self.key_mapping
            .entry(name.to_string())
            .or_default()
            .insert(old_key.clone(), new_key.clone());
        Ok(())
    }

    fn map_int_keys(&mut self, name: &str, _scope: &str, old_key: i32, new_key: i32) -> Result<(), String> {
        // Сохраняем маппинг integer ключей
        self.int_key_mapping
            .entry(name.to_string())
            .or_default()
            .insert(old_key, new_key);
        Ok(())
    }

    fn ignore_key_not_found(&self) -> bool {
        self.ignore_key_not_found
    }
}

impl DataExporter for ExporterChain {
    fn get_type_exporter(&mut self, name: &str) -> &mut dyn TypeExporter {
        self.exporters.get_mut(name)
            .map(|b| b.as_mut())
            .unwrap_or_else(|| panic!("Exporter {} not found", name))
    }
    
    fn get_loaded_keys(&self, name: &str, scope: &str) -> Result<Vec<EntityKey>, String> {
        if let Some(exporter) = self.exporters.get(name) {
            exporter.get_loaded_keys(scope)
        } else {
            Err(format!("Exporter {} not found", name))
        }
    }
    
    fn get_loaded_keys_int(&self, name: &str, scope: &str) -> Result<Vec<i32>, String> {
        let keys = self.get_loaded_keys(name, scope)?;
        Ok(keys.into_iter().filter_map(|k| k.parse::<i32>().ok()).collect())
    }
}

impl Default for ExporterChain {
    fn default() -> Self {
        Self::new()
    }
}

impl ExporterChain {
    pub fn new() -> Self {
        Self {
            exporters: HashMap::new(),
            key_mapping: HashMap::new(),
            int_key_mapping: HashMap::new(),
            ignore_key_not_found: false,
        }
    }

    /// Создаёт новый ExporterChain с настройками
    pub fn with_options(ignore_key_not_found: bool) -> Self {
        Self {
            exporters: HashMap::new(),
            key_mapping: HashMap::new(),
            int_key_mapping: HashMap::new(),
            ignore_key_not_found,
        }
    }

    pub fn add_exporter(&mut self, name: &str, exporter: Box<dyn TypeExporter>) {
        self.exporters.insert(name.to_string(), exporter);
    }

    pub fn get_type_exporter(&mut self, name: &str) -> Option<&mut Box<dyn TypeExporter>> {
        self.exporters.get_mut(name)
    }

    pub fn get_loaded_keys(&self, name: &str, scope: &str) -> Result<Vec<EntityKey>, String> {
        if let Some(exporter) = self.exporters.get(name) {
            exporter.get_loaded_keys(scope)
        } else {
            Err(format!("Exporter {} not found", name))
        }
    }

    /// Сортирует ключи по зависимостям
    pub fn get_sorted_keys(exporters: &HashMap<String, Box<dyn TypeExporter>>, depends_on: fn(&dyn TypeExporter) -> Vec<&str>) -> Result<Vec<String>, String> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        fn visit(name: &str, exporters: &HashMap<String, Box<dyn TypeExporter>>, sorted: &mut Vec<String>, visited: &mut HashSet<String>, visiting: &mut HashSet<String>, depends_on: fn(&dyn TypeExporter) -> Vec<&str>) -> Result<(), String> {
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

    /// Загружает данные из БД
    pub fn load(&mut self, store: &dyn crate::db::Store) -> Result<(), String> {
        let sorted_keys = Self::get_sorted_keys(&self.exporters, |e| e.export_depends_on())?;
        let len = self.exporters.len();

        for (i, key) in sorted_keys.iter().enumerate() {
            // Извлекаем экспортер, чтобы освободить заимствование self.exporters
            let mut exporter = match self.exporters.remove(key) {
                Some(e) => e,
                None => continue,
            };
            info!("Loading {}...", key);
            // self теперь доступен как &dyn DataExporter (после remove, нет mutable borrow)
            let data_exporter: &dyn DataExporter = self;
            let mut progress = ProgressBar::new(100.0 / len as f32);
            if let Err(e) = exporter.load(store, data_exporter, &mut progress) {
                self.exporters.insert(key.clone(), exporter);
                return Err(e);
            }
            progress.update((i + 1) as f32 * 100.0 / len as f32);
            self.exporters.insert(key.clone(), exporter);
        }

        Ok(())
    }

    /// Восстанавливает данные в БД
    pub fn restore(&mut self, store: &dyn crate::db::Store) -> Result<(), String> {
        let sorted_keys = Self::get_sorted_keys(&self.exporters, |e| e.import_depends_on())?;

        for key in sorted_keys {
            // Извлекаем экспортер, чтобы освободить заимствование self.exporters
            let mut exporter = match self.exporters.remove(&key) {
                Some(e) => e,
                None => continue,
            };
            info!("Restoring {}...", key);
            let data_exporter: &dyn DataExporter = self;
            let mut progress = ProgressBar::new(0.0);
            if let Err(e) = exporter.restore(store, data_exporter, &mut progress) {
                self.exporters.insert(key, exporter);
                return Err(e);
            }
            self.exporters.insert(key, exporter);
        }

        Ok(())
    }
}

/// ProgressBar - прогресс бар
pub struct ProgressBar {
    total: f32,
    current: f32,
}

impl ProgressBar {
    pub fn new(total: f32) -> Self {
        Self {
            total,
            current: 0.0,
        }
    }

    pub fn update(&mut self, progress: f32) {
        self.current = progress;
        info!("Progress: {:.2}%", self.current.min(self.total));
    }
}

impl Progress for ProgressBar {
    fn update(&mut self, progress: f32) {
        self.update(progress);
    }
}

/// Инициализирует экспортеры проекта
pub fn init_project_exporters(mapper: &mut dyn KeyMapper, skip_task_output: bool) -> ExporterChain {
    let mut chain = ExporterChain::new();

    // Добавляем экспортеры в порядке зависимостей
    chain.add_exporter(USER, Box::new(ValueMap::<crate::models::User>::new()));
    chain.add_exporter(ACCESS_KEY, Box::new(ValueMap::<crate::models::AccessKey>::new()));
    chain.add_exporter(ENVIRONMENT, Box::new(ValueMap::<crate::models::Environment>::new()));
    chain.add_exporter(REPOSITORY, Box::new(ValueMap::<crate::models::Repository>::new()));
    chain.add_exporter(INVENTORY, Box::new(ValueMap::<crate::models::Inventory>::new()));
    chain.add_exporter(TEMPLATE, Box::new(ValueMap::<crate::models::Template>::new()));
    chain.add_exporter(VIEW, Box::new(ValueMap::<crate::models::View>::new()));
    chain.add_exporter(SCHEDULE, Box::new(ValueMap::<crate::models::Schedule>::new()));
    chain.add_exporter(INTEGRATION, Box::new(ValueMap::<crate::models::Integration>::new()));

    if !skip_task_output {
        chain.add_exporter(TASK, Box::new(ValueMap::<crate::models::Task>::new()));
    }

    chain
}

/// Создаёт новый KeyMapper
pub fn new_key_mapper() -> TypeKeyMapper {
    TypeKeyMapper::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestErrorHandler;

    impl ErrorHandler for TestErrorHandler {
        fn on_error(&self, _err: &str) {}
    }

    #[test]
    fn test_new_key_from_int() {
        let key = new_key_from_int(123);
        assert_eq!(key, "123");
    }

    #[test]
    fn test_new_key() {
        let key = new_key("test");
        assert_eq!(key, "test");
    }

    #[test]
    fn test_type_key_mapper() {
        let mut mapper = TypeKeyMapper::new();
        let err_handler = TestErrorHandler;

        let old_key = new_key_from_int(1);
        let new_key = new_key_from_int(2);

        mapper.map_keys("test", "scope1", &old_key, &new_key).unwrap();

        let result = mapper.get_new_key("test", "scope1", &old_key, &err_handler).unwrap();
        assert_eq!(result, new_key);
    }

    #[test]
    fn test_value_map() {
        let mut value_map: ValueMap<String> = ValueMap::new();
        
        value_map.append_values(vec!["a".to_string(), "b".to_string()], "scope1").unwrap();
        
        let keys = value_map.get_loaded_keys("scope1").unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_progress_bar() {
        let mut progress = ProgressBar::new(100.0);
        progress.update(50.0);
        assert_eq!(progress.current, 50.0);
    }
}
