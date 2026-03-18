//! Exporter Entities - экспорт сущностей с реальными store-запросами
//!
//! Аналог services/export/ файлов из Go версии.
//! Мост async→sync реализован через tokio::task::block_in_place.

use crate::models::*;
use crate::db::store::Store;
use crate::services::exporter_main::{TypeExporter, DataExporter, ValueMap};

// ─────────────────────────────────────────────────────────────
// Вспомогательный макрос: вызов async fn из sync-контекста
// ─────────────────────────────────────────────────────────────
macro_rules! block_on {
    ($fut:expr) => {{
        let handle = tokio::runtime::Handle::current();
        tokio::task::block_in_place(|| handle.block_on($fut))
    }};
}

// ─────────────────────────────────────────────────────────────
// UserExporter
// ─────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct UserExporter {
    users: ValueMap<User>,
}

impl TypeExporter for UserExporter {
    fn load(&mut self, store: &dyn Store, _exporter: &dyn DataExporter) -> Result<(), String> {
        use crate::db::store::UserManager;
        let users = block_on!(store.get_users(crate::db::store::RetrieveQueryParams {
            offset: 0,
            count: Some(1000),
            sort_by: None,
            sort_inverted: false,
            filter: None,
        }))
        .map_err(|e| format!("UserExporter::load — ошибка загрузки пользователей: {e}"))?;

        self.users
            .append_values(users, "global")
            .map_err(|e| format!("UserExporter::load — ошибка сохранения: {e}"))
    }

    fn restore(&mut self, store: &dyn Store, _exporter: &dyn DataExporter) -> Result<(), String> {
        use crate::db::store::UserManager;
        let users = self.users.get_values("global");

        for user in users {
            // Проверяем, существует ли пользователь с таким логином
            let exists = block_on!(store.get_user_by_login_or_email(&user.username, &user.email))
                .is_ok();

            if !exists {
                // Создаём пользователя; пароль не восстанавливается — задаётся временный
                let mut new_user = user.clone();
                new_user.id = 0; // БД присвоит новый id
                let _ = block_on!(store.create_user(new_user, "ChangeMe123!"))
                    .map_err(|e| tracing::warn!("UserExporter::restore — не удалось создать пользователя {}: {e}", user.username));
            }
        }
        Ok(())
    }

    fn get_loaded_keys(&self, scope: &str) -> Result<Vec<String>, String> {
        self.users.get_loaded_keys(scope)
    }

    fn get_loaded_values(&self, _scope: &str) -> Result<Vec<Box<dyn std::any::Any>>, String> {
        Err("Not implemented".to_string())
    }

    fn get_name(&self) -> &str { "User" }
    fn export_depends_on(&self) -> Vec<&str> { vec![] }
    fn import_depends_on(&self) -> Vec<&str> { vec![] }
    fn get_errors(&self) -> Vec<String> { self.users.get_errors() }
    fn clear(&mut self) { self.users.clear() }
}

// ─────────────────────────────────────────────────────────────
// ProjectExporter
// ─────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct ProjectExporter {
    projects: ValueMap<Project>,
}

impl TypeExporter for ProjectExporter {
    fn load(&mut self, store: &dyn Store, _exporter: &dyn DataExporter) -> Result<(), String> {
        use crate::db::store::ProjectStore;
        // None — загружаем все проекты (без фильтра по пользователю)
        let projects = block_on!(store.get_projects(None))
            .map_err(|e| format!("ProjectExporter::load — ошибка загрузки проектов: {e}"))?;

        self.projects
            .append_values(projects, "global")
            .map_err(|e| format!("ProjectExporter::load — ошибка сохранения: {e}"))
    }

    fn restore(&mut self, store: &dyn Store, _exporter: &dyn DataExporter) -> Result<(), String> {
        use crate::db::store::ProjectStore;
        let projects = self.projects.get_values("global");

        for project in projects {
            let mut new_project = project.clone();
            new_project.id = 0; // БД присвоит новый id
            new_project.created = chrono::Utc::now();

            let _ = block_on!(store.create_project(new_project))
                .map_err(|e| tracing::warn!("ProjectExporter::restore — не удалось создать проект '{}': {e}", project.name));
        }
        Ok(())
    }

    fn get_loaded_keys(&self, scope: &str) -> Result<Vec<String>, String> {
        self.projects.get_loaded_keys(scope)
    }

