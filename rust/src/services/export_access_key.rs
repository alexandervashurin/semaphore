//! Access Key Exporter
//!
//! Экспорт ключей доступа

use std::sync::Arc;
use crate::error::{Error, Result};
use crate::models::AccessKey;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, ExportedValue, Progress};

/// Экспортёр ключей доступа
pub struct AccessKeyExporter {
    /// Карта значений
    pub value_map: ValueMap<AccessKey>,
}

impl AccessKeyExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for AccessKeyExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for AccessKeyExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        // Получаем ключи проектов
        let projs = exporter.get_loaded_keys_int("Project", "global")?;

        for proj in projs {
            let keys = store.get_access_keys(proj, crate::db::store::RetrieveQueryParams::default())
                .map_err(|e| Error::Other(format!("Failed to load access keys: {}", e)))?;

            for key in keys {
                self.value_map.append_value(key, proj.to_string())?;
            }
        }

        Ok(())
    }

    /// Восстанавливает данные
    fn restore(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        for val in &self.value_map.values {
            let mut old = val.value.clone();

            // Восстанавливаем ссылки
            old.environment_id = exporter.get_new_key_int_ref("Environment", &val.scope, old.environment_id, self)?;
            old.storage_id = exporter.get_new_key_int_ref("SecretStorage", &val.scope, old.storage_id, self)?;
            old.user_id = exporter.get_new_key_int_ref("User", &val.scope, old.user_id, self)?;
            old.project_id = exporter.get_new_key_int("Project", "global", old.project_id, self)?;
            old.source_storage_id = exporter.get_new_key_int_ref("SecretStorage", &val.scope, old.source_storage_id, self)?;

            let new_key = store.create_access_key(old)
                .map_err(|e| Error::Other(format!("Failed to create access key: {}", e)))?;

            exporter.map_int_keys(self.get_name(), &val.scope, val.value.id, new_key.id)?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "AccessKey"
    }

    /// Получает зависимости экспорта
    fn export_depends_on(&self) -> Vec<&str> {
        vec!["Project"]
    }

    /// Получает зависимости импорта
    fn import_depends_on(&self) -> Vec<&str> {
        vec!["Project", "SecretStorage", "Environment"]
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
    fn test_access_key_exporter_creation() {
        let exporter = AccessKeyExporter::new();
        assert_eq!(exporter.get_name(), "AccessKey");
    }

    #[test]
    fn test_access_key_exporter_depends_on() {
        let exporter = AccessKeyExporter::new();
        assert_eq!(exporter.export_depends_on(), vec!["Project"]);
    }
}
