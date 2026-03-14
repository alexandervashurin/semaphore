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
        S: PlaybookManager + RepositoryManager,
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
        let temp_dir = TempDir::new().map_err(|e| {
            Error::Other(format!("Не удалось создать временную директорию: {}", e))
        })?;

        info!(
            "Клонирование репозитория {} в {:?}",
            repository.git_url,
            temp_dir.path()
        );

        clone_repository(&repository, temp_dir.path()).await?;

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

        info!(
            "Playbook {} успешно синхронизирован из Git",
            playbook.name
        );

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
        S: PlaybookManager + RepositoryManager,
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
        let temp_dir = TempDir::new().map_err(|e| {
            Error::Other(format!("Не удалось создать временную директорию: {}", e))
        })?;

        clone_repository(&repository, temp_dir.path()).await?;

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
async fn clone_repository(repository: &crate::models::Repository, path: &Path) -> Result<()> {
    let mut fetch_options = FetchOptions::new();
    
    // Настраиваем callback для аутентификации
    let mut remote_callbacks = RemoteCallbacks::new();
    
    // Если есть SSH ключ, настраиваем SSH аутентификацию
    if repository.key_id != 0 {
        // TODO: Получить AccessKey из БД и настроить SSH аутентификацию
        // Временно используем заглушку
        warn!("SSH аутентификация для key_id={} требует реализации", repository.key_id);
        
        // Устанавливаем credentials callback для SSH
        remote_callbacks.credentials(|url, username_from_url, allowed_types| {
            info!("Git credentials callback: URL={}, allowed_types={:?}", url, allowed_types);
            
            if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                // Пытаемся использовать SSH агент
                Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
            } else {
                Cred::default()
            }
        });
    } else {
        // Без аутентификации или HTTP аутентификация
        remote_callbacks.credentials(|url, username_from_url, allowed_types| {
            info!("Git credentials callback (no key): URL={}", url);
            Cred::default()
        });
    }
    
    fetch_options.remote_callbacks(remote_callbacks);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_options);

    // Клонируем репозиторий
    let _repo = builder
        .clone(&repository.git_url, path)
        .map_err(|e| Error::Git(e))?;

    Ok(())
}

/// Определяет путь к файлу playbook
///
/// # Arguments
/// * `repo_path` - Путь к корню репозитория
/// * `playbook_name` - Название playbook
///
/// # Returns
/// * `PathBuf` - Полный путь к файлу
fn determine_playbook_path(repo_path: &Path, playbook_name: &str) -> PathBuf {
    // Пробуем несколько вариантов:
    // 1. playbook_name как полный путь (например, "playbooks/deploy.yml")
    // 2. playbook_name с расширением .yml
    // 3. playbook_name с расширением .yaml
    
    let possible_paths = vec![
        repo_path.join(playbook_name),
        repo_path.join(format!("{}.yml", playbook_name)),
        repo_path.join(format!("{}.yaml", playbook_name)),
        repo_path.join("playbooks").join(playbook_name),
        repo_path.join("playbooks").join(format!("{}.yml", playbook_name)),
    ];

    // Возвращаем первый существующий путь
    for path in &possible_paths {
        if path.exists() {
            return path.clone();
        }
    }

    // Если ни один путь не найден, возвращаем первый вариант
    possible_paths[0].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_playbook_path() {
        let temp_dir = TempDir::new().unwrap();

        // Создаем тестовые файлы
        std::fs::write(temp_dir.path().join("deploy.yml"), "---").unwrap();
        
        // Создаем директорию playbooks и файл в ней
        let playbooks_dir = temp_dir.path().join("playbooks");
        std::fs::create_dir_all(&playbooks_dir).unwrap();
        std::fs::write(playbooks_dir.join("site.yaml"), "---").unwrap();

        // Тест 1: Прямой путь
        let path = determine_playbook_path(temp_dir.path(), "deploy.yml");
        assert!(path.exists());

        // Тест 2: Путь без расширения
        let path = determine_playbook_path(temp_dir.path(), "deploy");
        assert!(path.exists());
        
        // Тест 3: Путь в поддиректории
        let path = determine_playbook_path(temp_dir.path(), "playbooks/site.yaml");
        assert!(path.exists());
    }
}
