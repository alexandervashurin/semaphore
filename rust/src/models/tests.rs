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
        assert_eq!(inventory.inventory_type, InventoryType::Static);
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
            alert: false,
            alert_chat: None,
            max_parallel_tasks: 0,
            r#type: "default".to_string(),
            default_secret_storage_id: None,
        };

        assert_eq!(project.name, "Test Project");
        assert_eq!(project.max_parallel_tasks, 0);
        assert!(project.default_secret_storage_id.is_none());
    }

    #[test]
    fn test_project_new() {
        let project = Project::new("New Project".to_string());

        assert_eq!(project.name, "New Project");
        assert_eq!(project.id, 0);
        assert!(!project.alert);
        assert!(project.alert_chat.is_none());
        assert_eq!(project.max_parallel_tasks, 0);
        assert_eq!(project.r#type, "default");
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
    use crate::models::access_key::{AccessKey, AccessKeyType, AccessKeyOwner, SshKeyData, LoginPasswordData};
    use super::*;

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

    #[test]
    fn test_access_key_type_display() {
        assert_eq!(AccessKeyType::None.to_string(), "none");
        assert_eq!(AccessKeyType::LoginPassword.to_string(), "login_password");
        assert_eq!(AccessKeyType::SSH.to_string(), "ssh");
        assert_eq!(AccessKeyType::AccessKey.to_string(), "access_key");
    }

    #[test]
    fn test_access_key_type_from_str() {
        assert_eq!("login_password".parse::<AccessKeyType>().unwrap(), AccessKeyType::LoginPassword);
        assert_eq!("ssh".parse::<AccessKeyType>().unwrap(), AccessKeyType::SSH);
        assert_eq!("access_key".parse::<AccessKeyType>().unwrap(), AccessKeyType::AccessKey);
        assert_eq!("invalid".parse::<AccessKeyType>().unwrap(), AccessKeyType::None);
    }

    #[test]
    fn test_access_key_owner_display() {
        assert_eq!(AccessKeyOwner::User.to_string(), "user");
        assert_eq!(AccessKeyOwner::Project.to_string(), "project");
        assert_eq!(AccessKeyOwner::Shared.to_string(), "shared");
    }

    #[test]
    fn test_access_key_owner_from_str() {
        assert_eq!("user".parse::<AccessKeyOwner>().unwrap(), AccessKeyOwner::User);
        assert_eq!("project".parse::<AccessKeyOwner>().unwrap(), AccessKeyOwner::Project);
        assert_eq!("shared".parse::<AccessKeyOwner>().unwrap(), AccessKeyOwner::Shared);
        assert_eq!("invalid".parse::<AccessKeyOwner>().unwrap(), AccessKeyOwner::Shared);
    }

    #[test]
    fn test_access_key_new() {
        let key = AccessKey::new("Test Key".to_string(), AccessKeyType::SSH);
        
        assert_eq!(key.name, "Test Key");
        assert_eq!(key.r#type, AccessKeyType::SSH);
        assert_eq!(key.id, 0);
        assert!(key.project_id.is_none());
        assert!(key.user_id.is_none());
        assert!(key.ssh_key.is_none());
    }

    #[test]
    fn test_access_key_new_ssh() {
        let key = AccessKey::new_ssh(
            1,
            "SSH Key".to_string(),
            "private_key_content".to_string(),
            "passphrase".to_string(),
            "admin".to_string(),
            Some(42),
        );

        assert_eq!(key.project_id, Some(1));
        assert_eq!(key.name, "SSH Key");
        assert_eq!(key.r#type, AccessKeyType::SSH);
        assert_eq!(key.ssh_key, Some("private_key_content".to_string()));
        assert_eq!(key.ssh_passphrase, Some("passphrase".to_string()));
        assert_eq!(key.login_password_login, Some("admin".to_string()));
        assert_eq!(key.user_id, Some(42));
    }

    #[test]
    fn test_access_key_new_ssh_empty_passphrase() {
        let key = AccessKey::new_ssh(
            1,
            "SSH Key".to_string(),
            "private_key".to_string(),
            "".to_string(),
            "root".to_string(),
            None,
        );

        assert!(key.ssh_passphrase.is_none());
    }

    #[test]
    fn test_access_key_new_login_password() {
        let key = AccessKey::new_login_password(
            1,
            "Login Password Key".to_string(),
            "myuser".to_string(),
            "mypassword".to_string(),
            Some(10),
        );

        assert_eq!(key.project_id, Some(1));
        assert_eq!(key.name, "Login Password Key");
        assert_eq!(key.r#type, AccessKeyType::LoginPassword);
        assert_eq!(key.login_password_login, Some("myuser".to_string()));
        assert_eq!(key.login_password_password, Some("mypassword".to_string()));
        assert_eq!(key.user_id, Some(10));
        assert!(key.ssh_key.is_none());
    }

    #[test]
    fn test_access_key_get_ssh_key_data() {
        let key = AccessKey::new_ssh(
            1,
            "SSH Key".to_string(),
            "private_key".to_string(),
            "passphrase".to_string(),
            "admin".to_string(),
            None,
        );

        let ssh_data = key.get_ssh_key_data().unwrap();
        assert_eq!(ssh_data.private_key, "private_key");
        assert_eq!(ssh_data.passphrase, Some("passphrase".to_string()));
        assert_eq!(ssh_data.login, "admin");
    }

    #[test]
    fn test_access_key_get_ssh_key_data_none() {
        let key = AccessKey::new("Test".to_string(), AccessKeyType::None);
        assert!(key.get_ssh_key_data().is_none());
    }

    #[test]
    fn test_access_key_get_login_password_data() {
        let key = AccessKey::new_login_password(
            1,
            "LP Key".to_string(),
            "user123".to_string(),
            "pass456".to_string(),
            None,
        );

        let lp_data = key.get_login_password_data().unwrap();
        assert_eq!(lp_data.login, "user123");
        assert_eq!(lp_data.password, "pass456");
    }

    #[test]
    fn test_access_key_get_login_password_data_none() {
        let key = AccessKey::new("Test".to_string(), AccessKeyType::SSH);
        assert!(key.get_login_password_data().is_none());
    }

    #[test]
    fn test_access_key_get_type() {
        let key = AccessKey::new("Test".to_string(), AccessKeyType::SSH);
        assert_eq!(key.get_type(), &AccessKeyType::SSH);
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

#[cfg(test)]
mod audit_log_tests {
    use crate::models::audit_log::{AuditAction, AuditObjectType, AuditLevel, AuditDetails, AuditLog, AuditLogFilter, AuditLogResult};
    use chrono::Utc;
    use serde_json::json;

    // ========================================================================
    // Тесты для AuditAction
    // ========================================================================

    #[test]
    fn test_audit_action_display() {
        // Аутентификация
        assert_eq!(AuditAction::Login.to_string(), "login");
        assert_eq!(AuditAction::Logout.to_string(), "logout");
        assert_eq!(AuditAction::LoginFailed.to_string(), "login_failed");
        
        // Пользователи
        assert_eq!(AuditAction::UserCreated.to_string(), "user_created");
        assert_eq!(AuditAction::UserUpdated.to_string(), "user_updated");
        assert_eq!(AuditAction::UserDeleted.to_string(), "user_deleted");
        
        // Проекты
        assert_eq!(AuditAction::ProjectCreated.to_string(), "project_created");
        assert_eq!(AuditAction::ProjectUpdated.to_string(), "project_updated");
        assert_eq!(AuditAction::ProjectDeleted.to_string(), "project_deleted");
        
        // Задачи
        assert_eq!(AuditAction::TaskCreated.to_string(), "task_created");
        assert_eq!(AuditAction::TaskStarted.to_string(), "task_started");
        assert_eq!(AuditAction::TaskCompleted.to_string(), "task_completed");
        assert_eq!(AuditAction::TaskFailed.to_string(), "task_failed");
        assert_eq!(AuditAction::TaskStopped.to_string(), "task_stopped");
        
        // Шаблоны
        assert_eq!(AuditAction::TemplateCreated.to_string(), "template_created");
        assert_eq!(AuditAction::TemplateRun.to_string(), "template_run");
        
        // Системные
        assert_eq!(AuditAction::ConfigChanged.to_string(), "config_changed");
        assert_eq!(AuditAction::BackupCreated.to_string(), "backup_created");
        assert_eq!(AuditAction::Other.to_string(), "other");
    }

    #[test]
    fn test_audit_action_serialization() {
        let actions = vec![
            AuditAction::Login,
            AuditAction::UserCreated,
            AuditAction::TaskCompleted,
            AuditAction::WebhookTriggered,
            AuditAction::Other,
        ];

        for action in actions {
            let json = serde_json::to_string(&action).unwrap();
            assert!(!json.is_empty());
        }
    }

    // ========================================================================
    // Тесты для AuditObjectType
    // ========================================================================

    #[test]
    fn test_audit_object_type_display() {
        assert_eq!(AuditObjectType::User.to_string(), "user");
        assert_eq!(AuditObjectType::Project.to_string(), "project");
        assert_eq!(AuditObjectType::Task.to_string(), "task");
        assert_eq!(AuditObjectType::Template.to_string(), "template");
        assert_eq!(AuditObjectType::Inventory.to_string(), "inventory");
        assert_eq!(AuditObjectType::Repository.to_string(), "repository");
        assert_eq!(AuditObjectType::Environment.to_string(), "environment");
        assert_eq!(AuditObjectType::AccessKey.to_string(), "access_key");
        assert_eq!(AuditObjectType::Integration.to_string(), "integration");
        assert_eq!(AuditObjectType::Schedule.to_string(), "schedule");
        assert_eq!(AuditObjectType::Runner.to_string(), "runner");
        assert_eq!(AuditObjectType::View.to_string(), "view");
        assert_eq!(AuditObjectType::Secret.to_string(), "secret");
        assert_eq!(AuditObjectType::System.to_string(), "system");
        assert_eq!(AuditObjectType::Other.to_string(), "other");
    }

    #[test]
    fn test_audit_object_type_serialization() {
        let types = vec![
            AuditObjectType::User,
            AuditObjectType::Project,
            AuditObjectType::Task,
            AuditObjectType::System,
            AuditObjectType::Other,
        ];

        for obj_type in types {
            let json = serde_json::to_string(&obj_type).unwrap();
            assert!(!json.is_empty());
        }
    }

    // ========================================================================
    // Тесты для AuditLevel
    // ========================================================================

    #[test]
    fn test_audit_level_display() {
        assert_eq!(AuditLevel::Info.to_string(), "info");
        assert_eq!(AuditLevel::Warning.to_string(), "warning");
        assert_eq!(AuditLevel::Error.to_string(), "error");
        assert_eq!(AuditLevel::Critical.to_string(), "critical");
    }

    #[test]
    fn test_audit_level_ordering() {
        // Проверка PartialOrd и Ord
        assert!(AuditLevel::Info < AuditLevel::Warning);
        assert!(AuditLevel::Warning < AuditLevel::Error);
        assert!(AuditLevel::Error < AuditLevel::Critical);
        assert!(AuditLevel::Info < AuditLevel::Critical);
        
        // Равные уровни
        assert!(AuditLevel::Info >= AuditLevel::Info);
        assert!(AuditLevel::Critical <= AuditLevel::Critical);
    }

    #[test]
    fn test_audit_level_serialization() {
        let levels = vec![
            AuditLevel::Info,
            AuditLevel::Warning,
            AuditLevel::Error,
            AuditLevel::Critical,
        ];

        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            assert!(!json.is_empty());
        }
    }

    // ========================================================================
    // Тесты для AuditDetails
    // ========================================================================

    #[test]
    fn test_audit_details_default() {
        let details = AuditDetails::default();
        assert!(details.ip_address.is_none());
        assert!(details.user_agent.is_none());
        assert!(details.changes.is_none());
        assert!(details.reason.is_none());
        assert!(details.metadata.is_none());
    }

    #[test]
    fn test_audit_details_with_values() {
        let details = AuditDetails {
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            changes: Some(json!({"old": "value1", "new": "value2"})),
            reason: Some("Security update".to_string()),
            metadata: Some(json!({"key": "value"})),
        };

        assert_eq!(details.ip_address, Some("192.168.1.1".to_string()));
        assert_eq!(details.user_agent, Some("Mozilla/5.0".to_string()));
        assert!(details.changes.is_some());
        assert_eq!(details.reason, Some("Security update".to_string()));
        assert!(details.metadata.is_some());
    }

    #[test]
    fn test_audit_details_serialization() {
        let details = AuditDetails {
            ip_address: Some("10.0.0.1".to_string()),
            user_agent: None,
            changes: Some(json!({"field": "value"})),
            reason: None,
            metadata: None,
        };

        let json = serde_json::to_string(&details).unwrap();
        assert!(json.contains("10.0.0.1"));
        // Поля с skip_serializing_if не должны сериализоваться
        assert!(!json.contains("user_agent"));
    }

    // ========================================================================
    // Тесты для AuditLog
    // ========================================================================

    #[test]
    fn test_audit_log_creation() {
        let audit_log = AuditLog {
            id: 1,
            project_id: Some(100),
            user_id: Some(42),
            username: Some("admin".to_string()),
            action: AuditAction::Login,
            object_type: AuditObjectType::User,
            object_id: Some(42),
            object_name: Some("admin".to_string()),
            description: "User logged in".to_string(),
            level: AuditLevel::Info,
            ip_address: Some("192.168.1.100".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            details: None,
            created: Utc::now(),
        };

        assert_eq!(audit_log.id, 1);
        assert_eq!(audit_log.project_id, Some(100));
        assert_eq!(audit_log.user_id, Some(42));
        assert_eq!(audit_log.action, AuditAction::Login);
        assert_eq!(audit_log.level, AuditLevel::Info);
    }

    #[test]
    fn test_audit_log_minimal() {
        let audit_log = AuditLog {
            id: 2,
            project_id: None,
            user_id: None,
            username: None,
            action: AuditAction::Other,
            object_type: AuditObjectType::Other,
            object_id: None,
            object_name: None,
            description: "Unknown action".to_string(),
            level: AuditLevel::Info,
            ip_address: None,
            user_agent: None,
            details: None,
            created: Utc::now(),
        };

        assert!(audit_log.project_id.is_none());
        assert!(audit_log.user_id.is_none());
        assert_eq!(audit_log.action, AuditAction::Other);
    }

    #[test]
    fn test_audit_log_serialization() {
        let audit_log = AuditLog {
            id: 3,
            project_id: Some(1),
            user_id: Some(1),
            username: Some("test".to_string()),
            action: AuditAction::TaskCreated,
            object_type: AuditObjectType::Task,
            object_id: Some(10),
            object_name: Some("Deploy".to_string()),
            description: "Task created".to_string(),
            level: AuditLevel::Info,
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: None,
            details: None,
            created: Utc::now(),
        };

        let json = serde_json::to_string(&audit_log).unwrap();
        assert!(json.contains("\"id\":3"));
        assert!(json.contains("\"action\":\"task_created\""));
        assert!(json.contains("\"level\":\"info\""));
    }

    // ========================================================================
    // Тесты для AuditLogFilter
    // ========================================================================

    #[test]
    fn test_audit_log_filter_default() {
        let filter = AuditLogFilter::default();
        
        assert!(filter.project_id.is_none());
        assert!(filter.user_id.is_none());
        assert!(filter.username.is_none());
        assert!(filter.action.is_none());
        assert!(filter.object_type.is_none());
        assert!(filter.object_id.is_none());
        assert!(filter.level.is_none());
        assert!(filter.search.is_none());
        assert!(filter.date_from.is_none());
        assert!(filter.date_to.is_none());
        // При derive(Default) поля получают значения по умолчанию для типа
        assert_eq!(filter.offset, 0);
        // sort и order используют serde default, но не работают с derive(Default)
    }

    #[test]
    fn test_audit_log_filter_custom() {
        let filter = AuditLogFilter {
            project_id: Some(100),
            user_id: Some(42),
            username: Some("admin".to_string()),
            action: Some(AuditAction::Login),
            object_type: Some(AuditObjectType::User),
            object_id: Some(42),
            level: Some(AuditLevel::Warning),
            search: Some("login".to_string()),
            date_from: Some(Utc::now()),
            date_to: Some(Utc::now()),
            limit: 100,
            offset: 10,
            sort: "username".to_string(),
            order: "asc".to_string(),
        };

        assert_eq!(filter.project_id, Some(100));
        assert_eq!(filter.user_id, Some(42));
        assert_eq!(filter.limit, 100);
        assert_eq!(filter.offset, 10);
        assert_eq!(filter.sort, "username");
        assert_eq!(filter.order, "asc");
    }

    #[test]
    fn test_audit_log_filter_serialization() {
        let filter = AuditLogFilter {
            project_id: Some(1),
            user_id: None,
            username: None,
            action: None,
            object_type: None,
            object_id: None,
            level: None,
            search: None,
            date_from: None,
            date_to: None,
            limit: 25,
            offset: 0,
            sort: "created".to_string(),
            order: "desc".to_string(),
        };

        let json = serde_json::to_string(&filter).unwrap();
        assert!(json.contains("\"project_id\":1"));
        assert!(json.contains("\"limit\":25"));
    }

    // ========================================================================
    // Тесты для AuditLogResult
    // ========================================================================

    #[test]
    fn test_audit_log_result() {
        let result = AuditLogResult {
            total: 100,
            records: vec![],
            limit: 50,
            offset: 0,
        };

        assert_eq!(result.total, 100);
        assert_eq!(result.records.len(), 0);
        assert_eq!(result.limit, 50);
        assert_eq!(result.offset, 0);
    }

    #[test]
    fn test_audit_log_result_with_records() {
        let records = vec![
            AuditLog {
                id: 1,
                project_id: Some(1),
                user_id: Some(1),
                username: Some("user1".to_string()),
                action: AuditAction::Login,
                object_type: AuditObjectType::User,
                object_id: Some(1),
                object_name: Some("user1".to_string()),
                description: "Login".to_string(),
                level: AuditLevel::Info,
                ip_address: None,
                user_agent: None,
                details: None,
                created: Utc::now(),
            },
            AuditLog {
                id: 2,
                project_id: Some(1),
                user_id: Some(2),
                username: Some("user2".to_string()),
                action: AuditAction::Logout,
                object_type: AuditObjectType::User,
                object_id: Some(2),
                object_name: Some("user2".to_string()),
                description: "Logout".to_string(),
                level: AuditLevel::Info,
                ip_address: None,
                user_agent: None,
                details: None,
                created: Utc::now(),
            },
        ];

        let result = AuditLogResult {
            total: 2,
            records: records.clone(),
            limit: 50,
            offset: 0,
        };

        assert_eq!(result.total, 2);
        assert_eq!(result.records.len(), 2);
        assert_eq!(result.records[0].action, AuditAction::Login);
        assert_eq!(result.records[1].action, AuditAction::Logout);
    }
}
