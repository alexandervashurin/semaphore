//! Модель ключа доступа (AccessKey)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{database::Database, decode::Decode, encode::Encode, FromRow, Type};

/// Данные SSH ключа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyData {
    pub private_key: String,
    pub passphrase: Option<String>,
    pub login: String,
}

/// Данные логина/пароля
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginPasswordData {
    pub login: String,
    pub password: String,
}

/// Тип ключа доступа
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AccessKeyType {
    None,
    LoginPassword,
    #[serde(rename = "ssh")]
    SSH,
    AccessKey,
}

impl std::fmt::Display for AccessKeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessKeyType::None => write!(f, "none"),
            AccessKeyType::LoginPassword => write!(f, "login_password"),
            AccessKeyType::SSH => write!(f, "ssh"),
            AccessKeyType::AccessKey => write!(f, "access_key"),
        }
    }
}

impl std::str::FromStr for AccessKeyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "login_password" => Ok(AccessKeyType::LoginPassword),
            "ssh" => Ok(AccessKeyType::SSH),
            "access_key" => Ok(AccessKeyType::AccessKey),
            _ => Ok(AccessKeyType::None),
        }
    }
}

impl<DB: Database> Type<DB> for AccessKeyType
where
    String: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for AccessKeyType
where
    String: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as Decode<'r, DB>>::decode(value)?;
        Ok(match s.as_str() {
            "login_password" => AccessKeyType::LoginPassword,
            "ssh" => AccessKeyType::SSH,
            "access_key" => AccessKeyType::AccessKey,
            _ => AccessKeyType::None,
        })
    }
}

impl<'q, DB: Database> Encode<'q, DB> for AccessKeyType
where
    DB: 'q,
    String: Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            AccessKeyType::None => "none",
            AccessKeyType::LoginPassword => "login_password",
            AccessKeyType::SSH => "ssh",
            AccessKeyType::AccessKey => "access_key",
        }
        .to_string();
        <String as Encode<'q, DB>>::encode(s, buf)
    }
}

/// Владелец ключа
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AccessKeyOwner {
    User,
    Project,
    Shared,
}

impl std::fmt::Display for AccessKeyOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessKeyOwner::User => write!(f, "user"),
            AccessKeyOwner::Project => write!(f, "project"),
            AccessKeyOwner::Shared => write!(f, "shared"),
        }
    }
}

impl std::str::FromStr for AccessKeyOwner {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(AccessKeyOwner::User),
            "project" => Ok(AccessKeyOwner::Project),
            "shared" => Ok(AccessKeyOwner::Shared),
            _ => Ok(AccessKeyOwner::Shared),
        }
    }
}

impl<DB: Database> Type<DB> for AccessKeyOwner
where
    String: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for AccessKeyOwner
where
    String: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as Decode<'r, DB>>::decode(value)?;
        Ok(match s.as_str() {
            "user" => AccessKeyOwner::User,
            "project" => AccessKeyOwner::Project,
            "shared" => AccessKeyOwner::Shared,
            _ => AccessKeyOwner::Shared,
        })
    }
}

impl<'q, DB: Database> Encode<'q, DB> for AccessKeyOwner
where
    DB: 'q,
    for<'a> &'a str: Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            AccessKeyOwner::User => "user",
            AccessKeyOwner::Project => "project",
            AccessKeyOwner::Shared => "shared",
        };
        <&str as Encode<'q, DB>>::encode(s, buf)
    }
}

/// Тип источника хранения ключа
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AccessKeySourceStorageType {
    #[default]
    DB,
    Storage,
    Env,
    File,
}

impl std::fmt::Display for AccessKeySourceStorageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessKeySourceStorageType::DB => write!(f, "db"),
            AccessKeySourceStorageType::Storage => write!(f, "storage"),
            AccessKeySourceStorageType::Env => write!(f, "env"),
            AccessKeySourceStorageType::File => write!(f, "file"),
        }
    }
}

impl std::str::FromStr for AccessKeySourceStorageType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "db" => Ok(AccessKeySourceStorageType::DB),
            "storage" => Ok(AccessKeySourceStorageType::Storage),
            "env" => Ok(AccessKeySourceStorageType::Env),
            "file" => Ok(AccessKeySourceStorageType::File),
            _ => Ok(AccessKeySourceStorageType::DB),
        }
    }
}

impl<DB: Database> Type<DB> for AccessKeySourceStorageType
where
    String: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for AccessKeySourceStorageType
where
    String: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as Decode<'r, DB>>::decode(value)?;
        Ok(match s.as_str() {
            "db" => AccessKeySourceStorageType::DB,
            "storage" => AccessKeySourceStorageType::Storage,
            "env" => AccessKeySourceStorageType::Env,
            "file" => AccessKeySourceStorageType::File,
            _ => AccessKeySourceStorageType::DB,
        })
    }
}

impl<'q, DB: Database> Encode<'q, DB> for AccessKeySourceStorageType
where
    DB: 'q,
    String: Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            AccessKeySourceStorageType::DB => "db",
            AccessKeySourceStorageType::Storage => "storage",
            AccessKeySourceStorageType::Env => "env",
            AccessKeySourceStorageType::File => "file",
        }
        .to_string();
        <String as Encode<'q, DB>>::encode(s, buf)
    }
}

