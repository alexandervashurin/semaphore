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
        NOTIFICATION_BOT
            .get()
            .and_then(|opt| opt.as_ref().cloned())
    }

    /// Создаёт бота если задан токен в конфиге / env.
    pub fn new(config: &Config) -> Option<Arc<Self>> {
        let token = config.telegram_bot_token.clone()
            .or_else(|| std::env::var("SEMAPHORE_TELEGRAM_TOKEN").ok())?;

        if token.is_empty() {
            return None;
        }

        let default_chat_id = std::env::var("SEMAPHORE_TELEGRAM_CHAT_ID").ok()
            .filter(|s| !s.is_empty());

        info!("Telegram bot configured (token: {}...)", &token[..token.len().min(10)]);

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

        let resp = self.client
            .post(self.api_url("sendMessage"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| crate::error::Error::Other(format!("Telegram HTTP: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(crate::error::Error::Other(format!("Telegram API error: {body}")));
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

        let resp = self.client
            .post(self.api_url("getUpdates"))
            .json(&payload)
            .timeout(std::time::Duration::from_secs(35))
            .send()
            .await
            .map_err(|e| crate::error::Error::Other(format!("Telegram getUpdates: {e}")))?;

        let body: serde_json::Value = resp.json().await
            .map_err(|e| crate::error::Error::Other(format!("Telegram parse: {e}")))?;

        let updates = body["result"]
            .as_array()
            .cloned()
            .unwrap_or_default();

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
                self.send_message(chat_id,
                    "<b>Команды:</b>\n\
                     /start — приветствие\n\
                     /help  — эта справка\n\
                     /status — статус сервера\n\n\
                     <i>Уведомления о задачах отправляются автоматически.</i>"
                ).await?;
            }
            "/status" => {
                let ver = env!("CARGO_PKG_VERSION");
                self.send_message(chat_id,
                    &format!("🟢 Velum v{ver} работает")
                ).await?;
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
}
