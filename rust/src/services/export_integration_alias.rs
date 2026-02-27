//! Integration Alias Exporter
//!
//! Экспорт псевдонимов интеграций

use crate::error::{Error, Result};
use crate::models::IntegrationAlias;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр псевдонимов интеграций
pub struct IntegrationAliasExporter {
    /// Карта значений
    pub value_map: ValueMap<IntegrationAlias>,
}

impl IntegrationAliasExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for IntegrationAliasExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for IntegrationAliasExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        // Получаем ключи проектов
        let projs = exporter.get_loaded_keys_int("Project", "global")?;

        for proj in projs {
            // Получаем все псевдонимы проекта
            let mut all_aliases = Vec::new();

            let aliases = store.get_integration_aliases(proj, None)
                .map_err(|e| Error::Other(format!("Failed to load integration aliases: {}", e)))?;

            all_aliases.extend(aliases);

            // Получаем псевдонимы для каждой интеграции
            let integrations = exporter.get_loaded_keys_int("Integration", &proj.to_string())?;

            for integration in integrations {
                let aliases = store.get_integration_aliases(proj, Some(integration))
                    .map_err(|e| Error::Other(format!("Failed to load integration aliases: {}", e)))?;

                all_aliases.extend(aliases);
            }

            for alias in all_aliases {
                self.value_map.append_value(alias, proj.to_string())?;
            }
        }

        Ok(())
    }

    /// Восстанавливает данные
    fn restore(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        for val in &self.value_map.values {
            let mut old = val.value.clone();

            // Восстанавливаем ссылки
            old.integration_id = exporter.get_new_key_int_ref("Integration", &val.scope, old.integration_id, self)?;
            old.project_id = exporter.get_new_key_int("Project", "global", old.project_id, self)?;

            let new_alias = store.create_integration_alias(old)
                .map_err(|e| Error::Other(format!("Failed to create integration alias: {}", e)))?;

            exporter.map_int_keys(self.get_name(), &val.scope, val.value.id, new_alias.id)?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "IntegrationAlias"
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
    fn test_integration_alias_exporter_creation() {
        let exporter = IntegrationAliasExporter::new();
        assert_eq!(exporter.get_name(), "IntegrationAlias");
    }

    #[test]
    fn test_integration_alias_exporter_depends_on() {
        let exporter = IntegrationAliasExporter::new();
        assert_eq!(exporter.export_depends_on(), vec!["Project", "Integration"]);
    }
}
