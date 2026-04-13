//! Сервис синхронизации Playbook из Git Repository
//!
//! Этот модуль предоставляет функциональность для загрузки и синхронизации
//! playbook файлов из Git репозиториев.

use crate::db::store::{AccessKeyManager, PlaybookManager, RepositoryManager};
use crate::error::{Error, Result};
use crate::models::playbook::{Playbook, PlaybookUpdate};
use crate::services::ssh_auth_service::SshAuthService;
use git2::{build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks, Repository};
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tracing::{info, warn};

/// Сервис для синхронизации playbook из Git
pub struct PlaybookSyncService;

impl PlaybookSyncService {
    /// Синхронизирует playbook из связанного Git репозитория
    ///
    /// # Arguments
    /// * `playbook_id` - ID playbook для синхронизации
    /// * `project_id` - ID проекта
    /// * `store` - Хранилище данных
    ///
    /// # Returns
    /// * `Result<Playbook>` - Обновленный playbook
    pub async fn sync_from_repository<S>(
        playbook_id: i32,
        project_id: i32,
        store: &S,
    ) -> Result<Playbook>
    where
        S: PlaybookManager + RepositoryManager + AccessKeyManager,
    {
        // 1. Получаем playbook
        let playbook = store.get_playbook(playbook_id, project_id).await?;

        // 2. Проверяем наличие repository_id
        let repository_id = playbook.repository_id.ok_or_else(|| {
            Error::Validation("Playbook не связан с Git репозиторием".to_string())
        })?;

        // 3. Получаем repository
        let repository = store.get_repository(project_id, repository_id).await?;

        // 4. Клонируем репозиторий во временную директорию
        let temp_dir = TempDir::new()
            .map_err(|e| Error::Other(format!("Не удалось создать временную директорию: {}", e)))?;

        info!(
            "Клонирование репозитория {} в {:?}",
            repository.git_url,
            temp_dir.path()
        );

        clone_repository(&repository, temp_dir.path(), project_id, store).await?;

        // 5. Читаем playbook файл
        // Путь к файлу берем из названия playbook или используем название как путь
        let playbook_file_path = determine_playbook_path(temp_dir.path(), &playbook.name);

        let content = std::fs::read_to_string(&playbook_file_path).map_err(|e| {
            Error::NotFound(format!(
                "Файл playbook не найден по пути {:?}: {}",
                playbook_file_path, e
            ))
        })?;

        // 6. Обновляем playbook в БД
        let updated_playbook = store
            .update_playbook(
                playbook_id,
                project_id,
                PlaybookUpdate {
                    name: playbook.name.clone(),
                    content,
                    description: playbook.description.clone(),
                    playbook_type: playbook.playbook_type.clone(),
                },
            )
            .await?;

        info!("Playbook {} успешно синхронизирован из Git", playbook.name);

        Ok(updated_playbook)
    }

    /// Предварительный просмотр содержимого playbook из Git
    ///
    /// # Arguments
    /// * `playbook_id` - ID playbook
    /// * `project_id` - ID проекта
    /// * `store` - Хранилище данных
    ///
    /// # Returns
    /// * `Result<String>` - Содержимое файла без сохранения в БД
    pub async fn preview_from_repository<S>(
        playbook_id: i32,
        project_id: i32,
        store: &S,
    ) -> Result<String>
    where
        S: PlaybookManager + RepositoryManager + AccessKeyManager,
    {
        // 1. Получаем playbook
        let playbook = store.get_playbook(playbook_id, project_id).await?;

        // 2. Проверяем наличие repository_id
        let repository_id = playbook.repository_id.ok_or_else(|| {
            Error::Validation("Playbook не связан с Git репозиторием".to_string())
        })?;

        // 3. Получаем repository
        let repository = store.get_repository(project_id, repository_id).await?;

        // 4. Клонируем репозиторий во временную директорию
        let temp_dir = TempDir::new()
            .map_err(|e| Error::Other(format!("Не удалось создать временную директорию: {}", e)))?;

        clone_repository(&repository, temp_dir.path(), project_id, store).await?;

        // 5. Читаем playbook файл
        let playbook_file_path = determine_playbook_path(temp_dir.path(), &playbook.name);

        let content = std::fs::read_to_string(&playbook_file_path).map_err(|e| {
            Error::NotFound(format!(
                "Файл playbook не найден по пути {:?}: {}",
                playbook_file_path, e
            ))
        })?;

        Ok(content)
    }
}

