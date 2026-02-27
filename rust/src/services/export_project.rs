//! Project Exporter
//!
//! Экспорт проектов

use crate::error::{Error, Result};
use crate::models::Project;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр проектов
pub struct ProjectExporter {
    /// Карта значений
    pub value_map: ValueMap<Project>,
}

impl ProjectExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for ProjectExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for ProjectExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        // Получаем ключи пользователей
        let users = exporter.get_loaded_keys_int("User", "global")?;

        let mut all_projects = Vec::new();
        let mut ids = std::collections::HashSet::new();

        for user_id in users {
            let projects = store.get_projects(Some(user_id))
                .map_err(|e| Error::Other(format!("Failed to load projects: {}", e)))?;

            for proj in projects {
                if ids.insert(proj.id) {
                    all_projects.push(proj);
                }
            }
        }

        for proj in all_projects {
            self.value_map.append_value(proj, "global".to_string())?;
        }

        Ok(())
    }

    /// Восстанавливает данные
    fn restore(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        for val in &self.value_map.values {
            let old = val.value.clone();

            let project = store.create_project(old)
                .map_err(|e| Error::Other(format!("Failed to create project: {}", e)))?;

            exporter.map_int_keys(self.get_name(), "global", val.value.id, project.id)?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "Project"
    }

    /// Получает зависимости экспорта
    fn export_depends_on(&self) -> Vec<&str> {
        vec!["User"]
    }

    /// Получает зависимости импорта
    fn import_depends_on(&self) -> Vec<&str> {
        vec!["User"]
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
    fn test_project_exporter_creation() {
        let exporter = ProjectExporter::new();
        assert_eq!(exporter.get_name(), "Project");
    }

    #[test]
    fn test_project_exporter_depends_on() {
        let exporter = ProjectExporter::new();
        assert_eq!(exporter.export_depends_on(), vec!["User"]);
    }
}
