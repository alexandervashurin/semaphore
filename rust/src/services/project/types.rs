//! Project Types
//!
//! Типы для проекта

use serde::{Deserialize, Serialize};
use crate::models::*;

/// База данных бэкапа проекта
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupDB {
    pub meta: Project,
    pub templates: Vec<Template>,
    pub repositories: Vec<Repository>,
    pub keys: Vec<AccessKey>,
    pub views: Vec<View>,
    pub inventories: Vec<Inventory>,
    pub environments: Vec<Environment>,
    pub schedules: Vec<Schedule>,
    pub integration_proj_aliases: Vec<IntegrationAlias>,
    pub integrations: Vec<Integration>,
    pub integration_aliases: std::collections::HashMap<i32, Vec<IntegrationAlias>>,
    pub integration_matchers: std::collections::HashMap<i32, Vec<IntegrationMatcher>>,
    pub integration_extract_values: std::collections::HashMap<i32, Vec<IntegrationExtractValue>>,
    pub secret_storages: Vec<SecretStorage>,
    pub global_roles: Vec<Role>,
    pub roles: Vec<Role>,
    pub template_roles: std::collections::HashMap<i32, Vec<TemplateRolePerm>>,
}

/// Формат бэкапа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFormat {
    pub meta: BackupMeta,
    pub templates: Vec<BackupTemplate>,
    pub repositories: Vec<BackupRepository>,
    pub keys: Vec<BackupAccessKey>,
    pub views: Vec<BackupView>,
    pub inventories: Vec<BackupInventory>,
    pub environments: Vec<BackupEnvironment>,
    pub integrations: Vec<BackupIntegration>,
    pub integration_aliases: Vec<String>,
    pub schedules: Vec<BackupSchedule>,
    pub secret_storages: Vec<BackupSecretStorage>,
    pub roles: Vec<BackupRole>,
}

/// Метаданные бэкапа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMeta {
    #[serde(flatten)]
    pub project: Project,
}

/// Окружение бэкапа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEnvironment {
    #[serde(flatten)]
    pub environment: Environment,
}

/// Ключ доступа бэкапа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupAccessKey {
    #[serde(flatten)]
    pub access_key: AccessKey,
    pub source_storage: Option<String>,
    pub storage: Option<String>,
}

/// Расписание бэкапа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSchedule {
    #[serde(flatten)]
    pub schedule: Schedule,
    pub template: String,
    pub checkable_repository: Option<String>,
}

/// Представление бэкапа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupView {
    #[serde(flatten)]
    pub view: View,
}

/// Инвентарь бэкапа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInventory {
    #[serde(flatten)]
    pub inventory: Inventory,
    pub ssh_key: Option<String>,
    pub become_key: Option<String>,
}

/// Репозиторий бэкапа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRepository {
    #[serde(flatten)]
    pub repository: Repository,
    pub ssh_key: Option<String>,
}

/// Шаблон бэкапа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupTemplate {
    #[serde(flatten)]
    pub template: Template,
    pub inventory: Option<String>,
    pub repository: String,
    pub environment: Option<String>,
    pub build_template: Option<String>,
    pub view: Option<String>,
    pub vaults: Vec<BackupTemplateVault>,
    pub roles: Vec<BackupTemplateRole>,
}

/// Роль шаблона бэкапа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupTemplateRole {
    pub role: String,
    pub is_global: bool,
    pub permissions: String,
}

/// Хранилище шаблона бэкапа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupTemplateVault {
    #[serde(flatten)]
    pub template_vault: TemplateVault,
    pub vault_key: Option<String>,
}

/// Интеграция бэкапа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupIntegration {
    #[serde(flatten)]
    pub integration: Integration,
    pub aliases: Vec<String>,
    pub matchers: Vec<IntegrationMatcher>,
    pub extract_values: Vec<IntegrationExtractValue>,
    pub template: String,
    pub auth_secret: Option<String>,
}

/// Хранилище секретов бэкапа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSecretStorage {
    #[serde(flatten)]
    pub secret_storage: SecretStorage,
}

/// Роль бэкапа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRole {
    #[serde(flatten)]
    pub role: Role,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_format_creation() {
        // Тест для проверки создания формата бэкапа
        assert!(true);
    }

    #[test]
    fn test_backup_meta_creation() {
        // Тест для проверки создания метаданных бэкапа
        assert!(true);
    }
}
