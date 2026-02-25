//! Модуль выполнения задач
//!
//! Предоставляет инфраструктуру для запуска задач Ansible, Terraform, Bash, PowerShell и других.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command as TokioCommand;

use crate::error::{Error, Result};
use crate::models::{Inventory, Repository, Template, Task};
use crate::services::task_logger::{TaskLogger, TaskStatus};

/// Тип приложения для выполнения
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppType {
    Ansible,
    Terraform,
    Tofu,
    Terragrunt,
    Bash,
    PowerShell,
    Python,
    Pulumi,
}

impl std::fmt::Display for AppType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppType::Ansible => write!(f, "ansible"),
            AppType::Terraform => write!(f, "terraform"),
            AppType::Tofu => write!(f, "tofu"),
            AppType::Terragrunt => write!(f, "terragrunt"),
            AppType::Bash => write!(f, "bash"),
            AppType::PowerShell => write!(f, "powershell"),
            AppType::Python => write!(f, "python3"),
            AppType::Pulumi => write!(f, "pulumi"),
        }
    }
}

/// Аргументы для запуска приложения
#[derive(Debug, Clone)]
pub struct AppRunArgs {
    /// Аргументы командной строки
    pub cli_args: Vec<String>,
    /// Переменные окружения
    pub environment_vars: Vec<String>,
    /// Входные данные (для интерактивных команд)
    pub inputs: HashMap<String, String>,
}

/// Результат выполнения приложения
#[derive(Debug)]
pub struct AppRunResult {
    /// Код возврата
    pub exit_code: i32,
    /// Были ли ошибки
    pub has_errors: bool,
    /// Путь к выводу (логам)
    pub output_path: Option<String>,
}

/// Трейт для исполняемых приложений
#[async_trait::async_trait]
pub trait ExecutableApp: Send + Sync {
    /// Устанавливает логгер
    fn set_logger(&mut self, logger: Arc<dyn TaskLogger>) -> Result<()>;

    /// Устанавливает параметры задачи
    fn set_task(&mut self, task: &Task, template: &Template, repository: &Repository, inventory: &Inventory);

    /// Устанавливает рабочую директорию
    fn set_work_dir(&mut self, path: PathBuf);

    /// Получает рабочую директорию
    fn get_work_dir(&self) -> &Path;

    /// Устанавливает переменные окружения
    fn set_environment(&mut self, vars: Vec<String>);

    /// Устанавливает аргументы командной строки
    fn set_cli_args(&mut self, args: Vec<String>);

    /// Проверяет наличие зависимостей и устанавливает их
    async fn install_requirements(&mut self) -> Result<()>;

    /// Выполняет приложение
    async fn run(&mut self) -> Result<AppRunResult>;

    /// Очищает ресурсы после выполнения
    fn cleanup(&mut self) -> Result<()>;
}

/// Базовая структура для всех приложений
pub struct BaseApp {
    /// Логгер задач
    logger: Option<Arc<dyn TaskLogger>>,
    /// Шаблон задачи
    template: Option<Template>,
    /// Репозиторий
    repository: Option<Repository>,
    /// Инвентарь
    inventory: Option<Inventory>,
    /// Задача
    task: Option<Task>,
    /// Рабочая директория
    work_dir: PathBuf,
    /// Переменные окружения
    environment_vars: Vec<String>,
    /// Аргументы командной строки
    cli_args: Vec<String>,
}

impl BaseApp {
    /// Создаёт новую базовую структуру приложения
    pub fn new() -> Self {
        Self {
            logger: None,
            template: None,
            repository: None,
            inventory: None,
            task: None,
            work_dir: PathBuf::new(),
            environment_vars: Vec::new(),
            cli_args: Vec::new(),
        }
    }

    /// Получает логгер или создаёт заглушку
    fn get_logger(&self) -> Arc<dyn TaskLogger> {
        self.logger.clone().unwrap_or_else(|| {
            Arc::new(NullLogger)
        })
    }

    /// Получает рабочую директорию
    pub fn get_work_dir(&self) -> &Path {
        &self.work_dir
    }

