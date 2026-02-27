//! Secret Storage Exporter
//!
//! Экспорт хранилищ секретов

use crate::error::{Error, Result};
use crate::models::SecretStorage;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр хранилищ секретов
pub struct SecretStorageExporter {
    /// Карта значений
    pub value_map: ValueMap<SecretStorage>,
}

impl SecretStorageExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for SecretStorageExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for SecretStorageExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        // Получаем ключи проектов
        let projs = exporter.get_loaded_keys_int("Project", "global")?;

        for proj_id in projs {
            let storages = store.get_secret_storages(proj_id)
                .map_err(|e| Error::Other(format!("Failed to load secret storages: {}", e)))?;

            for storage in storages {
                self.value_map.append_value(storage, proj_id.to_string())?;
            }
        }

        Ok(())
    }

    /// Восстанавливает данные
    fn restore(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        for val in &self.value_map.values {
            let mut old = val.value.clone();

            // Восстанавливаем ссылки
            old.project_id = exporter.get_new_key_int("Project", "global", old.project_id, self)?;

            let new_storage = store.create_secret_storage(old)
                .map_err(|e| Error::Other(format!("Failed to create secret storage: {}", e)))?;

            exporter.map_int_keys(self.get_name(), &val.scope, val.value.id, new_storage.id)?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "SecretStorage"
    }

    /// Получает зависимости экспорта
    fn export_depends_on(&self) -> Vec<&str> {
        vec!["Project"]
    }

    /// Получает зависимости импорта
    fn import_depends_on(&self) -> Vec<&str> {
        vec!["Project"]
    }

    /// Получает ошибки
    fn get_errors(&self) -> Vec<String> {
        self.value_map.errors.clone()
    }

    /// Очищает
    fn clear(&mut self) {
        self.value_map.clear();
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_storage_exporter_creation() {
        let exporter = SecretStorageExporter::new();
        assert_eq!(exporter.get_name(), "SecretStorage");
    }

    #[test]
    fn test_secret_storage_exporter_depends_on() {
        let exporter = SecretStorageExporter::new();
        assert_eq!(exporter.export_depends_on(), vec!["Project"]);
    }
}
