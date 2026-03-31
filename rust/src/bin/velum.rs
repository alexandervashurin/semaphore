//! velum — Developer CLI for Velum
//!
//! Управление Velum из командной строки: запуск задач, просмотр логов,
//! работа с проектами и шаблонами.
//!
//! # Конфигурация
//! - `VELUM_URL` — URL сервера (по умолчанию: http://localhost:3000)
//! - `VELUM_TOKEN` — JWT токен авторизации
//!
//! # Примеры
//! ```bash
//! velum projects                           # список проектов
//! velum templates --project 1             # шаблоны проекта
//! velum run --project 1 --template 5      # запустить задачу
//! velum status --project 1               # запущенные задачи
//! velum logs --project 1 --task 123      # логи задачи
//! velum approve --project 1 --task 123   # подтвердить задачу
//! velum whoami                            # текущий пользователь
//! ```

use clap::{Parser, Subcommand};

// ─── CLI structure ─────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "velum",
    about = "Velum Developer CLI — управление Ansible/Terraform задачами из терминала",
    version = env!("CARGO_PKG_VERSION"),
    long_about = None,
)]
struct Cli {
    /// URL Velum сервера
    #[arg(
        long,
        env = "VELUM_URL",
        default_value = "http://localhost:3000",
        global = true
    )]
    url: String,

    /// JWT токен авторизации (или установите VELUM_TOKEN)
    #[arg(long, env = "VELUM_TOKEN", global = true)]
    token: Option<String>,

    /// Формат вывода (table / json)
    #[arg(long, default_value = "table", global = true)]
    output: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Список проектов
    Projects,

    /// Список шаблонов
    Templates {
        /// ID проекта
        #[arg(short, long)]
        project: i32,
    },

    /// Запустить задачу из шаблона
    Run {
        /// ID проекта
        #[arg(short, long)]
        project: i32,

        /// ID шаблона
        #[arg(short, long)]
        template: i32,

        /// Сообщение/описание запуска
        #[arg(short, long)]
        message: Option<String>,

        /// Git ветка
        #[arg(short, long)]
        branch: Option<String>,

        /// Дополнительные аргументы CLI (например: --limit=web*)
        #[arg(short, long)]
        args: Option<String>,

        /// Режим dry-run (--check для Ansible, plan для Terraform)
        #[arg(long)]
        dry_run: bool,

        /// Ждать завершения задачи
        #[arg(long, short = 'w')]
        wait: bool,
    },

    /// Статус задач в проекте
    Status {
        /// ID проекта
        #[arg(short, long)]
        project: i32,

        /// Показать только запущенные задачи
        #[arg(long)]
        running: bool,
    },

    /// Логи задачи
    Logs {
        /// ID проекта
        #[arg(short, long)]
        project: i32,

        /// ID задачи
        #[arg(short, long)]
        task: i32,
    },

    /// Подтвердить задачу (для задач, ожидающих подтверждения)
    Approve {
        /// ID проекта
        #[arg(short, long)]
        project: i32,

        /// ID задачи
        #[arg(short, long)]
        task: i32,
    },

    /// Информация о текущем пользователе
    Whoami,

    /// Список задач
    Tasks {
        /// ID проекта
        #[arg(short, long)]
        project: i32,

        /// Количество последних задач
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Остановить задачу
    Stop {
        /// ID проекта
        #[arg(short, long)]
        project: i32,

        /// ID задачи
        #[arg(short, long)]
        task: i32,
    },

    /// Показать версию сервера
    Version,
}

// ─── API Client ────────────────────────────────────────────────────────────

struct Client {
    base_url: String,
    token: String,
    http: reqwest::blocking::Client,
    json_output: bool,
}

impl Client {
    fn new(base_url: String, token: String, json_output: bool) -> Self {
        let http = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        Self {
            base_url,
            token,
            http,
            json_output,
        }
    }

