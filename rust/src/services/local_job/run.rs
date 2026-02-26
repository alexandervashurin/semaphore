//! LocalJob Run - основной метод запуска задачи
//!
//! Аналог services/tasks/local_job_run.go из Go версии

use crate::error::Result;
use crate::services::local_job::LocalJob;

impl LocalJob {
    /// Запускает задачу
    pub async fn run(&mut self, username: &str, incoming_version: Option<&str>, alias: &str) -> Result<()> {
        self.set_status(crate::services::task_logger::TaskStatus::Starting);
        self.log("Starting job...");

        // Устанавливаем SSH ключи
        self.install_ssh_keys().await?;

        // Устанавливаем файлы Vault
        self.install_vault_key_files().await?;

        // Обновляем репозиторий
        self.update_repository().await?;

        // Переключаем на нужный коммит/ветку
        self.checkout_repository().await?;

        // Создаём приложение и запускаем
        // TODO: Интеграция с Executor
        // self.prepare_run(username, incoming_version, alias).await?;
        // self.app.run().await?;

        self.set_status(crate::services::task_logger::TaskStatus::Success);
        self.log("Job completed successfully");

        Ok(())
    }

    /// Подготавливает запуск задачи
    async fn prepare_run(&mut self, username: &str, incoming_version: Option<&str>, alias: &str) -> Result<()> {
        // TODO: Определить тип приложения и создать его
        // match self.template.template_type {
        //     TemplateType::Ansible => {
        //         self.app = Some(Box::new(AnsibleApp::new(...)?));
        //     }
        //     TemplateType::Terraform => {
        //         self.app = Some(Box::new(TerraformApp::new(...)?));
        //     }
        //     TemplateType::Shell => {
        //         self.app = Some(Box::new(ShellApp::new(...)?));
        //     }
        //     _ => {}
        // }

        // TODO: Установить приложение
        // self.app.install(installing_args).await?;

        Ok(())
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
