//! LocalJob - структура и базовые методы
//!
//! Аналог services/tasks/local_job_types.go из Go версии

use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Child;
use tracing::{error, info, warn};

use crate::db_lib::AccessKeyInstallerImpl;
use crate::error::Result;
use crate::models::{Environment, Inventory, Repository, Task, Template};
use crate::services::ssh_agent::AccessKeyInstallation;
use crate::services::task_logger::{TaskLogger, TaskStatus};
use crate::services::task_runner::Job;

/// Локальная задача для выполнения
pub struct LocalJob {
    /// Задача
    pub task: Task,
    /// Шаблон
    pub template: Template,
    /// Инвентарь
    pub inventory: Inventory,
    /// Репозиторий
    pub repository: Repository,
    /// Окружение
    pub environment: Environment,
    /// Секретные переменные из Survey
    pub secret: String,
    /// Логгер
    pub logger: Arc<dyn TaskLogger>,
    /// SSH ключи
    pub ssh_key_installation: Option<AccessKeyInstallation>,
    /// Become ключи
    pub become_key_installation: Option<AccessKeyInstallation>,
    /// Vault файлы
    pub vault_file_installations: std::collections::HashMap<String, AccessKeyInstallation>,
    /// Установщик ключей
    pub key_installer: AccessKeyInstallerImpl,
    /// Процесс
    pub process: Option<Child>,
    /// Флаг остановки
    pub killed: bool,
    /// Рабочая директория
    pub work_dir: PathBuf,
    /// Временная директория
    pub tmp_dir: PathBuf,
    /// Store для загрузки SSH ключей из БД (опционально)
    pub store: Option<Arc<dyn crate::db::store::Store + Send + Sync>>,
    /// Имя пользователя (для Job trait)
    pub username: String,
    /// Входящая версия (для Job trait)
    pub incoming_version: Option<String>,
    /// Alias для запуска (для Job trait)
    pub alias: String,
}

impl LocalJob {
    /// Создаёт новую локальную задачу
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        task: Task,
        template: Template,
        inventory: Inventory,
        repository: Repository,
        environment: Environment,
        logger: Arc<dyn TaskLogger>,
        key_installer: AccessKeyInstallerImpl,
        work_dir: PathBuf,
        tmp_dir: PathBuf,
    ) -> Self {
        Self {
            task,
            template,
            inventory,
            repository,
            environment,
            secret: String::new(),
            logger,
            ssh_key_installation: None,
            become_key_installation: None,
            vault_file_installations: std::collections::HashMap::new(),
            key_installer,
            process: None,
            killed: false,
            work_dir,
            tmp_dir,
            username: String::new(),
            incoming_version: None,
            alias: String::new(),
            store: None,
        }
    }

    /// Устанавливает параметры запуска (вызывается перед Job::run)
    pub fn set_run_params(
        &mut self,
        username: String,
        incoming_version: Option<String>,
        alias: String,
    ) {
        self.username = username;
        self.incoming_version = incoming_version;
        self.alias = alias;
    }

    /// Проверяет, убита ли задача
    pub fn is_killed(&self) -> bool {
        self.killed
    }

    /// Останавливает задачу
    pub fn kill(&mut self) {
        self.killed = true;
        if let Some(ref mut process) = self.process {
            let _ = process.start_kill();
            self.logger.log("Process killed");
        }
    }

    /// Логирует сообщение
    pub fn log(&self, msg: &str) {
        self.logger.log(msg);
    }

    /// Устанавливает статус
    pub fn set_status(&self, status: TaskStatus) {
        self.logger.set_status(status);
    }

    /// Устанавливает информацию о коммите
    pub fn set_commit(&self, hash: &str, message: &str) {
        self.logger.set_commit(hash, message);
    }
}

impl Drop for LocalJob {
    fn drop(&mut self) {
        // Очищаем SSH ключи
        self.ssh_key_installation = None;
        self.become_key_installation = None;
        self.vault_file_installations.clear();
    }
}

#[async_trait::async_trait]
impl Job for LocalJob {
    async fn run(&mut self) -> Result<()> {
        let username = self.username.clone();
        let incoming_version = self.incoming_version.clone();
        let alias = self.alias.clone();
        LocalJob::run(self, &username, incoming_version.as_deref(), &alias).await
    }

    fn kill(&mut self) {
        LocalJob::kill(self);
    }

    fn is_killed(&self) -> bool {
        LocalJob::is_killed(self)
    }
}

