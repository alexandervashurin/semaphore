//! Task Exporter
//!
//! Экспорт задач

use crate::error::{Error, Result};
use crate::models::Task;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр задач
pub struct TaskExporter {
    /// Карта значений
    pub value_map: ValueMap<Task>,
}

impl TaskExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for TaskExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for TaskExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        // Получаем ключи проектов
        let projs = exporter.get_loaded_keys_int("Project", "global")?;

        for proj in projs {
            let tasks = store.get_tasks(proj, crate::db::store::RetrieveQueryParams::default())
                .map_err(|e| Error::Other(format!("Failed to load tasks: {}", e)))?;

            // Получаем только Task из TaskWithTpl
            let task_list: Vec<Task> = tasks.into_iter().map(|t| t.task).collect();

            for task in task_list {
                self.value_map.append_value(task, proj.to_string())?;
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
            old.inventory_id = exporter.get_new_key_int_ref("Inventory", &val.scope, old.inventory_id, self)?;
            old.schedule_id = exporter.get_new_key_int_ref("Schedule", &val.scope, old.schedule_id, self)?;
            old.user_id = exporter.get_new_key_int_ref("User", "global", old.user_id, self)?;
            old.integration_id = exporter.get_new_key_int_ref("Integration", &val.scope, old.integration_id, self)?;
            old.build_task_id = exporter.get_new_key_int_ref("Task", &val.scope, old.build_task_id, self)?;

            let new_task = store.create_task(old)
                .map_err(|e| Error::Other(format!("Failed to create task: {}", e)))?;

            exporter.map_int_keys(self.get_name(), &val.scope, val.value.id, new_task.id)?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "Task"
    }

    /// Получает зависимости экспорта
    fn export_depends_on(&self) -> Vec<&str> {
        vec!["Project"]
    }

    /// Получает зависимости импорта
    fn import_depends_on(&self) -> Vec<&str> {
        vec!["Project", "Template", "Inventory", "Schedule", "User", "Integration", "Task"]
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
    fn test_task_exporter_creation() {
        let exporter = TaskExporter::new();
        assert_eq!(exporter.get_name(), "Task");
    }

    #[test]
    fn test_task_exporter_depends_on() {
        let exporter = TaskExporter::new();
        assert_eq!(exporter.export_depends_on(), vec!["Project"]);
    }
}
