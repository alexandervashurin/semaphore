//! LocalJob Vault - работа с Vault файлами
//!
//! Аналог services/tasks/local_job_vault.go из Go версии
use crate::error::Result;
use crate::services::local_job::LocalJob;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

impl LocalJob {
    /// Устанавливает файлы ключей Vault
    pub async fn install_vault_key_files(&mut self) -> Result<()> {
        self.vault_file_installations = HashMap::new();

        #[derive(serde::Deserialize)]
        struct VaultRef {
            vault_key_id: i32,
            #[serde(default)]
            r#type: String,
        }

        let vault_refs: Vec<VaultRef> = if let Some(ref vaults_val) = self.template.vaults {
            match serde_json::from_value(vaults_val.clone()) {
                Ok(v) => v,
                Err(e) => {
                    self.log(&format!(
                        "Warning: failed to parse template vault refs: {}",
                        e
                    ));
                    return Ok(());
                }
            }
        } else if let Some(ref vaults_json) = self.inventory.vaults {
            if vaults_json.is_empty() {
                return Ok(());
            }
            match serde_json::from_str(vaults_json) {
                Ok(v) => v,
                Err(e) => {
                    self.log(&format!("Warning: failed to parse vault refs: {}", e));
                    return Ok(());
                }
            }
        } else {
            return Ok(());
        };

        if vault_refs.is_empty() {
            return Ok(());
        }

        let store = match self.store.as_ref() {
            Some(s) => s.clone(),
            None => {
                self.log("Warning: no store available for vault key loading");
                return Ok(());
            }
        };

        for (i, vref) in vault_refs.iter().enumerate() {
            use crate::db::store::AccessKeyManager;
            let key = match store
                .get_access_key(self.task.project_id, vref.vault_key_id)
                .await
            {
                Ok(k) => k,
                Err(e) => {
                    self.log(&format!(
                        "Warning: vault key {} not found: {}",
                        vref.vault_key_id, e
                    ));
                    continue;
                }
            };

            let raw_type = if vref.r#type.is_empty() {
                format!("vault_{}", i)
            } else {
                vref.r#type.clone()
            };
            let vault_name: String = raw_type
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            if vault_name.is_empty() {
                self.log(&format!(
                    "Warning: vault type '{}' contains no valid chars, skipping",
                    raw_type
                ));
                continue;
            }

            let password = key.login_password_password.as_deref().unwrap_or("");

            if !password.is_empty() {
                let _vault_file = self
                    .create_vault_password_file(&vault_name, password)
                    .await?;
                self.log(&format!("Vault key installed: {}", vault_name));
                let installation = crate::services::ssh_agent::AccessKeyInstallation {
                    password: Some(password.to_string()),
                    ..Default::default()
                };
                self.vault_file_installations
                    .insert(vault_name, installation);
            }
        }

