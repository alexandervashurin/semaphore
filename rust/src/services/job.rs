//! Реализация Job для различных типов задач
//!
//! AnsibleJob, TerraformJob, ShellJob - исполнители задач

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command as TokioCommand;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{info, warn, error, debug};

use crate::error::{Error, Result};
use crate::models::{Task, Template, Inventory, Repository, Environment};
use crate::services::task_runner::Job;

/// Базовая структура для всех Job
pub struct BaseJob {
    /// Задача
    pub task: Task,
    /// Шаблон
    pub template: Template,
    /// Инвентарь
    pub inventory: Option<Inventory>,
    /// Репозиторий
    pub repository: Option<Repository>,
    /// Окружение
    pub environment: Option<Environment>,
    /// Рабочая директория
    pub work_dir: PathBuf,
    /// Переменные окружения
    pub env_vars: HashMap<String, String>,
    /// Флаг остановки
    pub killed: bool,
}

impl BaseJob {
    /// Создаёт базовый Job
    pub fn new(
        task: Task,
        template: Template,
        inventory: Option<Inventory>,
        repository: Option<Repository>,
        environment: Option<Environment>,
    ) -> Self {
        Self {
            task,
            template,
            inventory,
            repository,
            environment,
            work_dir: PathBuf::from("/tmp/semaphore"),
            env_vars: HashMap::new(),
            killed: false,
        }
    }

    /// Устанавливает рабочую директорию
    pub fn set_work_dir(&mut self, path: PathBuf) {
        self.work_dir = path;
    }

    /// Добавляет переменную окружения
    pub fn add_env_var(&mut self, key: String, value: String) {
        self.env_vars.insert(key, value);
    }

    /// Запускает команду и логирует вывод
    pub async fn run_command(
        &self,
        command: &str,
        args: &[String],
    ) -> Result<bool> {
        debug!("Запуск команды: {} {}", command, args.join(" "));

        let mut cmd = TokioCommand::new(command);
        cmd.args(args);
        cmd.current_dir(&self.work_dir);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Добавляем переменные окружения
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        let mut child = cmd.spawn().map_err(|e| {
            Error::Other(format!("Ошибка запуска команды: {}", e))
        })?;

        // Читаем stdout
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                info!("[{}] {}", command, line);
            }
        }

        // Читаем stderr
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                warn!("[{}] {}", command, line);
            }
        }

        let status = child.wait().await.map_err(|e| {
            Error::Other(format!("Ошибка ожидания команды: {}", e))
        })?;

        let exit_code = status.code().unwrap_or(-1);
        let success = status.success();

        debug!("Команда завершилась с кодом: {}", exit_code);

        Ok(success)
    }

    /// Получает путь к репозиторию
    pub fn get_repository_path(&self) -> PathBuf {
        if let Some(ref repo) = self.repository {
            PathBuf::from(format!("/tmp/semaphore/repo_{}", repo.id))
        } else {
            self.work_dir.clone()
        }
    }

    /// Получает путь к playbook
    pub fn get_playbook_path(&self) -> PathBuf {
        let repo_path = self.get_repository_path();
        repo_path.join(&self.template.playbook)
    }

    /// Получает путь к inventory
    pub fn get_inventory_path(&self) -> Option<PathBuf> {
        self.inventory.as_ref().map(|inv| {
            let repo_path = self.get_repository_path();
            repo_path.join(&inv.inventory_data)
        })
    }
}

/// Ansible Job
pub struct AnsibleJob {
    base: BaseJob,
    /// Дополнительные переменные
    pub extra_vars: HashMap<String, String>,
    /// Лимиты
    pub limit: Vec<String>,
    /// Теги
    pub tags: Vec<String>,
    /// Skip теги
    pub skip_tags: Vec<String>,
    /// Debug режим
    pub debug: bool,
    /// Dry run
    pub dry_run: bool,
    /// Diff режим
    pub diff: bool,
    /// Флаг остановки
    pub killed: bool,
}

