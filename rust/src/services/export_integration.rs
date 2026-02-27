//! Integration Exporter
//!
//! Экспорт интеграций

use crate::error::{Error, Result};
use crate::models::Integration;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр интеграций
pub struct IntegrationExporter {
    /// Карта значений
    pub value_map: ValueMap<Integration>,
}

impl IntegrationExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for IntegrationExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for IntegrationExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        // Получаем ключи проектов
        let projs = exporter.get_loaded_keys_int("Project", "global")?;

        for proj in projs {
            let integrations = store.get_integrations(proj, crate::db::store::RetrieveQueryParams::default())
                .map_err(|e| Error::Other(format!("Failed to load integrations: {}", e)))?;

            for integration in integrations {
                self.value_map.append_value(integration, proj.to_string())?;
            }
        }

        Ok(())
    }

    /// Восстанавливает данные
    fn restore(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        for val in &self.value_map.values {
            let mut old = val.value.clone();

            // Восстанавливаем ссылки
            if let Some(ref mut task_params) = old.task_params {
                task_params.inventory_id = exporter.get_new_key_int_ref("Inventory", &val.scope, task_params.inventory_id, self)?;
                task_params.project_id = exporter.get_new_key_int("Project", "global", old.project_id, self)?;
            }

            old.template_id = exporter.get_new_key_int("Template", &val.scope, old.template_id, self)?;
            old.auth_secret_id = exporter.get_new_key_int_ref("AccessKey", &val.scope, old.auth_secret_id, self)?;
            old.project_id = exporter.get_new_key_int("Project", "global", old.project_id, self)?;

            let integration = store.create_integration(old)
                .map_err(|e| Error::Other(format!("Failed to create integration: {}", e)))?;

            exporter.map_int_keys(self.get_name(), &val.scope, val.value.id, integration.id)?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "Integration"
    }

    /// Получает зависимости экспорта
    fn export_depends_on(&self) -> Vec<&str> {
        vec!["Project"]
    }

    /// Получает зависимости импорта
    fn import_depends_on(&self) -> Vec<&str> {
        vec!["Project", "Template", "AccessKey", "Inventory"]
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
    fn test_integration_exporter_creation() {
        let exporter = IntegrationExporter::new();
        assert_eq!(exporter.get_name(), "Integration");
    }

    #[test]
    fn test_integration_exporter_depends_on() {
        let exporter = IntegrationExporter::new();
        assert_eq!(exporter.export_depends_on(), vec!["Project"]);
    }
}
