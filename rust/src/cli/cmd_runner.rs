//! CLI - Runner Command
//!
//! Velum Runner Agent — запускается на удалённой машине, поллит сервер на задачи,
//! выполняет ansible/terraform/bash локально и стримит логи обратно.
//!
//! Использование:
//! ```text
//! velum runner --token `<TOKEN>` --server-url http://velum-server:3000
//! ```

use crate::cli::CliResult;
use clap::Args;

/// Команда runner
#[derive(Debug, Args)]
pub struct RunnerCommand {
    /// Токен раннера (выдаётся администратором через /api/runners)
    #[arg(long, env = "VELUM_RUNNER_TOKEN")]
    pub token: Option<String>,

    /// URL Velum-сервера
    #[arg(long, env = "VELUM_SERVER_URL", default_value = "http://localhost:3000")]
    pub server_url: String,

    /// Имя раннера (отображается в UI)
    #[arg(long, env = "VELUM_RUNNER_NAME")]
    pub name: Option<String>,

    /// Интервал опроса в секундах (по умолчанию 5)
    #[arg(long, default_value = "5")]
    pub poll_interval: u64,

    /// Максимальное число параллельных задач
    #[arg(long, default_value = "1")]
    pub max_parallel: u32,

    /// Тег раннера (для маршрутизации задач)
    #[arg(long)]
    pub tag: Option<String>,
}

impl RunnerCommand {
    /// Выполняет команду: запускает агент-цикл
    pub fn run(&self) -> CliResult<()> {
        let token = self.token.as_deref().unwrap_or("").to_string();
        if token.is_empty() {
            eprintln!("Error: --token is required. Generate a runner token in Velum UI → Settings → Runners.");
            std::process::exit(1);
        }

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| crate::error::Error::Other(e.to_string()))?;

        rt.block_on(async {
            self.run_agent(token).await
        }).map_err(|e| crate::error::Error::Other(e.to_string()))?;

        Ok(())
    }

    async fn run_agent(&self, token: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        // Регистрация / переподключение
        let runner_id = self.register(&client, &token).await?;
        println!("✅ Runner registered (id={runner_id}). Polling for tasks every {}s...", self.poll_interval);

        let mut heartbeat_tick = tokio::time::interval(std::time::Duration::from_secs(30));
        let poll_delay = std::time::Duration::from_secs(self.poll_interval);

        loop {
            tokio::select! {
                _ = heartbeat_tick.tick() => {
                    let _ = self.heartbeat(&client, runner_id).await;
                }
                _ = tokio::time::sleep(poll_delay) => {
                    match self.poll_task(&client, runner_id).await {
                        Ok(Some(assignment)) => {
                            println!("▶  Task #{} (project={}, template={})",
                                assignment.task_id, assignment.project_id, assignment.template_id);
                            if let Err(e) = self.execute_task(&client, &assignment).await {
                                eprintln!("   Task #{} failed: {e}", assignment.task_id);
                            }
                        }
                        Ok(None) => {} // нет задач — продолжаем поллить
                        Err(e) => eprintln!("Poll error: {e}"),
                    }
                }
            }
        }
    }

    /// Регистрирует раннер на сервере, возвращает runner_id
    async fn register(&self, client: &reqwest::Client, token: &str) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::json!({
            "token": token,
            "name": self.name.as_deref().unwrap_or("velum-runner"),
            "max_parallel_tasks": self.max_parallel,
            "tag": self.tag,
        });

        let resp = client
            .post(format!("{}/api/internal/runners", self.server_url))
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Registration failed ({status}): {body}").into());
        }

        let body: serde_json::Value = resp.json().await?;
        let id = body["id"].as_i64().ok_or("Missing 'id' in registration response")? as i32;
        Ok(id)
    }

    /// Heartbeat — обновляет last_active на сервере
    async fn heartbeat(&self, client: &reqwest::Client, runner_id: i32) -> Result<(), reqwest::Error> {
        let _ = client
            .post(format!("{}/api/internal/runners/{runner_id}", self.server_url))
            .json(&serde_json::json!({}))
            .send()
            .await?;
        Ok(())
    }

    /// Запрашивает задачу — 200 = есть задача, 204 = нет задач
    async fn poll_task(
        &self,
        client: &reqwest::Client,
        runner_id: i32,
    ) -> Result<Option<TaskAssignment>, Box<dyn std::error::Error + Send + Sync>> {
        let resp = client
            .get(format!("{}/api/internal/runners/{runner_id}/task", self.server_url))
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(None);
        }
        if !resp.status().is_success() {
            return Err(format!("Poll failed: {}", resp.status()).into());
        }

        let assignment: TaskAssignment = resp.json().await?;
        Ok(Some(assignment))
    }

    /// Выполняет задачу локально и стримит логи на сервер
    async fn execute_task(
        &self,
        client: &reqwest::Client,
        assignment: &TaskAssignment,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::io::{AsyncBufReadExt, BufReader};
        use tokio::process::Command;

        // Строим команду на основе типа задачи
        let (program, args) = self.build_command(assignment);

        println!("   Executing: {program} {}", args.join(" "));

        let mut child = Command::new(&program)
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().expect("stdout piped");
        let stderr = child.stderr.take().expect("stderr piped");

        let task_id = assignment.task_id;
        let server_url = self.server_url.clone();
        let client_clone = client.clone();

        // Читаем stdout/stderr в отдельных задачах и буферизуем логи
        let mut lines_stdout = BufReader::new(stdout).lines();
        let mut lines_stderr = BufReader::new(stderr).lines();

        let mut log_buffer: Vec<LogLine> = Vec::new();
        let flush_interval = std::time::Duration::from_secs(2);
        let mut last_flush = std::time::Instant::now();

        loop {
            tokio::select! {
                line = lines_stdout.next_line() => {
                    match line? {
                        Some(l) => {
                            println!("   | {l}");
                            log_buffer.push(LogLine::now(l, task_id));
                        }
                        None => break,
                    }
                }
                line = lines_stderr.next_line() => {
                    if let Ok(Some(l)) = line {
                        eprintln!("   ! {l}");
                        log_buffer.push(LogLine::now(format!("[stderr] {l}"), task_id));
                    }
                }
            }

            if last_flush.elapsed() >= flush_interval && !log_buffer.is_empty() {
                let _ = Self::flush_logs(&client_clone, &server_url, task_id, &log_buffer).await;
                log_buffer.clear();
                last_flush = std::time::Instant::now();
            }
        }

        // Ждём завершения процесса
        let exit = child.wait().await?;
        let final_line = if exit.success() {
            format!("Task #{task_id} completed successfully")
        } else {
            format!("Task #{task_id} failed with exit code {:?}", exit.code())
        };

        println!("   {final_line}");
        log_buffer.push(LogLine::now(final_line, task_id));

        // Финальный flush
        if !log_buffer.is_empty() {
            let _ = Self::flush_logs(client, &self.server_url, task_id, &log_buffer).await;
        }

        Ok(())
    }

    /// Строит команду для запуска задачи
    fn build_command(&self, assignment: &TaskAssignment) -> (String, Vec<String>) {
        // Ansible playbook
        if let Some(ref playbook) = assignment.playbook {
            let mut args = vec![playbook.clone()];
            if assignment.dry_run {
                args.push("--check".to_string());
            }
            if assignment.debug {
                args.push("-vvv".to_string());
            }
            if let Some(ref extra) = assignment.arguments {
                args.push("--extra-vars".to_string());
                args.push(extra.clone());
            }
            return ("ansible-playbook".to_string(), args);
        }

        // Terraform
        if let Some(ref env) = assignment.environment {
            if env.contains("terraform") {
                let cmd = if assignment.dry_run { "plan" } else { "apply" };
                return ("terraform".to_string(), vec![cmd.to_string(), "-auto-approve".to_string()]);
            }
        }

        // Bash fallback
        ("bash".to_string(), vec!["-c".to_string(), "echo 'No playbook configured'".to_string()])
    }

    /// Отправляет буфер логов на сервер
    async fn flush_logs(
        client: &reqwest::Client,
        server_url: &str,
        task_id: i32,
        lines: &[LogLine],
    ) -> Result<(), reqwest::Error> {
        let payload = serde_json::json!({ "output": lines });
        let _ = client
            .post(format!("{server_url}/api/internal/tasks/{task_id}/log"))
            .json(&payload)
            .send()
            .await?;
        Ok(())
    }
}

