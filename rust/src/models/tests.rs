//! Тесты для моделей (Models)

use crate::models::access_key::{AccessKey, AccessKeyType};
use crate::models::environment::Environment;
use crate::models::inventory::{Inventory, InventoryType};
use crate::models::repository::{Repository, RepositoryType};
use crate::models::schedule::Schedule;
use crate::models::*;
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================
    // Access Key Tests
    // ============================================

    #[test]
    fn test_access_key_type_display() {
        assert_eq!(AccessKeyType::SSH.to_string(), "ssh");
        assert_eq!(AccessKeyType::LoginPassword.to_string(), "login_password");
        assert_eq!(AccessKeyType::None.to_string(), "none");
        assert_eq!(AccessKeyType::AccessKey.to_string(), "access_key");
    }

    #[test]
    fn test_access_key_type_from_str() {
        assert_eq!("ssh".parse::<AccessKeyType>().unwrap(), AccessKeyType::SSH);
        assert_eq!(
            "login_password".parse::<AccessKeyType>().unwrap(),
            AccessKeyType::LoginPassword
        );
        assert_eq!(
            "none".parse::<AccessKeyType>().unwrap(),
            AccessKeyType::None
        );
        // unknown values fall back to None (no error)
        assert_eq!(
            "invalid".parse::<AccessKeyType>().unwrap(),
            AccessKeyType::None
        );
    }

    #[test]
    fn test_access_key_new_ssh() {
        let key = AccessKey::new_ssh(
            1,
            "Test SSH Key".to_string(),
            "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----".to_string(),
            "passphrase".to_string(),
            "git".to_string(),
            None,
        );

        assert_eq!(key.name, "Test SSH Key");
        assert_eq!(key.r#type, AccessKeyType::SSH);
        assert!(key.ssh_key.is_some());
    }

    #[test]
    fn test_access_key_new_login_password() {
        let key = AccessKey::new_login_password(
            1,
            "Test Login".to_string(),
            "username".to_string(),
            "password123".to_string(),
            None,
        );

        assert_eq!(key.name, "Test Login");
        assert_eq!(key.r#type, AccessKeyType::LoginPassword);
        assert_eq!(key.login_password_login.as_deref(), Some("username"));
        assert_eq!(key.login_password_password.as_deref(), Some("password123"));
    }

    #[test]
    fn test_access_key_type_serialization() {
        let json = serde_json::to_string(&AccessKeyType::SSH).unwrap();
        assert_eq!(json, "\"ssh\"");
        let json = serde_json::to_string(&AccessKeyType::LoginPassword).unwrap();
        assert_eq!(json, "\"login_password\"");
    }

    // ============================================
    // Inventory Tests
    // ============================================

    #[test]
    fn test_inventory_type_serialization() {
        let json = serde_json::to_string(&InventoryType::Static).unwrap();
        assert_eq!(json, "\"static\"");

        let json = serde_json::to_string(&InventoryType::File).unwrap();
        assert_eq!(json, "\"file\"");

        // StaticYaml uses underscore in serialization (not hyphen)
        let json = serde_json::to_string(&InventoryType::StaticYaml).unwrap();
        assert_eq!(json, "\"static_yaml\"");
    }

    #[test]
    fn test_inventory_type_from_str() {
        assert_eq!(
            "static".parse::<InventoryType>().unwrap(),
            InventoryType::Static
        );
        assert_eq!(
            "file".parse::<InventoryType>().unwrap(),
            InventoryType::File
        );
        assert_eq!(
            "static_yaml".parse::<InventoryType>().unwrap(),
            InventoryType::StaticYaml
        );
        // unknown values fall back to Static (no error)
        assert_eq!(
            "invalid".parse::<InventoryType>().unwrap(),
            InventoryType::Static
        );
    }

    #[test]
    fn test_inventory_new() {
        let inventory = Inventory::new(1, "Test Inventory".to_string(), InventoryType::Static);

        assert_eq!(inventory.name, "Test Inventory");
        assert_eq!(inventory.inventory_type, InventoryType::Static);
        assert_eq!(inventory.project_id, 1);
    }

    #[test]
    fn test_inventory_type_display() {
        assert_eq!(InventoryType::Static.to_string(), "static");
        assert_eq!(InventoryType::File.to_string(), "file");
        assert_eq!(InventoryType::StaticYaml.to_string(), "static_yaml");
    }

    // ============================================
    // Repository Tests
    // ============================================

    #[test]
    fn test_repository_new() {
        let repo = Repository::new(
            1,
            "Test Repo".to_string(),
            "https://github.com/test/repo.git".to_string(),
        );

        assert_eq!(repo.name, "Test Repo");
        assert_eq!(repo.git_url, "https://github.com/test/repo.git");
        assert_eq!(repo.git_type, RepositoryType::Git);
        assert_eq!(repo.project_id, 1);
    }

    #[test]
    fn test_repository_type_json_roundtrip() {
        // RepositoryType serializes as a DB-internal value; test via serde_json
        let json = serde_json::to_string(&RepositoryType::Git).unwrap();
        let parsed: RepositoryType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, RepositoryType::Git);
    }

    // ============================================
    // Environment Tests
    // ============================================

    #[test]
    fn test_environment_new() {
        let env = Environment::new(
            1,
            "Production".to_string(),
            r#"{"DB_HOST": "localhost", "DB_PORT": "5432"}"#.to_string(),
        );

        assert_eq!(env.name, "Production");
        assert_eq!(env.json, r#"{"DB_HOST": "localhost", "DB_PORT": "5432"}"#);
        assert_eq!(env.project_id, 1);
    }

    #[test]
    fn test_environment_json_parse() {
        let env = Environment::new(1, "Test".to_string(), r#"{"KEY": "value"}"#.to_string());

        let parsed: serde_json::Value = serde_json::from_str(&env.json).unwrap();
        assert_eq!(parsed["KEY"], "value");
    }

    // ============================================
    // Schedule Tests
    // ============================================

    #[test]
    fn test_schedule_default_active() {
        let schedule = Schedule {
            id: 0,
            template_id: 1,
            project_id: 1,
            cron: "0 0 * * *".to_string(),
            cron_format: None,
            name: "Daily".to_string(),
            active: true,
            last_commit_hash: None,
            repository_id: None,
            created: None,
            run_at: None,
            delete_after_run: false,
        };

        assert_eq!(schedule.template_id, 1);
        assert_eq!(schedule.cron, "0 0 * * *");
        assert!(schedule.active);
    }

    #[test]
    fn test_schedule_cron_validation() {
        let valid_crons = [
            "0 * * * *",
            "0 0 * * *",
            "0 0 * * 0",
            "0 0 1 * *",
            "*/5 * * * *",
        ];

        for cron in valid_crons.iter() {
            let parts: Vec<&str> = cron.split_whitespace().collect();
            assert_eq!(parts.len(), 5, "Invalid cron: {}", cron);
        }
    }

    // ============================================
    // User Tests
    // ============================================

    #[test]
    fn test_user_validate_empty_username() {
        let user = crate::models::user::User {
            id: 0,
            name: "Test".to_string(),
            username: "".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            created: chrono::Utc::now(),
            totp: None,
            email_otp: None,
        };

        assert!(user.validate().is_err());
    }

    #[test]
    fn test_user_validate_empty_email() {
        let user = crate::models::user::User {
            id: 0,
            name: "Test".to_string(),
            username: "testuser".to_string(),
            email: "".to_string(),
            password: "password".to_string(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            created: chrono::Utc::now(),
            totp: None,
            email_otp: None,
        };

        assert!(user.validate().is_err());
    }

    #[test]
    fn test_user_validate_success() {
        let user = crate::models::user::User {
            id: 0,
            name: "Test User".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            admin: false,
            external: false,
            alert: false,
            pro: false,
            created: chrono::Utc::now(),
            totp: None,
            email_otp: None,
        };

        assert!(user.validate().is_ok());
    }

    #[test]
    fn test_project_user_role_display() {
        use crate::models::user::ProjectUserRole;
        assert_eq!(ProjectUserRole::Owner.to_string(), "owner");
        assert_eq!(ProjectUserRole::Manager.to_string(), "manager");
        assert_eq!(ProjectUserRole::Guest.to_string(), "guest");
    }

    // ============================================
    // JSON serialization tests
    // ============================================

    #[test]
    fn test_access_key_type_json_roundtrip() {
        let types = [
            AccessKeyType::SSH,
            AccessKeyType::LoginPassword,
            AccessKeyType::None,
            AccessKeyType::AccessKey,
        ];
        for t in types.iter() {
            let json = serde_json::to_string(t).unwrap();
            let parsed: AccessKeyType = serde_json::from_str(&json).unwrap();
            assert_eq!(t, &parsed);
        }
    }

    #[test]
    fn test_inventory_type_json_roundtrip() {
        let types = [
            InventoryType::Static,
            InventoryType::File,
            InventoryType::StaticYaml,
        ];
        for t in types.iter() {
            let json = serde_json::to_string(t).unwrap();
            let parsed: InventoryType = serde_json::from_str(&json).unwrap();
            assert_eq!(t, &parsed);
        }
    }
}