    /// Получает полную директорию репозитория
    #[allow(dead_code)]
    fn get_repository_path(&self) -> PathBuf {
        if let Some(ref repo) = self.repository {
            // В реальной реализации здесь будет путь к репозиторию
            PathBuf::from(format!("/tmp/semaphore/repo_{}", repo.id))
        } else {
            PathBuf::from("/tmp/semaphore")
        }
    }

    /// Запускает команду и логирует вывод
    async fn run_command(
        &self,
        command: &str,
        args: &[String],
        env: &[String],
    ) -> Result<AppRunResult> {
        let logger = self.get_logger();

        logger.log(&format!("Запуск команды: {} {}", command, args.join(" ")));

        let mut cmd = TokioCommand::new(command);
        cmd.args(args);
        cmd.envs(env.iter().map(|e| {
            let parts: Vec<&str> = e.splitn(2, '=').collect();
            if parts.len() == 2 {
                (parts[0], parts[1])
            } else {
                (e.as_str(), "")
            }
        }));
        cmd.current_dir(&self.work_dir);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            Error::Other(format!("Ошибка запуска команды: {}", e))
        })?;

        // Читаем stdout
        if let Some(stdout) = child.stdout.take() {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            
            while let Ok(Some(line)) = lines.next_line().await {
                logger.log(&line);
            }
        }

        // Читаем stderr
        if let Some(stderr) = child.stderr.take() {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            
            while let Ok(Some(line)) = lines.next_line().await {
                logger.log(&format!("STDERR: {}", line));
            }
        }

        let status = child.wait().await.map_err(|e| {
            Error::Other(format!("Ошибка ожидания команды: {}", e))
        })?;

        let exit_code = status.code().unwrap_or(-1);
        let has_errors = !status.success();

        logger.log(&format!("Команда завершилась с кодом: {}", exit_code));

        Ok(AppRunResult {
            exit_code,
            has_errors,
            output_path: None,
        })
    }
}

impl Default for BaseApp {
    fn default() -> Self {
        Self::new()
    }
}

/// Заглушка логгера для использования по умолчанию
pub struct NullLogger;

impl TaskLogger for NullLogger {
    fn log(&self, _msg: &str) {}
    
    fn set_status(&self, _status: TaskStatus) {}
    
    fn get_status(&self) -> TaskStatus {
        TaskStatus::Running
    }
}

/// Ansible приложение
pub struct AnsibleApp {
    base: BaseApp,
    /// Путь к playbook
    playbook_path: PathBuf,
    /// Путь к inventory
    inventory_path: Option<PathBuf>,
    /// Дополнительные переменные
    extra_vars: HashMap<String, serde_json::Value>,
}

impl AnsibleApp {
    /// Создаёт новое Ansible приложение
    pub fn new() -> Self {
        Self {
            base: BaseApp::new(),
            playbook_path: PathBuf::new(),
            inventory_path: None,
            extra_vars: HashMap::new(),
        }
    }

    /// Устанавливает путь к playbook
    pub fn set_playbook(&mut self, path: PathBuf) {
        self.playbook_path = path;
    }

    /// Устанавливает путь к inventory
    pub fn set_inventory(&mut self, path: PathBuf) {
        self.inventory_path = Some(path);
    }

    /// Добавляет дополнительную переменную
    pub fn add_extra_var(&mut self, key: String, value: serde_json::Value) {
        self.extra_vars.insert(key, value);
    }

    /// Проверяет и устанавливает зависимости Ansible (roles, collections)
    async fn install_galaxy_requirements(&mut self) -> Result<()> {
        let logger = self.base.get_logger();
        let work_dir = self.base.get_work_dir();

        // Проверяем наличие requirements.yml
        let requirements_path = work_dir.join("requirements.yml");
        if !requirements_path.exists() {
            logger.log("requirements.yml не найден, пропускаем установку зависимостей");
            return Ok(());
        }

        logger.log("Установка зависимостей Ansible Galaxy...");

        let args = vec![
            "install".to_string(),
            "-r".to_string(),
            requirements_path.to_string_lossy().to_string(),
            "--force".to_string(),
        ];

        let result = self.base.run_command("ansible-galaxy", &args, &[]).await?;

        if result.has_errors {
            return Err(Error::Other("Ошибка установки зависимостей Ansible Galaxy".to_string()));
        }

        logger.log("Зависимости успешно установлены");
        Ok(())
    }
}

