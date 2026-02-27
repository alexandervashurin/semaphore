//! Integration Extract Value Exporter
//!
//! Экспорт значений извлечения интеграций

use crate::error::{Error, Result};
use crate::models::IntegrationExtractValue;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр значений извлечения интеграций
pub struct IntegrationExtractValueExporter {
    /// Карта значений
    pub value_map: ValueMap<IntegrationExtractValue>,
}

impl IntegrationExtractValueExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for IntegrationExtractValueExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for IntegrationExtractValueExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        // Получаем ключи проектов
        let projs = exporter.get_loaded_keys_int("Project", "global")?;

        for proj in projs {
            let mut all_values = Vec::new();

            // Получаем интеграции проекта
            let integrations = exporter.get_loaded_keys_int("Integration", &proj.to_string())?;

            for integration in integrations {
                let values = store.get_integration_extract_values(proj, crate::db::store::RetrieveQueryParams::default(), integration)
                    .map_err(|e| Error::Other(format!("Failed to load integration extract values: {}", e)))?;

                all_values.extend(values);
            }

            for value in all_values {
                self.value_map.append_value(value, proj.to_string())?;
            }
        }

        Ok(())
    }

    /// Восстанавливает данные
    fn restore(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        for val in &self.value_map.values {
            let mut old = val.value.clone();

            // Восстанавливаем ссылки
            old.integration_id = exporter.get_new_key_int("Integration", &val.scope, old.integration_id, self)?;

            let new_value = store.create_integration_extract_value(old.project_id, old)
                .map_err(|e| Error::Other(format!("Failed to create integration extract value: {}", e)))?;

            exporter.map_int_keys(self.get_name(), &val.scope, val.value.id, new_value.id)?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "IntegrationExtractValue"
    }

    /// Получает зависимости экспорта
    fn export_depends_on(&self) -> Vec<&str> {
        vec!["Project", "Integration"]
    }

    /// Получает зависимости импорта
    fn import_depends_on(&self) -> Vec<&str> {
        vec!["Project", "Integration"]
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
    fn test_integration_extract_value_exporter_creation() {
        let exporter = IntegrationExtractValueExporter::new();
        assert_eq!(exporter.get_name(), "IntegrationExtractValue");
    }

    #[test]
    fn test_integration_extract_value_exporter_depends_on() {
        let exporter = IntegrationExtractValueExporter::new();
        assert_eq!(exporter.export_depends_on(), vec!["Project", "Integration"]);
    }
}
