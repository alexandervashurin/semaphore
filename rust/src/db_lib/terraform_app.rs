//! TerraformApp - выполнение Terraform/OpenTofu/Terragrunt задач
//!
//! Аналог db_lib/TerraformApp.go из Go версии

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::{Child, Command as TokioCommand};
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{info, warn, error, debug};

use crate::error::{Error, Result};
use crate::models::{Template, Repository, Inventory, TerraformTaskParams};
use crate::services::task_logger::{TaskLogger, TaskStatus};

/// TerraformApp представляет приложение для выполнения Terraform команд
pub struct TerraformApp {
    /// Логгер
    pub logger: Box<dyn TaskLogger>,
    /// Шаблон
    pub template: Template,
    /// Репозиторий
    pub repository: Repository,
    /// Инвентарь
    pub inventory: Inventory,
    /// Имя бинарника (terraform, tofu, terragrunt)
    pub name: String,
    /// Plan не имеет изменений
    pub plan_has_no_changes: bool,
    /// Путь к backend файлу
    pub backend_filename: Option<String>,
    /// Рабочая директория
    pub work_dir: PathBuf,
}

impl TerraformApp {
    /// Создаёт новый TerraformApp
    pub fn new(
        logger: Box<dyn TaskLogger>,
        template: Template,
        repository: Repository,
        inventory: Inventory,
        name: String,
        work_dir: PathBuf,
    ) -> Self {
        Self {
            logger,
            template,
            repository,
            inventory,
            name,
            plan_has_no_changes: false,
            backend_filename: None,
            work_dir,
        }
    }

    /// Получает полный путь к рабочей директории
    pub fn get_full_path(&self) -> PathBuf {
        self.work_dir.join("repository")
    }

    /// Создаёт команду для выполнения
    fn make_cmd(&self, command: &str, args: Vec<String>, environment_vars: Vec<String>) -> TokioCommand {
        let mut cmd = TokioCommand::new(command);
        cmd.args(&args);
        cmd.current_dir(self.get_full_path());
        
        // Добавляем переменные окружения
        for (key, value) in self.get_environment_vars() {
            cmd.env(&key, &value);
        }
        
        for env_var in environment_vars {
            if let Some((key, value)) = env_var.split_once('=') {
                cmd.env(key, value);
            }
        }
        
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        cmd
    }