/// Ключ доступа - учётные данные для подключения
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccessKey {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта (None для глобальных ключей)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<i32>,

    /// Название ключа
    pub name: String,

    /// Тип ключа
    pub r#type: AccessKeyType,

    /// ID пользователя (для user-ключа)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i32>,

    /// Логин
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_password_login: Option<String>,

    /// Пароль (зашифрованный)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_password_password: Option<String>,

    /// SSH-ключ
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_key: Option<String>,

    /// SSH-пароль для ключа
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_passphrase: Option<String>,

    /// Access Key ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key_access_key: Option<String>,

    /// Secret Key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key_secret_key: Option<String>,

    /// ID хранилища секретов
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_storage_id: Option<i32>,

    /// Тип источника хранения
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_storage_type: Option<AccessKeySourceStorageType>,

    /// ID источника (для типа storage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_storage_id: Option<i32>,

    /// Ключ источника (путь/имя переменной для env/file)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_key: Option<String>,

    /// Владелец ключа
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<AccessKeyOwner>,

    /// ID окружения
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<i32>,

    /// Дата создания
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<Utc>>,
}

impl AccessKey {
    /// Создаёт новый ключ доступа
    pub fn new(name: String, key_type: AccessKeyType) -> Self {
        Self {
            id: 0,
            project_id: None,
            name,
            r#type: key_type,
            user_id: None,
            login_password_login: None,
            login_password_password: None,
            ssh_key: None,
            ssh_passphrase: None,
            access_key_access_key: None,
            access_key_secret_key: None,
            secret_storage_id: None,
            source_storage_type: Some(AccessKeySourceStorageType::DB),
            source_storage_id: None,
            source_key: None,
            owner: None,
            environment_id: None,
            created: None,
        }
    }

    /// Создаёт новый SSH ключ
    pub fn new_ssh(
        project_id: i32,
        name: String,
        private_key: String,
        passphrase: String,
        login: String,
        user_id: Option<i32>,
    ) -> Self {
        Self {
            id: 0,
            project_id: Some(project_id),
            name,
            r#type: AccessKeyType::SSH,
            user_id,
            login_password_login: Some(login),
            login_password_password: None,
            ssh_key: Some(private_key),
            ssh_passphrase: if passphrase.is_empty() {
                None
            } else {
                Some(passphrase)
            },
            access_key_access_key: None,
            access_key_secret_key: None,
            secret_storage_id: None,
            source_storage_type: Some(AccessKeySourceStorageType::DB),
            source_storage_id: None,
            source_key: None,
            owner: None,
            environment_id: None,
            created: None,
        }
    }

    /// Создаёт новый ключ логин/пароль
    pub fn new_login_password(
        project_id: i32,
        name: String,
        login: String,
        password: String,
        user_id: Option<i32>,
    ) -> Self {
        Self {
            id: 0,
            project_id: Some(project_id),
            name,
            r#type: AccessKeyType::LoginPassword,
            user_id,
            login_password_login: Some(login),
            login_password_password: Some(password),
            ssh_key: None,
            ssh_passphrase: None,
            access_key_access_key: None,
            access_key_secret_key: None,
            secret_storage_id: None,
            source_storage_type: Some(AccessKeySourceStorageType::DB),
            source_storage_id: None,
            source_key: None,
            owner: None,
            environment_id: None,
            created: None,
        }
    }

    /// Получает данные SSH ключа
    pub fn get_ssh_key_data(&self) -> Option<SshKeyData> {
        self.ssh_key.as_ref().map(|key| SshKeyData {
            private_key: key.clone(),
            passphrase: self.ssh_passphrase.clone(),
            login: self.login_password_login.clone().unwrap_or_default(),
        })
    }

    /// Получает данные логина/пароля
    pub fn get_login_password_data(&self) -> Option<LoginPasswordData> {
        match (&self.login_password_login, &self.login_password_password) {
            (Some(login), Some(password)) => Some(LoginPasswordData {
                login: login.clone(),
                password: password.clone(),
            }),
            _ => None,
        }
    }

    /// Получает тип ключа (алиас для r#type)
    pub fn get_type(&self) -> &AccessKeyType {
        &self.r#type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!("unknown".parse::<AccessKeyType>().unwrap(), AccessKeyType::None);
    }

    #[test]
    fn test_access_key_owner_display() {
        assert_eq!(AccessKeyOwner::User.to_string(), "user");
        assert_eq!(AccessKeyOwner::Project.to_string(), "project");
        assert_eq!(AccessKeyOwner::Shared.to_string(), "shared");
    }

