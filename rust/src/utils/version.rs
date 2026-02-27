//! Version Information
//!
//! Информация о версии приложения

/// Версия приложения
pub static VER: &str = env!("CARGO_PKG_VERSION");

/// Commit хэш
pub static COMMIT: &str = env!("GIT_COMMIT");

/// Дата сборки
pub static DATE: &str = env!("BUILD_DATE");

/// Получает полную версию приложения
pub fn version() -> String {
    format!("{}-{}-{}", VER, COMMIT, DATE)
}

/// Получает только версию
pub fn get_version() -> &'static str {
    VER
}

/// Получает commit хэш
pub fn get_commit() -> &'static str {
    COMMIT
}

/// Получает дату сборки
pub fn get_date() -> &'static str {
    DATE
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_format() {
        let ver = version();
        assert!(ver.contains(VER));
    }

    #[test]
    fn test_get_version() {
        assert!(!VER.is_empty());
    }

    #[test]
    fn test_get_commit() {
        // COMMIT может быть "00000000" если не установлен
        assert!(!COMMIT.is_empty());
    }
}
