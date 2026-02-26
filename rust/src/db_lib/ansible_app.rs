//! AnsibleApp - выполнение Ansible playbook
//!
//! Аналог db_lib/AnsibleApp.go из Go версии

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::{Child, Command as TokioCommand};
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{info, warn, error, debug};
use md5::{Md5, Digest};

use crate::error::{Error, Result};
use crate::models::{Template, Repository};
use crate::services::task_logger::{TaskLogger, TaskStatus};

/// Тип требований Galaxy
#[derive(Debug, Clone, Copy)]
pub enum GalaxyRequirementsType {
    Role,
    Collection,
}

impl GalaxyRequirementsType {
    fn to_string(&self) -> &'static str {
        match self {
            GalaxyRequirementsType::Role => "role",
            GalaxyRequirementsType::Collection => "collection",
        }
    }
}

/// AnsibleApp представляет приложение для выполнения Ansible playbook
pub struct AnsibleApp {
    /// Логгер
    pub logger: Box<dyn TaskLogger>,
    /// Playbook
    pub playbook: AnsiblePlaybook,
    /// Шаблон
    pub template: Template,
    /// Репозиторий
    pub repository: Repository,
}

/// AnsiblePlaybook для выполнения команд
pub struct AnsiblePlaybook {
    /// Логгер
    pub logger: Box<dyn TaskLogger>,
    /// Репозиторий
    pub repository: Repository,
    /// Шаблон
    pub template: Template,
    /// Рабочая директория
    pub work_dir: PathBuf,
}

impl AnsiblePlaybook {
    /// Создаёт новый AnsiblePlaybook
    pub fn new(
        logger: Box<dyn TaskLogger>,
        repository: Repository,
        template: Template,
        work_dir: PathBuf,
    ) -> Self {
        Self {
            logger,
            repository,
            template,
            work_dir,
        }
    }

    /// Получает путь к репозиторию
    fn get_repo_path(&self) -> PathBuf {
        self.work_dir.join("repository")
    }

    /// Получает директорию playbook
    fn get_playbook_dir(&self) -> PathBuf {
        self.get_repo_path().join(&self.template.playbook).parent().unwrap().to_path_buf()
    }

    /// Создаёт команду для выполнения
    fn make_cmd(&self, command: &str, args: Vec<String>, environment_vars: Vec<String>) -> TokioCommand {
        let mut cmd = TokioCommand::new(command);
        cmd.args(&args);
        cmd.current_dir(self.get_playbook_dir());
        
        // Добавляем переменные окружения
        for env_var in environment_vars {
            if let Some((key, value)) = env_var.split_once('=') {
                cmd.env(key, value);
            }
        }
        
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        cmd
    }

    /// Выполняет команду
    async fn run_cmd(&self, command: &str, args: Vec<String>) -> Result<()> {
        let mut cmd = self.make_cmd(command, args, vec![]);
        
        let mut child = cmd.spawn()?;
        
        // Читаем вывод
        if let Some(ref mut stdout) = child.stdout {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                self.logger.log(&line);
            }
        }
        
        if let Some(ref mut stderr) = child.stderr {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                self.logger.log(&line);
            }
        }
        
        let status = child.wait().await?;
        
        if !status.success() {
            return Err(Error::Other(format!("Command failed with status: {}", status)));
        }
        
        Ok(())
    }

    /// Запускает playbook
    pub async fn run_playbook(
        &self,
        cli_args: Vec<String>,
        environment_vars: Vec<String>,
        inputs: HashMap<String, String>,
        cb: Option<Box<dyn Fn(&Child) + Send>>,
    ) -> Result<()> {
        self.logger.log("Running Ansible playbook...");
        
        let mut args = vec![];
        
        // Extra vars
        if !inputs.is_empty() {
            let extra_vars = serde_json::to_string(&inputs).unwrap_or_default();
            args.push("-e".to_string());
            args.push(extra_vars);
        }
        
        // CLI аргументы
        args.extend(cli_args);
        
        // Playbook file
        args.push(self.template.playbook.clone());
        
        let mut cmd = self.make_cmd("ansible-playbook", args, environment_vars);
        
        let mut child = cmd.spawn()?;
        
        // Callback для процесса
        if let Some(callback) = cb {
            callback(&child);
        }
        
        let status = child.wait().await?;
        
        if !status.success() {
            return Err(Error::Other(format!("Playbook failed with status: {}", status)));
        }
        
        Ok(())
    }

    /// Запускает Galaxy команду
    pub async fn run_galaxy(&self, args: Vec<String>, environment_vars: Vec<String>) -> Result<()> {
        self.logger.log("Running Ansible Galaxy...");
        
        let mut cmd = self.make_cmd("ansible-galaxy", args, environment_vars);
        
        let status = cmd.spawn()?.wait().await?;
        
        if !status.success() {
            return Err(Error::Other(format!("Galaxy command failed with status: {}", status)));
        }
        
        Ok(())
    }
}