    fn get(&self, path: &str) -> Result<serde_json::Value, String> {
        let url = format!("{}/api{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .map_err(|e| format!("HTTP error: {}", e))?;
        let status = resp.status();
        let body: serde_json::Value = resp
            .json()
            .map_err(|e| format!("JSON parse error: {}", e))?;
        if !status.is_success() {
            let msg = body
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            return Err(format!("API error {}: {}", status, msg));
        }
        Ok(body)
    }

    fn post(&self, path: &str, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{}/api{}", self.base_url, path);
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(payload)
            .send()
            .map_err(|e| format!("HTTP error: {}", e))?;
        let status = resp.status();
        let body: serde_json::Value = resp
            .json()
            .map_err(|e| format!("JSON parse error: {}", e))?;
        if !status.is_success() {
            let msg = body
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            return Err(format!("API error {}: {}", status, msg));
        }
        Ok(body)
    }

    fn post_empty(&self, path: &str) -> Result<(), String> {
        let url = format!("{}/api{}", self.base_url, path);
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Length", "0")
            .send()
            .map_err(|e| format!("HTTP error: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("API error {}", resp.status()));
        }
        Ok(())
    }
}

// ─── Formatters ────────────────────────────────────────────────────────────

fn status_icon(status: &str) -> &'static str {
    match status {
        "success" => "✅",
        "error" | "failed" => "❌",
        "running" => "🔄",
        "waiting" | "queued" => "⏳",
        "stopped" | "cancelled" => "⛔",
        _ => "❓",
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

fn print_table(headers: &[&str], rows: Vec<Vec<String>>) {
    // Calculate column widths
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    // Print header
    let header_line = headers
        .iter()
        .enumerate()
        .map(|(i, h)| format!("{:<width$}", h, width = widths[i]))
        .collect::<Vec<_>>()
        .join("  ");
    println!("\x1b[1;34m{}\x1b[0m", header_line);
    println!("{}", "─".repeat(header_line.len()));

    // Print rows
    for row in &rows {
        let line = row
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let w = widths.get(i).copied().unwrap_or(0);
                format!("{:<width$}", cell, width = w)
            })
            .collect::<Vec<_>>()
            .join("  ");
        println!("{}", line);
    }

    if rows.is_empty() {
        println!("\x1b[33m(нет данных)\x1b[0m");
    }
}

// ─── Command handlers ──────────────────────────────────────────────────────

fn cmd_projects(client: &Client) -> Result<(), String> {
    let data = client.get("/projects")?;
    let projects = data.as_array().ok_or("Expected array")?;

    if client.json_output {
        println!("{}", serde_json::to_string_pretty(&data).unwrap());
        return Ok(());
    }

    print_table(
        &["ID", "Название", "Создан"],
        projects
            .iter()
            .map(|p| {
                vec![
                    p["id"].to_string(),
                    truncate(p["name"].as_str().unwrap_or("—"), 40),
                    p["created"].as_str().unwrap_or("—")
                        [..10.min(p["created"].as_str().unwrap_or("").len())]
                        .to_string(),
                ]
            })
            .collect(),
    );
    println!("\nВсего: {} проектов", projects.len());
    Ok(())
}

fn cmd_templates(client: &Client, project: i32) -> Result<(), String> {
    let data = client.get(&format!("/project/{}/templates", project))?;
    let templates = data.as_array().ok_or("Expected array")?;

    if client.json_output {
        println!("{}", serde_json::to_string_pretty(&data).unwrap());
        return Ok(());
    }

    print_table(
        &["ID", "Название", "Тип", "Playbook"],
        templates
            .iter()
            .map(|t| {
                vec![
                    t["id"].to_string(),
                    truncate(t["name"].as_str().unwrap_or("—"), 36),
                    t["app"]
                        .as_str()
                        .or_else(|| t["type"].as_str())
                        .unwrap_or("ansible")
                        .to_string(),
                    truncate(t["playbook"].as_str().unwrap_or("—"), 30),
                ]
            })
            .collect(),
    );
    println!("\nВсего: {} шаблонов", templates.len());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_run(
    client: &Client,
    project: i32,
    template: i32,
    message: Option<String>,
    branch: Option<String>,
    args: Option<String>,
    dry_run: bool,
    wait: bool,
) -> Result<(), String> {
    let mut payload = serde_json::json!({
        "template_id": template,
    });
    if let Some(m) = message {
        payload["message"] = serde_json::Value::String(m);
    }
    if let Some(b) = branch {
        payload["git_branch"] = serde_json::Value::String(b);
    }
    if let Some(a) = args {
        payload["arguments"] = serde_json::Value::String(a);
    }
    if dry_run {
        payload["dry_run"] = serde_json::Value::Bool(true);
    }

    let task = client.post(&format!("/project/{}/tasks", project), &payload)?;
    let task_id = task["id"].as_i64().unwrap_or(0);

    if client.json_output {
        println!("{}", serde_json::to_string_pretty(&task).unwrap());
        return Ok(());
    }

    println!("🚀 Задача #{} запущена", task_id);
    if dry_run {
        println!("   🔍 Dry Run / Plan Preview режим");
    }
    println!("   Шаблон: #{}", template);
    println!(
        "   Просмотр логов: velum logs --project {} --task {}",
        project, task_id
    );
    println!(
        "   URL: {}/project/{}/task/{}",
        client.base_url, project, task_id
    );

    if wait {
        println!("\nОжидание завершения задачи...");
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3));
            let status_data = client.get(&format!("/project/{}/tasks/{}", project, task_id))?;
            let status = status_data["status"].as_str().unwrap_or("unknown");
            match status {
                "success" => {
                    println!("✅ Задача #{} завершена успешно", task_id);
                    break;
                }
                "error" | "failed" => {
                    println!("❌ Задача #{} завершилась с ошибкой", task_id);
                    println!(
                        "   Логи: velum logs --project {} --task {}",
                        project, task_id
                    );
                    return Err("Task failed".to_string());
                }
                "stopped" | "cancelled" => {
                    println!("⛔ Задача #{} остановлена", task_id);
                    break;
                }
                s => print!("\r   Статус: {} {}... ", status_icon(s), s),
            }
        }
    }

    Ok(())
}

