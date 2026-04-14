//! AccessKeyManager - управление ключами доступа
//!
//! Реализация трейта AccessKeyManager для SqlStore

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::AccessKey;
use async_trait::async_trait;
use sqlx::Row;

#[async_trait]
impl AccessKeyManager for SqlStore {
    async fn get_access_keys(&self, project_id: i32) -> Result<Vec<AccessKey>> {
        let query = "SELECT * FROM access_key WHERE project_id = $1 ORDER BY name";
        let rows = sqlx::query(query)
            .bind(project_id)
            .fetch_all(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

        Ok(rows
            .into_iter()
            .map(|row| AccessKey {
                id: row.get("id"),
                project_id: row.get("project_id"),
                name: row.get("name"),
                r#type: row.get("type"),
                user_id: row.get("user_id"),
                login_password_login: row.get("login_password_login"),
                login_password_password: row.get("login_password_password"),
                ssh_key: row.get("ssh_key"),
                ssh_passphrase: row.get("ssh_passphrase"),
                access_key_access_key: row.get("access_key_access_key"),
                access_key_secret_key: row.get("access_key_secret_key"),
                secret_storage_id: row.get("secret_storage_id"),
                source_storage_type: row.try_get("source_storage_type").ok().flatten(),
                source_storage_id: row.try_get("source_storage_id").ok().flatten(),
                source_key: row.try_get("source_key").ok().flatten(),
                owner: row.get("owner"),
                environment_id: row.get("environment_id"),
                created: row.get("created"),
            })
            .collect())
    }

    async fn get_access_key(&self, project_id: i32, key_id: i32) -> Result<AccessKey> {
        let query = "SELECT * FROM access_key WHERE id = $1 AND project_id = $2";
        let row = sqlx::query(query)
            .bind(key_id)
            .bind(project_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound("Ключ доступа не найден".to_string()),
                _ => Error::Database(e),
            })?;

        Ok(AccessKey {
            id: row.get("id"),
            project_id: row.get("project_id"),
            name: row.get("name"),
            r#type: row.get("type"),
            user_id: row.get("user_id"),
            login_password_login: row.get("login_password_login"),
            login_password_password: row.get("login_password_password"),
            ssh_key: row.get("ssh_key"),
            ssh_passphrase: row.get("ssh_passphrase"),
            access_key_access_key: row.get("access_key_access_key"),
            access_key_secret_key: row.get("access_key_secret_key"),
            secret_storage_id: row.get("secret_storage_id"),
            source_storage_type: row.try_get("source_storage_type").ok().flatten(),
            source_storage_id: row.try_get("source_storage_id").ok().flatten(),
            source_key: row.try_get("source_key").ok().flatten(),
            owner: row.get("owner"),
            environment_id: row.get("environment_id"),
            created: row.get("created"),
        })
    }

