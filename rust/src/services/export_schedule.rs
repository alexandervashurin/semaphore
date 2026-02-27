//! Schedule Exporter
//!
//! Экспорт расписаний

use crate::error::{Error, Result};
use crate::models::Schedule;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр расписаний
pub struct ScheduleExporter {
    /// Карта значений
    pub value_map: ValueMap<Schedule>,
}

impl ScheduleExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for ScheduleExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for ScheduleExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        // Получаем ключи проектов
        let projs = exporter.get_loaded_keys_int("Project", "global")?;

        for proj in projs {
            let schedules = store.get_schedules(proj)
                .map_err(|e| Error::Other(format!("Failed to load schedules: {}", e)))?;

            for schedule in schedules {
                self.value_map.append_value(schedule, proj.to_string())?;
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
            old.template_id = exporter.get_new_key_int("Template", &val.scope, old.template_id, self)?;

            let new_schedule = store.create_schedule(old)
                .map_err(|e| Error::Other(format!("Failed to create schedule: {}", e)))?;

            exporter.map_int_keys(self.get_name(), &val.scope, val.value.id, new_schedule.id)?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "Schedule"
    }

    /// Получает зависимости экспорта
    fn export_depends_on(&self) -> Vec<&str> {
        vec!["Project", "Template"]
    }

    /// Получает зависимости импорта
    fn import_depends_on(&self) -> Vec<&str> {
        vec!["Project", "Template"]
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
    fn test_schedule_exporter_creation() {
        let exporter = ScheduleExporter::new();
        assert_eq!(exporter.get_name(), "Schedule");
    }

    #[test]
    fn test_schedule_exporter_depends_on() {
        let exporter = ScheduleExporter::new();
        assert_eq!(exporter.export_depends_on(), vec!["Project", "Template"]);
    }
}
