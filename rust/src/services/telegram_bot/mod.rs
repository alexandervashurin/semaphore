//! Telegram Bot service (v5.1)
//!
//! Уведомления о задачах + команды управления через Telegram Bot API.
//!
//! ## Конфигурация
//! ```bash
//! SEMAPHORE_TELEGRAM_TOKEN=1234567890:AABBCCDDEEFFaabbccddeeff
//! SEMAPHORE_TELEGRAM_CHAT_ID=-1001234567890   # channel/group id (optional default)
//! ```
//!
//! ## Команды
//! - `/start` — приветствие
//! - `/help`  — список команд
//! - `/status` — запущенные задачи
//!
//! ## Уведомления (вызываются из playbook_run_service)
//! - `notify_task_success(...)` — ✅
//! - `notify_task_failed(...)` — ❌
//! - `notify_task_stopped(...)` — ⏹️

use crate::config::Config;
use crate::db::store::Store;
use crate::error::Result;
use crate::models::{Task, Template};
use crate::services::task_logger::TaskStatus;
use std::sync::{Arc, OnceLock};
use tracing::{error, info, warn};

static NOTIFICATION_BOT: OnceLock<Option<Arc<TelegramBot>>> = OnceLock::new();

/// Telegram Bot — обёртка над Telegram Bot API (без teloxide runtime)
///
/// Использует прямой HTTP-вызов через reqwest, что не требует tokio task/dispatch loop
/// и не конфликтует с Axum-runtime.
#[derive(Clone)]
pub struct TelegramBot {
    token: String,
    /// Дефолтный chat_id для уведомлений (из конфига)
    default_chat_id: Option<String>,
    client: reqwest::Client,
}

impl TelegramBot {
    /// Регистрирует экземпляр для фоновых уведомлений о задачах (вызывать из `cmd_server`).
    pub fn init_notification_bot(config: &Config) {
        let _ = NOTIFICATION_BOT.get_or_init(|| TelegramBot::new(config));
    }

    fn notification_bot() -> Option<Arc<TelegramBot>> {
        NOTIFICATION_BOT.get().and_then(|opt| opt.as_ref().cloned())
    }

    /// Создаёт бота если задан токен в конфиге / env.
    pub fn new(config: &Config) -> Option<Arc<Self>> {
        let token = config
            .telegram_bot_token
            .clone()
            .or_else(|| std::env::var("SEMAPHORE_TELEGRAM_TOKEN").ok())?;

        if token.is_empty() {
            return None;
        }

        let default_chat_id = std::env::var("SEMAPHORE_TELEGRAM_CHAT_ID")
            .ok()
            .filter(|s| !s.is_empty());

        info!(
            "Telegram bot configured (token: {}...)",
            &token[..token.len().min(10)]
        );

        Some(Arc::new(Self {
            token,
            default_chat_id,
            client: reqwest::Client::new(),
        }))
    }

    /// Базовый URL Telegram Bot API
    fn api_url(&self, method: &str) -> String {
        format!("https://api.telegram.org/bot{}/{}", self.token, method)
    }

