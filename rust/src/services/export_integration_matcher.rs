//! Integration Matcher Exporter
//!
//! Экспорт матчеров интеграций

use crate::error::{Error, Result};
use crate::models::IntegrationMatcher;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр матчеров интеграций
pub struct IntegrationMatcherExporter {
    /// Карта значений
    pub value_map: ValueMap<IntegrationMatcher>,
}

impl IntegrationMatcherExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for IntegrationMatcherExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for IntegrationMatcherExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        // Получаем ключи проектов
        let projs = exporter.get_loaded_keys_int("Project", "global")?;

        for proj in projs {
            let mut all_matchers = Vec::new();

            // Получаем интеграции проекта
            let integrations = exporter.get_loaded_keys_int("Integration", &proj.to_string())?;

            for integration in integrations {
                let matchers = store.get_integration_matchers(proj, crate::db::store::RetrieveQueryParams::default(), integration)
                    .map_err(|e| Error::Other(format!("Failed to load integration matchers: {}", e)))?;

                all_matchers.extend(matchers);
            }

            for matcher in all_matchers {
                self.value_map.append_value(matcher, proj.to_string())?;
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

            let new_matcher = store.create_integration_matcher(old.project_id, old)
                .map_err(|e| Error::Other(format!("Failed to create integration matcher: {}", e)))?;

            exporter.map_int_keys(self.get_name(), &val.scope, val.value.id, new_matcher.id)?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "IntegrationMatcher"
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
    fn test_integration_matcher_exporter_creation() {
        let exporter = IntegrationMatcherExporter::new();
        assert_eq!(exporter.get_name(), "IntegrationMatcher");
    }

    #[test]
    fn test_integration_matcher_exporter_depends_on() {
        let exporter = IntegrationMatcherExporter::new();
        assert_eq!(exporter.export_depends_on(), vec!["Project", "Integration"]);
    }
}
