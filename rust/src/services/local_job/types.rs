//! LocalJob - структура и базовые методы
//!
//! Аналог services/tasks/local_job_types.go из Go версии

use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Child;
use tracing::{info, warn, error};

use crate::error::Result;
use crate::models::{Task, Template, Inventory, Repository, Environment};
use crate::services::task_logger::{TaskLogger, TaskStatus};
use crate::services::ssh_agent::AccessKeyInstallation;
use crate::services::task_runner::Job;
use crate::db_lib::AccessKeyInstallerImpl;

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
    pub fn set_run_params(&mut self, username: String, incoming_version: Option<String>, alias: String) {
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
        LocalJob::run(
            self,
            &username,
            incoming_version.as_deref(),
            &alias,
        )
        .await
    }

    fn kill(&mut self) {
        LocalJob::kill(self);
    }

    fn is_killed(&self) -> bool {
        LocalJob::is_killed(self)
    }
}

// TODO: Добавить тесты после завершения миграции всех модулей local_job
