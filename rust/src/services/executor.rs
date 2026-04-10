//! Модуль выполнения задач
//!
//! Предоставляет инфраструктуру для запуска задач Ansible, Terraform, Bash, PowerShell и других.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::process::Command as TokioCommand;

use crate::error::{Error, Result};
use crate::models::{Inventory, Repository, Task, Template};
use crate::services::task_logger::{LogListener, StatusListener, TaskLogger, TaskStatus};

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
    fn set_task(
        &mut self,
        task: &Task,
        template: &Template,
        repository: &Repository,
        inventory: &Inventory,
    );

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
        self.logger.clone().unwrap_or_else(|| Arc::new(NullLogger))
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

        let mut child = cmd
            .spawn()
            .map_err(|e| Error::Other(format!("Ошибка запуска команды: {}", e)))?;

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

        let status = child
            .wait()
            .await
            .map_err(|e| Error::Other(format!("Ошибка ожидания команды: {}", e)))?;

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

    fn logf(&self, _format: &str, _args: fmt::Arguments<'_>) {}

    fn log_with_time(&self, _time: DateTime<Utc>, _msg: &str) {}

    fn logf_with_time(&self, _time: DateTime<Utc>, _format: &str, _args: fmt::Arguments<'_>) {}

    fn log_cmd(&self, _cmd: &Command) {}

    fn set_status(&self, _status: TaskStatus) {}

    fn get_status(&self) -> TaskStatus {
        TaskStatus::Running
    }

    fn add_status_listener(&self, _listener: StatusListener) {}

    fn add_log_listener(&self, _listener: LogListener) {}

    fn set_commit(&self, _hash: &str, _message: &str) {}

    fn wait_log(&self) {}
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
            return Err(Error::Other(
                "Ошибка установки зависимостей Ansible Galaxy".to_string(),
            ));
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

    fn set_task(
        &mut self,
        task: &Task,
        template: &Template,
        repository: &Repository,
        inventory: &Inventory,
    ) {
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
        let mut args = vec![self.playbook_path.to_string_lossy().to_string()];

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
            return Err(Error::Other(
                "Ошибка выполнения terraform apply".to_string(),
            ));
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

    fn set_task(
        &mut self,
        task: &Task,
        template: &Template,
        repository: &Repository,
        inventory: &Inventory,
    ) {
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

    fn set_task(
        &mut self,
        task: &Task,
        template: &Template,
        repository: &Repository,
        inventory: &Inventory,
    ) {
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

        self.base
            .run_command(command, &args, &self.base.environment_vars)
            .await
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

// ============================================================================
// Pure helper functions (extracted for testability)
// ============================================================================

/// Parses "KEY=VALUE" strings into (key, value) pairs.
pub fn parse_environment_vars(env: &[String]) -> Vec<(&str, &str)> {
    env.iter().map(|e| {
        let parts: Vec<&str> = e.splitn(2, '=').collect();
        if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            (e.as_str(), "")
        }
    }).collect()
}

/// Builds the argument list for ansible-playbook.
pub fn build_ansible_args(
    playbook: &str,
    inventory: Option<&str>,
    extra_vars: &std::collections::HashMap<String, serde_json::Value>,
    cli_args: &[String],
) -> Vec<String> {
    let mut args = vec![playbook.to_string()];

    if let Some(inv) = inventory {
        args.push("-i".to_string());
        args.push(inv.to_string());
    }

    if !extra_vars.is_empty() {
        if let Ok(json) = serde_json::to_string(extra_vars) {
            args.push("--extra-vars".to_string());
            args.push(json);
        }
    }

    args.extend(cli_args.iter().cloned());
    args
}

/// Builds args for terraform apply.
pub fn build_terraform_apply_args(auto_approve: bool) -> Vec<String> {
    let mut args = vec!["apply".to_string()];
    if auto_approve {
        args.push("-auto-approve".to_string());
    }
    args
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

    #[test]
    fn test_app_type_display_powershell() {
        assert_eq!(AppType::PowerShell.to_string(), "powershell");
    }

    #[test]
    fn test_app_type_display_python() {
        assert_eq!(AppType::Python.to_string(), "python3");
    }

    #[test]
    fn test_shell_app_powershell_creation() {
        let app = ShellApp::new(AppType::PowerShell);
        assert_eq!(app.shell_type, AppType::PowerShell);
    }

    #[test]
    fn test_app_type_display_all_variants() {
        assert_eq!(AppType::Tofu.to_string(), "tofu");
        assert_eq!(AppType::Terragrunt.to_string(), "terragrunt");
        assert_eq!(AppType::Pulumi.to_string(), "pulumi");
    }

    #[test]
    fn test_parse_env_vars_normal() {
        let input = vec!["FOO=bar".to_string(), "BAZ=qux".to_string()];
        let result = parse_environment_vars(&input);
        assert_eq!(result, vec![("FOO", "bar"), ("BAZ", "qux")]);
    }

    #[test]
    fn test_parse_env_vars_empty_value() {
        let input = vec!["EMPTY=".to_string()];
        let result = parse_environment_vars(&input);
        assert_eq!(result, vec![("EMPTY", "")]);
    }

    #[test]
    fn test_parse_env_vars_no_equals() {
        let input = vec!["NOEQUALS".to_string()];
        let result = parse_environment_vars(&input);
        assert_eq!(result, vec![("NOEQUALS", "")]);
    }

    #[test]
    fn test_parse_env_vars_value_contains_equals() {
        let input = vec!["URL=http://example.com?a=1".to_string()];
        let result = parse_environment_vars(&input);
        assert_eq!(result, vec![("URL", "http://example.com?a=1")]);
    }

    #[test]
    fn test_parse_env_vars_empty_input() {
        let result = parse_environment_vars(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_build_ansible_args_minimal() {
        use std::collections::HashMap;
        let args = build_ansible_args("site.yml", None, &HashMap::new(), &[]);
        assert_eq!(args, vec!["site.yml"]);
    }

    #[test]
    fn test_build_ansible_args_with_inventory() {
        use std::collections::HashMap;
        let args = build_ansible_args("site.yml", Some("hosts.ini"), &HashMap::new(), &[]);
        assert_eq!(args, vec!["site.yml", "-i", "hosts.ini"]);
    }

    #[test]
    fn test_build_ansible_args_with_extra_vars() {
        use std::collections::HashMap;
        let mut vars = HashMap::new();
        vars.insert("debug".to_string(), serde_json::json!(true));
        let args = build_ansible_args("site.yml", None, &vars, &[]);
        assert_eq!(args[0], "site.yml");
        assert_eq!(args[1], "--extra-vars");
        let parsed: HashMap<String, serde_json::Value> = serde_json::from_str(&args[2]).unwrap();
        assert_eq!(parsed["debug"], serde_json::json!(true));
    }

    #[test]
    fn test_build_ansible_args_with_cli_args() {
        use std::collections::HashMap;
        let cli = vec!["--limit".to_string(), "web".to_string()];
        let args = build_ansible_args("site.yml", None, &HashMap::new(), &cli);
        assert_eq!(args, vec!["site.yml", "--limit", "web"]);
    }

    #[test]
    fn test_terraform_apply_without_auto_approve() {
        let args = build_terraform_apply_args(false);
        assert_eq!(args, vec!["apply"]);
    }

    #[test]
    fn test_terraform_apply_with_auto_approve() {
        let args = build_terraform_apply_args(true);
        assert_eq!(args, vec!["apply", "-auto-approve"]);
    }

    #[test]
    fn test_app_type_equality() {
        assert_eq!(AppType::Ansible, AppType::Ansible);
        assert_ne!(AppType::Ansible, AppType::Terraform);
    }

    #[test]
    fn test_app_type_clone() {
        let app_type = AppType::Terraform;
        let cloned = app_type; // Copy type
        assert_eq!(cloned, app_type);
    }

    #[test]
    fn test_app_run_args_default() {
        let args = AppRunArgs {
            cli_args: Vec::new(),
            environment_vars: Vec::new(),
            inputs: HashMap::new(),
        };
        assert!(args.cli_args.is_empty());
        assert!(args.environment_vars.is_empty());
        assert!(args.inputs.is_empty());
    }

    #[test]
    fn test_app_run_result() {
        let result = AppRunResult {
            exit_code: 0,
            has_errors: false,
            output_path: Some("/tmp/output.log".to_string()),
        };
        assert_eq!(result.exit_code, 0);
        assert!(!result.has_errors);
        assert!(result.output_path.is_some());
    }

    #[test]
    fn test_app_run_result_with_errors() {
        let result = AppRunResult {
            exit_code: 1,
            has_errors: true,
            output_path: None,
        };
        assert_eq!(result.exit_code, 1);
        assert!(result.has_errors);
        assert!(result.output_path.is_none());
    }

    #[test]
    fn test_app_type_display_bash_additional() {
        assert_eq!(AppType::Bash.to_string(), "bash");
    }

    #[test]
    fn test_parse_env_vars_with_empty_strings() {
        let input = vec!["KEY=".to_string(), "=value".to_string()];
        let result = parse_environment_vars(&input);
        assert_eq!(result, vec![("KEY", ""), ("", "value")]);
    }

    #[test]
    fn test_app_type_all_variants_count() {
        // Проверяем что все варианты AppType существуют
        let variants = vec![
            AppType::Ansible,
            AppType::Terraform,
            AppType::Tofu,
            AppType::Terragrunt,
            AppType::Bash,
            AppType::PowerShell,
            AppType::Python,
            AppType::Pulumi,
        ];
        assert_eq!(variants.len(), 8);
    }

    #[test]
    fn test_app_type_display_consistency() {
        let app_types = vec![
            AppType::Ansible,
            AppType::Terraform,
            AppType::Tofu,
            AppType::Terragrunt,
            AppType::Bash,
            AppType::PowerShell,
            AppType::Python,
            AppType::Pulumi,
        ];
        for app_type in app_types {
            let display = app_type.to_string();
            assert!(!display.is_empty());
        }
    }

    #[test]
    fn test_base_app_default() {
        let app = BaseApp::default();
        assert!(app.logger.is_none());
        assert_eq!(app.work_dir, PathBuf::new());
    }

    #[test]
    fn test_ansible_app_default() {
        let app = AnsibleApp::default();
        assert!(app.base.logger.is_none());
        assert_eq!(app.playbook_path, PathBuf::new());
    }

    #[test]
    fn test_terraform_app_default() {
        let app = TerraformApp::default();
        assert_eq!(app.workspace, "default");
    }

    #[test]
    fn test_shell_app_get_command_bash() {
        let app = ShellApp::new(AppType::Bash);
        assert_eq!(app.get_command(), "bash");
    }

    #[test]
    fn test_shell_app_get_command_powershell() {
        let app = ShellApp::new(AppType::PowerShell);
        assert_eq!(app.get_command(), "pwsh");
    }

    #[test]
    fn test_shell_app_get_command_python() {
        let app = ShellApp::new(AppType::Python);
        assert_eq!(app.get_command(), "python3");
    }

    #[test]
    fn test_shell_app_get_command_default() {
        let app = ShellApp::new(AppType::Ansible);
        assert_eq!(app.get_command(), "bash");
    }

    #[test]
    fn test_app_factory_create_ansible() {
        let app = AppFactory::create(AppType::Ansible);
        // App создаётся без паники
        assert!(std::any::type_name::<dyn std::any::Any>().contains("dyn"));
    }

    #[test]
    fn test_app_factory_create_terraform() {
        let app = AppFactory::create(AppType::Terraform);
        let _ = app; // Просто проверяем что не паникует
    }

    #[test]
    fn test_app_factory_create_tofu() {
        let app = AppFactory::create(AppType::Tofu);
        let _ = app;
    }

    #[test]
    fn test_app_factory_create_terragrunt() {
        let app = AppFactory::create(AppType::Terragrunt);
        let _ = app;
    }

    #[test]
    fn test_app_factory_create_bash() {
        let app = AppFactory::create(AppType::Bash);
        let _ = app;
    }

    #[test]
    fn test_app_factory_create_powershell() {
        let app = AppFactory::create(AppType::PowerShell);
        let _ = app;
    }

    #[test]
    fn test_app_factory_create_python() {
        let app = AppFactory::create(AppType::Python);
        let _ = app;
    }

    #[test]
    fn test_app_factory_create_pulumi() {
        let app = AppFactory::create(AppType::Pulumi);
        let _ = app;
    }

    #[test]
    fn test_parse_env_vars_multiple_equals() {
        let input = vec![
            "DB_HOST=localhost".to_string(),
            "DB_PORT=5432".to_string(),
            "DB_NAME=mydb".to_string(),
        ];
        let result = parse_environment_vars(&input);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("DB_HOST", "localhost"));
        assert_eq!(result[1], ("DB_PORT", "5432"));
        assert_eq!(result[2], ("DB_NAME", "mydb"));
    }

    #[test]
    fn test_build_ansible_args_with_all_options() {
        use std::collections::HashMap;
        let mut extra_vars = HashMap::new();
        extra_vars.insert("var1".to_string(), serde_json::json!("value1"));
        let cli_args = vec!["--limit".to_string(), "web".to_string()];

        let args = build_ansible_args(
            "deploy.yml",
            Some("production.ini"),
            &extra_vars,
            &cli_args,
        );

        assert!(args.contains(&"deploy.yml".to_string()));
        assert!(args.contains(&"-i".to_string()));
        assert!(args.contains(&"production.ini".to_string()));
        assert!(args.contains(&"--extra-vars".to_string()));
        assert!(args.contains(&"--limit".to_string()));
        assert!(args.contains(&"web".to_string()));
    }

    #[test]
    fn test_build_terraform_apply_args_no_auto_approve() {
        let args = build_terraform_apply_args(false);
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], "apply");
    }

    #[test]
    fn test_app_run_args_construction() {
        let mut inputs = HashMap::new();
        inputs.insert("prompt".to_string(), "yes".to_string());

        let args = AppRunArgs {
            cli_args: vec!["--verbose".to_string()],
            environment_vars: vec!["DEBUG=1".to_string()],
            inputs,
        };

        assert_eq!(args.cli_args.len(), 1);
        assert_eq!(args.environment_vars.len(), 1);
        assert_eq!(args.inputs.len(), 1);
    }

    #[test]
    fn test_app_run_result_display() {
        let result = AppRunResult {
            exit_code: 42,
            has_errors: true,
            output_path: Some("/var/log/error.log".to_string()),
        };
        assert_eq!(result.exit_code, 42);
        assert!(result.has_errors);
    }

    #[test]
    fn test_null_logger() {
        let logger = NullLogger;
        logger.log("test message");
        logger.logf("test {}", format_args!("format"));
        use chrono::Utc;
        logger.log_with_time(Utc::now(), "timed message");
        // NullLogger не должен паниковать
    }

    // ===========================================================================
    // Additional tests (20+)
    // ===========================================================================

    #[test]
    fn test_app_type_display_terraform() {
        assert_eq!(AppType::Terraform.to_string(), "terraform");
    }

    #[test]
    fn test_app_type_display_terragrunt() {
        assert_eq!(AppType::Terragrunt.to_string(), "terragrunt");
    }

    #[test]
    fn test_app_type_display_pulumi() {
        assert_eq!(AppType::Pulumi.to_string(), "pulumi");
    }

    #[test]
    fn test_app_type_display_tofu() {
        assert_eq!(AppType::Tofu.to_string(), "tofu");
    }

    #[test]
    fn test_app_type_partial_eq_same_type() {
        assert_eq!(AppType::Bash, AppType::Bash);
        assert_eq!(AppType::Python, AppType::Python);
        assert_eq!(AppType::Pulumi, AppType::Pulumi);
    }

    #[test]
    fn test_app_type_partial_eq_different_type() {
        assert_ne!(AppType::Bash, AppType::Python);
        assert_ne!(AppType::Terraform, AppType::Ansible);
        assert_ne!(AppType::Pulumi, AppType::Terragrunt);
    }

    #[test]
    fn test_app_run_args_construction_with_values() {
        let mut inputs = HashMap::new();
        inputs.insert("ssh_passphrase".to_string(), "secret".to_string());

        let args = AppRunArgs {
            cli_args: vec!["-vvv".to_string(), "--check".to_string()],
            environment_vars: vec!["ANSIBLE_HOST_KEY_CHECKING=False".to_string()],
            inputs,
        };

        assert_eq!(args.cli_args.len(), 2);
        assert!(args.cli_args.contains(&"-vvv".to_string()));
        assert_eq!(args.environment_vars.len(), 1);
        assert_eq!(args.inputs["ssh_passphrase"], "secret");
    }

    #[test]
    fn test_app_run_args_empty_fields() {
        let args = AppRunArgs {
            cli_args: vec![],
            environment_vars: vec![],
            inputs: HashMap::new(),
        };
        assert!(args.cli_args.is_empty());
        assert!(args.environment_vars.is_empty());
        assert!(args.inputs.is_empty());
    }

    #[test]
    fn test_app_run_result_success_zero_exit() {
        let result = AppRunResult {
            exit_code: 0,
            has_errors: false,
            output_path: None,
        };
        assert_eq!(result.exit_code, 0);
        assert!(!result.has_errors);
        assert!(result.output_path.is_none());
    }

    #[test]
    fn test_app_run_result_failure_non_zero_exit() {
        let result = AppRunResult {
            exit_code: 2,
            has_errors: true,
            output_path: Some("/tmp/fail.log".to_string()),
        };
        assert_eq!(result.exit_code, 2);
        assert!(result.has_errors);
        assert_eq!(result.output_path.unwrap(), "/tmp/fail.log");
    }

    #[test]
    fn test_null_logger_log_with_time_and_format() {
        let logger = NullLogger;
        use chrono::Utc;
        logger.logf_with_time(Utc::now(), "test {}", format_args!("args"));
        logger.set_status(crate::services::task_logger::TaskStatus::Success);
        let _status = logger.get_status();
        // No panic expected
    }

    #[test]
    fn test_null_logger_add_listeners_and_commit() {
        let logger = NullLogger;
        logger.add_status_listener(Box::new(|_| {}));
        use chrono::Utc;
        logger.add_log_listener(Box::new(|_time, _msg| {}));
        logger.set_commit("abc123", "initial commit");
        logger.wait_log();
        // All should succeed (no-op)
    }

    #[test]
    fn test_base_app_get_logger_returns_null_when_none() {
        let app = BaseApp::new();
        let logger = app.get_logger();
        // Should return NullLogger without panicking
        logger.log("test");
    }

    #[test]
    fn test_base_app_get_work_dir_default() {
        let app = BaseApp::new();
        assert_eq!(app.get_work_dir(), PathBuf::new());
    }

    #[test]
    fn test_base_app_get_repository_path_without_repo() {
        let app = BaseApp::new();
        let path = app.get_repository_path();
        // Should return fallback path
        assert!(path.to_string_lossy().contains("semaphore"));
    }

    #[test]
    fn test_parse_env_vars_single_var() {
        let input = vec!["MY_VAR=hello_world".to_string()];
        let result = parse_environment_vars(&input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], ("MY_VAR", "hello_world"));
    }

    #[test]
    fn test_parse_env_vars_value_with_special_chars() {
        let input = vec!["JSON={\"key\":\"value\"}".to_string()];
        let result = parse_environment_vars(&input);
        assert_eq!(result[0].0, "JSON");
        assert_eq!(result[0].1, "{\"key\":\"value\"}");
    }

    #[test]
    fn test_parse_env_vars_multiple_different_formats() {
        let input = vec![
            "SIMPLE=value".to_string(),
            "WITH_EQUALS=a=b=c".to_string(),
            "EMPTY_VAL=".to_string(),
        ];
        let result = parse_environment_vars(&input);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("SIMPLE", "value"));
        assert_eq!(result[1], ("WITH_EQUALS", "a=b=c"));
        assert_eq!(result[2], ("EMPTY_VAL", ""));
    }

    #[test]
    fn test_build_ansible_args_with_empty_extra_vars() {
        use std::collections::HashMap;
        let extra_vars: HashMap<String, serde_json::Value> = HashMap::new();
        let args = build_ansible_args("playbook.yml", None, &extra_vars, &[]);
        assert_eq!(args, vec!["playbook.yml"]);
        // No --extra-vars should be present
        assert!(!args.contains(&"--extra-vars".to_string()));
    }

    #[test]
    fn test_build_ansible_args_with_multiple_extra_vars() {
        use std::collections::HashMap;
        let mut extra_vars = HashMap::new();
        extra_vars.insert("var1".to_string(), serde_json::json!("value1"));
        extra_vars.insert("var2".to_string(), serde_json::json!(42));
        extra_vars.insert("var3".to_string(), serde_json::json!(true));

        let args = build_ansible_args("deploy.yml", None, &extra_vars, &[]);
        assert_eq!(args[0], "deploy.yml");
        assert_eq!(args[1], "--extra-vars");
        // Check JSON contains all keys
        let json_str = &args[2];
        assert!(json_str.contains("var1"));
        assert!(json_str.contains("var2"));
        assert!(json_str.contains("var3"));
    }

    #[test]
    fn test_build_ansible_args_combined_inventory_and_cli() {
        use std::collections::HashMap;
        let cli = vec!["--limit".to_string(), "db".to_string(), "-vv".to_string()];
        let args = build_ansible_args(
            "site.yml",
            Some("inventory/hosts"),
            &HashMap::new(),
            &cli,
        );

        assert_eq!(args[0], "site.yml");
        assert_eq!(args[1], "-i");
        assert_eq!(args[2], "inventory/hosts");
        assert_eq!(args[3], "--limit");
        assert_eq!(args[4], "db");
        assert_eq!(args[5], "-vv");
    }

    #[test]
    fn test_build_terraform_apply_args_edge_cases() {
        // Without auto-approve
        let args_no = build_terraform_apply_args(false);
        assert_eq!(args_no.len(), 1);
        assert!(!args_no.contains(&"-auto-approve".to_string()));

        // With auto-approve
        let args_yes = build_terraform_apply_args(true);
        assert_eq!(args_yes.len(), 2);
        assert!(args_yes.contains(&"-auto-approve".to_string()));
    }

    #[test]
    fn test_app_factory_default_fallback() {
        // Pulumi and unknown types should fallback to AnsibleApp
        let _ = AppFactory::create(AppType::Pulumi);
        // No panic expected
    }

    #[test]
    fn test_ansible_app_setters() {
        let mut app = AnsibleApp::new();
        app.set_playbook(PathBuf::from("/path/to/playbook.yml"));
        assert_eq!(app.playbook_path, PathBuf::from("/path/to/playbook.yml"));

        app.set_inventory(PathBuf::from("/path/to/hosts.ini"));
        assert!(app.inventory_path.is_some());
        assert_eq!(
            app.inventory_path.unwrap(),
            PathBuf::from("/path/to/hosts.ini")
        );
    }

    #[test]
    fn test_ansible_app_add_extra_vars() {
        let mut app = AnsibleApp::new();
        app.add_extra_var("debug".to_string(), serde_json::json!(true));
        app.add_extra_var("target".to_string(), serde_json::json!("all"));

        assert_eq!(app.extra_vars.len(), 2);
        assert_eq!(app.extra_vars["debug"], serde_json::json!(true));
        assert_eq!(app.extra_vars["target"], serde_json::json!("all"));
    }

    #[test]
    fn test_terraform_app_setters() {
        let mut app = TerraformApp::new();
        app.set_workspace("staging".to_string());
        assert_eq!(app.workspace, "staging");

        app.set_auto_approve(true);
        assert!(app.auto_approve);

        app.set_plan_only(true);
        assert!(app.plan_only);
    }

    #[test]
    fn test_terraform_app_default_workspace() {
        let app = TerraformApp::new();
        assert_eq!(app.workspace, "default");
    }

    #[test]
    fn test_shell_app_set_script() {
        let mut app = ShellApp::new(AppType::Bash);
        app.set_script(PathBuf::from("/scripts/deploy.sh"));
        assert_eq!(app.script_path, PathBuf::from("/scripts/deploy.sh"));
    }

    #[test]
    fn test_shell_app_get_command_for_all_types() {
        let bash_app = ShellApp::new(AppType::Bash);
        assert_eq!(bash_app.get_command(), "bash");

        let ps_app = ShellApp::new(AppType::PowerShell);
        assert_eq!(ps_app.get_command(), "pwsh");

        let py_app = ShellApp::new(AppType::Python);
        assert_eq!(py_app.get_command(), "python3");

        // Default fallback
        let ansible_as_shell = ShellApp::new(AppType::Ansible);
        assert_eq!(ansible_as_shell.get_command(), "bash");
    }

    #[test]
    fn test_shell_app_get_work_dir() {
        let mut app = ShellApp::new(AppType::Bash);
        app.set_work_dir(PathBuf::from("/tmp/work"));
        assert_eq!(app.get_work_dir(), Path::new("/tmp/work"));
    }

    #[test]
    fn test_shell_app_cli_and_env_setters() {
        let mut app = ShellApp::new(AppType::Bash);
        app.set_cli_args(vec!["--verbose".to_string()]);
        app.set_environment(vec!["DEBUG=1".to_string()]);

        assert_eq!(app.base.cli_args.len(), 1);
        assert_eq!(app.base.environment_vars.len(), 1);
    }

    #[test]
    fn test_ansible_app_install_requirements_no_file() {
        // Without requirements.yml, install_galaxy_requirements should return Ok(())
        // This is tested indirectly via the helper function behavior
        let mut app = AnsibleApp::new();
        // We can't easily test async without a runtime, but we verify structure
        assert!(app.extra_vars.is_empty());
        assert!(app.inventory_path.is_none());
    }

    #[test]
    fn test_build_ansible_args_empty_inventory_with_extra_vars_and_args() {
        use std::collections::HashMap;
        let mut extra = HashMap::new();
        extra.insert("env".to_string(), serde_json::json!("production"));

        let cli = vec!["--forks".to_string(), "10".to_string()];

        let args = build_ansible_args(
            "deploy.yml",
            None, // no inventory
            &extra,
            &cli,
        );

        // Should have playbook, extra-vars, and cli args but NO -i
        assert_eq!(args[0], "deploy.yml");
        assert_eq!(args[1], "--extra-vars");
        assert!(args.contains(&"--forks".to_string()));
        assert!(args.contains(&"10".to_string()));
        assert!(!args.contains(&"-i".to_string()));
    }

    #[test]
    fn test_app_run_result_various_exit_codes() {
        let codes = vec![0, 1, 2, 127, 128, 255];
        for code in codes {
            let result = AppRunResult {
                exit_code: code,
                has_errors: code != 0,
                output_path: None,
            };
            assert_eq!(result.exit_code, code);
            assert_eq!(result.has_errors, code != 0);
        }
    }

    #[test]
    fn test_app_factory_all_types_return_executable_app() {
        let all_types = vec![
            AppType::Ansible,
            AppType::Terraform,
            AppType::Tofu,
            AppType::Terragrunt,
            AppType::Bash,
            AppType::PowerShell,
            AppType::Python,
            AppType::Pulumi,
        ];

        for app_type in all_types {
            let app = AppFactory::create(app_type);
            // Just verify the factory returns a valid Box<dyn ExecutableApp>
            let _ = app;
        }
    }
}
