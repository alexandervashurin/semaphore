//! Alert - система уведомлений
//!
//! Аналог services/tasks/alert.go из Go версии

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};

use crate::error::{Error, Result};
use crate::models::{Task, User};
use crate::services::task_logger::TaskLogger;
use crate::services::task_logger::TaskStatus;

/// Alert представляет уведомление
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub name: String,
    pub author: String,
    pub color: String,
    pub task: AlertTask,
    pub chat: AlertChat,
}

/// AlertTask - информация о задаче в уведомлении
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertTask {
    pub id: String,
    pub url: String,
    pub result: String,
    pub desc: String,
    pub version: String,
}

/// AlertChat - информация о чате
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertChat {
    pub id: String,
}

/// AlertService - сервис для отправки уведомлений
pub struct AlertService {
    client: Client,
    task: Task,
    template_name: String,
    username: String,
}

impl AlertService {
    /// Создаёт новый AlertService
    pub fn new(task: Task, template_name: String, username: String) -> Self {
        Self {
            client: Client::new(),
            task,
            template_name,
            username,
        }
    }

    /// Получает информацию для alert'а
    fn alert_infos(&self) -> (String, String) {
        let author = self.username.clone();
        let version = self.task.version.clone().unwrap_or_default();
        (author, version)
    }

    /// Получает цвет alert'а
    fn alert_color(&self, kind: &str) -> String {
        match self.task.status {
            TaskStatus::Success => match kind {
                "telegram" => "✅".to_string(),
                "slack" => "good".to_string(),
                "teams" => "8BC34A".to_string(),
                _ => "green".to_string(),
            },
            TaskStatus::Error => match kind {
                "telegram" => "❌".to_string(),
                "slack" => "danger".to_string(),
                "teams" => "F44336".to_string(),
                _ => "red".to_string(),
            },
            TaskStatus::Stopped => match kind {
                "telegram" => "⏹️".to_string(),
                "slack" => "warning".to_string(),
                "teams" => "FFC107".to_string(),
                _ => "yellow".to_string(),
            },
            _ => "gray".to_string(),
        }
    }

    /// Получает ссылку на задачу
    fn task_link(&self) -> String {
        format!(
            "{}/project/{}/tasks/{}",
            crate::config::get_public_host(),
            self.task.project_id,
            self.task.id
        )
    }

    /// Создаёт Alert объект
    fn create_alert(&self) -> Alert {
        let (author, version) = self.alert_infos();

        Alert {
            name: self.template_name.clone(),
            author,
            color: self.alert_color("generic"),
            task: AlertTask {
                id: self.task.id.to_string(),
                url: self.task_link(),
                result: self.task.status.to_string(),
                desc: self.task.message.clone().unwrap_or_default(),
                version,
            },
            chat: AlertChat { id: String::new() },
        }
    }

    /// Отправляет email уведомление
    pub async fn send_email_alert(&self, users: Vec<User>) -> Result<()> {
        use crate::utils::mailer::{send_email, Email};

        if !crate::config::email_alert_enabled() {
            return Ok(());
        }

        let alert = self.create_alert();

        // Формируем тело письма
        let body = format!(
            "Alert: {}\nAuthor: {}\nResult: {}\nVersion: {}\nDescription: {}\nURL: {}",
            alert.name,
            alert.author,
            alert.task.result,
            alert.task.version,
            alert.task.desc,
            alert.task.url
        );

        for user in users {
            if !user.alert {
                continue;
            }

            let user_email = user.email.clone();
            info!("Attempting to send email alert to {}", user_email);

            let config = crate::config::get_smtp_config();
            let email = Email::new(
                crate::config::get_email_sender(),
                user.email,
                format!("Alert: {}", alert.name),
                body.clone(),
            );

            if let Err(e) = send_email(&config, &email).await {
                error!("Failed to send email to {}: {}", user_email, e);
            }
        }

        Ok(())
    }

    /// Отправляет Telegram уведомление
    pub async fn send_telegram_alert(&self, chat_id: &str, token: &str) -> Result<()> {
        let alert = self.create_alert();

        let text = format!(
            "{} *Alert: {}*\n*Author:* {}\n*Result:* {}\n*Version:* {}\n*Description:* {}\n[View Task]({})",
            alert.color,
            alert.name,
            alert.author,
            alert.task.result,
            alert.task.version,
            alert.task.desc,
            alert.task.url
        );

        let url = format!("https://api.telegram.org/bot{}/sendMessage", token);

        let mut params = HashMap::new();
        params.insert("chat_id", chat_id);
        params.insert("text", &text);
        params.insert("parse_mode", "Markdown");

        let response = self.client.post(&url).json(&params).send().await?;

        if !response.status().is_success() {
            return Err(Error::Other(format!(
                "Telegram API error: {}",
                response.text().await?
            )));
        }

        info!("Telegram alert sent to {}", chat_id);
        Ok(())
    }