impl AnsibleJob {
    /// Создаёт новый Ansible Job
    pub fn new(
        task: Task,
        template: Template,
        inventory: Option<Inventory>,
        repository: Option<Repository>,
        environment: Option<Environment>,
    ) -> Self {
        let mut base = BaseJob::new(task, template, inventory, repository, environment);
        
        // Добавляем ANSIBLE_* переменные
        base.add_env_var("ANSIBLE_FORCE_COLOR".to_string(), "0".to_string());
        base.add_env_var("PYTHONUNBUFFERED".to_string(), "1".to_string());

        Self {
            base,
            extra_vars: HashMap::new(),
            limit: Vec::new(),
            tags: Vec::new(),
            skip_tags: Vec::new(),
            debug: false,
            dry_run: false,
            diff: false,
            killed: false,
        }
    }

    /// Добавляет дополнительную переменную
    pub fn add_extra_var(&mut self, key: String, value: String) {
        self.extra_vars.insert(key, value);
    }

    /// Устанавливает лимиты
    pub fn set_limit(&mut self, limit: Vec<String>) {
        self.limit = limit;
    }

    /// Устанавливает теги
    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = tags;
    }

    /// Устанавливает skip теги
    pub fn set_skip_tags(&mut self, skip_tags: Vec<String>) {
        self.skip_tags = skip_tags;
    }

    /// Включает debug режим
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    /// Включает dry run
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }

    /// Включает diff режим
    pub fn set_diff(&mut self, diff: bool) {
        self.diff = diff;
    }

    /// Устанавливает зависимости Ansible (roles, collections)
    pub async fn install_galaxy_requirements(&self) -> Result<()> {
        let work_dir = self.base.get_repository_path();
        let requirements_path = work_dir.join("requirements.yml");

        if !requirements_path.exists() {
            debug!("requirements.yml не найден, пропускаем установку зависимостей");
            return Ok(());
        }

        info!("Установка зависимостей Ansible Galaxy...");

        let args = vec![
            "install".to_string(),
            "-r".to_string(),
            requirements_path.to_string_lossy().to_string(),
            "--force".to_string(),
        ];

        let success = self.base.run_command("ansible-galaxy", &args).await?;

        if !success {
            return Err(Error::Other("Ошибка установки зависимостей Ansible Galaxy".to_string()));
        }

        info!("Зависимости успешно установлены");
        Ok(())
    }

    /// Формирует аргументы для ansible-playbook
    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Playbook
        args.push(self.base.get_playbook_path().to_string_lossy().to_string());

        // Inventory
        if let Some(inv_path) = self.base.get_inventory_path() {
            args.push("-i".to_string());
            args.push(inv_path.to_string_lossy().to_string());
        }

        // Extra vars
        if !self.extra_vars.is_empty() {
            let extra_vars_json: Vec<String> = self.extra_vars
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            
            args.push("--extra-vars".to_string());
            args.push(extra_vars_json.join(" "));
        }

        // Limit
        if !self.limit.is_empty() {
            args.push("--limit".to_string());
            args.push(self.limit.join(","));
        }

        // Tags
        if !self.tags.is_empty() {
            args.push("--tags".to_string());
            args.push(self.tags.join(","));
        }

        // Skip tags
        if !self.skip_tags.is_empty() {
            args.push("--skip-tags".to_string());
            args.push(self.skip_tags.join(","));
        }

        // Debug
        if self.debug {
            args.push("-vvv".to_string());
        }

        // Dry run
        if self.dry_run {
            args.push("--check".to_string());
        }

        // Diff
        if self.diff {
            args.push("--diff".to_string());
        }

        args
    }
}

#[async_trait::async_trait]
impl Job for AnsibleJob {
    async fn run(&mut self) -> Result<()> {
        info!("Запуск Ansible playbook...");

        // Устанавливаем зависимости
        self.install_galaxy_requirements().await?;

        // Формируем аргументы
        let args = self.build_args();

        // Запускаем ansible-playbook
        let success = self.base.run_command("ansible-playbook", &args).await?;

        if !success {
            return Err(Error::Other("Ansible playbook завершился с ошибкой".to_string()));
        }

        info!("Ansible playbook успешно завершён");
        Ok(())
    }