impl AnsibleApp {
    /// Создаёт новый AnsibleApp
    pub fn new(
        logger: Box<dyn TaskLogger>,
        template: Template,
        repository: Repository,
        work_dir: PathBuf,
    ) -> Self {
        let playbook = AnsiblePlaybook::new(
            Box::new(logger.clone()),
            repository.clone(),
            template.clone(),
            work_dir,
        );
        
        Self {
            logger,
            playbook,
            template,
            repository,
        }
    }

    /// Устанавливает логгер
    pub fn set_logger(&mut self, logger: Box<dyn TaskLogger>) -> Box<dyn TaskLogger> {
        let old_logger = Box::new(self.logger.clone());
        self.logger = logger;
        self.playbook.logger = Box::new(self.logger.clone());
        old_logger
    }

    /// Логирует сообщение
    pub fn log(&self, msg: &str) {
        self.logger.log(msg);
    }

    /// Получает путь к репозиторию
    fn get_repo_path(&self) -> PathBuf {
        self.playbook.get_repo_path()
    }

    /// Получает директорию playbook
    fn get_playbook_dir(&self) -> PathBuf {
        self.playbook.get_playbook_dir()
    }

    /// Вычисляет MD5 хеш файла
    fn get_md5_hash(filepath: &Path) -> Result<String> {
        let contents = fs::read(filepath)?;
        let mut hasher = Md5::new();
        hasher.update(&contents);
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    /// Проверяет изменились ли требования
    fn has_requirements_changes(requirements_path: &Path, hash_path: &Path) -> bool {
        // Читаем старый хеш
        let old_hash = match fs::read_to_string(hash_path) {
            Ok(content) => content,
            Err(_) => return true,
        };

        // Вычисляем новый хеш
        let new_hash = match Self::get_md5_hash(requirements_path) {
            Ok(hash) => hash,
            Err(_) => return true,
        };

        old_hash != new_hash
    }

    /// Записывает MD5 хеш
    fn write_md5_hash(requirements_path: &Path, hash_path: &Path) -> Result<()> {
        let new_hash = Self::get_md5_hash(requirements_path)?;
        fs::write(hash_path, new_hash)?;
        Ok(())
    }

    /// Устанавливает требования Galaxy
    pub async fn install_galaxy_requirements_file(
        &self,
        requirements_type: GalaxyRequirementsType,
        requirements_path: &Path,
        environment_vars: Vec<String>,
    ) -> Result<()> {
        let hash_path = requirements_path.with_extension(format!("yml.{}.md5", requirements_type.to_string()));

        if !requirements_path.exists() {
            self.log(&format!("No {} file found. Skip galaxy install process.", requirements_path.display()));
            return Ok(());
        }

        if Self::has_requirements_changes(requirements_path, &hash_path) {
            self.log(&format!("Installing {} requirements from {}", requirements_type.to_string(), requirements_path.display()));
            
            let args = vec![
                requirements_type.to_string().to_string(),
                "install".to_string(),
                "-r".to_string(),
                requirements_path.display().to_string(),
                "--force".to_string(),
            ];
            
            self.playbook.run_galaxy(args, environment_vars).await?;
            
            Self::write_md5_hash(requirements_path, &hash_path)?;
        } else {
            self.log(&format!("{} has no changes. Skip galaxy install process.", requirements_path.display()));
        }

        Ok(())
    }

    /// Устанавливает требования ролей
    pub async fn install_roles_requirements(&self, environment_vars: Vec<String>) -> Result<()> {
        let playbook_dir = self.get_playbook_dir();
        let repo_path = self.get_repo_path();

        // default roles path
        self.install_galaxy_requirements_file(
            GalaxyRequirementsType::Role,
            &playbook_dir.join("roles").join("requirements.yml"),
            environment_vars.clone(),
        ).await?;

        self.install_galaxy_requirements_file(
            GalaxyRequirementsType::Role,
            &playbook_dir.join("requirements.yml"),
            environment_vars.clone(),
        ).await?;

        // alternative roles path
        self.install_galaxy_requirements_file(
            GalaxyRequirementsType::Role,
            &repo_path.join("roles").join("requirements.yml"),
            environment_vars.clone(),
        ).await?;

        self.install_galaxy_requirements_file(
            GalaxyRequirementsType::Role,
            &repo_path.join("requirements.yml"),
            environment_vars,
        ).await?;

        Ok(())
    }

    /// Устанавливает требования коллекций
    pub async fn install_collections_requirements(&self, environment_vars: Vec<String>) -> Result<()> {
        let playbook_dir = self.get_playbook_dir();
        let repo_path = self.get_repo_path();

        // default collections path
        self.install_galaxy_requirements_file(
            GalaxyRequirementsType::Collection,
            &playbook_dir.join("collections").join("requirements.yml"),
            environment_vars.clone(),
        ).await?;

        self.install_galaxy_requirements_file(
            GalaxyRequirementsType::Collection,
            &playbook_dir.join("requirements.yml"),
            environment_vars.clone(),
        ).await?;

        // alternative collections path
        self.install_galaxy_requirements_file(
            GalaxyRequirementsType::Collection,
            &repo_path.join("collections").join("requirements.yml"),
            environment_vars.clone(),
        ).await?;

        self.install_galaxy_requirements_file(
            GalaxyRequirementsType::Collection,
            &repo_path.join("requirements.yml"),
            environment_vars,
        ).await?;

        Ok(())
    }

    /// Устанавливает зависимости
    pub async fn install_requirements(&self, args: crate::db_lib::LocalAppInstallingArgs) -> Result<()> {
        self.log("Installing Ansible requirements...");
        
        if let Err(e) = self.install_collections_requirements(args.environment_vars.clone()).await {
            self.log(&format!("Failed to install collections: {}", e));
        }
        
        if let Err(e) = self.install_roles_requirements(args.environment_vars).await {
            self.log(&format!("Failed to install roles: {}", e));
        }
        
        Ok(())
    }

    /// Запускает задачу
    pub async fn run(&self, args: crate::db_lib::LocalAppRunningArgs) -> Result<()> {
        // Получаем аргументы для "default" ключа
        let cli_args = args.cli_args.get("default").cloned().unwrap_or_default();
        
        self.playbook.run_playbook(
            cli_args,
            args.environment_vars,
            args.inputs,
            args.callback,
        ).await
    }

    /// Очищает ресурсы
    pub fn clear(&self) {
        // Ansible не требует очистки
    }
}

impl Drop for AnsibleApp {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::task_logger::BasicLogger;
    use std::sync::Arc;

    fn create_test_ansible_app() -> AnsibleApp {
        let logger = Box::new(BasicLogger::new());
        let template = Template::default();
        let repository = Repository::default();
        let work_dir = PathBuf::from("/tmp/test_ansible");
        
        AnsibleApp::new(logger, template, repository, work_dir)
    }

    #[test]
    fn test_ansible_app_creation() {
        let app = create_test_ansible_app();
        assert_eq!(app.template.playbook, "");
    }

    #[test]
    fn test_get_playbook_dir() {
        let app = create_test_ansible_app();
        let dir = app.get_playbook_dir();
        assert!(dir.ends_with("repository"));
    }

    #[test]
    fn test_get_md5_hash() {
        use std::fs;
        use tempfile::NamedTempFile;
        use std::io::Write;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();
        
        let hash = AnsibleApp::get_md5_hash(temp_file.path()).unwrap();
        assert_eq!(hash.len(), 32); // MD5 hash is 32 hex characters
    }
}