/// Задача, полученная от сервера
#[derive(Debug, serde::Deserialize)]
struct TaskAssignment {
    task_id: i32,
    project_id: i32,
    template_id: i32,
    playbook: Option<String>,
    environment: Option<String>,
    arguments: Option<String>,
    git_branch: Option<String>,
    debug: bool,
    dry_run: bool,
}

/// Строка лога для отправки на сервер
#[derive(Debug, serde::Serialize)]
struct LogLine {
    time: String,
    output: String,
    task_id: i32,
}

impl LogLine {
    fn now(line: String, task_id: i32) -> Self {
        Self {
            time: chrono::Utc::now().to_rfc3339(),
            output: line,
            task_id,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_command_creation() {
        let cmd = RunnerCommand {
            token: Some("test-token".to_string()),
            server_url: "http://localhost:3000".to_string(),
            name: Some("test-runner".to_string()),
            poll_interval: 5,
            max_parallel: 1,
            tag: None,
        };
        assert_eq!(cmd.server_url, "http://localhost:3000");
    }

    #[test]
    fn test_log_line_creation() {
        let line = LogLine::now("hello".to_string(), 42);
        assert_eq!(line.output, "hello");
        assert_eq!(line.task_id, 42);
        assert!(!line.time.is_empty());
    }

    #[test]
    fn test_build_command_ansible() {
        let cmd = RunnerCommand {
            token: None,
            server_url: "http://localhost:3000".to_string(),
            name: None,
            poll_interval: 5,
            max_parallel: 1,
            tag: None,
        };
        let assignment = TaskAssignment {
            task_id: 1,
            project_id: 1,
            template_id: 1,
            playbook: Some("site.yml".to_string()),
            environment: None,
            arguments: None,
            git_branch: None,
            debug: false,
            dry_run: false,
        };
        let (prog, args) = cmd.build_command(&assignment);
        assert_eq!(prog, "ansible-playbook");
        assert_eq!(args[0], "site.yml");
    }

    #[test]
    fn test_build_command_dry_run() {
        let cmd = RunnerCommand {
            token: None,
            server_url: "http://localhost:3000".to_string(),
            name: None,
            poll_interval: 5,
            max_parallel: 1,
            tag: None,
        };
        let assignment = TaskAssignment {
            task_id: 1,
            project_id: 1,
            template_id: 1,
            playbook: Some("deploy.yml".to_string()),
            environment: None,
            arguments: None,
            git_branch: None,
            debug: false,
            dry_run: true,
        };
        let (_, args) = cmd.build_command(&assignment);
        assert!(args.contains(&"--check".to_string()));
    }
}