    fn kill(&mut self) {
        warn!("Остановка Ansible Job (не реализована)");
        self.killed = true;
    }

    fn is_killed(&self) -> bool {
        self.killed
    }
}

/// Terraform Job
pub struct TerraformJob {
    base: BaseJob,
    /// Рабочее пространство
    pub workspace: String,
    /// Auto approve
    pub auto_approve: bool,
    /// Plan only
    pub plan_only: bool,
    /// Destroy
    pub destroy: bool,
    /// Reconfigure
    pub reconfigure: bool,
    /// Upgrade провайдеров
    pub upgrade: bool,
    /// Флаг остановки
    pub killed: bool,
}

impl TerraformJob {
    /// Создаёт новый Terraform Job
    pub fn new(
        task: Task,
        template: Template,
        inventory: Option<Inventory>,
        repository: Option<Repository>,
        environment: Option<Environment>,
    ) -> Self {
        let mut base = BaseJob::new(task, template, inventory, repository, environment);
        
        // Добавляем TF_* переменные
        base.add_env_var("TF_INPUT".to_string(), "false".to_string());
        base.add_env_var("TF_IN_AUTOMATION".to_string(), "true".to_string());

        Self {
            base,
            workspace: "default".to_string(),
            auto_approve: false,
            plan_only: false,
            destroy: false,
            reconfigure: false,
            upgrade: false,
            killed: false,
        }
    }

    /// Устанавливает рабочее пространство
    pub fn set_workspace(&mut self, workspace: String) {
        self.workspace = workspace;
    }

    /// Устанавливает auto approve
    pub fn set_auto_approve(&mut self, auto_approve: bool) {
        self.auto_approve = auto_approve;
    }

    /// Устанавливает plan only
    pub fn set_plan_only(&mut self, plan_only: bool) {
        self.plan_only = plan_only;
    }

    /// Устанавливает destroy
    pub fn set_destroy(&mut self, destroy: bool) {
        self.destroy = destroy;
    }

    /// Устанавливает reconfigure
    pub fn set_reconfigure(&mut self, reconfigure: bool) {
        self.reconfigure = reconfigure;
    }

    /// Устанавливает upgrade
    pub fn set_upgrade(&mut self, upgrade: bool) {
        self.upgrade = upgrade;
    }

    /// Инициализирует Terraform
    pub async fn init(&self) -> Result<()> {
        info!("Инициализация Terraform...");

        let mut args = vec!["init".to_string()];

        if self.reconfigure {
            args.push("-reconfigure".to_string());
        }

        if self.upgrade {
            args.push("-upgrade".to_string());
        }

        let success = self.base.run_command("terraform", &args).await?;

        if !success {
            return Err(Error::Other("Ошибка инициализации Terraform".to_string()));
        }

        Ok(())
    }

    /// Выбирает рабочее пространство
    pub async fn select_workspace(&self) -> Result<()> {
        if self.workspace == "default" {
            return Ok(());
        }

        info!("Выбор workspace: {}", self.workspace);

        let args = vec![
            "workspace".to_string(),
            "select".to_string(),
            "-or-create=true".to_string(),
            self.workspace.clone(),
        ];

        let success = self.base.run_command("terraform", &args).await?;

        if !success {
            return Err(Error::Other("Ошибка выбора workspace".to_string()));
        }

        Ok(())
    }

    /// Выполняет terraform plan
    pub async fn plan(&self) -> Result<bool> {
        info!("Выполнение terraform plan...");

        let args = vec!["plan".to_string()];
        let success = self.base.run_command("terraform", &args).await?;

        Ok(success)
    }

    /// Выполняет terraform apply
    pub async fn apply(&self) -> Result<()> {
        info!("Выполнение terraform apply...");

        let mut args = vec!["apply".to_string()];

        if self.auto_approve {
            args.push("-auto-approve".to_string());
        }

        let success = self.base.run_command("terraform", &args).await?;

        if !success {
            return Err(Error::Other("Ошибка выполнения terraform apply".to_string()));
        }

        Ok(())
    }

