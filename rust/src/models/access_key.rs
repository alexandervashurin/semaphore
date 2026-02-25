//! Модель ключа доступа (AccessKey)

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Тип ключа доступа
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AccessKeyType {
    None,
    LoginPassword,
    SSH,
    AccessKey,
}

/// Владелец ключа
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AccessKeyOwner {
    User,
    Project,
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
        }
    }
}
