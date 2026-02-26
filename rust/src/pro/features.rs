//! PRO Features - функциональные флаги
//!
//! Аналог pro/pkg/features/features.go из Go версии

use crate::models::User;

/// Доступные функции проекта
pub struct ProjectFeatures {
    /// Раннеры проектов
    pub project_runners: bool,
    /// Terraform backend
    pub terraform_backend: bool,
    /// Сводка задач
    pub task_summary: bool,
    /// Секретные хранилища
    pub secret_storages: bool,
}

impl Default for ProjectFeatures {
    fn default() -> Self {
        Self {
            project_runners: false,
            terraform_backend: false,
            task_summary: false,
            secret_storages: false,
        }
    }
}

/// Получает доступные функции для пользователя
pub fn get_features(user: &User, plan: &str) -> ProjectFeatures {
    // По умолчанию все функции отключены
    // В PRO версии могут быть включены в зависимости от плана
    ProjectFeatures::default()
}

/// Проверяет, доступна ли функция
pub fn is_feature_enabled(user: &User, plan: &str, feature: &str) -> bool {
    let features = get_features(user, plan);
    
    match feature {
        "project_runners" => features.project_runners,
        "terraform_backend" => features.terraform_backend,
        "task_summary" => features.task_summary,
        "secret_storages" => features.secret_storages,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_user() -> User {
        User {
            id: 1,
            created: Utc::now(),
            username: "testuser".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password: String::new(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        }
    }

    #[test]
    fn test_get_features_default() {
        let user = create_test_user();
        let features = get_features(&user, "free");

        assert!(!features.project_runners);
        assert!(!features.terraform_backend);
        assert!(!features.task_summary);
        assert!(!features.secret_storages);
    }

    #[test]
    fn test_is_feature_enabled_unknown() {
        let user = create_test_user();
        assert!(!is_feature_enabled(&user, "free", "unknown_feature"));
    }

    #[test]
    fn test_is_feature_enabled_project_runners() {
        let user = create_test_user();
        assert!(!is_feature_enabled(&user, "free", "project_runners"));
    }

    #[test]
    fn test_is_feature_enabled_terraform_backend() {
        let user = create_test_user();
        assert!(!is_feature_enabled(&user, "free", "terraform_backend"));
    }
}
