//! Модель репозитория

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type, decode::Decode, encode::Encode, database::Database};

/// Тип репозитория
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryType {
    Git,
    Http,
    Https,
    File,
}

impl<DB: Database> Type<DB> for RepositoryType
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

impl<'r, DB: Database> Decode<'r, DB> for RepositoryType
where
    String: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as Decode<'r, DB>>::decode(value)?;
        Ok(match s.as_str() {
            "git" => RepositoryType::Git,
            "http" => RepositoryType::Http,
            "https" => RepositoryType::Https,
            "file" => RepositoryType::File,
            _ => RepositoryType::Git,
        })
    }
}

impl<'q, DB: Database> Encode<'q, DB> for RepositoryType
where
    DB: 'q,
    String: Encode<'q, DB>,
{
    fn encode_by_ref(&self, buf: &mut <DB as Database>::ArgumentBuffer<'q>) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            RepositoryType::Git => "git",
            RepositoryType::Http => "http",
            RepositoryType::Https => "https",
            RepositoryType::File => "file",
        }.to_string();
        <String as Encode<'q, DB>>::encode(s, buf)
    }
}

/// Репозиторий - хранилище кода (Git, HTTP, файл)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Repository {
    /// Уникальный идентификатор
    pub id: i32,

    /// ID проекта
    pub project_id: i32,

    /// Название репозитория
    pub name: String,

    /// URL репозитория
    pub git_url: String,

    /// Тип репозитория
    pub git_type: RepositoryType,

    /// Ветка Git
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,

    /// ID ключа доступа
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_id: Option<i32>,

    /// Путь к файлу (для file-типа)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_path: Option<String>,

    /// Дата создания
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<Utc>>,
}

impl Repository {
    /// Создаёт новый репозиторий
    pub fn new(project_id: i32, name: String, git_url: String) -> Self {
        Self {
            id: 0,
            project_id,
            name,
            git_url,
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        }
    }

    /// Получает URL для клонирования
    pub fn get_clone_url(&self) -> &str {
        &self.git_url
    }

    /// Получает полный путь к репозиторию
    pub fn get_full_path(&self) -> String {
        self.git_path.clone().unwrap_or_else(|| self.git_url.clone())
    }
}

impl Default for Repository {
    fn default() -> Self {
        Self {
            id: 0,
            project_id: 0,
            name: String::new(),
            git_url: String::new(),
            git_type: RepositoryType::Git,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        }
    }
}
