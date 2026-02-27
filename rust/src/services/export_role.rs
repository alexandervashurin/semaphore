//! Role Exporter
//!
//! Экспорт ролей

use crate::error::{Error, Result};
use crate::models::Role;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр ролей
pub struct RoleExporter {
    /// Карта значений
    pub value_map: ValueMap<Role>,
}

impl RoleExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for RoleExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for RoleExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        // Получаем ключи проектов
        let projs = exporter.get_loaded_keys_int("Project", "global")?;

        for proj in projs {
            let roles = store.get_roles(proj)
                .map_err(|e| Error::Other(format!("Failed to load roles: {}", e)))?;

            for role in roles {
                self.value_map.append_value(role, proj.to_string())?;
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

            let new_role = store.create_role(old)
                .map_err(|e| Error::Other(format!("Failed to create role: {}", e)))?;

            exporter.map_int_keys(self.get_name(), &val.scope, val.value.id, new_role.id)?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "Role"
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
    fn test_role_exporter_creation() {
        let exporter = RoleExporter::new();
        assert_eq!(exporter.get_name(), "Role");
    }

    #[test]
    fn test_role_exporter_depends_on() {
        let exporter = RoleExporter::new();
        assert_eq!(exporter.export_depends_on(), vec!["Project"]);
    }
}
