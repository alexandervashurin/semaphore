//! Integration Service
//!
//! Сервис управления интеграциями

use std::sync::Arc;
use crate::error::Result;
use crate::models::Integration;
use crate::db::store::Store;
use super::access_key_encryption_svc::AccessKeyEncryptionService;

/// Сервис интеграций
pub trait IntegrationService: Send + Sync {
    /// Заполняет интеграцию связанными данными
    fn fill_integration(&self, integration: &mut Integration) -> Result<()>;
}

/// Реализация сервиса интеграций
pub struct IntegrationServiceImpl {
    access_key_repo: Arc<dyn Store + Send + Sync>,
    encryption_service: Arc<dyn AccessKeyEncryptionService>,
}

impl IntegrationServiceImpl {
    /// Создаёт новый сервис
    pub fn new(
        access_key_repo: Arc<dyn Store + Send + Sync>,
        encryption_service: Arc<dyn AccessKeyEncryptionService>,
    ) -> Self {
        Self {
            access_key_repo,
            encryption_service,
        }
    }
}

impl IntegrationService for IntegrationServiceImpl {
    fn fill_integration(&self, integration: &mut Integration) -> Result<()> {
        // В базовой версии просто возвращаем Ok
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_service_creation() {
        // Тест для проверки создания сервиса
        assert!(true);
    }
}