/// Клонирует Git репозиторий в указанную директорию
async fn clone_repository<S>(
    repository: &crate::models::Repository,
    path: &Path,
    project_id: i32,
    store: &S,
) -> Result<()>
where
    S: AccessKeyManager,
{
    // Загружаем данные ключа async перед входом в spawn_blocking
    let (ssh_key, ssh_passphrase, login, password) = if let Some(key_id) = repository.key_id {
        match store.get_access_key(project_id, key_id).await {
            Ok(key) => (
                key.ssh_key,
                key.ssh_passphrase,
                key.login_password_login,
                key.login_password_password,
            ),
            Err(e) => {
                warn!(
                    "Failed to load access key {:?} for repository: {}",
                    repository.key_id, e
                );
                (None, None, None, None)
            }
        }
    } else {
        (None, None, None, None)
    };

    let git_url = repository.git_url.clone();
    let path = path.to_path_buf();

    // Используем spawn_blocking т.к. git2 не Send
    tokio::task::spawn_blocking(move || {
        let mut fetch_options = FetchOptions::new();
        let mut remote_callbacks = RemoteCallbacks::new();

        remote_callbacks.credentials(move |_url, username_from_url, allowed_types| {
            if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                if let Some(ref key_pem) = ssh_key {
                    return Cred::ssh_key_from_memory(
                        username_from_url.unwrap_or("git"),
                        None,
                        key_pem,
                        ssh_passphrase.as_deref(),
                    );
                }
                Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
            } else if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
                Cred::userpass_plaintext(
                    login.as_deref().unwrap_or(""),
                    password.as_deref().unwrap_or(""),
                )
            } else {
                Cred::default()
            }
        });

        fetch_options.remote_callbacks(remote_callbacks);

        let mut builder = RepoBuilder::new();
        builder.fetch_options(fetch_options);

        builder.clone(&git_url, &path).map_err(Error::Git)?;
        Ok(())
    })
    .await
    .map_err(|e| Error::Other(format!("spawn_blocking error: {}", e)))?
}