    /// Выполняет terraform destroy
    pub async fn destroy_all(&self) -> Result<()> {
        info!("Выполнение terraform destroy...");

        let mut args = vec!["destroy".to_string()];

        if self.auto_approve {
            args.push("-auto-approve".to_string());
        }

        let success = self.base.run_command("terraform", &args).await?;

        if !success {
            return Err(Error::Other("Ошибка выполнения terraform destroy".to_string()));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Job for TerraformJob {
    async fn run(&mut self) -> Result<()> {
        info!("Запуск Terraform...");

        // Инициализация
        self.init().await?;

        // Выбор workspace
        self.select_workspace().await?;

        // Plan
        let has_changes = self.plan().await?;

        if self.plan_only {
            info!("Режим plan-only, завершаем выполнение");
            return Ok(());
        }

        // Apply или Destroy
        if self.destroy {
            self.destroy_all().await?;
        } else if has_changes {
            self.apply().await?;
        } else {
            info!("Изменений нет");
        }

        info!("Terraform успешно завершён");
        Ok(())
    }

    fn kill(&mut self) {
        warn!("Остановка Terraform Job (не реализована)");
        self.killed = true;
    }

    fn is_killed(&self) -> bool {
        self.killed
    }
}

/// Shell Job (Bash, PowerShell, Python)
pub struct ShellJob {
    base: BaseJob,
    /// Тип оболочки
    pub shell_type: ShellType,
    /// Скрипт
    pub script: Option<String>,
    /// Флаг остановки
    pub killed: bool,
}

/// Тип оболочки
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShellType {
    Bash,
    PowerShell,
    Python,
}

impl ShellType {
    /// Получает команду для выполнения
    pub fn get_command(&self) -> &'static str {
        match self {
            ShellType::Bash => "bash",
            ShellType::PowerShell => "pwsh",
            ShellType::Python => "python3",
        }
    }
}

impl ShellJob {
    /// Создаёт новый Shell Job
    pub fn new(
        task: Task,
        template: Template,
        inventory: Option<Inventory>,
        repository: Option<Repository>,
        environment: Option<Environment>,
        shell_type: ShellType,
    ) -> Self {
        let base = BaseJob::new(task, template, inventory, repository, environment);

        Self {
            base,
            shell_type,
            script: None,
            killed: false,
        }
    }

    /// Устанавливает скрипт
    pub fn set_script(&mut self, script: String) {
        self.script = Some(script);
    }

    /// Получает путь к скрипту
    pub fn get_script_path(&self) -> PathBuf {
        let repo_path = self.base.get_repository_path();
        repo_path.join(&self.base.template.playbook)
    }
}

#[async_trait::async_trait]
impl Job for ShellJob {
    async fn run(&mut self) -> Result<()> {
        let command = self.shell_type.get_command();
        let script_path = self.get_script_path();

        info!("Запуск скрипта: {:?} через {}", script_path, command);

        let args = vec![script_path.to_string_lossy().to_string()];
        let success = self.base.run_command(command, &args).await?;

        if !success {
            return Err(Error::Other(format!("Скрипт {} завершился с ошибкой", command)));
        }

        info!("Скрипт успешно завершён");
        Ok(())
    }

    fn kill(&mut self) {
        warn!("Остановка Shell Job (не реализована)");
        self.killed = true;
    }

    fn is_killed(&self) -> bool {
        self.killed
    }
}

/// Фабрика для создания Job
pub struct JobFactory;

impl JobFactory {
    /// Создаёт Job на основе типа шаблона
    pub fn create(
        task: Task,
        template: Template,
        inventory: Option<Inventory>,
        repository: Option<Repository>,
        environment: Option<Environment>,
    ) -> Result<Box<dyn Job>> {
        use crate::models::template::TemplateApp;

        match &template.app {
            TemplateApp::Ansible => {
                Ok(Box::new(AnsibleJob::new(task, template, inventory, repository, environment)))
            }
            TemplateApp::Terraform | TemplateApp::Tofu | TemplateApp::Terragrunt => {
                Ok(Box::new(TerraformJob::new(task, template, inventory, repository, environment)))
            }
            TemplateApp::Bash => {
                Ok(Box::new(ShellJob::new(task, template, inventory, repository, environment, ShellType::Bash)))
            }
            TemplateApp::PowerShell => {
                Ok(Box::new(ShellJob::new(task, template, inventory, repository, environment, ShellType::PowerShell)))
            }
            TemplateApp::Python => {
                Ok(Box::new(ShellJob::new(task, template, inventory, repository, environment, ShellType::Python)))
            }
            TemplateApp::Pulumi | TemplateApp::Default => {
                Err(Error::Other(format!("Неподдерживаемый тип приложения: {:?}", template.app)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::template::TemplateApp;
    use chrono::Utc;

    #[test]
    fn test_ansible_job_creation() {
        let task = Task {
            id: 1,
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
            created: Utc::now(),
            start: None,
            end: None,
            message: None,
            commit_hash: None,
            commit_message: None,
            build_task_id: None,
            version: None,
            inventory_id: None,
            params: None,
        };

        let template = Template {
            id: 1,
            project_id: 1,
            name: "Test".to_string(),
            playbook: "test.yml".to_string(),
            description: "Test".to_string(),
            inventory_id: 1,
            repository_id: 1,
            environment_id: 0,
            app: TemplateApp::Ansible,
            r#type: crate::models::template::TemplateType::Default,
            git_branch: "main".to_string(),
            deleted: false,
            created: Utc::now(),
        };

        let job = AnsibleJob::new(task, template, None, None, None);
        
        assert!(!job.killed);
        assert!(job.extra_vars.is_empty());
    }

    #[test]
    fn test_terraform_job_creation() {
        let task = Task {
            id: 1,
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
            created: Utc::now(),
            start: None,
            end: None,
            message: None,
            commit_hash: None,
            commit_message: None,
            build_task_id: None,
            version: None,
            inventory_id: None,
            params: None,
        };

        let template = Template {
            id: 1,
            project_id: 1,
            name: "Test".to_string(),
            playbook: "test.tf".to_string(),
            description: "Test".to_string(),
            inventory_id: 1,
            repository_id: 1,
            environment_id: 0,
            app: TemplateApp::Terraform,
            r#type: crate::models::template::TemplateType::Default,
            git_branch: "main".to_string(),
            deleted: false,
            created: Utc::now(),
        };

        let mut job = TerraformJob::new(task, template, None, None, None);
        job.set_workspace("dev".to_string());
        job.set_auto_approve(true);
        
        assert_eq!(job.workspace, "dev");
        assert!(job.auto_approve);
    }

    #[test]
    fn test_shell_job_creation() {
        let task = Task {
            id: 1,
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
            created: Utc::now(),
            start: None,
            end: None,
            message: None,
            commit_hash: None,
            commit_message: None,
            build_task_id: None,
            version: None,
            inventory_id: None,
            params: None,
        };

        let template = Template {
            id: 1,
            project_id: 1,
            name: "Test".to_string(),
            playbook: "test.sh".to_string(),
            description: "Test".to_string(),
            inventory_id: 1,
            repository_id: 1,
            environment_id: 0,
            app: TemplateApp::Bash,
            r#type: crate::models::template::TemplateType::Default,
            git_branch: "main".to_string(),
            deleted: false,
            created: Utc::now(),
        };

        let job = ShellJob::new(task, template, None, None, None, ShellType::Bash);
        
        assert_eq!(job.shell_type, ShellType::Bash);
    }

    #[test]
    fn test_shell_type_command() {
        assert_eq!(ShellType::Bash.get_command(), "bash");
        assert_eq!(ShellType::PowerShell.get_command(), "pwsh");
        assert_eq!(ShellType::Python.get_command(), "python3");
    }
}