// TODO: Добавить тесты после завершения миграции всех модулей local_job

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::task_logger::BasicLogger;
    use chrono::Utc;

    fn create_test_local_job() -> LocalJob {
        let logger = Arc::new(BasicLogger::new());
        let key_installer = AccessKeyInstallerImpl::new();

        let task = Task {
            id: 1,
            created: Utc::now(),
            template_id: 1,
            status: TaskStatus::Waiting,
            message: None,
            commit_hash: None,
            commit_message: None,
            version: None,
            project_id: 1,
            arguments: None,
            params: None,
            playbook: None,
            environment: None,
            secret: None,
            git_branch: None,
            user_id: None,
            integration_id: None,
            schedule_id: None,
            start: None,
            end: None,
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            build_task_id: None,
        };

        LocalJob::new(
            task,
            Template::default(),
            Inventory::default(),
            Repository::default(),
            Environment::default(),
            logger,
            key_installer,
            PathBuf::from("/tmp/work"),
            PathBuf::from("/tmp/tmp"),
        )
    }

    #[test]
    fn test_local_job_creation() {
        let job = create_test_local_job();
        assert_eq!(job.task.id, 1);
        assert!(job.ssh_key_installation.is_none());
        assert!(job.become_key_installation.is_none());
        assert!(job.vault_file_installations.is_empty());
        assert!(!job.killed);
        assert_eq!(job.work_dir, PathBuf::from("/tmp/work"));
        assert_eq!(job.tmp_dir, PathBuf::from("/tmp/tmp"));
    }

    #[test]
    fn test_local_job_set_run_params() {
        let mut job = create_test_local_job();
        job.set_run_params(
            "testuser".to_string(),
            Some("v1.0".to_string()),
            "deploy".to_string(),
        );
        assert_eq!(job.username, "testuser");
        assert_eq!(job.incoming_version, Some("v1.0".to_string()));
        assert_eq!(job.alias, "deploy");
    }

    #[test]
    fn test_local_job_is_killed_default() {
        let job = create_test_local_job();
        assert!(!job.is_killed());
    }

    #[test]
    fn test_local_job_kill_sets_flag() {
        let mut job = create_test_local_job();
        job.kill();
        assert!(job.is_killed());
    }

    #[test]
    fn test_local_job_log() {
        let job = create_test_local_job();
        // Просто проверяем что метод не паникует
        job.log("Test log message");
    }

    #[test]
    fn test_local_job_set_status() {
        let job = create_test_local_job();
        job.set_status(TaskStatus::Running);
    }

    #[test]
    fn test_local_job_set_commit() {
        let job = create_test_local_job();
        job.set_commit("abc123", "Test commit");
    }

    #[test]
    fn test_local_job_drop() {
        let job = create_test_local_job();
        // Drop вызывается автоматически, проверяем что не паникует
        drop(job);
    }

    #[test]
    fn test_kill_without_process() {
        let mut job = create_test_local_job();
        assert!(job.process.is_none());
        assert!(!job.killed);

        job.kill();

        assert!(job.killed);
        // process остаётся None -- никакая паника
        assert!(job.process.is_none());
    }

    #[test]
    fn test_set_run_params_with_none_version() {
        let mut job = create_test_local_job();
        job.set_run_params("admin".to_string(), None, "rollback".to_string());

        assert_eq!(job.username, "admin");
        assert!(job.incoming_version.is_none());
        assert_eq!(job.alias, "rollback");
    }

    #[test]
    fn test_local_job_set_run_params_with_version() {
        let mut job = create_test_local_job();
        job.set_run_params("deploy_user".to_string(), Some("v1.2.3".to_string()), "deploy".to_string());

        assert_eq!(job.username, "deploy_user");
        assert_eq!(job.incoming_version, Some("v1.2.3".to_string()));
        assert_eq!(job.alias, "deploy");
    }

    #[test]
    fn test_local_job_secret_initially_empty() {
        let job = create_test_local_job();
        assert!(job.secret.is_empty());
    }

    #[test]
    fn test_local_job_vault_installations_initially_empty() {
        let job = create_test_local_job();
        assert!(job.vault_file_installations.is_empty());
    }

    #[test]
    fn test_local_job_process_initially_none() {
        let job = create_test_local_job();
        assert!(job.process.is_none());
    }

    #[test]
    fn test_local_job_store_initially_none() {
        let job = create_test_local_job();
        assert!(job.store.is_none());
    }
}
