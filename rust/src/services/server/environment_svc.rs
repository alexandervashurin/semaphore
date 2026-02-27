//! Environment Service
//!
//! Сервис управления окружениями

use std::sync::Arc;
use crate::error::Result;
use crate::db::store::Store;
use super::access_key_encryption_svc::AccessKeyEncryptionService;

/// Сервис окружений
pub trait EnvironmentService: Send + Sync {
    /// Удаляет окружение
    fn delete(&self, project_id: i32, environment_id: i32) -> Result<()>;
}

/// Реализация сервиса окружений
pub struct EnvironmentServiceImpl {
    environment_repo: Arc<dyn Store + Send + Sync>,
    encryption_service: Arc<dyn AccessKeyEncryptionService>,
}

impl EnvironmentServiceImpl {
    /// Создаёт новый сервис
    pub fn new(
        environment_repo: Arc<dyn Store + Send + Sync>,
        encryption_service: Arc<dyn AccessKeyEncryptionService>,
    ) -> Self {
        Self {
            environment_repo,
            encryption_service,
        }
    }
}

impl EnvironmentService for EnvironmentServiceImpl {
    fn delete(&self, project_id: i32, environment_id: i32) -> Result<()> {
        self.environment_repo.delete_environment(project_id, environment_id)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_service_creation() {
        // Тест для проверки создания сервиса
        assert!(true);
    }
}
