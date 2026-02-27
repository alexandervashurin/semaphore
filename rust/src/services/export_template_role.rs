//! Template Role Exporter
//!
//! Экспорт ролей шаблонов

use crate::error::{Error, Result};
use crate::models::TemplateRolePerm;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр ролей шаблонов
pub struct TemplateRoleExporter {
    /// Карта значений
    pub value_map: ValueMap<TemplateRolePerm>,
}

impl TemplateRoleExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for TemplateRoleExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for TemplateRoleExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        // Получаем ключи проектов
        let projs = exporter.get_loaded_keys_int("Project", "global")?;

        for proj_id in projs {
            let templates = exporter.get_loaded_keys_int("Template", &proj_id.to_string())?;

            let mut all_roles = Vec::new();
            for template in templates {
                let roles = store.get_template_roles(proj_id, template)
                    .map_err(|e| Error::Other(format!("Failed to load template roles: {}", e)))?;

                all_roles.extend(roles);
            }

            for role in all_roles {
                self.value_map.append_value(role, proj_id.to_string())?;
            }
        }

        Ok(())
    }

    /// Восстанавливает данные
    fn restore(&mut self, store: &dyn Store, exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        for val in &self.value_map.values {
            let mut old = val.value.clone();

            // Восстанавливаем ссылки
            old.role_slug = exporter.get_new_key("Role", &val.scope, old.role_slug, self)?;
            old.template_id = exporter.get_new_key_int("Template", &val.scope, old.template_id, self)?;
            old.project_id = exporter.get_new_key_int("Project", "global", old.project_id, self)?;

            let new_role = store.create_template_role(old)
                .map_err(|e| Error::Other(format!("Failed to create template role: {}", e)))?;

            exporter.map_int_keys(self.get_name(), &val.scope, val.value.id, new_role.id)?;
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "TemplateRole"
    }

    /// Получает зависимости экспорта
    fn export_depends_on(&self) -> Vec<&str> {
        vec!["Template", "Project"]
    }

    /// Получает зависимости импорта
    fn import_depends_on(&self) -> Vec<&str> {
        vec!["Template", "Project"]
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
    fn test_template_role_exporter_creation() {
        let exporter = TemplateRoleExporter::new();
        assert_eq!(exporter.get_name(), "TemplateRole");
    }

    #[test]
    fn test_template_role_exporter_depends_on() {
        let exporter = TemplateRoleExporter::new();
        assert_eq!(exporter.export_depends_on(), vec!["Template", "Project"]);
    }
}
