//! Repository Exporter
//!
//! Экспорт репозиториев

use crate::error::{Error, Result};
use crate::models::Repository;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр репозиториев
pub struct RepositoryExporter {
    /// Карта значений
    pub value_map: ValueMap<Repository>,
}

impl RepositoryExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for RepositoryExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for RepositoryExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        // Получаем ключи проектов
        let projs = exporter.get_loaded_keys_int("Project", "global")?;

        for proj in projs {
            let repos = store.get_repositories(proj)
                .map_err(|e| Error::Other(format!("Failed to load repositories: {}", e)))?;

            for repo in repos {
                self.value_map.append_value(repo, proj.to_string())?;
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
            old.ssh_key_id = exporter.get_new_key_int_ref("AccessKey", &val.scope, old.ssh_key_id, self)?;

            let new_repo = store.create_repository(old)
                .map_err(|e| Error::Other(format!("Failed to create repository: {}", e)))?;

            exporter.map_int_keys(self.get_name(), &val.scope, val.value.id, new_repo.id)?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "Repository"
    }

    /// Получает зависимости экспорта
    fn export_depends_on(&self) -> Vec<&str> {
        vec!["Project"]
    }

    /// Получает зависимости импорта
    fn import_depends_on(&self) -> Vec<&str> {
        vec!["Project", "AccessKey"]
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
    fn test_repository_exporter_creation() {
        let exporter = RepositoryExporter::new();
        assert_eq!(exporter.get_name(), "Repository");
    }

    #[test]
    fn test_repository_exporter_depends_on() {
        let exporter = RepositoryExporter::new();
        assert_eq!(exporter.export_depends_on(), vec!["Project"]);
    }
}
