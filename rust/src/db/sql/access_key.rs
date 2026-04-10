//! Access Key CRUD Operations
//!
//! Операции с ключами доступа в SQL

use chrono::Utc;
use crate::db::sql::types::SqlDb;
use crate::error::{Error, Result};
use crate::models::AccessKey;
use sqlx::Row;

impl SqlDb {
    fn pg_pool_access_key(&self) -> Result<&sqlx::PgPool> {
        self.get_postgres_pool()
            .ok_or_else(|| Error::Other("PostgreSQL pool not found".to_string()))
    }

    /// Получает ключи доступа проекта
    pub async fn get_access_keys(&self, project_id: i32) -> Result<Vec<AccessKey>> {
        let rows = sqlx::query("SELECT * FROM access_key WHERE project_id = $1 ORDER BY name")
            .bind(project_id)
            .fetch_all(self.pg_pool_access_key()?)
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

    /// Получает ключ доступа по ID
    pub async fn get_access_key(&self, project_id: i32, key_id: i32) -> Result<AccessKey> {
        let row = sqlx::query("SELECT * FROM access_key WHERE id = $1 AND project_id = $2")
            .bind(key_id)
            .bind(project_id)
            .fetch_one(self.pg_pool_access_key()?)
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

    /// Создаёт ключ доступа
    pub async fn create_access_key(&self, mut key: AccessKey) -> Result<AccessKey> {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO access_key (project_id, name, type, user_id, login_password_login, \
             login_password_password, ssh_key, ssh_passphrase, access_key_access_key, \
             access_key_secret_key, secret_storage_id, owner, environment_id) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING id",
        )
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
        .fetch_one(self.pg_pool_access_key()?)
        .await
        .map_err(Error::Database)?;

        key.id = id;
        Ok(key)
    }

    /// Обновляет ключ доступа
    pub async fn update_access_key(&self, key: AccessKey) -> Result<()> {
        sqlx::query(
            "UPDATE access_key SET name = $1, type = $2, user_id = $3, \
             login_password_login = $4, login_password_password = $5, ssh_key = $6, \
             ssh_passphrase = $7, access_key_access_key = $8, access_key_secret_key = $9, \
             secret_storage_id = $10, owner = $11, environment_id = $12 \
             WHERE id = $13 AND project_id = $14",
        )
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
        .bind(key.project_id)
        .execute(self.pg_pool_access_key()?)
        .await
        .map_err(Error::Database)?;
        Ok(())
    }

    /// Удаляет ключ доступа
    pub async fn delete_access_key(&self, project_id: i32, key_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM access_key WHERE id = $1 AND project_id = $2")
            .bind(key_id)
            .bind(project_id)
            .execute(self.pg_pool_access_key()?)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::access_key::{AccessKeyType, AccessKeyOwner, AccessKeySourceStorageType};

    #[test]
    fn test_access_key_type_display() {
        assert_eq!(AccessKeyType::None.to_string(), "none");
        assert_eq!(AccessKeyType::LoginPassword.to_string(), "login_password");
        assert_eq!(AccessKeyType::SSH.to_string(), "ssh");
        assert_eq!(AccessKeyType::AccessKey.to_string(), "access_key");
    }

    #[test]
    fn test_access_key_type_from_str() {
        assert_eq!("ssh".parse::<AccessKeyType>().unwrap(), AccessKeyType::SSH);
        assert_eq!("login_password".parse::<AccessKeyType>().unwrap(), AccessKeyType::LoginPassword);
        assert_eq!("access_key".parse::<AccessKeyType>().unwrap(), AccessKeyType::AccessKey);
        assert_eq!("none".parse::<AccessKeyType>().unwrap(), AccessKeyType::None);
        assert_eq!("unknown".parse::<AccessKeyType>().unwrap(), AccessKeyType::None);
    }

    #[test]
    fn test_access_key_type_serialize_all() {
        let types = [
            AccessKeyType::None,
            AccessKeyType::LoginPassword,
            AccessKeyType::SSH,
            AccessKeyType::AccessKey,
        ];
        for t in &types {
            let json = serde_json::to_string(t).unwrap();
            assert!(json.starts_with('"') && json.ends_with('"'));
        }
    }

    #[test]
    fn test_access_key_type_deserialize_all() {
        assert_eq!(
            serde_json::from_str::<AccessKeyType>("\"ssh\"").unwrap(),
            AccessKeyType::SSH
        );
        assert_eq!(
            serde_json::from_str::<AccessKeyType>("\"login_password\"").unwrap(),
            AccessKeyType::LoginPassword
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
        assert_eq!("user".parse::<AccessKeyOwner>().unwrap(), AccessKeyOwner::User);
        assert_eq!("project".parse::<AccessKeyOwner>().unwrap(), AccessKeyOwner::Project);
        assert_eq!("shared".parse::<AccessKeyOwner>().unwrap(), AccessKeyOwner::Shared);
        assert_eq!("unknown".parse::<AccessKeyOwner>().unwrap(), AccessKeyOwner::Shared);
    }

    #[test]
    fn test_access_key_source_storage_type_display() {
        assert_eq!(AccessKeySourceStorageType::DB.to_string(), "db");
        assert_eq!(AccessKeySourceStorageType::Storage.to_string(), "storage");
        assert_eq!(AccessKeySourceStorageType::Env.to_string(), "env");
        assert_eq!(AccessKeySourceStorageType::File.to_string(), "file");
    }

    #[test]
    fn test_access_key_source_storage_type_from_str() {
        assert_eq!("db".parse::<AccessKeySourceStorageType>().unwrap(), AccessKeySourceStorageType::DB);
        assert_eq!("storage".parse::<AccessKeySourceStorageType>().unwrap(), AccessKeySourceStorageType::Storage);
        assert_eq!("env".parse::<AccessKeySourceStorageType>().unwrap(), AccessKeySourceStorageType::Env);
        assert_eq!("file".parse::<AccessKeySourceStorageType>().unwrap(), AccessKeySourceStorageType::File);
        assert_eq!("unknown".parse::<AccessKeySourceStorageType>().unwrap(), AccessKeySourceStorageType::DB);
    }

    #[test]
    fn test_access_key_struct_fields() {
        let key = AccessKey {
            id: 1,
            project_id: Some(10),
            name: "Deploy Key".to_string(),
            r#type: AccessKeyType::SSH,
            user_id: Some(5),
            login_password_login: None,
            login_password_password: None,
            ssh_key: Some("-----BEGIN RSA PRIVATE KEY-----".to_string()),
            ssh_passphrase: Some("passphrase".to_string()),
            access_key_access_key: None,
            access_key_secret_key: None,
            secret_storage_id: None,
            source_storage_type: Some(AccessKeySourceStorageType::DB),
            source_storage_id: None,
            source_key: None,
            owner: Some(AccessKeyOwner::Project),
            environment_id: None,
            created: Some(Utc::now()),
        };
        assert_eq!(key.id, 1);
        assert_eq!(key.name, "Deploy Key");
        assert_eq!(key.r#type, AccessKeyType::SSH);
        assert_eq!(key.owner, Some(AccessKeyOwner::Project));
    }

    #[test]
    fn test_access_key_clone() {
        let key = AccessKey {
            id: 1,
            project_id: None,
            name: "Test Key".to_string(),
            r#type: AccessKeyType::None,
            user_id: None,
            login_password_login: None,
            login_password_password: None,
            ssh_key: None,
            ssh_passphrase: None,
            access_key_access_key: None,
            access_key_secret_key: None,
            secret_storage_id: None,
            source_storage_type: None,
            source_storage_id: None,
            source_key: None,
            owner: None,
            environment_id: None,
            created: None,
        };
        let cloned = key.clone();
        assert_eq!(cloned.id, key.id);
        assert_eq!(cloned.name, key.name);
        assert_eq!(cloned.r#type, key.r#type);
    }

    #[test]
    fn test_access_key_serialization() {
        let key = AccessKey {
            id: 1,
            project_id: Some(1),
            name: "Test".to_string(),
            r#type: AccessKeyType::SSH,
            user_id: None,
            login_password_login: None,
            login_password_password: None,
            ssh_key: None,
            ssh_passphrase: None,
            access_key_access_key: None,
            access_key_secret_key: None,
            secret_storage_id: None,
            source_storage_type: None,
            source_storage_id: None,
            source_key: None,
            owner: None,
            environment_id: None,
            created: None,
        };
        let json = serde_json::to_string(&key).unwrap();
        assert!(json.contains("\"name\":\"Test\""));
        assert!(json.contains("\"type\":\"ssh\""));
    }

    #[test]
    fn test_access_key_new_constructor() {
        let key = AccessKey::new("My Key".to_string(), AccessKeyType::SSH);
        assert_eq!(key.id, 0);
        assert_eq!(key.name, "My Key");
        assert_eq!(key.r#type, AccessKeyType::SSH);
        assert!(key.project_id.is_none());
    }
}