fn cmd_status(client: &Client, project: i32, running_only: bool) -> Result<(), String> {
    let data = client.get(&format!("/project/{}/tasks?limit=20", project))?;
    let tasks = data.as_array().ok_or("Expected array")?;

    let tasks: Vec<_> = if running_only {
        tasks
            .iter()
            .filter(|t| {
                matches!(
                    t["status"].as_str().unwrap_or(""),
                    "running" | "waiting" | "queued"
                )
            })
            .collect()
    } else {
        tasks.iter().collect()
    };

    if client.json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::Value::Array(
                tasks.iter().map(|t| (*t).clone()).collect()
            ))
            .unwrap()
        );
        return Ok(());
    }

    if tasks.is_empty() {
        if running_only {
            println!("✨ Нет запущенных задач");
        } else {
            println!("Нет задач");
        }
        return Ok(());
    }

    print_table(
        &["ID", "St", "Шаблон", "Пользователь", "Начало", "Длит."],
        tasks
            .iter()
            .map(|t| {
                let status = t["status"].as_str().unwrap_or("—");
                let start = t["start"]
                    .as_str()
                    .or_else(|| t["start_time"].as_str())
                    .unwrap_or("—");
                let start_short = if start.len() > 16 {
                    &start[..16]
                } else {
                    start
                };
                vec![
                    t["id"].to_string(),
                    format!("{} {}", status_icon(status), status),
                    format!("#{}", t["template_id"].as_i64().unwrap_or(0)),
                    truncate(
                        t["user_name"]
                            .as_str()
                            .or_else(|| t["username"].as_str())
                            .unwrap_or("—"),
                        16,
                    ),
                    start_short.to_string(),
                    "—".to_string(),
                ]
            })
            .collect(),
    );
    Ok(())
}

fn cmd_logs(client: &Client, project: i32, task: i32) -> Result<(), String> {
    // First get task status
    let task_data = client.get(&format!("/project/{}/tasks/{}", project, task))?;
    let status = task_data["status"].as_str().unwrap_or("unknown");
    println!("📋 Задача #{} [{}{}]", task, status_icon(status), status);
    println!("{}", "─".repeat(60));

    // Get task output
    let output = client.get(&format!("/project/{}/tasks/{}/output", project, task))?;
    let lines = output.as_array().ok_or("Expected array")?;

    for line in lines {
        let text = line["output"]
            .as_str()
            .or_else(|| line["text"].as_str())
            .or_else(|| line.as_str())
            .unwrap_or("");
        // Colorize common patterns
        let colored = colorize_log_line(text);
        println!("{}", colored);
    }

    if lines.is_empty() {
        println!("\x1b[33m(лог пуст)\x1b[0m");
    }

    println!("{}", "─".repeat(60));
    println!("Всего {} строк", lines.len());
    Ok(())
}

fn colorize_log_line(line: &str) -> String {
    let t = line.trim();
    if t.starts_with("PLAY ") || t.starts_with("TASK [") {
        format!("\x1b[1;34m{}\x1b[0m", line) // bold blue
    } else if t.contains("fatal:") || t.contains("UNREACHABLE") {
        format!("\x1b[1;31m{}\x1b[0m", line) // bold red
    } else if t.starts_with("ok:") {
        format!("\x1b[32m{}\x1b[0m", line) // green
    } else if t.starts_with("changed:") {
        format!("\x1b[33m{}\x1b[0m", line) // yellow
    } else if t.starts_with("skipping:") || t.starts_with("skipped:") {
        format!("\x1b[36m{}\x1b[0m", line) // cyan
    } else if t.starts_with("  + ") {
        format!("\x1b[32m{}\x1b[0m", line) // green (terraform add)
    } else if t.starts_with("  - ") {
        format!("\x1b[31m{}\x1b[0m", line) // red (terraform remove)
    } else if t.starts_with("  ~ ") {
        format!("\x1b[33m{}\x1b[0m", line) // yellow (terraform change)
    } else {
        line.to_string()
    }
}

