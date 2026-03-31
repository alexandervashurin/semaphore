//! LocalJob Run - основной метод запуска задачи
//!
//! Аналог services/tasks/local_job_run.go из Go версии

use crate::db_lib::local_app::{LocalApp, LocalAppInstallingArgs, LocalAppRunningArgs};
use crate::db_lib::{create_app, AnsibleApp, TerraformApp};
use crate::error::Result;
use crate::models::template::TemplateApp;
use crate::services::local_job::LocalJob;
use crate::services::task_logger::TaskStatus;

impl LocalJob {
    /// Запускает задачу
    pub async fn run(
        &mut self,
        username: &str,
        incoming_version: Option<&str>,
        alias: &str,
    ) -> Result<()> {
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
    async fn prepare_run(
        &mut self,
        _username: &str,
        _incoming_version: Option<&str>,
        _alias: &str,
    ) -> Result<()> {
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
                let cli_args = run_args
                    .cli_args
                    .entry("default".to_string())
                    .or_insert_with(Vec::new);
                cli_args.push("-i".to_string());
                cli_args.push(inv_path.to_string_lossy().to_string());
            }
        }

        match self.template.app {
            TemplateApp::Ansible => {
                self.log("Running Ansible playbook...");

                // Конвертируем task_params шаблона в флаги ansible-playbook
                {
                    let cli_args = run_args
                        .cli_args
                        .entry("default".to_string())
                        .or_insert_with(Vec::new);

                    if let Some(ref params) = self.template.task_params {
                        // --forks N
                        if let Some(forks) = params
                            .get("forks")
                            .and_then(|v| v.as_i64())
                            .filter(|&f| f > 0)
                        {
                            cli_args.push("--forks".to_string());
                            cli_args.push(forks.to_string());
                        }
                        // --connection <type>
                        if let Some(conn) = params
                            .get("connection")
                            .and_then(|v| v.as_str())
                            .filter(|s| !s.is_empty() && *s != "ssh")
                        {
                            cli_args.push("--connection".to_string());
                            cli_args.push(conn.to_string());
                        }
                        // -v / -vv / -vvv / -vvvv (skip if allow_override_debug — task.params.verbosity takes over)
                        let allow_debug_pre = params
                            .get("allow_override_debug")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        if !allow_debug_pre {
                            if let Some(v) = params
                                .get("verbosity")
                                .and_then(|v| v.as_i64())
                                .filter(|&v| v > 0 && v <= 4)
                            {
                                cli_args.push(format!("-{}", "v".repeat(v as usize)));
                            }
                        }
                        // --user <remote_user>
                        if let Some(user) = params
                            .get("remote_user")
                            .and_then(|v| v.as_str())
                            .filter(|s| !s.is_empty())
                        {
                            cli_args.push("--user".to_string());
                            cli_args.push(user.to_string());
                        }
                        // --timeout N
                        if let Some(timeout) = params
                            .get("timeout")
                            .and_then(|v| v.as_i64())
                            .filter(|&t| t > 0)
                        {
                            cli_args.push("--timeout".to_string());
                            cli_args.push(timeout.to_string());
                        }
                        // --become [--become-method <m>] [--become-user <u>]
                        if params
                            .get("become")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false)
                        {
                            cli_args.push("--become".to_string());
                            if let Some(method) = params
                                .get("become_method")
                                .and_then(|v| v.as_str())
                                .filter(|s| !s.is_empty() && *s != "sudo")
                            {
                                cli_args.push("--become-method".to_string());
                                cli_args.push(method.to_string());
                            }
                            if let Some(user) = params
                                .get("become_user")
                                .and_then(|v| v.as_str())
                                .filter(|s| !s.is_empty() && *s != "root")
                            {
                                cli_args.push("--become-user".to_string());
                                cli_args.push(user.to_string());
                            }
                        }

                        // Runtime-переопределения из task.params (limit / tags / skip-tags)
                        let allow_limit = params
                            .get("allow_override_limit")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let allow_tags = params
                            .get("allow_override_tags")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let allow_skip_tags = params
                            .get("allow_override_skip_tags")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let allow_debug = params
                            .get("allow_override_debug")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        if let Some(ref task_p) = self.task.params {
                            if allow_limit {
                                if let Some(limit) = task_p
                                    .get("limit")
                                    .and_then(|v| v.as_str())
                                    .filter(|s| !s.is_empty())
                                {
                                    cli_args.push("--limit".to_string());
                                    cli_args.push(limit.to_string());
                                }
                            }
                            if allow_tags {
                                if let Some(tags) = task_p
                                    .get("tags")
                                    .and_then(|v| v.as_str())
                                    .filter(|s| !s.is_empty())
                                {
                                    cli_args.push("--tags".to_string());
                                    cli_args.push(tags.to_string());
                                }
                            }
                            if allow_skip_tags {
                                if let Some(skip_tags) = task_p
                                    .get("skip_tags")
                                    .and_then(|v| v.as_str())
                                    .filter(|s| !s.is_empty())
                                {
                                    cli_args.push("--skip-tags".to_string());
                                    cli_args.push(skip_tags.to_string());
                                }
                            }
                            if allow_debug {
                                if let Some(v) = task_p
                                    .get("verbosity")
                                    .and_then(|v| v.as_i64())
                                    .filter(|&v| v > 0 && v <= 4)
                                {
                                    cli_args.push(format!("-{}", "v".repeat(v as usize)));
                                }
                            }
                        }
                    }

                    // --vault-password-file для каждого установленного vault ключа
                    let vault_names: Vec<String> =
                        self.vault_file_installations.keys().cloned().collect();
                    for vault_name in vault_names {
                        let vault_file =
                            self.tmp_dir.join(format!("vault_{}_password", vault_name));
                        if vault_file.exists() {
                            cli_args.push("--vault-password-file".to_string());
                            cli_args.push(vault_file.to_string_lossy().to_string());
                        }
                    }
                }

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
    use crate::db_lib::AccessKeyInstallerImpl;
    use crate::services::task_logger::BasicLogger;
    use chrono::Utc;
    use std::path::PathBuf;
    use std::sync::Arc;

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