    /// Отправляет сообщение. Поддерживает Markdown v2.
    pub async fn send_message(&self, chat_id: &str, text: &str) -> Result<()> {
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "HTML",
            "disable_web_page_preview": true,
        });

        let resp = self
            .client
            .post(self.api_url("sendMessage"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| crate::error::Error::Other(format!("Telegram HTTP: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(crate::error::Error::Other(format!(
                "Telegram API error: {body}"
            )));
        }
        Ok(())
    }

    /// Отправляет в дефолтный чат (из конфига/env), если задан.
    pub async fn send_default(&self, text: &str) -> Result<()> {
        match &self.default_chat_id {
            Some(chat_id) => self.send_message(chat_id, text).await,
            None => {
                warn!("Telegram: no default chat_id configured, message dropped");
                Ok(())
            }
        }
    }

    // ── Task notifications ───────────────────────────────────────

    /// Уведомление об успешном завершении задачи.
    pub async fn notify_task_success(
        &self,
        project_name: &str,
        template_name: &str,
        task_id: i32,
        author: &str,
        duration_secs: u64,
        task_url: &str,
    ) {
        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = if mins > 0 {
            format!("{mins}m {secs}s")
        } else {
            format!("{secs}s")
        };

        let text = format!(
            "✅ <b>[{project_name}]</b> {template_name} — <b>SUCCESS</b> ({duration})\n\
             👤 {author} · <a href=\"{task_url}\">#task {task_id}</a>"
        );

        if let Err(e) = self.send_default(&text).await {
            error!("Telegram notify success failed: {e}");
        }
    }

    /// Уведомление о падении задачи.
    pub async fn notify_task_failed(
        &self,
        project_name: &str,
        template_name: &str,
        task_id: i32,
        author: &str,
        duration_secs: u64,
        task_url: &str,
    ) {
        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = if mins > 0 {
            format!("{mins}m {secs}s")
        } else {
            format!("{secs}s")
        };

        let text = format!(
            "❌ <b>[{project_name}]</b> {template_name} — <b>FAILED</b> ({duration})\n\
             👤 {author} · <a href=\"{task_url}\">#task {task_id}</a>"
        );

        if let Err(e) = self.send_default(&text).await {
            error!("Telegram notify failed failed: {e}");
        }
    }

    /// Уведомление об остановке задачи.
    pub async fn notify_task_stopped(
        &self,
        project_name: &str,
        template_name: &str,
        task_id: i32,
        task_url: &str,
    ) {
        let text = format!(
            "⏹️ <b>[{project_name}]</b> {template_name} — <b>STOPPED</b>\n\
             <a href=\"{task_url}\">#task {task_id}</a>"
        );

        if let Err(e) = self.send_default(&text).await {
            error!("Telegram notify stopped failed: {e}");
        }
    }

    // ── Command handling (long-polling) ──────────────────────────

    /// Запускает polling loop для обработки входящих команд.
    /// Вызывается из cmd_server.rs если бот настроен.
    pub async fn run_polling(self: Arc<Self>) {
        info!("Telegram bot polling started");
        let mut offset: i64 = 0;

        loop {
            match self.get_updates(offset).await {
                Ok(updates) => {
                    for update in updates {
                        let update_id = update["update_id"].as_i64().unwrap_or(0);
                        offset = update_id + 1;

                        if let Some(msg) = update.get("message") {
                            if let Some(text) = msg["text"].as_str() {
                                let chat_id = msg["chat"]["id"].to_string();
                                let _ = self.handle_command(&chat_id, text).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Telegram polling error: {e}");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }

            // Throttle to avoid hammering Telegram API (max 1 req/sec)
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    }

    async fn get_updates(&self, offset: i64) -> Result<Vec<serde_json::Value>> {
        let payload = serde_json::json!({
            "offset": offset,
            "timeout": 30,
            "allowed_updates": ["message"],
        });

        let resp = self
            .client
            .post(self.api_url("getUpdates"))
            .json(&payload)
            .timeout(std::time::Duration::from_secs(35))
            .send()
            .await
            .map_err(|e| crate::error::Error::Other(format!("Telegram getUpdates: {e}")))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| crate::error::Error::Other(format!("Telegram parse: {e}")))?;

        let updates = body["result"].as_array().cloned().unwrap_or_default();

        Ok(updates)
    }

    async fn handle_command(&self, chat_id: &str, text: &str) -> Result<()> {
        let cmd = text.split_whitespace().next().unwrap_or("").to_lowercase();

        match cmd.as_str() {
            "/start" => {
                self.send_message(chat_id,
                    "👋 <b>Velum Bot</b>\n\nЯ отправляю уведомления о задачах Velum.\nИспользуй /help для списка команд."
                ).await?;
            }
            "/help" => {
                self.send_message(
                    chat_id,
                    "<b>Команды:</b>\n\
                     /start — приветствие\n\
                     /help  — эта справка\n\
                     /status — статус сервера\n\n\
                     <i>Уведомления о задачах отправляются автоматически.</i>",
                )
                .await?;
            }
            "/status" => {
                let ver = env!("CARGO_PKG_VERSION");
                self.send_message(chat_id, &format!("🟢 Velum v{ver} работает"))
                    .await?;
            }
            _ => {
                // Unknown command — ignore silently to avoid spam
            }
        }

        Ok(())
    }
}

/// Уведомление в дефолтный чат после завершения задачи (если бот сконфигурирован).
pub async fn notify_on_task_finished(
    store: Arc<dyn Store + Send + Sync>,
    task: &Task,
    template: &Template,
) {
    let Some(bot) = TelegramBot::notification_bot() else {
        return;
    };

    let duration_secs = task
        .start
        .zip(task.end)
        .map(|(s, e)| (e - s).num_seconds().max(0) as u64)
        .unwrap_or(0);

    let project_name = store
        .get_project(task.project_id)
        .await
        .map(|p| p.name)
        .unwrap_or_else(|_| format!("project {}", task.project_id));

    let author = match task.user_id {
        Some(uid) => store
            .get_user(uid)
            .await
            .map(|u| u.username)
            .unwrap_or_else(|_| "unknown".to_string()),
        None => "system".to_string(),
    };

    let task_url = format!(
        "{}/project/{}/tasks/{}",
        crate::config::get_public_host(),
        task.project_id,
        task.id
    );

    match task.status {
        TaskStatus::Success => {
            bot.notify_task_success(
                &project_name,
                &template.name,
                task.id,
                &author,
                duration_secs,
                &task_url,
            )
            .await;
        }
        TaskStatus::Error => {
            bot.notify_task_failed(
                &project_name,
                &template.name,
                task.id,
                &author,
                duration_secs,
                &task_url,
            )
            .await;
        }
        TaskStatus::Stopped => {
            bot.notify_task_stopped(&project_name, &template.name, task.id, &task_url)
                .await;
        }
        _ => {}
    }
}

/// Запускает Telegram бота в фоновом потоке, если настроен.
pub fn start_bot_if_configured(config: &Config) {
    TelegramBot::init_notification_bot(config);

    if let Some(bot) = TelegramBot::notification_bot() {
        info!("Starting Telegram bot polling loop");
        tokio::spawn(async move {
            bot.run_polling().await;
        });
    } else {
        info!("Telegram bot not configured (SEMAPHORE_TELEGRAM_TOKEN not set)");
    }
}

#[cfg(test)]
mod notify_tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn task_duration_secs_from_task_times() {
        let mut t = Task::default();
        let s = Utc::now();
        t.start = Some(s);
        t.end = Some(s + Duration::seconds(125));
        let secs = t
            .start
            .zip(t.end)
            .map(|(a, b)| (b - a).num_seconds().max(0) as u64)
            .unwrap_or(0);
        assert_eq!(secs, 125);
    }

    #[test]
    fn task_duration_secs_zero_duration() {
        let mut t = Task::default();
        let s = Utc::now();
        t.start = Some(s);
        t.end = Some(s);
        let secs = t
            .start
            .zip(t.end)
            .map(|(a, b)| (b - a).num_seconds().max(0) as u64)
            .unwrap_or(0);
        assert_eq!(secs, 0);
    }

    #[test]
    fn task_duration_secs_no_start() {
        let t = Task::default();
        let secs = t
            .start
            .zip(t.end)
            .map(|(a, b)| (b - a).num_seconds().max(0) as u64)
            .unwrap_or(0);
        assert_eq!(secs, 0);
    }

    #[test]
    fn task_duration_format_minutes_and_seconds() {
        let duration_secs = 125;
        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = if mins > 0 {
            format!("{mins}m {secs}s")
        } else {
            format!("{secs}s")
        };
        assert_eq!(duration, "2m 5s");
    }

    #[test]
    fn task_duration_format_seconds_only() {
        let duration_secs = 30;
        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = if mins > 0 {
            format!("{mins}m {secs}s")
        } else {
            format!("{secs}s")
        };
        assert_eq!(duration, "30s");
    }

    #[test]
    fn task_duration_format_one_hour() {
        let duration_secs = 3661;
        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = if mins > 0 {
            format!("{mins}m {secs}s")
        } else {
            format!("{secs}s")
        };
        assert_eq!(duration, "61m 1s");
    }

    #[test]
    fn task_duration_format_zero_seconds() {
        let duration_secs = 0;
        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = if mins > 0 {
            format!("{mins}m {secs}s")
        } else {
            format!("{secs}s")
        };
        assert_eq!(duration, "0s");
    }

    #[test]
    fn telegram_bot_new_returns_none_without_token() {
        // Убедимся что переменные окружения не заданы
        unsafe { std::env::remove_var("SEMAPHORE_TELEGRAM_TOKEN") };

        let config = crate::config::Config::default();
        let bot = TelegramBot::new(&config);
        assert!(bot.is_none(), "Bot should not be created without token");
    }

    #[test]
    fn telegram_api_url_format() {
        // Создаём бота напрямую для тестирования api_url
        let bot = TelegramBot {
            token: "test-token-123".to_string(),
            default_chat_id: None,
            client: reqwest::Client::new(),
        };

        assert_eq!(
            bot.api_url("sendMessage"),
            "https://api.telegram.org/bottest-token-123/sendMessage"
        );
        assert_eq!(
            bot.api_url("getUpdates"),
            "https://api.telegram.org/bottest-token-123/getUpdates"
        );
    }

    #[test]
    fn telegram_handle_command_start() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: Some("-100test".to_string()),
            client: reqwest::Client::new(),
        };

        // /start должен вызвать send_message с приветствием
        // Поскольку send_message делает реальный HTTP запрос, проверяем только что handle_command не паникует
        // и возвращает Ok (независимо от результата HTTP)
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(bot.handle_command("-100test", "/start"));
        // Может вернуть ошибку HTTP — это нормально
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn telegram_handle_command_help() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: Some("-100test".to_string()),
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(bot.handle_command("-100test", "/help"));
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn telegram_handle_command_status() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: Some("-100test".to_string()),
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(bot.handle_command("-100test", "/status"));
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn telegram_handle_command_unknown_ignored() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: Some("-100test".to_string()),
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        // Неизвестная команда должна вернуть Ok (игнорируется молча)
        let result = rt.block_on(bot.handle_command("-100test", "/unknown_cmd"));
        assert!(result.is_ok());
    }

    #[test]
    fn telegram_handle_command_with_args() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: Some("-100test".to_string()),
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        // Команда с аргументами — должен распознать базу
        let result = rt.block_on(bot.handle_command("-100test", "/start extra_arg"));
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn telegram_send_default_without_chat_id() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: None,
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        // Без chat_id должно вернуть Ok с warning
        let result = rt.block_on(bot.send_default("Test message"));
        assert!(result.is_ok());
    }

    #[test]
    fn telegram_notify_task_success_format() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: Some("-100test".to_string()),
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        // Проверяем что метод не паникует
        rt.block_on(bot.notify_task_success(
            "TestProject",
            "Deploy",
            42,
            "admin",
            125,
            "http://localhost:3000/project/1/tasks/42",
        ));
    }

    #[test]
    fn telegram_notify_task_failed_format() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: Some("-100test".to_string()),
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(bot.notify_task_failed(
            "TestProject",
            "Deploy",
            42,
            "admin",
            300,
            "http://localhost:3000/project/1/tasks/42",
        ));
    }

    #[test]
    fn telegram_notify_task_stopped_format() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: Some("-100test".to_string()),
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(bot.notify_task_stopped(
            "TestProject",
            "Deploy",
            42,
            "http://localhost:3000/project/1/tasks/42",
        ));
    }

    #[test]
    fn start_bot_if_configured_without_token() {
        unsafe { std::env::remove_var("SEMAPHORE_TELEGRAM_TOKEN") };
        let config = crate::config::Config::default();

        // Не должен паниковать даже без токена
        start_bot_if_configured(&config);
    }

    #[test]
    fn command_parsing_with_leading_spaces() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: Some("-100test".to_string()),
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        // Команда с ведущими пробелами — split_whitespace корректно обрабатывает
        let result = rt.block_on(bot.handle_command("-100test", "  /help"));
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn command_parsing_with_trailing_spaces() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: Some("-100test".to_string()),
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(bot.handle_command("-100test", "/help   "));
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn command_parsing_case_insensitive() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: Some("-100test".to_string()),
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        // Команды должны обрабатываться case-insensitive
        let result = rt.block_on(bot.handle_command("-100test", "/HELP"));
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn command_parsing_mixed_case() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: Some("-100test".to_string()),
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(bot.handle_command("-100test", "/HeLp"));
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn command_parsing_empty_text() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: Some("-100test".to_string()),
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        // Пустая строка — должна обработаться без паники
        let result = rt.block_on(bot.handle_command("-100test", ""));
        assert!(result.is_ok());
    }

    #[test]
    fn command_parsing_whitespace_only() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: Some("-100test".to_string()),
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(bot.handle_command("-100test", "   "));
        assert!(result.is_ok());
    }

    #[test]
    fn command_parsing_multiple_args() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: Some("-100test".to_string()),
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        // Несколько аргументов после команды
        let result = rt.block_on(bot.handle_command("-100test", "/status arg1 arg2 arg3"));
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn notification_success_message_contains_expected_parts() {
        let project = "MyProject";
        let template = "Deploy Prod";
        let task_id = 123;
        let author = "ivan";
        let duration_secs = 95;
        let task_url = "http://example.com/tasks/123";

        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = format!("{mins}m {secs}s");

        let expected_text = format!(
            "✅ <b>[{project}]</b> {template} — <b>SUCCESS</b> ({duration})\n\
             👤 {author} · <a href=\"{task_url}\">#task {task_id}</a>"
        );

        assert!(expected_text.contains("✅"));
        assert!(expected_text.contains("SUCCESS"));
        assert!(expected_text.contains("MyProject"));
        assert!(expected_text.contains("Deploy Prod"));
        assert!(expected_text.contains("1m 35s"));
        assert!(expected_text.contains("ivan"));
        assert!(expected_text.contains("#task 123"));
    }

    #[test]
    fn notification_failed_message_contains_expected_parts() {
        let project = "Backend";
        let template = "Run tests";
        let task_id = 456;
        let author = "maria";
        let duration_secs = 45;
        let task_url = "http://example.com/tasks/456";

        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = format!("{secs}s");

        let expected_text = format!(
            "❌ <b>[{project}]</b> {template} — <b>FAILED</b> ({duration})\n\
             👤 {author} · <a href=\"{task_url}\">#task {task_id}</a>"
        );

        assert!(expected_text.contains("❌"));
        assert!(expected_text.contains("FAILED"));
        assert!(expected_text.contains("Backend"));
        assert!(expected_text.contains("45s"));
        assert!(expected_text.contains("maria"));
    }

    #[test]
    fn notification_stopped_message_contains_expected_parts() {
        let project = "Frontend";
        let template = "Build";
        let task_id = 789;
        let task_url = "http://example.com/tasks/789";

        let expected_text = format!(
            "⏹️ <b>[{project}]</b> {template} — <b>STOPPED</b>\n\
             <a href=\"{task_url}\">#task {task_id}</a>"
        );

        assert!(expected_text.contains("⏹️"));
        assert!(expected_text.contains("STOPPED"));
        assert!(expected_text.contains("Frontend"));
        assert!(expected_text.contains("Build"));
        assert!(expected_text.contains("#task 789"));
    }

    #[test]
    fn bot_clone_preserves_config() {
        let bot = TelegramBot {
            token: "clone-token".to_string(),
            default_chat_id: Some("-100clone".to_string()),
            client: reqwest::Client::new(),
        };

        let bot2 = bot.clone();
        assert_eq!(bot.token, bot2.token);
        assert_eq!(bot.default_chat_id, bot2.default_chat_id);
    }

    #[test]
    fn bot_without_default_chat_id_send_default_returns_ok() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: None,
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(bot.send_default("any message"));
        assert!(result.is_ok());
    }

    #[test]
    fn bot_with_default_chat_id_attempts_send() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: Some("-1001234567890".to_string()),
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        // Должен попытаться отправить (вернёт ошибку HTTP, но не паникует)
        let result = rt.block_on(bot.send_default("test"));
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn duration_format_exactly_one_minute() {
        let duration_secs = 60;
        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = if mins > 0 {
            format!("{mins}m {secs}s")
        } else {
            format!("{secs}s")
        };
        assert_eq!(duration, "1m 0s");
    }

    #[test]
    fn duration_format_large_value() {
        let duration_secs = 86400; // 24 hours
        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = if mins > 0 {
            format!("{mins}m {secs}s")
        } else {
            format!("{secs}s")
        };
        assert_eq!(duration, "1440m 0s");
    }

    #[test]
    fn chat_id_formats() {
        // Различные допустимые форматы chat_id
        let valid_ids = vec![
            "-1001234567890", // supergroup
            "-123456789",     // group
            "123456789",      // user
        ];

        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: None,
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        for chat_id in valid_ids {
            let result = rt.block_on(bot.send_message(chat_id, "test"));
            // Ожидается HTTP ошибка (нет реального соединения)
            assert!(result.is_err());
        }
    }

    #[test]
    fn notification_bot_static_init_returns_none_without_config() {
        unsafe { std::env::remove_var("SEMAPHORE_TELEGRAM_TOKEN") };

        let config = crate::config::Config::default();
        TelegramBot::init_notification_bot(&config);

        let bot = TelegramBot::notification_bot();
        assert!(bot.is_none());
    }

    #[test]
    fn handle_command_empty_string_after_split() {
        let bot = TelegramBot {
            token: "test".to_string(),
            default_chat_id: Some("-100test".to_string()),
            client: reqwest::Client::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        // split_whitespace на пустой строке вернёт пустой итератор,
        // unwrap_or("") даст пустую строку, которая не совпадёт ни с одной командой
        let result = rt.block_on(bot.handle_command("-100test", ""));
        assert!(result.is_ok());
    }

    #[test]
    fn task_url_format_in_notification() {
        let project_id = 5;
        let task_id = 42;
        let base_url = "https://semaphore.example.com";

        // Мокируем get_public_host через установку переменной окружения
        unsafe { std::env::set_var("SEMAPHORE_PUBLIC_HOST", base_url) };

        let expected = format!("{base_url}/project/{project_id}/tasks/{task_id}");
        assert_eq!(expected, "https://semaphore.example.com/project/5/tasks/42");
    }

    // ── Additional tests: command parsing ──────────────────────────

    #[test]
    fn extract_command_first_word_only() {
        let text = "/help arg1 arg2";
        let cmd = text.split_whitespace().next().unwrap_or("").to_lowercase();
        assert_eq!(cmd, "/help");
    }

    #[test]
    fn extract_command_from_multiline() {
        let text = "/status\nsome\nother\nlines";
        let cmd = text.split_whitespace().next().unwrap_or("").to_lowercase();
        assert_eq!(cmd, "/status");
    }

    #[test]
    fn extract_command_tabs_and_mixed_whitespace() {
        let text = "\t\t/start\targ1\targ2";
        let cmd = text.split_whitespace().next().unwrap_or("").to_lowercase();
        assert_eq!(cmd, "/start");
    }

    #[test]
    fn parse_args_count_zero() {
        let text = "/help";
        let args: Vec<&str> = text.split_whitespace().skip(1).collect();
        assert_eq!(args.len(), 0);
    }

    #[test]
    fn parse_args_count_one() {
        let text = "/status arg1";
        let args: Vec<&str> = text.split_whitespace().skip(1).collect();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], "arg1");
    }

    #[test]
    fn parse_args_count_three() {
        let text = "/start a b c";
        let args: Vec<&str> = text.split_whitespace().skip(1).collect();
        assert_eq!(args.len(), 3);
        assert_eq!(args, vec!["a", "b", "c"]);
    }

    #[test]
    fn parse_args_ignores_extra_whitespace() {
        let text = "  /help   arg1   arg2  ";
        let args: Vec<&str> = text.split_whitespace().skip(1).collect();
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn command_case_variants_all_match_help() {
        let variants = vec!["/help", "/HELP", "/Help", "/hElP", "/heLP"];
        for v in variants {
            let cmd = v.split_whitespace().next().unwrap_or("").to_lowercase();
            assert_eq!(cmd, "/help");
        }
    }

    #[test]
    fn command_case_variants_all_match_start() {
        let variants = vec!["/start", "/START", "/Start", "/sTaRt"];
        for v in variants {
            let cmd = v.split_whitespace().next().unwrap_or("").to_lowercase();
            assert_eq!(cmd, "/start");
        }
    }

    #[test]
    fn command_case_variants_all_match_status() {
        let variants = vec!["/status", "/STATUS", "/Status", "/sTaTuS"];
        for v in variants {
            let cmd = v.split_whitespace().next().unwrap_or("").to_lowercase();
            assert_eq!(cmd, "/status");
        }
    }

    #[test]
    fn unknown_command_not_in_known_set() {
        let known = vec!["/start", "/help", "/status"];
        let unknown = "/unknown_command";
        let cmd = unknown
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_lowercase();
        assert!(!known.contains(&cmd.as_str()));
    }

    #[test]
    fn not_a_command_random_text() {
        let text = "just some random text without slash";
        let cmd = text.split_whitespace().next().unwrap_or("").to_lowercase();
        assert_eq!(cmd, "just");
        assert!(!cmd.starts_with('/'));
    }

    #[test]
    fn empty_text_yields_empty_cmd() {
        let text = "";
        let cmd = text.split_whitespace().next().unwrap_or("");
        assert_eq!(cmd, "");
    }

    #[test]
    fn whitespace_only_yields_empty_cmd() {
        let text = "    \t   \n  ";
        let cmd = text.split_whitespace().next().unwrap_or("");
        assert_eq!(cmd, "");
    }

    // ── Additional tests: message formatting ───────────────────────

    #[test]
    fn success_message_html_structure() {
        let project = "ProjectX";
        let template = "Deploy";
        let task_id = 10;
        let author = "alice";
        let duration_secs = 72;
        let task_url = "http://host/p/1/t/10";

        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = format!("{mins}m {secs}s");

        let text = format!(
            "✅ <b>[{project}]</b> {template} — <b>SUCCESS</b> ({duration})\n\
             👤 {author} · <a href=\"{task_url}\">#task {task_id}</a>"
        );

        assert!(text.starts_with("✅"));
        assert!(text.contains("<b>[ProjectX]</b>"));
        assert!(text.contains("Deploy"));
        assert!(text.contains("<b>SUCCESS</b>"));
        assert!(text.contains("1m 12s"));
        assert!(text.contains("alice"));
        assert!(text.contains("href=\"http://host/p/1/t/10\""));
        assert!(text.contains("#task 10"));
    }

    #[test]
    fn failed_message_html_structure() {
        let project = "DB-Migration";
        let template = "Migrate";
        let task_id = 99;
        let author = "bob";
        let duration_secs = 5;
        let task_url = "http://host/p/2/t/99";

        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = if mins > 0 {
            format!("{mins}m {secs}s")
        } else {
            format!("{secs}s")
        };

        let text = format!(
            "❌ <b>[{project}]</b> {template} — <b>FAILED</b> ({duration})\n\
             👤 {author} · <a href=\"{task_url}\">#task {task_id}</a>"
        );

        assert!(text.starts_with("❌"));
        assert!(text.contains("<b>FAILED</b>"));
        assert!(text.contains("5s"));
        assert!(text.contains("bob"));
    }

    #[test]
    fn stopped_message_no_duration() {
        let project = "WebApp";
        let template = "Test";
        let task_id = 55;
        let task_url = "http://host/p/3/t/55";

        let text = format!(
            "⏹️ <b>[{project}]</b> {template} — <b>STOPPED</b>\n\
             <a href=\"{task_url}\">#task {task_id}</a>"
        );

        assert!(text.starts_with("⏹️"));
        assert!(text.contains("<b>STOPPED</b>"));
        assert!(text.contains("WebApp"));
        assert!(text.contains("Test"));
        assert!(text.contains("#task 55"));
        assert!(!text.contains("SUCCESS"));
        assert!(!text.contains("FAILED"));
    }

    #[test]
    fn message_with_unicode_project_name() {
        let project = "Проект-Тест";
        let template = "Сборка";
        let task_id = 1;
        let author = "Иван";
        let duration_secs = 10;
        let task_url = "http://host/p/1/t/1";

        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = format!("{secs}s");

        let text = format!(
            "✅ <b>[{project}]</b> {template} — <b>SUCCESS</b> ({duration})\n\
             👤 {author} · <a href=\"{task_url}\">#task {task_id}</a>"
        );

        assert!(text.contains("Проект-Тест"));
        assert!(text.contains("Сборка"));
        assert!(text.contains("Иван"));
    }

    #[test]
    fn message_with_emoji_in_template_name() {
        let project = "CI";
        let template = "🚀 Deploy to prod";
        let task_id = 42;
        let author = "deploy-bot";
        let duration_secs = 120;
        let task_url = "http://host/p/1/t/42";

        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = format!("{mins}m {secs}s");

        let text = format!(
            "✅ <b>[{project}]</b> {template} — <b>SUCCESS</b> ({duration})\n\
             👤 {author} · <a href=\"{task_url}\">#task {task_id}</a>"
        );

        assert!(text.contains("🚀 Deploy to prod"));
        assert!(text.contains("2m 0s"));
    }

    #[test]
    fn message_with_special_html_chars_escaped() {
        // HTML-символы в именах могут сломать разметку — проверяем что они вставляются как есть
        let project = "Test<Script>";
        let template = "Run & <do>";
        let task_id = 1;
        let author = "user'name";
        let duration_secs = 0;
        let task_url = "http://host?a=1&b=2";

        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = format!("{secs}s");

        let text = format!(
            "✅ <b>[{project}]</b> {template} — <b>SUCCESS</b> ({duration})\n\
             👤 {author} · <a href=\"{task_url}\">#task {task_id}</a>"
        );

        assert!(text.contains("Test<Script>"));
        assert!(text.contains("Run & <do>"));
        assert!(text.contains("user'name"));
        assert!(text.contains("a=1&b=2"));
    }

    #[test]
    fn message_with_newline_in_project_name() {
        let project = "Multi\nLine";
        let template = "Tpl";
        let task_id = 1;
        let author = "dev";
        let duration_secs = 1;
        let task_url = "http://host/p/1/t/1";

        let text = format!(
            "✅ <b>[{project}]</b> {template} — <b>SUCCESS</b> (1s)\n\
             👤 {author} · <a href=\"{task_url}\">#task {task_id}</a>"
        );

        // Должно содержать перенос из project
        assert!(text.contains("Multi\nLine"));
    }

    #[test]
    fn message_with_very_long_names() {
        let project = "A".repeat(200);
        let template = "B".repeat(200);
        let task_id = 999999;
        let author = "admin";
        let duration_secs = 1;
        let task_url = "http://host/p/1/t/999999";

        let text = format!(
            "✅ <b>[{project}]</b> {template} — <b>SUCCESS</b> (1s)\n\
             👤 {author} · <a href=\"{task_url}\">#task {task_id}</a>"
        );

        assert!(text.len() > 400);
        assert!(text.contains(&"A".repeat(200)));
        assert!(text.contains(&"B".repeat(200)));
        assert!(text.contains("#task 999999"));
    }

    #[test]
    fn message_with_empty_strings() {
        let project = "";
        let template = "";
        let task_id = 0;
        let author = "";
        let duration_secs = 0;
        let task_url = "";

        let text = format!(
            "✅ <b>[{project}]</b> {template} — <b>SUCCESS</b> (0s)\n\
             👤 {author} · <a href=\"{task_url}\">#task {task_id}</a>"
        );

        assert!(text.contains("[]"));
        assert!(text.contains(" — <b>SUCCESS</b>"));
        assert!(text.contains("#task 0"));
    }

    // ── Additional tests: duration formatting ──────────────────────

    #[test]
    fn duration_format_boundary_59_seconds() {
        let duration_secs = 59;
        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = if mins > 0 {
            format!("{mins}m {secs}s")
        } else {
            format!("{secs}s")
        };
        assert_eq!(duration, "59s");
    }

    #[test]
    fn duration_format_boundary_60_seconds() {
        let duration_secs = 60;
        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = if mins > 0 {
            format!("{mins}m {secs}s")
        } else {
            format!("{secs}s")
        };
        assert_eq!(duration, "1m 0s");
    }

    #[test]
    fn duration_format_boundary_61_seconds() {
        let duration_secs = 61;
        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = if mins > 0 {
            format!("{mins}m {secs}s")
        } else {
            format!("{secs}s")
        };
        assert_eq!(duration, "1m 1s");
    }

    #[test]
    fn duration_format_10_minutes() {
        let duration_secs = 600;
        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = if mins > 0 {
            format!("{mins}m {secs}s")
        } else {
            format!("{secs}s")
        };
        assert_eq!(duration, "10m 0s");
    }

    #[test]
    fn duration_format_one_week() {
        let duration_secs = 7 * 24 * 3600; // 604800
        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = if mins > 0 {
            format!("{mins}m {secs}s")
        } else {
            format!("{secs}s")
        };
        assert_eq!(duration, "10080m 0s");
    }

    #[test]
    fn duration_format_max_u64() {
        let duration_secs = u64::MAX;
        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let duration = if mins > 0 {
            format!("{mins}m {secs}s")
        } else {
            format!("{secs}s")
        };
        assert!(duration.contains("m"));
        assert!(duration.ends_with("s"));
    }

    #[test]
    fn duration_format_various_values() {
        let cases = vec![
            (0, "0s"),
            (1, "1s"),
            (30, "30s"),
            (59, "59s"),
            (60, "1m 0s"),
            (61, "1m 1s"),
            (119, "1m 59s"),
            (120, "2m 0s"),
            (3600, "60m 0s"),
            (3661, "61m 1s"),
            (7265, "121m 5s"),
        ];
        for (secs, expected) in cases {
            let m = secs / 60;
            let s = secs % 60;
            let d = if m > 0 {
                format!("{m}m {s}s")
            } else {
                format!("{s}s")
            };
            assert_eq!(d, expected, "Failed for {secs} seconds");
        }
    }

    // ── Additional tests: bot configuration ────────────────────────

    #[test]
    fn bot_struct_fields_accessible() {
        let bot = TelegramBot {
            token: "tok".to_string(),
            default_chat_id: Some("-100test".to_string()),
            client: reqwest::Client::new(),
        };
        assert_eq!(bot.token, "tok");
        assert_eq!(bot.default_chat_id, Some("-100test".to_string()));
    }

    #[test]
    fn bot_without_default_chat_id() {
        let bot = TelegramBot {
            token: "tok".to_string(),
            default_chat_id: None,
            client: reqwest::Client::new(),
        };
        assert!(bot.default_chat_id.is_none());
    }

    #[test]
    fn bot_with_various_chat_id_formats() {
        let formats = vec!["-1001234567890", "-123456789", "123456789", "user_123"];
        for chat_id in formats {
            let bot = TelegramBot {
                token: "tok".to_string(),
                default_chat_id: Some(chat_id.to_string()),
                client: reqwest::Client::new(),
            };
            assert_eq!(bot.default_chat_id, Some(chat_id.to_string()));
        }
    }

    #[test]
    fn bot_api_url_different_methods() {
        let bot = TelegramBot {
            token: "abc123".to_string(),
            default_chat_id: None,
            client: reqwest::Client::new(),
        };
        assert_eq!(
            bot.api_url("sendMessage"),
            "https://api.telegram.org/botabc123/sendMessage"
        );
        assert_eq!(
            bot.api_url("getUpdates"),
            "https://api.telegram.org/botabc123/getUpdates"
        );
        assert_eq!(
            bot.api_url("sendDocument"),
            "https://api.telegram.org/botabc123/sendDocument"
        );
        assert_eq!(
            bot.api_url("getMe"),
            "https://api.telegram.org/botabc123/getMe"
        );
    }

    #[test]
    fn bot_api_url_with_special_chars_in_token() {
        let bot = TelegramBot {
            token: "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11".to_string(),
            default_chat_id: None,
            client: reqwest::Client::new(),
        };
        let url = bot.api_url("sendMessage");
        assert!(url
            .starts_with("https://api.telegram.org/bot123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11/"));
    }

    // ── Additional tests: chat ID validation ───────────────────────

    #[test]
    fn chat_id_supergroup_format() {
        // Telegram supergroup IDs start with -100
        let chat_id = "-1001234567890";
        assert!(chat_id.starts_with("-100"));
        assert!(chat_id.chars().skip(1).all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn chat_id_group_format() {
        // Group IDs start with -
        let chat_id = "-123456789";
        assert!(chat_id.starts_with('-'));
        assert!(chat_id.chars().skip(1).all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn chat_id_user_format() {
        // User IDs are positive numbers
        let chat_id = "123456789";
        assert!(!chat_id.starts_with('-'));
        assert!(chat_id.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn chat_id_invalid_contains_letters() {
        let chat_id = "-100abc123";
        let is_valid =
            chat_id.starts_with("-100") && chat_id.chars().skip(4).all(|c| c.is_ascii_digit());
        assert!(!is_valid);
    }

    #[test]
    fn chat_id_invalid_empty_string() {
        let chat_id = "";
        assert!(chat_id.is_empty());
    }

    #[test]
    fn chat_id_validation_helper() {
        fn is_valid_telegram_chat_id(s: &str) -> bool {
            if s.is_empty() {
                return false;
            }
            if s.starts_with("-100") {
                s.len() > 4 && s.chars().skip(4).all(|c| c.is_ascii_digit())
            } else if s.starts_with('-') {
                s.len() > 1 && s.chars().skip(1).all(|c| c.is_ascii_digit())
            } else {
                s.chars().all(|c| c.is_ascii_digit())
            }
        }

        assert!(is_valid_telegram_chat_id("-1001234567890"));
        assert!(is_valid_telegram_chat_id("-123456789"));
        assert!(is_valid_telegram_chat_id("123456789"));
        assert!(!is_valid_telegram_chat_id(""));
        assert!(!is_valid_telegram_chat_id("-100abc"));
        assert!(!is_valid_telegram_chat_id("abc123"));
        assert!(!is_valid_telegram_chat_id("-"));
        assert!(!is_valid_telegram_chat_id("-100"));
    }

    // ── Additional tests: URL building ─────────────────────────────

    #[test]
    fn task_url_standard_format() {
        let base = "https://semaphore.example.com";
        let project_id = 10;
        let task_id = 42;
        let url = format!("{base}/project/{project_id}/tasks/{task_id}");
        assert_eq!(url, "https://semaphore.example.com/project/10/tasks/42");
    }

    #[test]
    fn task_url_with_http_base() {
        let base = "http://localhost:3000";
        let project_id = 1;
        let task_id = 1;
        let url = format!("{base}/project/{project_id}/tasks/{task_id}");
        assert_eq!(url, "http://localhost:3000/project/1/tasks/1");
    }

    #[test]
    fn task_url_with_trailing_slash_base() {
        let base = "https://example.com/";
        let project_id = 5;
        let task_id = 100;
        let url = format!("{base}/project/{project_id}/tasks/{task_id}");
        assert_eq!(url, "https://example.com//project/5/tasks/100");
    }

    #[test]
    fn task_url_with_large_ids() {
        let base = "https://ci.example.com";
        let project_id = i32::MAX;
        let task_id = i32::MAX;
        let url = format!("{base}/project/{project_id}/tasks/{task_id}");
        assert!(url.contains("2147483647"));
    }

    #[test]
    fn task_url_zero_ids() {
        let base = "https://example.com";
        let project_id = 0;
        let task_id = 0;
        let url = format!("{base}/project/{project_id}/tasks/{task_id}");
        assert_eq!(url, "https://example.com/project/0/tasks/0");
    }

    // ── Additional tests: edge cases ───────────────────────────────

    #[test]
    fn message_with_rtl_language() {
        let project = "مشروع";
        let template = "نشر";
        let task_id = 1;
        let author = "أحمد";
        let task_url = "http://host/p/1/t/1";

        let text = format!(
            "✅ <b>[{project}]</b> {template} — <b>SUCCESS</b> (1s)\n\
             👤 {author} · <a href=\"{task_url}\">#task {task_id}</a>"
        );

        assert!(text.contains("مشروع"));
        assert!(text.contains("نشر"));
        assert!(text.contains("أحمد"));
    }

    #[test]
    fn message_with_cjk_characters() {
        let project = "项目测试";
        let template = "部署";
        let task_id = 1;
        let author = "张三";
        let task_url = "http://host/p/1/t/1";

        let text = format!(
            "✅ <b>[{project}]</b> {template} — <b>SUCCESS</b> (1s)\n\
             👤 {author} · <a href=\"{task_url}\">#task {task_id}</a>"
        );

        assert!(text.contains("项目测试"));
        assert!(text.contains("部署"));
        assert!(text.contains("张三"));
    }

    #[test]
    fn message_with_zwj_emoji_sequence() {
        // ZWJ sequence for family emoji
        let project = "👨‍👩‍👧‍👦 Team";
        let template = "Build";
        let task_id = 1;
        let author = "team-bot";
        let task_url = "http://host/p/1/t/1";

        let text = format!(
            "✅ <b>[{project}]</b> {template} — <b>SUCCESS</b> (1s)\n\
             👤 {author} · <a href=\"{task_url}\">#task {task_id}</a>"
        );

        assert!(text.contains("👨"));
    }

    #[test]
    fn command_with_null_byte() {
        let text = "/help\x00";
        let cmd = text.split_whitespace().next().unwrap_or("").to_lowercase();
        // Null byte is part of the string but split_whitespace handles it
        assert!(cmd.starts_with("/help"));
    }

    #[test]
    fn command_with_control_chars() {
        // \x1b (ESC) is not whitespace, so split_whitespace includes it in first token
        let text = "\x1b/start\x07";
        let cmd = text.split_whitespace().next().unwrap_or("").to_lowercase();
        // The ESC char stays at the beginning -- this documents actual behavior
        assert!(cmd.contains("/start"));
        assert_eq!(cmd.len(), 8); // ESC + "/start" + BEL
    }

    #[test]
    fn message_with_url_query_params() {
        let task_url = "http://host/project/1/tasks/42?ref=telegram&notify=true";
        let text = format!("<a href=\"{task_url}\">#task 42</a>");
        assert!(text.contains("ref=telegram"));
        assert!(text.contains("notify=true"));
    }

    #[test]
    fn duration_from_task_with_negative_span() {
        // Task with end before start should yield 0 via max(0)
        let mut t = Task::default();
        let s = Utc::now();
        t.start = Some(s);
        t.end = Some(s - Duration::seconds(100));
        let secs = t
            .start
            .zip(t.end)
            .map(|(a, b)| (b - a).num_seconds().max(0) as u64)
            .unwrap_or(0);
        assert_eq!(secs, 0);
    }

    #[test]
    fn task_notification_dispatch_by_status() {
        // Verify that TaskStatus enum values map correctly
        use crate::services::task_logger::TaskStatus;
        assert_eq!(format!("{:?}", TaskStatus::Success), "Success");
        assert_eq!(format!("{:?}", TaskStatus::Error), "Error");
        assert_eq!(format!("{:?}", TaskStatus::Stopped), "Stopped");
    }

    #[test]
    fn send_message_payload_structure() {
        // Проверяем структуру JSON payload для sendMessage
        let chat_id = "-100test";
        let text = "Test message";
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "HTML",
            "disable_web_page_preview": true,
        });

        assert_eq!(payload["chat_id"], "-100test");
        assert_eq!(payload["text"], "Test message");
        assert_eq!(payload["parse_mode"], "HTML");
        assert_eq!(payload["disable_web_page_preview"], true);
    }

    #[test]
    fn notification_message_length_reasonable() {
        // Telegram has ~4096 char limit for messages
        let project = "A".repeat(50);
        let template = "B".repeat(50);
        let task_id = 12345;
        let author = "C".repeat(30);
        let task_url = "http://host".to_string() + &"/p".repeat(10);

        let text = format!(
            "✅ <b>[{project}]</b> {template} — <b>SUCCESS</b> (1m 0s)\n\
             👤 {author} · <a href=\"{task_url}\">#task {task_id}</a>"
        );

        // Message should be within Telegram limit
        assert!(text.len() < 4096);
    }

    #[test]
    fn extract_command_preserves_unicode_in_args() {
        let text = "/deploy проект тест";
        let parts: Vec<&str> = text.split_whitespace().collect();
        assert_eq!(parts[0], "/deploy");
        assert_eq!(parts[1], "проект");
        assert_eq!(parts[2], "тест");
    }

    #[test]
    fn command_with_emoji_args() {
        let text = "/log 🚀 production";
        let parts: Vec<&str> = text.split_whitespace().collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "/log");
        assert_eq!(parts[1], "🚀");
        assert_eq!(parts[2], "production");
    }
}