    fn get_loaded_values(&self, _scope: &str) -> Result<Vec<Box<dyn std::any::Any>>, String> {
        Err("Not implemented".to_string())
    }

    fn get_name(&self) -> &str { "Project" }
    fn export_depends_on(&self) -> Vec<&str> { vec!["User"] }
    fn import_depends_on(&self) -> Vec<&str> { vec!["User"] }
    fn get_errors(&self) -> Vec<String> { self.projects.get_errors() }
    fn clear(&mut self) { self.projects.clear() }
}

// ─────────────────────────────────────────────────────────────
// Остальные экспортеры — заглушки (stub-экспортеры), функционал
// делегируется backup_restore.rs для проектных сущностей.
// ─────────────────────────────────────────────────────────────

macro_rules! stub_exporter {
    ($name:ident, $field:ident, $ty:ty, $label:literal, $export_deps:expr, $import_deps:expr) => {
        pub struct $name {
            $field: ValueMap<$ty>,
        }

        impl Default for $name {
            fn default() -> Self {
                Self { $field: ValueMap::new() }
            }
        }

        impl TypeExporter for $name {
            fn get_name(&self) -> &str { $label }
            fn export_depends_on(&self) -> Vec<&str> { $export_deps }
            fn import_depends_on(&self) -> Vec<&str> { $import_deps }
            fn load(&mut self, _store: &dyn Store, _: &dyn DataExporter) -> Result<(), String> { Ok(()) }
            fn restore(&mut self, _store: &dyn Store, _: &dyn DataExporter) -> Result<(), String> { Ok(()) }
            fn get_loaded_keys(&self, scope: &str) -> Result<Vec<String>, String> { self.$field.get_loaded_keys(scope) }
            fn get_loaded_values(&self, _: &str) -> Result<Vec<Box<dyn std::any::Any>>, String> { Err("Not implemented".to_string()) }
            fn get_errors(&self) -> Vec<String> { self.$field.get_errors() }
            fn clear(&mut self) { self.$field.clear() }
        }
    };
}

stub_exporter!(AccessKeyExporter,    access_keys,   AccessKey,   "AccessKey",   vec!["Project"], vec!["Project"]);
stub_exporter!(EnvironmentExporter,  environments,  Environment, "Environment", vec!["Project"], vec!["Project"]);
stub_exporter!(RepositoryExporter,   repositories,  Repository,  "Repository",  vec!["Project", "AccessKey"], vec!["Project", "AccessKey"]);
stub_exporter!(InventoryExporter,    inventories,   Inventory,   "Inventory",   vec!["Project", "AccessKey"], vec!["Project", "AccessKey"]);
stub_exporter!(TemplateExporter,     templates,     Template,    "Template",    vec!["Project", "Inventory", "Repository", "Environment"], vec!["Project", "Inventory", "Repository", "Environment"]);
stub_exporter!(ViewExporter,         views,         View,        "View",        vec!["Project"], vec!["Project"]);
stub_exporter!(ScheduleExporter,     schedules,     Schedule,    "Schedule",    vec!["Project", "Template"], vec!["Project", "Template"]);
stub_exporter!(IntegrationExporter,  integrations,  Integration, "Integration", vec!["Project", "Template"], vec!["Project", "Template"]);
stub_exporter!(TaskExporter,         tasks,         Task,        "Task",        vec!["Project", "Template"], vec!["Project", "Template"]);

// ─────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_exporter_default() {
        let exp = UserExporter::default();
        assert_eq!(exp.get_name(), "User");
        assert!(exp.export_depends_on().is_empty());
    }

    #[test]
    fn test_project_exporter_default() {
        let exp = ProjectExporter::default();
        assert_eq!(exp.get_name(), "Project");
        assert_eq!(exp.export_depends_on(), vec!["User"]);
    }

    #[test]
    fn test_template_exporter_dependencies() {
        let exp = TemplateExporter::default();
        let deps = exp.export_depends_on();
        assert!(deps.contains(&"Project"));
        assert!(deps.contains(&"Inventory"));
        assert!(deps.contains(&"Repository"));
        assert!(deps.contains(&"Environment"));
    }

    #[test]
    fn test_schedule_exporter_dependencies() {
        let exp = ScheduleExporter::default();
        assert!(exp.export_depends_on().contains(&"Template"));
    }
}
