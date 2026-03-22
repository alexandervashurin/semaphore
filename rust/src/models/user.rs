//! Модель пользователя

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Пользователь системы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Уникальный идентификатор
    pub id: i32,

    /// Дата создания
    pub created: DateTime<Utc>,

    /// Имя пользователя (логин)
    pub username: String,

    /// Полное имя
    pub name: String,

    /// Электронная почта
    pub email: String,

    /// Хэш пароля
    #[serde(skip_serializing)]
    pub password: String,

    /// Является ли администратором
    pub admin: bool,

    /// Внешний пользователь (из LDAP/OIDC)
    pub external: bool,

    /// Получать уведомления
    pub alert: bool,

    /// Pro-пользователь
    pub pro: bool,

    /// Двухфакторная аутентификация TOTP
    #[serde(skip_serializing, skip_deserializing)]
    pub totp: Option<UserTotp>,

    /// OTP по электронной почте
    #[serde(skip_serializing, skip_deserializing)]
    pub email_otp: Option<UserEmailOtp>,
}

/// TOTP-конфигурация пользователя
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserTotp {
    pub id: i32,
    pub created: DateTime<Utc>,
    pub user_id: i32,
    pub url: String,
    #[serde(skip_serializing)]
    pub recovery_hash: String,
    #[serde(skip_serializing)]
    pub recovery_code: Option<String>,
}

/// OTP по электронной почте
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserEmailOtp {
    pub id: i32,
    pub created: DateTime<Utc>,
    pub user_id: i32,
    pub code: String,
}

/// Пользователь с ролью в проекте
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWithProjectRole {
    #[serde(flatten)]
    pub user: User,
    pub role: ProjectUserRole,
}

/// Роль пользователя в проекте
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectUserRole {
    Owner,
    Manager,
    TaskRunner,
    Guest,
    None,
}

impl std::fmt::Display for ProjectUserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectUserRole::Owner => write!(f, "owner"),
            ProjectUserRole::Manager => write!(f, "manager"),
            ProjectUserRole::TaskRunner => write!(f, "task_runner"),
            ProjectUserRole::Guest => write!(f, "guest"),
            ProjectUserRole::None => write!(f, "none"),
        }
    }
}

// PostgreSQL поддержка
impl sqlx::Type<sqlx::Postgres> for ProjectUserRole {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ProjectUserRole {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
        Ok(match s.as_str() {
            "owner" => ProjectUserRole::Owner,
            "manager" => ProjectUserRole::Manager,
            "task_runner" => ProjectUserRole::TaskRunner,
            "guest" => ProjectUserRole::Guest,
            "none" => ProjectUserRole::None,
            _ => ProjectUserRole::None,
        })
    }
}


/// Пользователь с паролем (для создания/обновления)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWithPwd {
    #[serde(skip_serializing)]
    pub pwd: String,
    #[serde(flatten)]
    pub user: User,
}

impl User {
    /// Проверяет валидность пользователя
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.username.is_empty() {
            return Err(ValidationError::UsernameEmpty);
        }
        if self.email.is_empty() {
            return Err(ValidationError::EmailEmpty);
        }
        if self.name.is_empty() {
            return Err(ValidationError::NameEmpty);
        }
        Ok(())
    }
}

impl UserEmailOtp {
    /// Проверяет, истёк ли срок действия OTP
    /// OTP действителен в течение 10 минут
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let expires_at = self.created + chrono::Duration::minutes(10);
        now > expires_at
    }
}

/// Ошибка валидации
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Имя пользователя не может быть пустым")]
    UsernameEmpty,
    #[error("Электронная почта не может быть пустой")]
    EmailEmpty,
    #[error("Имя не может быть пустым")]
    NameEmpty,
}
