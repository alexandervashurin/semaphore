//! Option Exporter
//!
//! Экспорт опций

use crate::error::{Error, Result};
use crate::models::OptionItem;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр опций
pub struct OptionExporter {
    /// Карта значений
    pub value_map: ValueMap<OptionItem>,
}

impl OptionExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for OptionExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for OptionExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, _exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        let options = store.get_options(crate::db::store::RetrieveQueryParams::default())
            .map_err(|e| Error::Other(format!("Failed to load options: {}", e)))?;

        // Конвертируем HashMap в Vec<Option>
        let option_list: Vec<OptionItem> = options.into_iter()
            .map(|(key, value)| OptionItem::new(0, key, value))
            .collect();

        for option in option_list {
            self.value_map.append_value(option, "global".to_string())?;
        }

        Ok(())
    }

    /// Восстанавливает данные
    fn restore(&mut self, store: &dyn Store, _exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        for val in &self.value_map.values {
            let old = val.value.clone();

            store.set_option(&old.key, &old.value)
                .map_err(|e| Error::Other(format!("Failed to set option: {}", e)))?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "Option"
    }

    /// Получает зависимости экспорта
    fn export_depends_on(&self) -> Vec<&str> {
        vec![]
    }

    /// Получает зависимости импорта
    fn import_depends_on(&self) -> Vec<&str> {
        vec![]
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
    fn test_option_exporter_creation() {
        let exporter = OptionExporter::new();
        assert_eq!(exporter.get_name(), "Option");
    }

    #[test]
    fn test_option_exporter_depends_on() {
        let exporter = OptionExporter::new();
        assert!(exporter.export_depends_on().is_empty());
    }
}
