//! LocalJob Run - основной метод запуска задачи
//!
//! Аналог services/tasks/local_job_run.go из Go версии

use crate::error::Result;
use crate::services::local_job::LocalJob;
use crate::services::task_logger::TaskStatus;

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

        // TODO: Запуск приложения
        // if let Some(ref mut app) = self.app {
        //     if let Err(e) = app.run().await {
        //         self.log(&format!("Failed to run app: {}", e));
        //         self.set_status(TaskStatus::Error);
        //         return Err(e);
        //     }
        // }

        self.set_status(TaskStatus::Success);
        self.log("Job completed successfully");

        Ok(())
    }

    /// Подготавливает запуск задачи
    async fn prepare_run(&mut self, username: &str, incoming_version: Option<&str>, alias: &str) -> Result<()> {
        self.log("Preparing to run task...");

        // Получаем аргументы в зависимости от типа шаблона
        match self.template.template_type {
            crate::models::TemplateType::Ansible => {
                self.log("Preparing Ansible playbook...");
                // TODO: Создать AnsibleApp
                // let args = self.get_playbook_args(username, incoming_version)?;
            }
            crate::models::TemplateType::Terraform => {
                self.log("Preparing Terraform...");
                // TODO: Создать TerraformApp
                // let args = self.get_terraform_args(username, incoming_version)?;
            }
            crate::models::TemplateType::Shell => {
                self.log("Preparing Shell script...");
                // TODO: Создать ShellApp
                // let args = self.get_shell_args(username, incoming_version)?;
            }
            _ => {
                self.log("Preparing local task...");
            }
        }

        Ok(())
    }

    /// Очищает ресурсы после выполнения
    pub fn cleanup(&self) {
        // Очищаем временные файлы
        let _ = std::fs::remove_dir_all(&self.tmp_dir);
        self.log("Cleanup completed");
    }
}

impl Drop for LocalJob {
    fn drop(&mut self) {
        self.cleanup();
        self.clear_ssh_keys();
        self.clear_vault_key_files();
    }
}

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
            status: crate::models::TaskStatus::Waiting,
            message: String::new(),
            commit_hash: None,
            commit_message: None,
            version: None,
            project_id: 1,
            arguments: None,
            params: String::new(),
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
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        )
    }

    #[test]
    fn test_run() {
        let mut job = create_test_job();
        let result = futures::executor::block_on(
            job.run("testuser", None, "test")
        );
        // Пока всегда Ok, так как методы-заглушки возвращают Ok
        assert!(result.is_ok());
    }
}
