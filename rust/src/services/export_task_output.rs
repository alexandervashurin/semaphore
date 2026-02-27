//! Task Output Exporter
//!
//! Экспорт вывода задач

use crate::error::{Error, Result};
use crate::models::TaskOutput;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр вывода задач
pub struct TaskOutputExporter {
    /// Карта значений
    pub value_map: ValueMap<TaskOutput>,
}

impl TaskOutputExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for TaskOutputExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for TaskOutputExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, exporter: &dyn DataExporter, progress: Progress) -> Result<()> {
        // Получаем ключи проектов
        let projs = exporter.get_loaded_keys_int("Project", "global")?;

        let task_count = self.task_count(exporter)?;
        let mut task_index = 0;

        for proj_id in projs {
            let tasks = exporter.get_loaded_keys_int("Task", &proj_id.to_string())?;

            let mut all_outputs = Vec::new();
            for task in tasks {
                let outputs = store.get_task_outputs(proj_id, task, crate::db::store::RetrieveQueryParams::default())
                    .map_err(|e| Error::Other(format!("Failed to load task outputs: {}", e)))?;

                all_outputs.extend(outputs);
                task_index += 1;
                progress.update(task_index as f32 / task_count as f32);
            }

            for output in all_outputs {
                self.value_map.append_value(output, proj_id.to_string())?;
            }
        }

        Ok(())
    }

    /// Восстанавливает данные
    fn restore(&mut self, store: &dyn Store, exporter: &dyn DataExporter, progress: Progress) -> Result<()> {
        let mut outputs = Vec::new();
        let size = self.value_map.values.len();

        for (index, val) in self.value_map.values.iter().enumerate() {
            let mut old = val.value.clone();

            // Восстанавливаем ссылки
            old.task_id = exporter.get_new_key_int("Task", &val.scope, old.task_id, self)?;
            old.stage_id = exporter.get_new_key_int_ref("TaskStage", &val.scope, old.stage_id, self)?;

            outputs.push(old);

            if outputs.len() >= 1000 {
                store.insert_task_output_batch(outputs)
                    .map_err(|e| Error::Other(format!("Failed to insert task output batch: {}", e)))?;
                outputs = Vec::new();
            }

            progress.update(index as f32 / size as f32);
        }

        if !outputs.is_empty() {
            store.insert_task_output_batch(outputs)
                .map_err(|e| Error::Other(format!("Failed to insert task output batch: {}", e)))?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "TaskOutput"
    }

    /// Получает зависимости экспорта
    fn export_depends_on(&self) -> Vec<&str> {
        vec!["Task"]
    }

    /// Получает зависимости импорта
    fn import_depends_on(&self) -> Vec<&str> {
        vec!["Task", "TaskStage"]
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

impl TaskOutputExporter {
    /// Подсчитывает количество задач
    fn task_count(&self, exporter: &dyn DataExporter) -> Result<usize> {
        let projs = exporter.get_loaded_keys_int("Project", "global")?;
        let mut count = 0;

        for proj_id in projs {
            let tasks = exporter.get_loaded_keys_int("Task", &proj_id.to_string())?;
            count += tasks.len();
        }

        Ok(count)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_output_exporter_creation() {
        let exporter = TaskOutputExporter::new();
        assert_eq!(exporter.get_name(), "TaskOutput");
    }

    #[test]
    fn test_task_output_exporter_depends_on() {
        let exporter = TaskOutputExporter::new();
        assert_eq!(exporter.export_depends_on(), vec!["Task"]);
    }
}
