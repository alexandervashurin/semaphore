//! View Exporter
//!
//! Экспорт представлений

use crate::error::{Error, Result};
use crate::models::View;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр представлений
pub struct ViewExporter {
    /// Карта значений
    pub value_map: ValueMap<View>,
}

impl ViewExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for ViewExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for ViewExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        // Получаем ключи проектов
        let projs = exporter.get_loaded_keys_int("Project", "global")?;

        for proj in projs {
            let views = store.get_views(proj)
                .map_err(|e| Error::Other(format!("Failed to load views: {}", e)))?;

            for view in views {
                self.value_map.append_value(view, proj.to_string())?;
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

            let new_view = store.create_view(old)
                .map_err(|e| Error::Other(format!("Failed to create view: {}", e)))?;

            exporter.map_int_keys(self.get_name(), &val.scope, val.value.id, new_view.id)?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "View"
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
    fn test_view_exporter_creation() {
        let exporter = ViewExporter::new();
        assert_eq!(exporter.get_name(), "View");
    }

    #[test]
    fn test_view_exporter_depends_on() {
        let exporter = ViewExporter::new();
        assert_eq!(exporter.export_depends_on(), vec!["Project"]);
    }
}
