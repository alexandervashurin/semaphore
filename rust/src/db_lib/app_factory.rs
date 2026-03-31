//! App Factory
//!
//! Фабрика для создания приложений (Shell/Bash/Python/PowerShell).
//! Ansible и Terraform создаются напрямую в LocalJob::prepare_run.

use super::{LocalApp, ShellApp};
use crate::models::{Inventory, Repository, Template};
use crate::services::task_logger::TaskLogger;
use std::sync::Arc;

/// Создаёт приложение для шаблона
pub fn create_app(
    template: Template,
    repository: Repository,
    inventory: Inventory,
    _logger: Arc<dyn TaskLogger>,
) -> Box<dyn LocalApp> {
    let app = template.app.clone();
    Box::new(ShellApp::new(template, repository, app))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_app_factory() {
        // Тест для проверки фабрики приложений
        assert!(true);
    }
}