    /// Отправляет Slack уведомление
    pub async fn send_slack_alert(&self, webhook_url: &str) -> Result<()> {
        let alert = self.create_alert();

        let payload = serde_json::json!({
            "attachments": [
                {
                    "color": alert.color,
                    "title": alert.name,
                    "text": format!("Author: {}\nResult: {}\nVersion: {}\nDescription: {}",
                        alert.author, alert.task.result, alert.task.version, alert.task.desc),
                    "fields": [
                        {
                            "title": "Task",
                            "value": format!("<{}|View Task>", alert.task.url),
                            "short": false
                        }
                    ]
                }
            ]
        });

        let response = self.client.post(webhook_url).json(&payload).send().await?;

        if !response.status().is_success() {
            return Err(Error::Other(format!(
                "Slack webhook error: {}",
                response.text().await?
            )));
        }

        info!("Slack alert sent");
        Ok(())
    }

    /// Отправляет Rocket.Chat уведомление
    pub async fn send_rocket_chat_alert(&self, webhook_url: &str) -> Result<()> {
        let alert = self.create_alert();

        let payload = serde_json::json!({
            "attachments": [
                {
                    "color": alert.color,
                    "title": alert.name,
                    "text": format!("Author: {}\nResult: {}\nVersion: {}",
                        alert.author, alert.task.result, alert.task.version),
                    "fields": [
                        {
                            "title": "Description",
                            "value": alert.task.desc,
                            "short": false
                        },
                        {
                            "title": "Task",
                            "value": format!("[View Task]({})", alert.task.url),
                            "short": false
                        }
                    ]
                }
            ]
        });

        let response = self.client.post(webhook_url).json(&payload).send().await?;

        if !response.status().is_success() {
            return Err(Error::Other(format!(
                "Rocket.Chat webhook error: {}",
                response.text().await?
            )));
        }

        info!("Rocket.Chat alert sent");
        Ok(())
    }

    /// Отправляет Microsoft Teams уведомление
    pub async fn send_teams_alert(&self, webhook_url: &str) -> Result<()> {
        let alert = self.create_alert();

        let payload = serde_json::json!({
            "@type": "MessageCard",
            "@context": "http://schema.org/extensions",
            "themeColor": alert.color,
            "summary": alert.name,
            "sections": [
                {
                    "activityTitle": alert.name,
                    "facts": [
                        {"name": "Author", "value": alert.author},
                        {"name": "Result", "value": alert.task.result},
                        {"name": "Version", "value": alert.task.version},
                        {"name": "Description", "value": alert.task.desc}
                    ],
                    "potentialAction": [
                        {
                            "@type": "OpenUri",
                            "name": "View Task",
                            "targets": [{"os": "default", "uri": alert.task.url}]
                        }
                    ]
                }
            ]
        });

        let response = self.client.post(webhook_url).json(&payload).send().await?;

        if !response.status().is_success() {
            return Err(Error::Other(format!(
                "Teams webhook error: {}",
                response.text().await?
            )));
        }

        info!("Teams alert sent");
        Ok(())
    }

    /// Отправляет DingTalk уведомление
    pub async fn send_dingtalk_alert(&self, webhook_url: &str, secret: Option<&str>) -> Result<()> {
        let alert = self.create_alert();

        let mut payload = serde_json::json!({
            "msgtype": "markdown",
            "markdown": {
                "title": alert.name,
                "text": format!(
                    "## {} Alert: {}\n**Author:** {}\n**Result:** {}\n**Version:** {}\n**Description:** {}\n[View Task]({})",
                    alert.color,
                    alert.name,
                    alert.author,
                    alert.task.result,
                    alert.task.version,
                    alert.task.desc,
                    alert.task.url
                )
            }
        });

        // DingTalk requires timestamp-based HMAC-SHA256: sign = base64(HMAC(timestamp+"\n"+secret, secret))
        let url = if let Some(secret_key) = secret {
            use base64::Engine as _;
            use hmac::{Hmac, Mac};
            use sha2::Sha256;
            type HmacSha256 = Hmac<Sha256>;

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                .to_string();
            let string_to_sign = format!("{}\n{}", timestamp, secret_key);
            let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
                .expect("HMAC accepts any key length");
            mac.update(string_to_sign.as_bytes());
            let sign_bytes = mac.finalize().into_bytes();
            let sign = base64::engine::general_purpose::STANDARD.encode(sign_bytes);
            // Percent-encode base64 chars that are unsafe in URL query params
            let encoded_sign = sign.replace('+', "%2B").replace('/', "%2F").replace('=', "%3D");
            format!("{webhook_url}&timestamp={timestamp}&sign={encoded_sign}")
        } else {
            webhook_url.to_string()
        };

        let response = self.client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            return Err(Error::Other(format!(
                "DingTalk webhook error: {}",
                response.text().await?
            )));
        }