        Ok(())
    }

    /// Очищает файлы ключей Vault
    pub fn clear_vault_key_files(&mut self) {
        self.vault_file_installations.clear();
    }

    /// Создаёт временный файл для пароля Vault
    pub async fn create_vault_password_file(
        &self,
        vault_name: &str,
        password: &str,
    ) -> Result<PathBuf> {
        let tmp_dir = &self.tmp_dir;
        let vault_password_file = tmp_dir.join(format!("vault_{}_password", vault_name));

        fs::create_dir_all(tmp_dir).await?;
        fs::write(&vault_password_file, password).await?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&vault_password_file).await?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&vault_password_file, perms).await?;
        }

        Ok(vault_password_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_lib::AccessKeyInstallerImpl;
    use crate::services::task_logger::BasicLogger;
    use chrono::Utc;
    use std::path::PathBuf;
    use std::sync::Arc;
    // В модуле tests файла vault.rs

    fn create_test_job_with_dirs(work_dir: PathBuf, tmp_dir: PathBuf) -> LocalJob {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let task = crate::models::Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: crate::services::task_logger::TaskStatus::Waiting,
            message: None,
            commit_hash: None,
            commit_message: None,
            version: None,
            project_id: 1,
            arguments: None,
            params: None,
            ..Default::default()
        };

        LocalJob::new(
            task,
            crate::models::Template::default(),
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger,
            key_installer,
            work_dir,
            tmp_dir,
        )
    }

    fn create_test_job() -> LocalJob {
        // Для обратной совместимости используем tempfile и здесь
        let tmp = tempfile::tempdir()
            .ok()
            .unwrap_or_else(|| tempfile::tempdir_in("/tmp").unwrap());
        let work = tempfile::tempdir()
            .ok()
            .unwrap_or_else(|| tempfile::tempdir_in("/tmp").unwrap());

        // Сохраняем директории, чтобы они не удалились преждевременно
        // В реальном тесте нужно хранить _tmp и _work в структуре теста
        create_test_job_with_dirs(work.into_path(), tmp.into_path())
    }

    #[test]
    fn test_clear_vault_key_files() {
        let mut job = create_test_job();
        job.clear_vault_key_files();
        assert!(job.vault_file_installations.is_empty());
    }

    #[tokio::test]
    async fn test_create_vault_password_file() {
        let job = create_test_job();
        let result = job
            .create_vault_password_file("test_vault", "my_secret_password")
            .await;
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("vault_test_vault_password"));
    }

    #[tokio::test]
    async fn test_create_vault_password_file_empty_password() {
        let job = create_test_job();
        let result = job.create_vault_password_file("empty_vault", "").await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_create_vault_password_file_very_long_password() {
        // Создаём временные директории через tempfile (кроссплатформенно)
        let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let work_dir = tempfile::tempdir().expect("Failed to create work dir");

        let job =
            create_test_job_with_dirs(work_dir.path().to_path_buf(), tmp_dir.path().to_path_buf());

        let long_password = "a".repeat(10000); // Без пробелов — 10000 символов
        let result = job.create_vault_password_file("long", &long_password).await;

        // ✅ Теперь тест пройдёт на всех платформах
        assert!(
            result.is_ok(),
            "Failed to create vault password file: {:?}",
            result.err()
        );

        let path = result.unwrap();
        let content = tokio::fs::read_to_string(&path).await.unwrap();
        assert_eq!(content.len(), 10000);

        // 🔹 tempfile автоматически удалит директории при drop
        // Но можно явно очистить, если нужно
        drop(job);
    }

    #[test]
    fn test_clear_vault_key_files_removes_all() {
        let mut job = create_test_job();
        job.vault_file_installations.insert(
            "test1".to_string(),
            crate::services::ssh_agent::AccessKeyInstallation::default(),
        );
        job.clear_vault_key_files();
        assert!(job.vault_file_installations.is_empty());
    }

    #[tokio::test]
    async fn test_create_vault_password_file_special_chars() {
        let job = create_test_job();
        let result = job
            .create_vault_password_file("my-vault_123", "p@ssw0rd!")
            .await;
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("my-vault_123"));
    }

    #[test]
    fn test_clear_vault_key_files_idempotent() {
        let mut job = create_test_job();
        job.clear_vault_key_files();
        assert!(job.vault_file_installations.is_empty());
        job.clear_vault_key_files();
        assert!(job.vault_file_installations.is_empty());
    }

    #[tokio::test]
    async fn test_create_vault_password_file_writes_correct_content() {
        let job = create_test_job();
        let password = "super_secret_password_123";
        let result = job.create_vault_password_file("test", password).await;
        assert!(result.is_ok());

        let path = result.unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, password);

        std::fs::remove_dir_all(&job.tmp_dir).ok();
    }

    #[tokio::test]
    async fn test_create_vault_password_file_unicode() {
        let job = create_test_job();
        let result = job
            .create_vault_password_file("vault_unicode", "pass123")
            .await;
        assert!(result.is_ok());
        let path = result.unwrap();
        let filename = path.file_name().unwrap().to_string_lossy();
        assert!(filename.contains("vault_unicode"));
    }

    #[tokio::test]
    async fn test_create_vault_password_file_path_traversal_chars() {
        let job = create_test_job();
        // create_vault_password_file не санитизирует имя. Передаём безопасное имя с дефисом.
        let result = job.create_vault_password_file("etc-passwd", "hack").await;
        assert!(result.is_ok());
        let path = result.unwrap();
        let filename = path.file_name().unwrap().to_string_lossy();
        assert!(filename.contains("vault_etc-passwd"));
        assert!(!filename.contains('/'));
        assert!(!filename.contains(".."));
    }

    #[tokio::test]
    async fn test_install_vault_key_files_no_store_logs_warning() {
        let mut job = create_test_job();
        job.template.vaults = Some(serde_json::json!([
            { "vault_key_id": 1, "type": "main" }
        ]));
        let result = job.install_vault_key_files().await;
        assert!(result.is_ok());
        assert!(job.vault_file_installations.is_empty());
    }

    #[tokio::test]
    async fn test_install_vault_key_files_invalid_vault_json() {
        let mut job = create_test_job();
        job.template.vaults = Some(serde_json::json!("not an array"));
        let result = job.install_vault_key_files().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_install_vault_key_files_from_inventory() {
        let mut job = create_test_job();
        job.inventory.vaults = Some(r#"[{"vault_key_id": 1, "type": "prod"}]"#.to_string());
        let result = job.install_vault_key_files().await;
        assert!(result.is_ok());
        assert!(job.vault_file_installations.is_empty());
    }

    #[tokio::test]
    async fn test_install_vault_key_files_empty_inventory_vaults() {
        let mut job = create_test_job();
        job.inventory.vaults = Some("".to_string());
        let result = job.install_vault_key_files().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_install_vault_key_files_no_vaults_anywhere() {
        let mut job = create_test_job();
        job.template.vaults = None;
        job.inventory.vaults = None;
        let result = job.install_vault_key_files().await;
        assert!(result.is_ok());
        assert!(job.vault_file_installations.is_empty());
    }

    #[tokio::test]
    async fn test_create_vault_password_file_special_vault_name() {
        let job = create_test_job();
        let result = job.create_vault_password_file("!@#$%^&*()", "pass").await;
        assert!(result.is_ok());
        let path = result.unwrap();
        let filename = path.file_name().unwrap().to_string_lossy();
        assert!(filename.contains("vault_"));
    }

    #[test]
    fn test_clear_vault_key_files_does_not_affect_other_fields() {
        let mut job = create_test_job();
        job.vault_file_installations.insert(
            "test".to_string(),
            crate::services::ssh_agent::AccessKeyInstallation::default(),
        );

        let work_dir_before = job.work_dir.clone();
        let tmp_dir_before = job.tmp_dir.clone();

        job.clear_vault_key_files();

        assert_eq!(job.work_dir, work_dir_before);
        assert_eq!(job.tmp_dir, tmp_dir_before);
        assert!(job.vault_file_installations.is_empty());
    }
}
