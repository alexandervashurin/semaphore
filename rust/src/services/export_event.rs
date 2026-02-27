//! Event Exporter
//!
//! Экспорт событий

use crate::error::{Error, Result};
use crate::models::Event;
use crate::db::store::Store;
use super::exporter_main::{TypeExporter, DataExporter, ValueMap, Progress};

/// Экспортёр событий
pub struct EventExporter {
    /// Карта значений
    pub value_map: ValueMap<Event>,
}

impl EventExporter {
    /// Создаёт новый экспортёр
    pub fn new() -> Self {
        Self {
            value_map: ValueMap::new(),
        }
    }
}

impl Default for EventExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeExporter for EventExporter {
    /// Загружает данные
    fn load(&mut self, store: &dyn Store, _exporter: &dyn DataExporter, _progress: Progress) -> Result<()> {
        // Получаем все события
        let events = store.get_events(None, 10000)
            .map_err(|e| Error::Other(format!("Failed to load events: {}", e)))?;

        for event in events {
            self.value_map.append_value(event, "global".to_string())?;
        }

        Ok(())
    }

    /// Восстанавливает данные
    fn restore(&mut self, store: &dyn Store, exporter: &dyn DataExporter, progress: Progress) -> Result<()> {
        let size = self.value_map.values.len();

        for (index, val) in self.value_map.values.iter().enumerate() {
            let mut old = val.value.clone();

            // Сбрасываем ID
            old.id = -1;

            // Восстанавливаем ссылки
            old.project_id = exporter.get_new_key_int_ref("Project", "global", old.project_id, self)?;
            old.user_id = exporter.get_new_key_int_ref("User", "global", old.user_id, self)?;

            let scope = if old.project_id.is_some() {
                old.project_id.unwrap().to_string()
            } else {
                "global".to_string()
            };

            old.integration_id = exporter.get_new_key_int_ref("Integration", &scope, old.integration_id, self)?;

            // Восстанавливаем объект события
            self.restore_event_object(&mut old, exporter, &scope)?;

            store.create_event(old)
                .map_err(|e| Error::Other(format!("Failed to create event: {}", e)))?;

            progress.update(index as f32 / size as f32);
        }

        Ok(())
    }

    /// Получает имя
    fn get_name(&self) -> &str {
        "Event"
    }

    /// Получает зависимости экспорта
    fn export_depends_on(&self) -> Vec<&str> {
        vec!["Project", "User"]
    }

    /// Получает зависимости импорта
    fn import_depends_on(&self) -> Vec<&str> {
        vec!["Project", "User", "Integration", "AccessKey", "Schedule", "Environment", "Template", "Task", "Inventory", "Repository", "View"]
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

impl EventExporter {
    /// Восстанавливает объект события
    fn restore_event_object(&self, event: &mut Event, exporter: &dyn DataExporter, scope: &str) -> Result<()> {
        if let Some(object_type) = &event.object_type {
            let entity_name = self.event_object_type_to_entity_name(object_type)?;
            event.object_id = exporter.get_new_key_int_ref(&entity_name, scope, event.object_id, self)?;
        }
        Ok(())
    }

    /// Преобразует тип объекта события в имя сущности
    fn event_object_type_to_entity_name(&self, object_type: &str) -> Result<String> {
        match object_type {
            "task" => Ok("Task".to_string()),
            "repository" => Ok("Repository".to_string()),
            "environment" => Ok("Environment".to_string()),
            "inventory" => Ok("Inventory".to_string()),
            "access_key" => Ok("AccessKey".to_string()),
            "project" => Ok("Project".to_string()),
            "schedule" => Ok("Schedule".to_string()),
            "template" => Ok("Template".to_string()),
            "user" => Ok("User".to_string()),
            "view" => Ok("View".to_string()),
            "integration" => Ok("Integration".to_string()),
            "integration_extract_value" => Ok("IntegrationExtractValue".to_string()),
            "integration_matcher" => Ok("IntegrationMatcher".to_string()),
            _ => Err(Error::Other(format!("Unknown event object type: {}", object_type))),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_exporter_creation() {
        let exporter = EventExporter::new();
        assert_eq!(exporter.get_name(), "Event");
    }

    #[test]
    fn test_event_exporter_depends_on() {
        let exporter = EventExporter::new();
        assert_eq!(exporter.export_depends_on(), vec!["Project", "User"]);
    }
}