fn cmd_approve(client: &Client, project: i32, task: i32) -> Result<(), String> {
    client.post_empty(&format!("/project/{}/tasks/{}/confirm", project, task))?;
    println!("✅ Задача #{} подтверждена", task);
    Ok(())
}

fn cmd_stop(client: &Client, project: i32, task: i32) -> Result<(), String> {
    client.post_empty(&format!("/project/{}/tasks/{}/stop", project, task))?;
    println!("⛔ Задача #{} остановлена", task);
    Ok(())
}

fn cmd_whoami(client: &Client) -> Result<(), String> {
    let data = client.get("/user")?;

    if client.json_output {
        println!("{}", serde_json::to_string_pretty(&data).unwrap());
        return Ok(());
    }

    println!("👤 Текущий пользователь:");
    println!("   ID:       {}", data["id"]);
    println!("   Логин:    {}", data["username"].as_str().unwrap_or("—"));
    println!("   Имя:      {}", data["name"].as_str().unwrap_or("—"));
    println!("   Email:    {}", data["email"].as_str().unwrap_or("—"));
    println!("   Admin:    {}", data["admin"].as_bool().unwrap_or(false));
    Ok(())
}

fn cmd_version(client: &Client) -> Result<(), String> {
    println!("velum CLI v{}", env!("CARGO_PKG_VERSION"));
    match client.get("/ping") {
        Ok(data) => println!(
            "Сервер: {} ({})",
            client.base_url,
            data.as_str().unwrap_or("ok")
        ),
        Err(_) => println!("Сервер: {} (недоступен)", client.base_url),
    }
    Ok(())
}

fn cmd_tasks(client: &Client, project: i32, limit: usize) -> Result<(), String> {
    let data = client.get(&format!("/project/{}/tasks?limit={}", project, limit))?;
    let tasks = data.as_array().ok_or("Expected array")?;

    if client.json_output {
        println!("{}", serde_json::to_string_pretty(&data).unwrap());
        return Ok(());
    }

    print_table(
        &[
            "ID",
            "Статус",
            "Шаблон",
            "Пользователь",
            "Создана",
            "Сообщение",
        ],
        tasks
            .iter()
            .map(|t| {
                let status = t["status"].as_str().unwrap_or("—");
                let created = t["created"].as_str().unwrap_or("—");
                let created_short = if created.len() > 16 {
                    &created[..16]
                } else {
                    created
                };
                vec![
                    t["id"].to_string(),
                    format!("{} {}", status_icon(status), status),
                    format!("#{}", t["template_id"].as_i64().unwrap_or(0)),
                    truncate(
                        t["user_name"]
                            .as_str()
                            .or_else(|| t["username"].as_str())
                            .unwrap_or("—"),
                        14,
                    ),
                    created_short.to_string(),
                    truncate(t["message"].as_str().unwrap_or("—"), 28),
                ]
            })
            .collect(),
    );
    println!("\nПоказано: {} из последних задач", tasks.len());
    Ok(())
}

// ─── Main ──────────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();

    // Get token
    let token = match cli.token.clone() {
        Some(t) if !t.is_empty() => t,
        _ => {
            // Try reading from ~/.velum/token
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_default();
            let token_path = format!("{}/.velum/token", home);
            std::fs::read_to_string(&token_path)
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        }
    };

    if token.is_empty() {
        match &cli.command {
            Commands::Version => {}
            _ => {
                eprintln!("❌ Требуется токен авторизации.");
                eprintln!(
                    "   Установите переменную VELUM_TOKEN или сохраните токен в ~/.velum/token"
                );
                eprintln!("   Получить токен: POST /api/auth/login → скопируйте поле 'token'");
                std::process::exit(1);
            }
        }
    }

    let json_output = cli.output == "json";
    let client = Client::new(
        cli.url.trim_end_matches('/').to_string(),
        token,
        json_output,
    );

    let result = match cli.command {
        Commands::Projects => cmd_projects(&client),
        Commands::Templates { project } => cmd_templates(&client, project),
        Commands::Run {
            project,
            template,
            message,
            branch,
            args,
            dry_run,
            wait,
        } => cmd_run(
            &client, project, template, message, branch, args, dry_run, wait,
        ),
        Commands::Status { project, running } => cmd_status(&client, project, running),
        Commands::Logs { project, task } => cmd_logs(&client, project, task),
        Commands::Approve { project, task } => cmd_approve(&client, project, task),
        Commands::Stop { project, task } => cmd_stop(&client, project, task),
        Commands::Whoami => cmd_whoami(&client),
        Commands::Version => cmd_version(&client),
        Commands::Tasks { project, limit } => cmd_tasks(&client, project, limit),
    };

    if let Err(e) = result {
        eprintln!("❌ {}", e);
        std::process::exit(1);
    }
}
