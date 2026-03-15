//! LocalJob Run - основной метод запуска задачи
//!
//! Аналог services/tasks/local_job_run.go из Go версии

use crate::error::Result;
use crate::models::template::TemplateApp;
use crate::services::local_job::LocalJob;
use crate::services::task_logger::TaskStatus;
use crate::db_lib::{create_app, AnsibleApp, TerraformApp};
use crate::db_lib::local_app::{LocalApp, LocalAppRunningArgs, LocalAppInstallingArgs};

impl LocalJob {
    /// Запускает задачу
    pub async fn run(&mut self, username: &str, incoming_version: Option<&str>, alias: &str) -> Result<()> {
        self.set_status(TaskStatus::Starting);
        self.log("Starting job...");

        // Устанавливаем SSH ключи
        if let Err(e) = self.install_ssh_keys().await {
            self.log(&format!("Failed to install SSH keys: {}", e));
            self.set_status(TaskStatus::Error);
            return Err(e);
        }

        // Устанавливаем файлы Vault
        if let Err(e) = self.install_vault_key_files().await {
            self.log(&format!("Failed to install Vault keys: {}", e));
            self.set_status(TaskStatus::Error);
            return Err(e);
        }

        // Обновляем репозиторий
        if let Err(e) = self.update_repository().await {
            self.log(&format!("Failed to update repository: {}", e));
            self.set_status(TaskStatus::Error);
            return Err(e);
        }

        // Переключаем на нужный коммит/ветку
        if let Err(e) = self.checkout_repository().await {
            self.log(&format!("Failed to checkout repository: {}", e));
            self.set_status(TaskStatus::Error);
            return Err(e);
        }

        // Создаём приложение и запускаем
        if let Err(e) = self.prepare_run(username, incoming_version, alias).await {
            self.log(&format!("Failed to prepare run: {}", e));
            self.set_status(TaskStatus::Error);
            return Err(e);
        }

        self.set_status(TaskStatus::Success);
        self.log("Job completed successfully");

        Ok(())
    }

    /// Подготавливает запуск задачи — создаёт и выполняет приложение
    async fn prepare_run(&mut self, _username: &str, _incoming_version: Option<&str>, _alias: &str) -> Result<()> {
        self.log("Preparing to run task...");

        let repo_path = self.work_dir.join("repository");
        let mut repository = self.repository.clone();
        repository.git_path = Some(repo_path.to_string_lossy().to_string());

        let install_args = LocalAppInstallingArgs::default();
        let mut run_args = LocalAppRunningArgs::default();

        // Write inventory data to temp file and pass -i to ansible-playbook
        if !self.inventory.inventory_data.is_empty() {
            let inv_path = self.tmp_dir.join("inventory");
            if let Err(e) = std::fs::write(&inv_path, &self.inventory.inventory_data) {
                self.log(&format!("Warning: could not write inventory file: {e}"));
            } else {
                let cli_args = run_args.cli_args.entry("default".to_string()).or_insert_with(Vec::new);
                cli_args.push("-i".to_string());
                cli_args.push(inv_path.to_string_lossy().to_string());
            }
        }

        match self.template.app {
            TemplateApp::Ansible => {
                self.log("Running Ansible playbook...");
                let app = AnsibleApp::new(
                    self.logger.clone(),
                    self.template.clone(),
                    repository,
                    self.work_dir.clone(),
                );
                app.install_requirements(install_args).await?;
                app.run(run_args).await?;
            }
            TemplateApp::Terraform | TemplateApp::Tofu | TemplateApp::Terragrunt => {
                let name = match self.template.app {
                    TemplateApp::Terraform => "terraform",
                    TemplateApp::Tofu => "tofu",
                    TemplateApp::Terragrunt => "terragrunt",
                    _ => "terraform",
                };
                self.log(&format!("Running {}...", name));
                let app = TerraformApp::new(
                    self.logger.clone(),
                    self.template.clone(),
                    repository,
                    self.inventory.clone(),
                    name.to_string(),
                    self.work_dir.clone(),
                );
                app.run(run_args).await?;
            }
            _ => {
                self.log("Running Shell script...");
                let mut app = create_app(
                    self.template.clone(),
                    repository,
                    self.inventory.clone(),
                    self.logger.clone(),
                );
                app.install_requirements(install_args)?;
                tokio::task::spawn_blocking(move || app.run(run_args))
                    .await
                    .map_err(|e| crate::error::Error::Other(format!("Task join error: {}", e)))??;
            }
        }

        Ok(())
    }

    /// Очищает ресурсы после выполнения
    pub fn cleanup(&self) {
        // Очищаем рабочую директорию
        let _ = std::fs::remove_dir_all(&self.work_dir);
        self.log("Cleanup completed");
    }
}

// Drop реализация находится в types.rs

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::sync::Arc;
    use crate::services::task_logger::BasicLogger;
    use crate::db_lib::AccessKeyInstallerImpl;
    use std::path::PathBuf;

    fn create_test_job() -> LocalJob {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let task = crate::models::Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            project_id: 1,
            status: crate::services::task_logger::TaskStatus::Waiting,
            playbook: None,
            environment: None,
            secret: None,
            arguments: None,
            git_branch: None,
            user_id: None,
            integration_id: None,
            schedule_id: None,
            start: None,
            end: None,
            message: None,
            commit_hash: None,
            commit_message: None,
            build_task_id: None,
            version: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            params: None,
        };

        LocalJob::new(
            task,
            crate::models::Template::default(),
            crate::models::Inventory::default(),
            crate::models::Repository::default(),
            crate::models::Environment::default(),
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        )
    }

    #[tokio::test]
    async fn test_run() {
        let mut job = create_test_job();
        job.set_run_params("testuser".to_string(), None, "default".to_string());
        let result = job.run("testuser", None, "default").await;
        assert!(result.is_ok());
    }
}