impl Default for AnsibleApp {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ExecutableApp for AnsibleApp {
    fn set_logger(&mut self, logger: Arc<dyn TaskLogger>) -> Result<()> {
        self.base.logger = Some(logger);
        Ok(())
    }

    fn set_task(&mut self, task: &Task, template: &Template, repository: &Repository, inventory: &Inventory) {
        self.base.task = Some(task.clone());
        self.base.template = Some(template.clone());
        self.base.repository = Some(repository.clone());
        self.base.inventory = Some(inventory.clone());
        
        if let Some(ref tpl) = self.base.template {
            self.playbook_path = PathBuf::from(&tpl.playbook);
        }
    }

    fn set_work_dir(&mut self, path: PathBuf) {
        self.base.work_dir = path;
    }

    fn get_work_dir(&self) -> &Path {
        &self.base.work_dir
    }

    fn set_environment(&mut self, vars: Vec<String>) {
        self.base.environment_vars = vars;
    }

    fn set_cli_args(&mut self, args: Vec<String>) {
        self.base.cli_args = args;
    }

    async fn install_requirements(&mut self) -> Result<()> {
        self.install_galaxy_requirements().await
    }

    async fn run(&mut self) -> Result<AppRunResult> {
        let logger = self.base.get_logger();
        
        logger.log("Запуск Ansible playbook...");

        // Формируем команду ansible-playbook
        let mut args = vec![
            self.playbook_path.to_string_lossy().to_string(),
        ];

        // Добавляем inventory
        if let Some(ref inv_path) = self.inventory_path {
            args.push("-i".to_string());
            args.push(inv_path.to_string_lossy().to_string());
        }

        // Добавляем extra vars
        if !self.extra_vars.is_empty() {
            let extra_vars_json = serde_json::to_string(&self.extra_vars)
                .map_err(|e| Error::Other(format!("Ошибка сериализации extra_vars: {}", e)))?;
            args.push("--extra-vars".to_string());
            args.push(extra_vars_json);
        }

        // Добавляем пользовательские аргументы
        args.extend(self.base.cli_args.clone());

        // Добавляем переменные окружения
        let mut env = self.base.environment_vars.clone();
        env.push("ANSIBLE_FORCE_COLOR=0".to_string());
        env.push("PYTHONUNBUFFERED=1".to_string());

        self.base.run_command("ansible-playbook", &args, &env).await
    }

    fn cleanup(&mut self) -> Result<()> {
        // Очистка временных файлов
        Ok(())
    }
}

/// Terraform приложение
pub struct TerraformApp {
    base: BaseApp,
    /// Рабочее пространство
    workspace: String,
    /// Флаг auto-approve
    auto_approve: bool,
    /// Флаг plan only
    plan_only: bool,
}

impl TerraformApp {
    /// Создаёт новое Terraform приложение
    pub fn new() -> Self {
        Self {
            base: BaseApp::new(),
            workspace: "default".to_string(),
            auto_approve: false,
            plan_only: false,
        }
    }

    /// Устанавливает рабочее пространство
    pub fn set_workspace(&mut self, workspace: String) {
        self.workspace = workspace;
    }

    /// Устанавливает флаг auto-approve
    pub fn set_auto_approve(&mut self, value: bool) {
        self.auto_approve = value;
    }

    /// Устанавливает флаг plan only
    pub fn set_plan_only(&mut self, value: bool) {
        self.plan_only = value;
    }

    /// Инициализирует Terraform
    async fn init(&mut self) -> Result<()> {
        let logger = self.base.get_logger();
        logger.log("Инициализация Terraform...");

        let args = vec!["init".to_string(), "-input=false".to_string()];
        let result = self.base.run_command("terraform", &args, &[]).await?;

        if result.has_errors {
            return Err(Error::Other("Ошибка инициализации Terraform".to_string()));
        }

        Ok(())
    }

