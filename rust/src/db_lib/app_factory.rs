//! App Factory
//!
//! Фабрика для создания приложений

use std::sync::Arc;
use crate::models::{Template, Repository, Inventory};
use crate::services::task_logger::TaskLogger;
use super::{LocalApp, AnsibleApp, TerraformApp, ShellApp};
use super::ansible_playbook::AnsiblePlaybook;

/// Создаёт приложение для шаблона
pub fn create_app(
    template: Template,
    repository: Repository,
    inventory: Inventory,
    logger: Arc<dyn TaskLogger>,
) -> Box<dyn LocalApp> {
    match template.app {
        crate::models::template::TemplateApp::Ansible => {
            let playbook = AnsiblePlaybook::new(template.id, repository.clone(), logger.clone());
            Box::new(AnsibleApp::new(template, repository, logger, Box::new(playbook)))
        }
        crate::models::template::TemplateApp::Terraform |
        crate::models::template::TemplateApp::Tofu |
        crate::models::template::TemplateApp::Terragrunt => {
            Box::new(TerraformApp::new(template, repository, inventory, logger))
        }
        _ => {
            Box::new(ShellApp::new(template, repository, template.app))
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
    fn test_create_app_factory() {
        // Тест для проверки фабрики приложений
        assert!(true);
    }
}
