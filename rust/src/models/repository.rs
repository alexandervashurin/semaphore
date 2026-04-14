//! Модель репозитория

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type, database::Database, decode::Decode, encode::Encode};

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
    fn encode_by_ref(
        &self,
        buf: &mut <DB as Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            RepositoryType::Git => "git",
            RepositoryType::Http => "http",
            RepositoryType::Https => "https",
            RepositoryType::File => "file",
        }
        .to_string();
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
        self.git_path
            .clone()
            .unwrap_or_else(|| self.git_url.clone())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_type_serialization() {
        assert_eq!(
            serde_json::to_string(&RepositoryType::Git).unwrap(),
            "\"git\""
        );
        assert_eq!(
            serde_json::to_string(&RepositoryType::Http).unwrap(),
            "\"http\""
        );
        assert_eq!(
            serde_json::to_string(&RepositoryType::Https).unwrap(),
            "\"https\""
        );
        assert_eq!(
            serde_json::to_string(&RepositoryType::File).unwrap(),
            "\"file\""
        );
    }

    #[test]
    fn test_repository_new() {
        let repo = Repository::new(
            10,
            "my-repo".to_string(),
            "git@github.com:user/repo.git".to_string(),
        );
        assert_eq!(repo.id, 0);
        assert_eq!(repo.project_id, 10);
        assert_eq!(repo.name, "my-repo");
        assert_eq!(repo.git_type, RepositoryType::Git);
        assert!(repo.git_branch.is_none());
    }

    #[test]
    fn test_repository_default() {
        let repo = Repository::default();
        assert_eq!(repo.id, 0);
        assert!(repo.name.is_empty());
        assert!(repo.git_url.is_empty());
        assert_eq!(repo.git_type, RepositoryType::Git);
    }

    #[test]
    fn test_repository_serialization() {
        let repo = Repository {
            id: 1,
            project_id: 5,
            name: "deploy-repo".to_string(),
            git_url: "git@github.com:org/deploy.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: Some("main".to_string()),
            key_id: Some(3),
            git_path: None,
            created: Some(Utc::now()),
        };
        let json = serde_json::to_string(&repo).unwrap();
        assert!(json.contains("\"name\":\"deploy-repo\""));
        assert!(json.contains("\"git_branch\":\"main\""));
        assert!(json.contains("\"key_id\":3"));
    }

    #[test]
    fn test_repository_skip_nulls() {
        let repo = Repository {
            id: 1,
            project_id: 5,
            name: "simple".to_string(),
            git_url: "https://example.com/repo.git".to_string(),
            git_type: RepositoryType::Https,
            git_branch: None,
            key_id: None,
            git_path: None,
            created: None,
        };
        let json = serde_json::to_string(&repo).unwrap();
        assert!(!json.contains("\"git_branch\":"));
        assert!(!json.contains("\"key_id\":"));
        assert!(!json.contains("\"git_path\":"));
    }

    #[test]
    fn test_repository_get_clone_url() {
        let repo = Repository::new(
            1,
            "repo".to_string(),
            "https://github.com/user/repo.git".to_string(),
        );
        assert_eq!(repo.get_clone_url(), "https://github.com/user/repo.git");
    }

    #[test]
    fn test_repository_get_full_path() {
        // Without git_path
        let repo = Repository::new(
            1,
            "repo".to_string(),
            "https://example.com/repo.git".to_string(),
        );
        assert_eq!(repo.get_full_path(), "https://example.com/repo.git");

        // With git_path
        let mut repo2 = repo.clone();
        repo2.git_path = Some("/path/to/repo".to_string());
        assert_eq!(repo2.get_full_path(), "/path/to/repo");
    }

    #[test]
    fn test_repository_clone() {
        let repo = Repository::new(
            1,
            "clone-repo".to_string(),
            "https://github.com/user/repo.git".to_string(),
        );
        let cloned = repo.clone();
        assert_eq!(cloned.name, repo.name);
        assert_eq!(cloned.git_url, repo.git_url);
        assert_eq!(cloned.git_type, repo.git_type);
    }

    #[test]
    fn test_repository_with_file_type() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "local-repo".to_string(),
            git_url: String::new(),
            git_type: RepositoryType::File,
            git_branch: None,
            key_id: None,
            git_path: Some("/var/repo".to_string()),
            created: None,
        };
        assert_eq!(repo.git_type, RepositoryType::File);
        assert_eq!(repo.git_path, Some("/var/repo".to_string()));
    }

    #[test]
    fn test_repository_with_http_type() {
        let repo = Repository::new(
            1,
            "http-repo".to_string(),
            "http://example.com/repo.git".to_string(),
        );
        assert_eq!(repo.git_type, RepositoryType::Git); // new() always uses Git type
        assert_eq!(repo.git_url, "http://example.com/repo.git");
    }

    #[test]
    fn test_repository_get_clone_url_with_branch() {
        let mut repo = Repository::new(
            1,
            "repo".to_string(),
            "https://github.com/user/repo.git".to_string(),
        );
        repo.git_branch = Some("develop".to_string());
        // get_clone_url returns git_url regardless of branch
        assert_eq!(repo.get_clone_url(), "https://github.com/user/repo.git");
    }

    #[test]
    fn test_repository_serialization_with_all_types() {
        let types = [
            RepositoryType::Git,
            RepositoryType::Http,
            RepositoryType::Https,
            RepositoryType::File,
        ];
        for repo_type in &types {
            let repo = Repository {
                id: 1,
                project_id: 1,
                name: "test".to_string(),
                git_url: "https://example.com/repo.git".to_string(),
                git_type: repo_type.clone(),
                git_branch: None,
                key_id: None,
                git_path: None,
                created: None,
            };
            let json = serde_json::to_string(&repo).unwrap();
            assert!(json.contains("\"git_type\":"));
        }
    }

    #[test]
    fn test_repository_decode_git_type() {
        // Test decoding RepositoryType from JSON values
        let git: RepositoryType = serde_json::from_value(serde_json::json!("git")).unwrap();
        assert_eq!(git, RepositoryType::Git);

        let http: RepositoryType = serde_json::from_value(serde_json::json!("http")).unwrap();
        assert_eq!(http, RepositoryType::Http);

        let https: RepositoryType = serde_json::from_value(serde_json::json!("https")).unwrap();
        assert_eq!(https, RepositoryType::Https);

        let file: RepositoryType = serde_json::from_value(serde_json::json!("file")).unwrap();
        assert_eq!(file, RepositoryType::File);
    }

    #[test]
    fn test_repository_clone_equality() {
        let repo1 = Repository::new(
            1,
            "repo1".to_string(),
            "https://github.com/user/repo1.git".to_string(),
        );
        let repo2 = repo1.clone();
        // Cloned repositories should have identical field values
        assert_eq!(repo1.id, repo2.id);
        assert_eq!(repo1.project_id, repo2.project_id);
        assert_eq!(repo1.name, repo2.name);
        assert_eq!(repo1.git_url, repo2.git_url);
    }

    #[test]
    fn test_repository_unicode_name_and_url() {
        let repo = Repository {
            id: 1,
            project_id: 1,
            name: "Репозиторий".to_string(),
            git_url: "https://github.com/user/repo.git".to_string(),
            git_type: RepositoryType::Git,
            git_branch: Some("main".to_string()),
            key_id: None,
            git_path: None,
            created: None,
        };
        let json = serde_json::to_string(&repo).unwrap();
        let restored: Repository = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, "Репозиторий");
    }
}