    /// Выбирает рабочее пространство
    async fn select_workspace(&mut self) -> Result<()> {
        if self.workspace == "default" {
            return Ok(());
        }

        let logger = self.base.get_logger();
        logger.log(&format!("Выбор workspace: {}", self.workspace));

        let args = vec![
            "workspace".to_string(),
            "select".to_string(),
            "-or-create=true".to_string(),
            self.workspace.clone(),
        ];

        let result = self.base.run_command("terraform", &args, &[]).await?;

        if result.has_errors {
            return Err(Error::Other("Ошибка выбора workspace".to_string()));
        }

        Ok(())
    }

    /// Выполняет terraform plan
    async fn plan(&mut self) -> Result<bool> {
        let logger = self.base.get_logger();
        logger.log("Выполнение terraform plan...");

        let args = vec!["plan".to_string()];
        let result = self.base.run_command("terraform", &args, &[]).await?;

        // Проверяем, есть ли изменения
        let has_changes = !result.has_errors;
        
        if self.plan_only {
            logger.log("Режим plan-only, завершаем выполнение");
            return Ok(false); // Нет необходимости в apply
        }

        Ok(has_changes)
    }

    /// Выполняет terraform apply
    async fn apply(&mut self) -> Result<()> {
        let logger = self.base.get_logger();
        logger.log("Выполнение terraform apply...");

        let mut args = vec!["apply".to_string()];
        
        if self.auto_approve {
            args.push("-auto-approve".to_string());
        }

        let result = self.base.run_command("terraform", &args, &[]).await?;

        if result.has_errors {
            return Err(Error::Other("Ошибка выполнения terraform apply".to_string()));
        }

        Ok(())
    }
}

impl Default for TerraformApp {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ExecutableApp for TerraformApp {
    fn set_logger(&mut self, logger: Arc<dyn TaskLogger>) -> Result<()> {
        self.base.logger = Some(logger);
        Ok(())
    }

    fn set_task(&mut self, task: &Task, template: &Template, repository: &Repository, inventory: &Inventory) {
        self.base.task = Some(task.clone());
        self.base.template = Some(template.clone());
        self.base.repository = Some(repository.clone());
        self.base.inventory = Some(inventory.clone());

        if let Some(ref inv) = self.base.inventory {
            // Используем name инвентаря как workspace
            if !inv.name.is_empty() {
                self.workspace = inv.name.clone();
            }
        }
    }

    fn set_work_dir(&mut self, path: PathBuf) {
        self.base.work_dir = path;
    }

    fn get_work_dir(&self) -> &Path {
        &self.base.work_dir
    }

    fn set_environment(&mut self, vars: Vec<String>) {
        self.base.environment_vars = vars;
    }

    fn set_cli_args(&mut self, args: Vec<String>) {
        self.base.cli_args = args;
    }

    async fn install_requirements(&mut self) -> Result<()> {
        // Terraform не имеет зависимостей в традиционном понимании
        // Но можем проверить наличие providers
        self.init().await
    }

    async fn run(&mut self) -> Result<AppRunResult> {
        let logger = self.base.get_logger();
        
        // Инициализация
        self.init().await?;
        
        // Выбор workspace
        self.select_workspace().await?;
        
        // Plan
        let has_changes = self.plan().await?;
        
        if !has_changes || self.plan_only {
            logger.log("Изменений нет или режим plan-only");
            return Ok(AppRunResult {
                exit_code: 0,
                has_errors: false,
                output_path: None,
            });
        }

        // Apply
        if self.auto_approve {
            self.apply().await?;
        } else {
            // В реальном режиме ожидаем подтверждения
            logger.log("Ожидание подтверждения для apply...");
            // Здесь должна быть логика ожидания подтверждения
        }

        Ok(AppRunResult {
            exit_code: 0,
            has_errors: false,
            output_path: None,
        })
    }

