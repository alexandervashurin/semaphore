//! Модель ключа доступа (AccessKey)

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type, decode::Decode, encode::Encode, database::Database};

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

impl<DB: Database> Type<DB> for AccessKeyType {
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
    fn encode_by_ref(&self, buf: &mut <DB as Database>::ArgumentBuffer<'q>) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            AccessKeyType::None => "none",
            AccessKeyType::LoginPassword => "login_password",
            AccessKeyType::SSH => "ssh",
            AccessKeyType::AccessKey => "access_key",
        }.to_string();
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

impl<DB: Database> Type<DB> for AccessKeyOwner {
    fn type_info() -> DB::TypeInfo {
        <&str as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <&str as Type<DB>>::compatible(ty)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for AccessKeyOwner
where
    &'r str: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as Decode<'r, DB>>::decode(value)?;
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
    fn encode_by_ref(&self, buf: &mut <DB as Database>::ArgumentBuffer<'q>) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            AccessKeyOwner::User => "user",
            AccessKeyOwner::Project => "project",
            AccessKeyOwner::Shared => "shared",
        };
        <&str as Encode<'q, DB>>::encode(&s, buf)
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

    /// Владелец ключа
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<AccessKeyOwner>,

    /// ID окружения
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_id: Option<i32>,
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
            owner: None,
            environment_id: None,
        }
    }
}