    #[test]
    fn test_access_key_new() {
        let key = AccessKey::new("My Key".to_string(), AccessKeyType::SSH);
        assert_eq!(key.id, 0);
        assert_eq!(key.name, "My Key");
        assert_eq!(key.r#type, AccessKeyType::SSH);
        assert!(key.project_id.is_none());
    }

    #[test]
    fn test_access_key_new_ssh() {
        let key = AccessKey::new_ssh(
            10,
            "SSH Key".to_string(),
            "-----BEGIN RSA PRIVATE KEY-----".to_string(),
            "".to_string(),
            "deploy".to_string(),
            Some(1),
        );
        assert_eq!(key.project_id, Some(10));
        assert_eq!(key.r#type, AccessKeyType::SSH);
        assert_eq!(key.login_password_login, Some("deploy".to_string()));
        assert!(key.ssh_key.is_some());
    }

    #[test]
    fn test_access_key_new_login_password() {
        let key = AccessKey::new_login_password(
            5,
            "Login Key".to_string(),
            "admin".to_string(),
            "secret".to_string(),
            None,
        );
        assert_eq!(key.project_id, Some(5));
        assert_eq!(key.r#type, AccessKeyType::LoginPassword);
        assert_eq!(key.login_password_login, Some("admin".to_string()));
    }

    #[test]
    fn test_access_key_serialization() {
        let key = AccessKey::new("Test".to_string(), AccessKeyType::SSH);
        let json = serde_json::to_string(&key).unwrap();
        assert!(json.contains("\"name\":\"Test\""));
        assert!(json.contains("\"type\":\"ssh\""));
    }

    #[test]
    fn test_access_key_serialization_skip_nulls() {
        let key = AccessKey::new("Minimal".to_string(), AccessKeyType::None);
        let json = serde_json::to_string(&key).unwrap();
        assert!(!json.contains("\"project_id\":"));
        assert!(!json.contains("\"ssh_key\":"));
    }

    #[test]
    fn test_get_ssh_key_data() {
        let key = AccessKey::new_ssh(
            1, "key".to_string(), "private_key".to_string(), "pass".to_string(), "user".to_string(), None,
        );
        let data = key.get_ssh_key_data().unwrap();
        assert_eq!(data.private_key, "private_key");
        assert_eq!(data.passphrase, Some("pass".to_string()));
        assert_eq!(data.login, "user".to_string());
    }

    #[test]
    fn test_get_login_password_data() {
        let key = AccessKey::new_login_password(
            1, "key".to_string(), "admin".to_string(), "secret".to_string(), None,
        );
        let data = key.get_login_password_data().unwrap();
        assert_eq!(data.login, "admin");
        assert_eq!(data.password, "secret");
    }

    #[test]
    fn test_get_type() {
        let key = AccessKey::new("key".to_string(), AccessKeyType::SSH);
        assert_eq!(key.get_type(), &AccessKeyType::SSH);
    }

    #[test]
    fn test_access_key_type_serialize_all_variants() {
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
    fn test_access_key_type_deserialize_all_variants() {
        assert_eq!(
            serde_json::from_str::<AccessKeyType>("\"none\"").unwrap(),
            AccessKeyType::None
        );
        assert_eq!(
            serde_json::from_str::<AccessKeyType>("\"login_password\"").unwrap(),
            AccessKeyType::LoginPassword
        );
        assert_eq!(
            serde_json::from_str::<AccessKeyType>("\"ssh\"").unwrap(),
            AccessKeyType::SSH
        );
        assert_eq!(
            serde_json::from_str::<AccessKeyType>("\"access_key\"").unwrap(),
            AccessKeyType::AccessKey
        );
    }

    #[test]
    fn test_access_key_clone() {
        let key = AccessKey::new("clone-key".to_string(), AccessKeyType::LoginPassword);
        let cloned = key.clone();
        assert_eq!(cloned.name, key.name);
        assert_eq!(cloned.r#type, key.r#type);
    }

    #[test]
    fn test_access_key_owner_serialize() {
        assert_eq!(
            serde_json::to_string(&AccessKeyOwner::User).unwrap(),
            "\"user\""
        );
        assert_eq!(
            serde_json::to_string(&AccessKeyOwner::Project).unwrap(),
            "\"project\""
        );
        assert_eq!(
            serde_json::to_string(&AccessKeyOwner::Shared).unwrap(),
            "\"shared\""
        );
    }

    #[test]
    fn test_access_key_owner_from_str() {
        assert_eq!("user".parse::<AccessKeyOwner>().unwrap(), AccessKeyOwner::User);
        assert_eq!("project".parse::<AccessKeyOwner>().unwrap(), AccessKeyOwner::Project);
        assert_eq!("shared".parse::<AccessKeyOwner>().unwrap(), AccessKeyOwner::Shared);
        assert_eq!("unknown".parse::<AccessKeyOwner>().unwrap(), AccessKeyOwner::Shared);
    }

    #[test]
    fn test_ssh_key_data_clone() {
        let data = SshKeyData {
            private_key: "key".to_string(),
            passphrase: Some("pass".to_string()),
            login: "user".to_string(),
        };
        let cloned = data.clone();
        assert_eq!(cloned.private_key, data.private_key);
        assert_eq!(cloned.passphrase, data.passphrase);
    }

    #[test]
    fn test_login_password_data_clone() {
        let data = LoginPasswordData {
            login: "admin".to_string(),
            password: "secret".to_string(),
        };
        let cloned = data.clone();
        assert_eq!(cloned.login, data.login);
        assert_eq!(cloned.password, data.password);
    }
}
