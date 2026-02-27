//! Task Stage Exporter
//!
//! Экспорт этапов задач

use crate::error::{Error, Result};
use crate::models::TaskStage;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр этапов задач
pub struct TaskStageExporter {
    /// Карта значений
    pub value_map: ValueMap<TaskStage>,
}

impl TaskStageExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for TaskStageExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for TaskStageExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        // Получаем ключи проектов
        let projs = exporter.get_loaded_keys_int("Project", "global")?;

        for proj_id in projs {
            let tasks = exporter.get_loaded_keys_int("Task", &proj_id.to_string())?;

            let mut all_stages = Vec::new();
            for task in tasks {
                let stages = store.get_task_stages(proj_id, task)
                    .map_err(|e| Error::Other(format!("Failed to load task stages: {}", e)))?;

                // Получаем только TaskStage из TaskStageWithResult
                let stage_list: Vec<TaskStage> = stages.into_iter().map(|s| s.stage).collect();
                all_stages.extend(stage_list);
            }

            for stage in all_stages {
                self.value_map.append_value(stage, proj_id.to_string())?;
            }
        }

        Ok(())
    }

    /// Восстанавливает данные
    fn restore(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        for val in &self.value_map.values {
            let mut old = val.value.clone();

            // Восстанавливаем ссылки
            old.task_id = exporter.get_new_key_int("Task", &val.scope, old.task_id, self)?;

            let new_stage = store.create_task_stage(old)
                .map_err(|e| Error::Other(format!("Failed to create task stage: {}", e)))?;

            exporter.map_int_keys(self.get_name(), &val.scope, val.value.id, new_stage.id)?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "TaskStage"
    }

    /// Получает зависимости экспорта
    fn export_depends_on(&self) -> Vec<&str> {
        vec!["Task"]
    }

    /// Получает зависимости импорта
    fn import_depends_on(&self) -> Vec<&str> {
        vec!["Task"]
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
    fn test_task_stage_exporter_creation() {
        let exporter = TaskStageExporter::new();
        assert_eq!(exporter.get_name(), "TaskStage");
    }

    #[test]
    fn test_task_stage_exporter_depends_on() {
        let exporter = TaskStageExporter::new();
        assert_eq!(exporter.export_depends_on(), vec!["Task"]);
    }
}