        info!("DingTalk alert sent");
        Ok(())
    }

    /// Вычисляет HMAC-SHA256 подпись тела запроса
    ///
    /// Возвращает `sha256=<hex>` как используется в GitHub-стиле webhook подписях.
    pub fn compute_hmac_signature(secret: &str, body: &[u8]) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC accepts any key length");
        mac.update(body);
        let result = mac.finalize().into_bytes();
        format!("sha256={}", hex::encode(result))
    }

    /// Отправляет generic webhook с HMAC-SHA256 подписью (если задан секрет)
    ///
    /// Заголовки при наличии `webhook_secret`:
    /// - `X-Velum-Signature: sha256=<HMAC-SHA256(secret, body)>`
    /// - `X-Velum-Timestamp: <unix_timestamp>`
    pub async fn send_generic_webhook(
        &self,
        webhook_url: &str,
        webhook_secret: Option<&str>,
    ) -> Result<()> {
        let alert = self.create_alert();

        let payload = serde_json::json!({
            "event": "task_result",
            "task": {
                "id": alert.task.id,
                "result": alert.task.result,
                "version": alert.task.version,
                "description": alert.task.desc,
                "url": alert.task.url,
            },
            "template": {
                "name": alert.name,
            },
            "author": alert.author,
        });

        let body = serde_json::to_vec(&payload)
            .map_err(|e| Error::Other(format!("JSON serialization error: {e}")))?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();

        let mut req = self.client
            .post(webhook_url)
            .header("Content-Type", "application/json")
            .header("X-Velum-Timestamp", &timestamp);

        if let Some(secret) = webhook_secret {
            let sig = Self::compute_hmac_signature(secret, &body);
            req = req.header("X-Velum-Signature", sig);
        }

        let response = req.body(body).send().await?;

        if !response.status().is_success() {
            return Err(Error::Other(format!(
                "Generic webhook error {}: {}",
                response.status(),
                response.text().await?
            )));
        }

        info!("Generic webhook sent to {}", webhook_url);
        Ok(())
    }

    /// Отправляет Gotify уведомление
    pub async fn send_gotify_alert(&self, server_url: &str, app_token: &str) -> Result<()> {
        let alert = self.create_alert();

        let payload = serde_json::json!({
            "title": format!("Alert: {}", alert.name),
            "message": format!(
                "Author: {}\nResult: {}\nVersion: {}\nDescription: {}\nURL: {}",
                alert.author, alert.task.result, alert.task.version, alert.task.desc, alert.task.url
            ),
            "priority": 5,
        });

        let url = format!("{}/message?token={}", server_url, app_token);

        let response = self.client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            return Err(Error::Other(format!(
                "Gotify API error: {}",
                response.text().await?
            )));
        }

        info!("Gotify alert sent");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_task() -> Task {
        let mut task = Task::default();
        task.id = 1;
        task.created = Utc::now();
        task.project_id = 1;
        task.template_id = 1;
        task.status = TaskStatus::Success;
        task.message = Some("Test message".to_string());
        task.version = Some("1.0.0".to_string());
        task.end = None;
        task
    }

    #[test]
    fn test_alert_service_creation() {
        let task = create_test_task();
        let service = AlertService::new(task, "Test Template".to_string(), "testuser".to_string());
        assert_eq!(service.template_name, "Test Template");
    }

    #[test]
    fn test_alert_color() {
        let task = create_test_task();
        let service = AlertService::new(task, "Test".to_string(), "testuser".to_string());

        assert_eq!(service.alert_color("telegram"), "✅");
        assert_eq!(service.alert_color("slack"), "good");
    }

    #[test]
    fn test_alert_infos() {
        let task = create_test_task();
        let service = AlertService::new(task, "Test".to_string(), "testuser".to_string());

        let (author, version) = service.alert_infos();
        assert_eq!(author, "testuser");
        assert_eq!(version, "1.0.0");
    }

    #[test]
    fn test_hmac_webhook() {
        let signature = AlertService::compute_hmac_signature("secret", br#"{"ping":"pong"}"#);
        assert_eq!(
            signature,
            "sha256=d4a0a190424950646d97770a70dbd0b331d19775a0024a3e762fd0cdf933c498"
        );
    }
}
