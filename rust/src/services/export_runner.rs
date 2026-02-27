//! Runner Exporter
//!
//! Экспорт раннеров

use crate::error::{Error, Result};
use crate::models::Runner;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр раннеров
pub struct RunnerExporter {
    /// Карта значений
    pub value_map: ValueMap<Runner>,
}

impl RunnerExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for RunnerExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for RunnerExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        // Получаем ключи проектов (или глобальные раннеры)
        let projs = exporter.get_loaded_keys_int("Project", "global")?;

        for proj in projs {
            let runners = store.get_runners(Some(proj))
                .map_err(|e| Error::Other(format!("Failed to load runners: {}", e)))?;

            for runner in runners {
                self.value_map.append_value(runner, proj.to_string())?;
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

            let new_runner = store.create_runner(old)
                .map_err(|e| Error::Other(format!("Failed to create runner: {}", e)))?;

            exporter.map_int_keys(self.get_name(), &val.scope, val.value.id, new_runner.id)?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "Runner"
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
    fn test_runner_exporter_creation() {
        let exporter = RunnerExporter::new();
        assert_eq!(exporter.get_name(), "Runner");
    }

    #[test]
    fn test_runner_exporter_depends_on() {
        let exporter = RunnerExporter::new();
        assert_eq!(exporter.export_depends_on(), vec!["Project"]);
    }
}