/// Определяет путь к файлу playbook
fn determine_playbook_path(repo_path: &Path, playbook_name: &str) -> PathBuf {
    let possible_paths = vec![
        repo_path.join(playbook_name),
        repo_path.join(format!("{}.yml", playbook_name)),
        repo_path.join(format!("{}.yaml", playbook_name)),
        repo_path.join("playbooks").join(playbook_name),
        repo_path
            .join("playbooks")
            .join(format!("{}.yml", playbook_name)),
    ];

    for path in &possible_paths {
        if path.exists() {
            return path.clone();
        }
    }

    possible_paths[0].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_playbook_path() {
        let temp_dir = TempDir::new().unwrap();

        std::fs::write(temp_dir.path().join("deploy.yml"), "---").unwrap();

        let playbooks_dir = temp_dir.path().join("playbooks");
        std::fs::create_dir_all(&playbooks_dir).unwrap();
        std::fs::write(playbooks_dir.join("site.yaml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "deploy.yml");
        assert!(path.exists());

        let path = determine_playbook_path(temp_dir.path(), "deploy");
        assert!(path.exists());

        let path = determine_playbook_path(temp_dir.path(), "playbooks/site.yaml");
        assert!(path.exists());
    }

    #[test]
    fn test_determine_playbook_path_yml_extension() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("deploy.yml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "deploy");
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with("deploy.yml"));
    }

    #[test]
    fn test_determine_playbook_path_yaml_extension() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("deploy.yaml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "deploy");
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with("deploy.yaml"));
    }

    #[test]
    fn test_determine_playbook_path_in_playbooks_dir() {
        let temp_dir = TempDir::new().unwrap();
        let playbooks_dir = temp_dir.path().join("playbooks");
        std::fs::create_dir_all(&playbooks_dir).unwrap();
        std::fs::write(playbooks_dir.join("site.yaml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "site.yaml");
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("playbooks"));
        assert!(path.to_string_lossy().ends_with("site.yaml"));
    }

    #[test]
    fn test_determine_playbook_path_exact_name() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("my_playbook.yml"), "---").unwrap();

        // Точное имя файла должно находиться первым
        let path = determine_playbook_path(temp_dir.path(), "my_playbook.yml");
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with("my_playbook.yml"));
    }

    #[test]
    fn test_determine_playbook_path_fallback_to_first() {
        let temp_dir = TempDir::new().unwrap();
        // Ни один файл не существует — должен вернуть первый путь (не существует)
        let path = determine_playbook_path(temp_dir.path(), "nonexistent");
        assert!(!path.exists());
        assert!(path.to_string_lossy().ends_with("nonexistent"));
    }

    #[test]
    fn test_determine_playbook_path_nested_playbooks_dir() {
        let temp_dir = TempDir::new().unwrap();
        let playbooks_dir = temp_dir.path().join("playbooks");
        std::fs::create_dir_all(&playbooks_dir).unwrap();
        std::fs::write(playbooks_dir.join("main.yml"), "---").unwrap();

        // Ищем playbook в поддиректории playbooks
        let path = determine_playbook_path(temp_dir.path(), "main.yml");
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("playbooks"));
    }

    #[test]
    fn test_determine_playbook_path_prefers_yml_over_yaml() {
        let temp_dir = TempDir::new().unwrap();
        // Создаём оба файла — должен найти .yml первым
        std::fs::write(temp_dir.path().join("deploy.yml"), "---").unwrap();
        std::fs::write(temp_dir.path().join("deploy.yaml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "deploy");
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with("deploy.yml"));
    }

    #[test]
    fn test_determine_playbook_path_with_full_path_in_name() {
        let temp_dir = TempDir::new().unwrap();
        let playbooks_dir = temp_dir.path().join("playbooks");
        std::fs::create_dir_all(&playbooks_dir).unwrap();
        std::fs::write(playbooks_dir.join("site.yaml"), "---").unwrap();

        // Когда указываем полный путь — ищем в playbooks/site.yaml
        let path = determine_playbook_path(temp_dir.path(), "playbooks/site.yaml");
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with("playbooks/site.yaml"));
    }

    #[test]
    fn test_determine_playbook_path_empty_name() {
        let temp_dir = TempDir::new().unwrap();
        let path = determine_playbook_path(temp_dir.path(), "");
        // Ожидаем: первый путь -- просто корень temp_dir
        assert_eq!(path, temp_dir.path().to_path_buf());
    }

    #[test]
    fn test_determine_playbook_path_missing_playbooks_dir() {
        let temp_dir = TempDir::new().unwrap();
        // Директория playbooks НЕ создана
        let path = determine_playbook_path(temp_dir.path(), "deploy");
        // Должен вернуть первый существующий путь (repo_path/deploy)
        assert!(!path.exists());
        assert_eq!(path, temp_dir.path().join("deploy"));
    }

    #[test]
    fn test_determine_playbook_path_deeply_nested_name() {
        let temp_dir = TempDir::new().unwrap();
        let nested = temp_dir.path().join("group1/group2");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("task.yaml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "group1/group2/task.yaml");
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("group1/group2/task.yaml"));
    }

    #[test]
    fn test_determine_playbook_path_yaml_in_playbooks_dir() {
        let temp_dir = TempDir::new().unwrap();
        let playbooks_dir = temp_dir.path().join("playbooks");
        std::fs::create_dir_all(&playbooks_dir).unwrap();
        std::fs::write(playbooks_dir.join("setup.yml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "setup");
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("playbooks"));
        assert!(path.to_string_lossy().ends_with("setup.yml"));
    }

    #[test]
    fn test_determine_playbook_path_empty_name_additional() {
        let temp_dir = TempDir::new().unwrap();
        let path = determine_playbook_path(temp_dir.path(), "");
        assert_eq!(path, temp_dir.path());
    }

    #[test]
    fn test_determine_playbook_path_falls_back_to_root_additional() {
        let temp_dir = TempDir::new().unwrap();
        let path = determine_playbook_path(temp_dir.path(), "nonexistent.yml");
        assert_eq!(path, temp_dir.path().join("nonexistent.yml"));
    }

    #[test]
    fn test_playbook_update_clone() {
        let update = PlaybookUpdate {
            name: "Test".to_string(),
            content: "---".to_string(),
            description: Some("Desc".to_string()),
            playbook_type: "ansible".to_string(),
        };
        let cloned = update.clone();
        assert_eq!(cloned.name, update.name);
        assert_eq!(cloned.content, update.content);
    }

    #[test]
    fn test_determine_playbook_path_prefers_exact_match_over_extensions() {
        let temp_dir = TempDir::new().unwrap();
        // Создаём playbook без расширения и с .yml
        std::fs::write(temp_dir.path().join("setup"), "#no ext").unwrap();
        std::fs::write(temp_dir.path().join("setup.yml"), "#yml").unwrap();

        // Точное совпадение должно быть первым
        let path = determine_playbook_path(temp_dir.path(), "setup");
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with("setup"));
    }

    #[test]
    fn test_determine_playbook_path_yaml_in_root_before_playbooks() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("deploy.yaml"), "#root").unwrap();

        let playbooks_dir = temp_dir.path().join("playbooks");
        std::fs::create_dir_all(&playbooks_dir).unwrap();
        std::fs::write(playbooks_dir.join("deploy.yaml"), "#playbooks").unwrap();

        // Должен найти root версию первым
        let path = determine_playbook_path(temp_dir.path(), "deploy.yaml");
        assert!(path.exists());
        assert!(!path.to_string_lossy().contains("playbooks"));
    }

    #[test]
    fn test_determine_playbook_path_returns_first_path_when_none_exist() {
        let temp_dir = TempDir::new().unwrap();
        let path = determine_playbook_path(temp_dir.path(), "missing.yml");
        assert!(!path.exists());
        assert_eq!(path, temp_dir.path().join("missing.yml"));
    }

    #[test]
    fn test_playbook_update_debug_format() {
        let update = PlaybookUpdate {
            name: "Debug Test".to_string(),
            content: "content".to_string(),
            description: Some("desc".to_string()),
            playbook_type: "terraform".to_string(),
        };
        let debug_str = format!("{:?}", update);
        assert!(debug_str.contains("PlaybookUpdate"));
        assert!(debug_str.contains("Debug Test"));
    }

    #[test]
    fn test_determine_playbook_path_with_yaml_in_playbooks_and_yml_in_root() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("site.yml"), "#root yml").unwrap();

        let playbooks_dir = temp_dir.path().join("playbooks");
        std::fs::create_dir_all(&playbooks_dir).unwrap();
        std::fs::write(playbooks_dir.join("site.yaml"), "#playbooks yaml").unwrap();

        // Ищем site.yaml -- должен найти playbooks/site.yaml
        let path = determine_playbook_path(temp_dir.path(), "site.yaml");
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("playbooks"));
    }

    #[test]
    fn test_determine_playbook_path_long_name_with_dashes() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("deploy-to-production.yml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "deploy-to-production");
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with("deploy-to-production.yml"));
    }

    #[test]
    fn test_playbook_update_with_none_description() {
        let update = PlaybookUpdate {
            name: "Test".to_string(),
            content: "---".to_string(),
            description: None,
            playbook_type: "ansible".to_string(),
        };
        assert!(update.description.is_none());
        assert_eq!(update.name, "Test");
    }

    #[test]
    fn test_playbook_update_equality() {
        let u1 = PlaybookUpdate {
            name: "A".to_string(),
            content: "---".to_string(),
            description: Some("d".to_string()),
            playbook_type: "ansible".to_string(),
        };
        let u2 = PlaybookUpdate {
            name: "A".to_string(),
            content: "---".to_string(),
            description: Some("d".to_string()),
            playbook_type: "ansible".to_string(),
        };
        assert_eq!(u1.name, u2.name);
        assert_eq!(u1.content, u2.content);
        assert_eq!(u1.description, u2.description);
        assert_eq!(u1.playbook_type, u2.playbook_type);
    }

    #[test]
    fn test_determine_playbook_path_with_underscore_name() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("my_playbook.yml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "my_playbook");
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with("my_playbook.yml"));
    }

    #[test]
    fn test_determine_playbook_path_numbered_name() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("playbook2.yml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "playbook2");
        assert!(path.exists());
    }

    #[test]
    fn test_playbook_service_struct_exists() {
        // Just verify the struct can be referenced
        let _ = std::mem::size_of::<PlaybookSyncService>();
    }

    #[test]
    fn test_determine_playbook_path_with_dotted_directory() {
        let temp_dir = TempDir::new().unwrap();
        let playbooks_dir = temp_dir.path().join("playbooks");
        std::fs::create_dir_all(&playbooks_dir).unwrap();
        std::fs::write(playbooks_dir.join("main.yaml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "main.yaml");
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("playbooks/main.yaml"));
    }

    #[test]
    fn test_determine_playbook_path_root_yml_first() {
        let temp_dir = TempDir::new().unwrap();
        // Создаём только .yml в корне
        std::fs::write(temp_dir.path().join("deploy.yml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "deploy");
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with("deploy.yml"));
    }

    #[test]
    fn test_playbook_update_clone_with_empty_content() {
        let update = PlaybookUpdate {
            name: "".to_string(),
            content: "".to_string(),
            description: None,
            playbook_type: "".to_string(),
        };
        let cloned = update.clone();
        assert_eq!(cloned.name, "");
        assert_eq!(cloned.content, "");
        assert!(cloned.description.is_none());
    }

    #[test]
    fn test_determine_playbook_path_with_capital_extension() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("Deploy.YML"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "Deploy.YML");
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with("Deploy.YML"));
    }

    #[test]
    fn test_determine_playbook_path_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("deploy.yml"), "---").unwrap();
        std::fs::write(temp_dir.path().join("setup.yaml"), "---").unwrap();
        std::fs::write(temp_dir.path().join("config.yml"), "---").unwrap();

        // Ищем deploy -- должен найти deploy.yml
        let path = determine_playbook_path(temp_dir.path(), "deploy");
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with("deploy.yml"));
    }

    #[test]
    fn test_determine_playbook_path_exact_yml_match() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("my_playbook.yml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "my_playbook.yml");
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with("my_playbook.yml"));
    }

    #[test]
    fn test_determine_playbook_path_exact_yaml_match() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("my_playbook.yaml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "my_playbook.yaml");
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with("my_playbook.yaml"));
    }

    #[test]
    fn test_determine_playbook_path_playbooks_dir_yml() {
        let temp_dir = TempDir::new().unwrap();
        let playbooks_dir = temp_dir.path().join("playbooks");
        std::fs::create_dir_all(&playbooks_dir).unwrap();
        std::fs::write(playbooks_dir.join("task.yml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "task");
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("playbooks"));
        assert!(path.to_string_lossy().ends_with("task.yml"));
    }

    #[test]
    fn test_determine_playbook_path_playbooks_dir_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let playbooks_dir = temp_dir.path().join("playbooks");
        std::fs::create_dir_all(&playbooks_dir).unwrap();
        // Файл в playbooks с точным именем (без добавления расширения)
        std::fs::write(playbooks_dir.join("task.yaml"), "---").unwrap();

        // Ищем с полным именем файла -- найдёт playbooks/task.yaml через repo_path.join(playbook_name)
        let path = determine_playbook_path(temp_dir.path(), "playbooks/task.yaml");
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("playbooks"));
        assert!(path.to_string_lossy().ends_with("playbooks/task.yaml"));
    }

    #[test]
    fn test_determine_playbook_path_no_playbooks_dir() {
        let temp_dir = TempDir::new().unwrap();
        // playbooks директория не существует

        let path = determine_playbook_path(temp_dir.path(), "nonexistent");
        assert!(!path.exists());
        assert_eq!(path, temp_dir.path().join("nonexistent"));
    }

    #[test]
    fn test_determine_playbook_path_both_root_and_playbooks() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("site.yml"), "#root").unwrap();

        let playbooks_dir = temp_dir.path().join("playbooks");
        std::fs::create_dir_all(&playbooks_dir).unwrap();
        std::fs::write(playbooks_dir.join("site.yml"), "#playbooks").unwrap();

        // Точное совпадение в корне находится первым
        let path = determine_playbook_path(temp_dir.path(), "site.yml");
        assert!(path.exists());
        assert!(!path.to_string_lossy().contains("playbooks"));
    }

    #[test]
    fn test_determine_playbook_path_only_in_playbooks() {
        let temp_dir = TempDir::new().unwrap();
        let playbooks_dir = temp_dir.path().join("playbooks");
        std::fs::create_dir_all(&playbooks_dir).unwrap();
        std::fs::write(playbooks_dir.join("unique.yml"), "---").unwrap();

        // Файла в корне нет, но есть в playbooks
        let path = determine_playbook_path(temp_dir.path(), "unique");
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("playbooks"));
    }

    #[test]
    fn test_playbook_update_with_special_characters() {
        let update = PlaybookUpdate {
            name: "Deploy toProduction (v2.0)".to_string(),
            content: "--- # special chars".to_string(),
            description: Some("Test with special characters & symbols".to_string()),
            playbook_type: "ansible".to_string(),
        };
        assert_eq!(update.name, "Deploy toProduction (v2.0)");
        assert!(update.description.unwrap().contains("&"));
    }

    #[test]
    fn test_playbook_update_with_multiline_content() {
        let update = PlaybookUpdate {
            name: "multiline".to_string(),
            content: "---\n- hosts: all\n  tasks:\n    - name: test\n      debug:\n        msg: hello".to_string(),
            description: None,
            playbook_type: "ansible".to_string(),
        };
        assert!(update.content.contains('\n'));
        assert_eq!(update.content.lines().count(), 6);
    }

    #[test]
    fn test_determine_playbook_path_with_hyphenated_name() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("deploy-staging.yml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "deploy-staging");
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with("deploy-staging.yml"));
    }

    #[test]
    fn test_determine_playbook_path_with_dotted_name() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("site.v2.yml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "site.v2.yml");
        assert!(path.exists());
    }

    #[test]
    fn test_determine_playbook_path_returns_clone_of_pathbuf() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("task.yml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "task");
        // Pathbuf должен быть клоном, не ссылкой
        assert!(path.exists());
        let path2 = determine_playbook_path(temp_dir.path(), "task");
        assert_eq!(path, path2);
    }

    #[test]
    fn test_playbook_update_clone_preserves_all_fields() {
        let update = PlaybookUpdate {
            name: "full".to_string(),
            content: "content".to_string(),
            description: Some("desc".to_string()),
            playbook_type: "terraform".to_string(),
        };
        let cloned = update.clone();

        assert_eq!(cloned.name, update.name);
        assert_eq!(cloned.content, update.content);
        assert_eq!(cloned.description, update.description);
        assert_eq!(cloned.playbook_type, update.playbook_type);
    }

    #[test]
    fn test_playbook_update_debug_contains_fields() {
        let update = PlaybookUpdate {
            name: "debug_test".to_string(),
            content: "content".to_string(),
            description: Some("debug desc".to_string()),
            playbook_type: "shell".to_string(),
        };
        let debug_str = format!("{:?}", update);
        assert!(debug_str.contains("debug_test"));
        assert!(debug_str.contains("debug desc"));
    }

    #[test]
    fn test_determine_playbook_path_temp_dir_isolation() {
        // Проверяем что разные TempDir не влияют друг на друга
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        std::fs::write(temp_dir1.path().join("task.yml"), "---").unwrap();
        std::fs::write(temp_dir2.path().join("task.yml"), "---").unwrap();

        let path1 = determine_playbook_path(temp_dir1.path(), "task");
        let path2 = determine_playbook_path(temp_dir2.path(), "task");

        assert!(path1.exists());
        assert!(path2.exists());
        assert_ne!(path1, path2);
    }

    #[test]
    fn test_determine_playbook_path_empty_playbooks_dir() {
        let temp_dir = TempDir::new().unwrap();
        let playbooks_dir = temp_dir.path().join("playbooks");
        std::fs::create_dir_all(&playbooks_dir).unwrap();
        // playbooks пустой

        let path = determine_playbook_path(temp_dir.path(), "missing");
        assert!(!path.exists());
    }

    #[test]
    fn test_playbook_sync_service_size() {
        // PlaybookSyncService -- zero-sized struct
        assert_eq!(std::mem::size_of::<PlaybookSyncService>(), 0);
    }

    #[test]
    fn test_determine_playbook_path_yml_and_yaml_both_exist_find_yml() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("deploy.yml"), "#yml").unwrap();
        std::fs::write(temp_dir.path().join("deploy.yaml"), "#yaml").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "deploy");
        assert!(path.exists());
        // .yml находится раньше .yaml в списке possible_paths
        assert!(path.to_string_lossy().ends_with("deploy.yml"));
    }

    #[test]
    fn test_determine_playbook_path_with_extension_finds_exact() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("task.yml"), "---").unwrap();
        std::fs::write(temp_dir.path().join("task.yaml"), "---").unwrap();

        // Ищем с .yaml -- должен найти .yaml
        let path = determine_playbook_path(temp_dir.path(), "task.yaml");
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with("task.yaml"));
    }

    #[test]
    fn test_playbook_update_name_uniqueness() {
        let u1 = PlaybookUpdate {
            name: "unique1".to_string(),
            content: "c".to_string(),
            description: None,
            playbook_type: "a".to_string(),
        };
        let u2 = PlaybookUpdate {
            name: "unique2".to_string(),
            content: "c".to_string(),
            description: None,
            playbook_type: "a".to_string(),
        };
        assert_ne!(u1.name, u2.name);
    }

    #[test]
    fn test_determine_playbook_path_case_sensitive_search() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("Deploy.yml"), "---").unwrap();

        // Ищем с другой casing -- не найдёт
        let path = determine_playbook_path(temp_dir.path(), "deploy");
        assert!(!path.exists());
    }

    #[test]
    fn test_determine_playbook_path_with_numeric_suffix() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("playbook_v2.yml"), "---").unwrap();

        let path = determine_playbook_path(temp_dir.path(), "playbook_v2");
        assert!(path.exists());
        assert!(path.to_string_lossy().ends_with("playbook_v2.yml"));
    }
}