    fn cleanup(&mut self) -> Result<()> {
        // Очистка временных файлов .terraform
        Ok(())
    }
}

/// Bash/Shell приложение
pub struct ShellApp {
    base: BaseApp,
    /// Тип оболочки
    shell_type: AppType,
    /// Скрипт для выполнения
    script_path: PathBuf,
}

impl ShellApp {
    /// Создаёт новое Shell приложение
    pub fn new(shell_type: AppType) -> Self {
        Self {
            base: BaseApp::new(),
            shell_type,
            script_path: PathBuf::new(),
        }
    }

    /// Устанавливает путь к скрипту
    pub fn set_script(&mut self, path: PathBuf) {
        self.script_path = path;
    }

    /// Получает команду для выполнения
    fn get_command(&self) -> &'static str {
        match self.shell_type {
            AppType::Bash => "bash",
            AppType::PowerShell => "pwsh",
            AppType::Python => "python3",
            _ => "bash",
        }
    }
}

#[async_trait::async_trait]
impl ExecutableApp for ShellApp {
    fn set_logger(&mut self, logger: Arc<dyn TaskLogger>) -> Result<()> {
        self.base.logger = Some(logger);
        Ok(())
    }

    fn set_task(&mut self, task: &Task, template: &Template, repository: &Repository, inventory: &Inventory) {
        self.base.task = Some(task.clone());
        self.base.template = Some(template.clone());
        self.base.repository = Some(repository.clone());
        self.base.inventory = Some(inventory.clone());
        
        if let Some(ref tpl) = self.base.template {
            self.script_path = PathBuf::from(&tpl.playbook);
        }
    }

    fn set_work_dir(&mut self, path: PathBuf) {
        self.base.work_dir = path;
    }

    fn get_work_dir(&self) -> &Path {
        &self.base.work_dir
    }

    fn set_environment(&mut self, vars: Vec<String>) {
        self.base.environment_vars = vars;
    }

    fn set_cli_args(&mut self, args: Vec<String>) {
        self.base.cli_args = args;
    }

    async fn install_requirements(&mut self) -> Result<()> {
        // Shell приложения обычно не имеют зависимостей
        Ok(())
    }

    async fn run(&mut self) -> Result<AppRunResult> {
        let logger = self.base.get_logger();
        logger.log(&format!("Запуск скрипта: {:?}", self.script_path));

        let command = self.get_command();
        let mut args = vec![self.script_path.to_string_lossy().to_string()];
        args.extend(self.base.cli_args.clone());

        self.base.run_command(command, &args, &self.base.environment_vars).await
    }

    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Фабрика для создания приложений
pub struct AppFactory;

impl AppFactory {
    /// Создаёт приложение нужного типа
    pub fn create(app_type: AppType) -> Box<dyn ExecutableApp> {
        match app_type {
            AppType::Ansible => Box::new(AnsibleApp::new()),
            AppType::Terraform | AppType::Tofu => Box::new(TerraformApp::new()),
            AppType::Terragrunt => Box::new(TerraformApp::new()),
            AppType::Bash | AppType::PowerShell | AppType::Python => {
                Box::new(ShellApp::new(app_type))
            }
            _ => Box::new(AnsibleApp::new()), // По умолчанию Ansible
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_type_display() {
        assert_eq!(AppType::Ansible.to_string(), "ansible");
        assert_eq!(AppType::Terraform.to_string(), "terraform");
        assert_eq!(AppType::Bash.to_string(), "bash");
    }

    #[test]
    fn test_base_app_creation() {
        let app = BaseApp::new();
        assert!(app.logger.is_none());
        assert!(app.template.is_none());
    }

    #[test]
    fn test_ansible_app_creation() {
        let app = AnsibleApp::new();
        assert!(app.base.logger.is_none());
        assert!(app.extra_vars.is_empty());
    }

    #[test]
    fn test_terraform_app_creation() {
        let app = TerraformApp::new();
        assert_eq!(app.workspace, "default");
        assert!(!app.auto_approve);
        assert!(!app.plan_only);
    }

    #[test]
    fn test_shell_app_creation() {
        let app = ShellApp::new(AppType::Bash);
        assert_eq!(app.shell_type, AppType::Bash);
    }
}
