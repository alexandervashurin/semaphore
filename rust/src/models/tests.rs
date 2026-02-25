//! Тесты для моделей данных

use chrono::Utc;

#[cfg(test)]
mod user_tests {
    use crate::models::user::{User, ValidationError};
    use super::*;

    fn create_test_user() -> User {
        User {
            id: 1,
            created: Utc::now(),
            username: "testuser".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password: "hashed_password".to_string(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        }
    }

    #[test]
    fn test_user_validate_success() {
        let user = create_test_user();
        assert!(user.validate().is_ok());
    }

    #[test]
    fn test_user_validate_empty_username() {
        let mut user = create_test_user();
        user.username = String::new();
        assert!(matches!(user.validate(), Err(ValidationError::UsernameEmpty)));
    }

    #[test]
    fn test_user_validate_empty_email() {
        let mut user = create_test_user();
        user.email = String::new();
        assert!(matches!(user.validate(), Err(ValidationError::EmailEmpty)));
    }

    #[test]
    fn test_user_validate_empty_name() {
        let mut user = create_test_user();
        user.name = String::new();
        assert!(matches!(user.validate(), Err(ValidationError::NameEmpty)));
    }
}

#[cfg(test)]
mod inventory_tests {
    use crate::models::inventory::{Inventory, InventoryType};

    #[test]
    fn test_inventory_new() {
        let inventory = Inventory::new(
            1,
            "Test Inventory".to_string(),
            InventoryType::Static,
        );

        assert_eq!(inventory.project_id, 1);
        assert_eq!(inventory.name, "Test Inventory");
        assert_eq!(inventory.inventory, InventoryType::Static);
        assert_eq!(inventory.id, 0);
        assert_eq!(inventory.ssh_login, "root");
        assert_eq!(inventory.ssh_port, 22);
        assert!(inventory.extra_vars.is_none());
    }

    #[test]
    fn test_inventory_type_serialization() {
        let types = vec![
            InventoryType::Static,
            InventoryType::StaticYaml,
            InventoryType::StaticJson,
            InventoryType::File,
            InventoryType::TerraformInventory,
        ];

        for inv_type in types {
            let json = serde_json::to_string(&inv_type).unwrap();
            assert!(!json.is_empty());
        }
    }
}

#[cfg(test)]
mod project_tests {
    use crate::models::project::Project;
    use chrono::Utc;

    #[test]
    fn test_project_creation() {
        let project = Project {
            id: 0,
            created: Utc::now(),
            name: "Test Project".to_string(),
            alert: None,
            alert_chat: None,
            max_parallel_tasks: None,
            r#type: None,
            default_secret_storage_id: None,
        };

        assert_eq!(project.name, "Test Project");
        assert!(project.max_parallel_tasks.is_none());
        assert!(project.default_secret_storage_id.is_none());
    }

    #[test]
    fn test_project_new() {
        let project = Project::new("New Project".to_string());

        assert_eq!(project.name, "New Project");
        assert_eq!(project.id, 0);
        assert!(project.alert.is_none());
        assert!(project.alert_chat.is_none());
        assert!(project.max_parallel_tasks.is_none());
        assert!(project.r#type.is_none());
        assert!(project.default_secret_storage_id.is_none());
    }

    #[test]
    fn test_project_validate_success() {
        let project = Project::new("Valid Project".to_string());
        assert!(project.validate().is_ok());
    }

    #[test]
    fn test_project_validate_empty_name() {
        let project = Project::new("".to_string());
        assert!(project.validate().is_err());
    }
}

#[cfg(test)]
mod template_tests {
    use crate::models::template::{TemplateApp, TemplateType};

    #[test]
    fn test_template_type_serialization() {
        let types = vec![
            TemplateType::Default,
            TemplateType::Build,
        ];

        for template_type in types {
            let json = serde_json::to_string(&template_type).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_template_app_serialization() {
        let apps = vec![
            TemplateApp::Ansible,
            TemplateApp::Terraform,
            TemplateApp::Tofu,
            TemplateApp::Bash,
            TemplateApp::PowerShell,
        ];

        for app in apps {
            let json = serde_json::to_string(&app).unwrap();
            assert!(!json.is_empty());
        }
    }
}

#[cfg(test)]
mod access_key_tests {
    use crate::models::access_key::AccessKeyType;

    #[test]
    fn test_access_key_type_serialization() {
        let types = vec![
            AccessKeyType::None,
            AccessKeyType::LoginPassword,
            AccessKeyType::SSH,
            AccessKeyType::AccessKey,
        ];

        for key_type in types {
            let json = serde_json::to_string(&key_type).unwrap();
            assert!(!json.is_empty());
        }
    }
}

#[cfg(test)]
mod session_tests {
    use crate::models::session::SessionVerificationMethod;

    #[test]
    fn test_session_verification_method_serialization() {
        let methods = vec![
            SessionVerificationMethod::None,
            SessionVerificationMethod::Totp,
            SessionVerificationMethod::EmailOtp,
        ];

        for method in methods {
            let json = serde_json::to_string(&method).unwrap();
            assert!(!json.is_empty());
        }
    }
}