    async fn create_access_key(&self, mut key: AccessKey) -> Result<AccessKey> {
        let query = "INSERT INTO access_key (project_id, name, type, user_id, login_password_login, login_password_password, ssh_key, ssh_passphrase, access_key_access_key, access_key_secret_key, secret_storage_id, owner, environment_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING id";
        let id: i32 = sqlx::query_scalar(query)
            .bind(key.project_id)
            .bind(&key.name)
            .bind(&key.r#type)
            .bind(key.user_id)
            .bind(&key.login_password_login)
            .bind(&key.login_password_password)
            .bind(&key.ssh_key)
            .bind(&key.ssh_passphrase)
            .bind(&key.access_key_access_key)
            .bind(&key.access_key_secret_key)
            .bind(key.secret_storage_id)
            .bind(key.owner.as_ref().map(|o| o.to_string()))
            .bind(key.environment_id)
            .fetch_one(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;

        key.id = id;
        Ok(key)
    }

    async fn update_access_key(&self, key: AccessKey) -> Result<()> {
        let query = "UPDATE access_key SET name = $1, type = $2, user_id = $3, login_password_login = $4, login_password_password = $5, ssh_key = $6, ssh_passphrase = $7, access_key_access_key = $8, access_key_secret_key = $9, secret_storage_id = $10, owner = $11, environment_id = $12 WHERE id = $13 AND project_id = $14";
        sqlx::query(query)
            .bind(&key.name)
            .bind(&key.r#type)
            .bind(key.user_id)
            .bind(&key.login_password_login)
            .bind(&key.login_password_password)
            .bind(&key.ssh_key)
            .bind(&key.ssh_passphrase)
            .bind(&key.access_key_access_key)
            .bind(&key.access_key_secret_key)
            .bind(key.secret_storage_id)
            .bind(key.owner.as_ref().map(|o| o.to_string()))
            .bind(key.environment_id)
            .bind(key.id)
            .bind(key.project_id.unwrap_or(0))
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }

    async fn delete_access_key(&self, project_id: i32, key_id: i32) -> Result<()> {
        let query = "DELETE FROM access_key WHERE id = $1 AND project_id = $2";
        sqlx::query(query)
            .bind(key_id)
            .bind(project_id)
            .execute(self.get_postgres_pool()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::models::access_key::{
        AccessKey, AccessKeyOwner, AccessKeySourceStorageType, AccessKeyType,
    };

    #[test]
    fn test_access_key_type_none_display() {
        assert_eq!(AccessKeyType::None.to_string(), "none");
    }

    #[test]
    fn test_access_key_type_ssh_display() {
        assert_eq!(AccessKeyType::SSH.to_string(), "ssh");
    }

    #[test]
    fn test_access_key_type_login_password_display() {
        assert_eq!(AccessKeyType::LoginPassword.to_string(), "login_password");
    }

    #[test]
    fn test_access_key_type_access_key_display() {
        assert_eq!(AccessKeyType::AccessKey.to_string(), "access_key");
    }

    #[test]
    fn test_access_key_type_from_str_all() {
        assert_eq!("ssh".parse::<AccessKeyType>().unwrap(), AccessKeyType::SSH);
        assert_eq!(
            "login_password".parse::<AccessKeyType>().unwrap(),
            AccessKeyType::LoginPassword
        );
        assert_eq!(
            "access_key".parse::<AccessKeyType>().unwrap(),
            AccessKeyType::AccessKey
        );
        assert_eq!(
            "invalid".parse::<AccessKeyType>().unwrap(),
            AccessKeyType::None
        );
    }

    #[test]
    fn test_access_key_owner_display() {
        assert_eq!(AccessKeyOwner::User.to_string(), "user");
        assert_eq!(AccessKeyOwner::Project.to_string(), "project");
        assert_eq!(AccessKeyOwner::Shared.to_string(), "shared");
    }

    #[test]
    fn test_access_key_owner_from_str() {
        assert_eq!(
            "user".parse::<AccessKeyOwner>().unwrap(),
            AccessKeyOwner::User
        );
        assert_eq!(
            "project".parse::<AccessKeyOwner>().unwrap(),
            AccessKeyOwner::Project
        );
        assert_eq!(
            "shared".parse::<AccessKeyOwner>().unwrap(),
            AccessKeyOwner::Shared
        );
        assert_eq!(
            "unknown".parse::<AccessKeyOwner>().unwrap(),
            AccessKeyOwner::Shared
        );
    }

    #[test]
    fn test_source_storage_type_display() {
        assert_eq!(AccessKeySourceStorageType::DB.to_string(), "db");
        assert_eq!(AccessKeySourceStorageType::Storage.to_string(), "storage");
        assert_eq!(AccessKeySourceStorageType::Env.to_string(), "env");
        assert_eq!(AccessKeySourceStorageType::File.to_string(), "file");
    }

    #[test]
    fn test_source_storage_type_from_str() {
        assert_eq!(
            "db".parse::<AccessKeySourceStorageType>().unwrap(),
            AccessKeySourceStorageType::DB
        );
        assert_eq!(
            "storage".parse::<AccessKeySourceStorageType>().unwrap(),
            AccessKeySourceStorageType::Storage
        );
        assert_eq!(
            "env".parse::<AccessKeySourceStorageType>().unwrap(),
            AccessKeySourceStorageType::Env
        );
        assert_eq!(
            "file".parse::<AccessKeySourceStorageType>().unwrap(),
            AccessKeySourceStorageType::File
        );
        assert_eq!(
            "unknown".parse::<AccessKeySourceStorageType>().unwrap(),
            AccessKeySourceStorageType::DB
        );
    }

    #[test]
    fn test_access_key_new_none() {
        let key = AccessKey::new("test-key".to_string(), AccessKeyType::None);
        assert_eq!(key.name, "test-key");
        assert_eq!(key.r#type, AccessKeyType::None);
        assert!(key.project_id.is_none());
    }

    #[test]
    fn test_access_key_new_ssh() {
        let key = AccessKey::new_ssh(
            1,
            "ssh-key".to_string(),
            "private_key_data".to_string(),
            "passphrase".to_string(),
            "deploy".to_string(),
            Some(42),
        );
        assert_eq!(key.project_id, Some(1));
        assert_eq!(key.r#type, AccessKeyType::SSH);
        assert_eq!(key.login_password_login, Some("deploy".to_string()));
        assert_eq!(key.ssh_key, Some("private_key_data".to_string()));
        assert_eq!(key.ssh_passphrase, Some("passphrase".to_string()));
        assert_eq!(key.user_id, Some(42));
    }

    #[test]
    fn test_access_key_new_login_password() {
        let key = AccessKey::new_login_password(
            5,
            "lp-key".to_string(),
            "admin".to_string(),
            "secret123".to_string(),
            Some(10),
        );
        assert_eq!(key.project_id, Some(5));
        assert_eq!(key.r#type, AccessKeyType::LoginPassword);
        assert_eq!(key.login_password_login, Some("admin".to_string()));
        assert_eq!(key.login_password_password, Some("secret123".to_string()));
    }

    #[test]
    fn test_access_key_serialize() {
        let key = AccessKey::new("ser-test".to_string(), AccessKeyType::SSH);
        let json = serde_json::to_string(&key).unwrap();
        assert!(json.contains("\"name\":\"ser-test\""));
        assert!(json.contains("\"type\":\"ssh\""));
    }

    #[test]
    fn test_access_key_serialize_skip_nulls() {
        let key = AccessKey::new("minimal".to_string(), AccessKeyType::None);
        let json = serde_json::to_string(&key).unwrap();
        assert!(!json.contains("\"ssh_key\":"));
        assert!(!json.contains("\"login_password_login\":"));
    }

    #[test]
    fn test_access_key_get_ssh_key_data() {
        let key = AccessKey::new_ssh(
            1,
            "key".to_string(),
            "pk".to_string(),
            "pp".to_string(),
            "user".to_string(),
            None,
        );
        let data = key.get_ssh_key_data().unwrap();
        assert_eq!(data.private_key, "pk");
        assert_eq!(data.passphrase, Some("pp".to_string()));
        assert_eq!(data.login, "user");
    }

    #[test]
    fn test_access_key_get_login_password_data() {
        let key = AccessKey::new_login_password(
            1,
            "key".to_string(),
            "u".to_string(),
            "p".to_string(),
            None,
        );
        let data = key.get_login_password_data().unwrap();
        assert_eq!(data.login, "u");
        assert_eq!(data.password, "p");
    }

    #[test]
    fn test_access_key_get_type() {
        let key = AccessKey::new("x".to_string(), AccessKeyType::AccessKey);
        assert_eq!(*key.get_type(), AccessKeyType::AccessKey);
    }

    #[test]
    fn test_access_key_clone() {
        let key = AccessKey::new("clone".to_string(), AccessKeyType::LoginPassword);
        let cloned = key.clone();
        assert_eq!(cloned.name, key.name);
        assert_eq!(cloned.r#type, key.r#type);
    }

    #[test]
    fn test_access_key_type_serialize_roundtrip() {
        let variants = vec![
            AccessKeyType::None,
            AccessKeyType::LoginPassword,
            AccessKeyType::SSH,
            AccessKeyType::AccessKey,
        ];
        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let parsed: AccessKeyType = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, variant);
        }
    }

    #[test]
    fn test_access_key_owner_serialize_roundtrip() {
        let variants = vec![
            AccessKeyOwner::User,
            AccessKeyOwner::Project,
            AccessKeyOwner::Shared,
        ];
        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let parsed: AccessKeyOwner = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, variant);
        }
    }
}