    /// Получает переменные окружения
    fn get_environment_vars(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();
        
        // TF_INPUT=0 - отключить интерактивный ввод
        env.insert("TF_INPUT".to_string(), "0".to_string());
        
        // TF_VAR_* переменные из инвентаря
        if let Some(ref inventory_vars) = self.inventory.variables {
            for (key, value) in inventory_vars {
                env.insert(format!("TF_VAR_{}", key), value.clone());
            }
        }
        
        env
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

    /// Инициализирует Terraform
    pub async fn init(&self, environment_vars: Vec<String>, params: &TerraformTaskParams, extra_args: Vec<String>) -> Result<()> {
        self.logger.log("Initializing Terraform...");
        
        let mut args = vec!["init".to_string(), "-input=false".to_string()];
        
        // Backend аргументы
        if params.backend_init_required {
            if let Some(ref backend_config) = params.backend_config {
                args.push(format!("-backend-config={}", backend_config));
            }
        } else {
            args.push("-backend=false".to_string());
        }
        
        // Upgrade
        if params.upgrade {
            args.push("-upgrade".to_string());
        }
        
        // Reconfigure
        if params.reconfigure {
            args.push("-reconfigure".to_string());
        }
        
        // Дополнительные аргументы
        args.extend(extra_args);
        
        self.run_cmd(&self.name, args).await?;
        
        Ok(())
    }

    /// Проверяет поддержку workspaces
    pub async fn is_workspaces_supported(&self, environment_vars: Vec<String>) -> Result<bool> {
        let args = vec!["workspace".to_string(), "list".to_string()];
        
        let mut cmd = self.make_cmd(&self.name, args, environment_vars);
        
        match cmd.output().await {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    /// Выбирает workspace
    pub async fn select_workspace(&self, workspace: &str, environment_vars: Vec<String>) -> Result<()> {
        self.logger.log(&format!("Selecting workspace: {}", workspace));
        
        // Проверяем существует ли workspace
        let list_args = vec!["workspace".to_string(), "list".to_string()];
        let output = self.make_cmd(&self.name, list_args, environment_vars.clone()).output().await?;
        
        let workspace_exists = String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line.trim() == workspace || line.trim().starts_with(&format!("* {}", workspace)));
        
        if workspace_exists {
            // Выбираем существующий
            let args = vec!["workspace".to_string(), "select".to_string(), workspace.to_string()];
            self.run_cmd(&self.name, args).await?;
        } else {
            // Создаём новый
            let args = vec!["workspace".to_string(), "new".to_string(), workspace.to_string()];
            self.run_cmd(&self.name, args).await?;
        }
        
        Ok(())
    }

    /// Выполняет plan
    pub async fn plan(&self, args: Vec<String>, environment_vars: Vec<String>, inputs: HashMap<String, String>, cb: Option<Box<dyn Fn(&Child) + Send>>) -> Result<bool> {
        self.logger.log("Running Terraform plan...");
        
        let mut plan_args = vec!["plan".to_string(), "-input=false".to_string(), "-no-color".to_string()];
        
        // Plan file
        let plan_file = self.work_dir.join("tfplan");
        plan_args.push(format!("-out={}", plan_file.display()));
        
        // Дополнительные аргументы
        plan_args.extend(args);
        
        let mut cmd = self.make_cmd(&self.name, plan_args, environment_vars);
        
        let mut child = cmd.spawn()?;
        
        // Callback для процесса
        if let Some(callback) = cb {
            callback(&child);
        }
        
        let status = child.wait().await?;
        
        // Проверяем есть ли изменения
        self.plan_has_no_changes = status.code() == Some(0);
        
        Ok(!self.plan_has_no_changes)
    }

    /// Выполняет apply
    pub async fn apply(&self, args: Vec<String>, environment_vars: Vec<String>, inputs: HashMap<String, String>, cb: Option<Box<dyn Fn(&Child) + Send>>) -> Result<()> {
        self.logger.log("Running Terraform apply...");
        
        let mut apply_args = vec!["apply".to_string(), "-input=false".to_string(), "-auto-approve".to_string()];
        
        // Plan file или дополнительные аргументы
        let plan_file = self.work_dir.join("tfplan");
        if plan_file.exists() {
            apply_args.push(plan_file.display().to_string());
        }
        
        apply_args.extend(args);
        
        self.run_cmd(&self.name, apply_args).await?;
        
        Ok(())
    }

    /// Выполняет destroy
    pub async fn destroy(&self, args: Vec<String>, environment_vars: Vec<String>) -> Result<()> {
        self.logger.log("Running Terraform destroy...");
        
        let mut destroy_args = vec!["destroy".to_string(), "-input=false".to_string(), "-auto-approve".to_string()];
        
        destroy_args.extend(args);
        
        self.run_cmd(&self.name, destroy_args).await?;
        
        Ok(())
    }

    /// Очищает временные файлы
    pub fn clear(&self) {
        let _ = std::fs::remove_dir_all(&self.work_dir);
        self.logger.log("Cleaned up temporary files");
    }

    /// Устанавливает зависимости
    pub async fn install_requirements(&self, args: crate::db_lib::LocalAppInstallingArgs) -> Result<()> {
        self.logger.log("Installing Terraform requirements...");
        
        // Инициализация
        let params = TerraformTaskParams {
            backend_init_required: true,
            backend_config: None,
            upgrade: false,
            reconfigure: false,
            destroy: false,
            workspace: None,
        };
        
        self.init(vec![], &params, vec![]).await?;
        
        Ok(())
    }

    /// Запускает задачу
    pub async fn run(&self, args: crate::db_lib::LocalAppRunningArgs) -> Result<()> {
        let params: TerraformTaskParams = self.template.extract_params()?;
        
        // Инициализация
        self.install_requirements(args.install_args).await?;
        
        // Workspace
        if let Some(ref workspace) = params.workspace {
            if self.is_workspaces_supported(vec![]).await? {
                self.select_workspace(workspace, vec![]).await?;
            }
        }
        
        // Plan
        let has_changes = self.plan(vec![], vec![], HashMap::new(), None).await?;
        
        // Apply или Destroy
        if params.destroy {
            self.destroy(vec![], vec![]).await?;
        } else if has_changes {
            self.apply(vec![], vec![], HashMap::new(), None).await?;
        } else {
            self.logger.log("No changes to apply");
        }
        
        Ok(())
    }
}

impl Drop for TerraformApp {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::task_logger::BasicLogger;
    use std::sync::Arc;

    fn create_test_terraform_app() -> TerraformApp {
        let logger = Box::new(BasicLogger::new());
        let template = Template::default();
        let repository = Repository::default();
        let inventory = Inventory::default();
        let work_dir = PathBuf::from("/tmp/test_terraform");
        
        TerraformApp::new(logger, template, repository, inventory, "terraform".to_string(), work_dir)
    }

    #[test]
    fn test_terraform_app_creation() {
        let app = create_test_terraform_app();
        assert_eq!(app.name, "terraform");
    }

    #[test]
    fn test_get_full_path() {
        let app = create_test_terraform_app();
        let path = app.get_full_path();
        assert!(path.ends_with("repository"));
    }

    #[test]
    fn test_get_environment_vars() {
        let app = create_test_terraform_app();
        let env = app.get_environment_vars();
        assert!(env.contains_key("TF_INPUT"));
        assert_eq!(env.get("TF_INPUT").unwrap(), "0");
    }
}
