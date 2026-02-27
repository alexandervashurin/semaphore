//! Template Vault Exporter
//!
//! Экспорт хранилищ шаблонов

use crate::error::{Error, Result};
use crate::models::TemplateVault;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр хранилищ шаблонов
pub struct TemplateVaultExporter {
    /// Карта значений
    pub value_map: ValueMap<TemplateVault>,
}

impl TemplateVaultExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for TemplateVaultExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for TemplateVaultExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        // Получаем ключи проектов
        let projs = exporter.get_loaded_keys_int("Project", "global")?;

        for proj_id in projs {
            let templates = exporter.get_loaded_keys_int("Template", &proj_id.to_string())?;

            let mut all_vaults = Vec::new();
            for template in templates {
                let vaults = store.get_template_vaults(proj_id, template)
                    .map_err(|e| Error::Other(format!("Failed to load template vaults: {}", e)))?;

                all_vaults.extend(vaults);
            }

            for vault in all_vaults {
                self.value_map.append_value(vault, proj_id.to_string())?;
            }
        }

        Ok(())
    }

    /// Восстанавливает данные
    fn restore(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        for val in &self.value_map.values {
            let mut old = val.value.clone();

            // Восстанавливаем ссылки
            old.vault_key_id = exporter.get_new_key_int_ref("AccessKey", &val.scope, old.vault_key_id, self)?;
            old.template_id = exporter.get_new_key_int("Template", &val.scope, old.template_id, self)?;
            old.project_id = exporter.get_new_key_int("Project", "global", old.project_id, self)?;

            let new_vault = store.create_template_vault(old)
                .map_err(|e| Error::Other(format!("Failed to create template vault: {}", e)))?;

            exporter.map_int_keys(self.get_name(), &val.scope, val.value.id, new_vault.id)?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "TemplateVault"
    }

    /// Получает зависимости экспорта
    fn export_depends_on(&self) -> Vec<&str> {
        vec!["Template"]
    }

    /// Получает зависимости импорта
    fn import_depends_on(&self) -> Vec<&str> {
        vec!["Template", "AccessKey"]
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
    fn test_template_vault_exporter_creation() {
        let exporter = TemplateVaultExporter::new();
        assert_eq!(exporter.get_name(), "TemplateVault");
    }

    #[test]
    fn test_template_vault_exporter_depends_on() {
        let exporter = TemplateVaultExporter::new();
        assert_eq!(exporter.export_depends_on(), vec!["Template"]);
    }
}
