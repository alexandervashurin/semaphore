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

    /// Вычисляет HMAC-SHA256 только по телу (без timestamp). Для новых интеграций используйте
    /// [`Self::compute_webhook_request_signature`].
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

    /// HMAC-SHA256(secret, `{timestamp_unix}.` + body) — защита от replay: получатель сверяет время.
    /// Заголовки исходящего запроса: `X-Semaphore-Signature: sha256=<hex>`, `X-Semaphore-Timestamp`.
    pub fn compute_webhook_request_signature(
        secret: &str,
        timestamp_unix: &str,
        body: &[u8],
    ) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC accepts any key length");
        mac.update(timestamp_unix.as_bytes());
        mac.update(b".");
        mac.update(body);
        let result = mac.finalize().into_bytes();
        format!("sha256={}", hex::encode(result))
    }

    /// Отправляет generic webhook с HMAC-SHA256 подписью (если задан секрет)
    ///
    /// Заголовки при наличии `webhook_secret`:
    /// - `X-Semaphore-Signature: sha256=<HMAC(secret, "{ts}." + body)>`
    /// - `X-Semaphore-Timestamp: <unix_secs>`
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

        let mut req = self
            .client
            .post(webhook_url)
            .header("Content-Type", "application/json")
            .header("X-Semaphore-Timestamp", &timestamp);

        if let Some(secret) = webhook_secret {
            let sig = Self::compute_webhook_request_signature(secret, &timestamp, &body);
            req = req.header("X-Semaphore-Signature", sig);
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
    fn test_hmac_webhook_body_only_legacy() {
        let signature = AlertService::compute_hmac_signature("secret", br#"{"ping":"pong"}"#);
        assert_eq!(
            signature,
            "sha256=d4a0a190424950646d97770a70dbd0b331d19775a0024a3e762fd0cdf933c498"
        );
    }

    #[test]
    fn test_webhook_request_signature_includes_timestamp() {
        let secret = "mysecret";
        let ts = "1704067200";
        let body = br#"{"event":"task_result"}"#;
        let s1 = AlertService::compute_webhook_request_signature(secret, ts, body);
        let s2 = AlertService::compute_webhook_request_signature(secret, ts, body);
        assert_eq!(s1, s2);
        assert!(s1.starts_with("sha256="));
        let s3 = AlertService::compute_webhook_request_signature(secret, "1704067201", body);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_alert_color_error_status() {
        let mut task = create_test_task();
        task.status = TaskStatus::Error;
        let service = AlertService::new(task, "Test".to_string(), "user".to_string());

        assert_eq!(service.alert_color("telegram"), "❌");
        assert_eq!(service.alert_color("slack"), "danger");
        assert_eq!(service.alert_color("teams"), "F44336");
        assert_eq!(service.alert_color("generic"), "red");
    }

    #[test]
    fn test_alert_color_stopped_status() {
        let mut task = create_test_task();
        task.status = TaskStatus::Stopped;
        let service = AlertService::new(task, "Test".to_string(), "user".to_string());

        assert_eq!(service.alert_color("telegram"), "⏹️");
        assert_eq!(service.alert_color("slack"), "warning");
        assert_eq!(service.alert_color("teams"), "FFC107");
        assert_eq!(service.alert_color("generic"), "yellow");
    }

    #[test]
    fn test_alert_color_default_for_unknown_status() {
        let mut task = create_test_task();
        task.status = TaskStatus::Waiting;
        let service = AlertService::new(task, "Test".to_string(), "user".to_string());

        assert_eq!(service.alert_color("telegram"), "gray");
        assert_eq!(service.alert_color("slack"), "gray");
        assert_eq!(service.alert_color("generic"), "gray");
    }

    #[test]
    fn test_task_link_contains_project_and_task_id() {
        let task = create_test_task();
        let service = AlertService::new(task, "Test".to_string(), "user".to_string());

        let link = service.task_link();
        assert!(link.contains("/project/"));
        assert!(link.contains("/tasks/1"));
    }

    #[test]
    fn test_alert_color_for_running_status() {
        let mut task = create_test_task();
        task.status = TaskStatus::Running;
        let service = AlertService::new(task, "Test".to_string(), "user".to_string());

        assert_eq!(service.alert_color("telegram"), "gray");
    }

    #[test]
    fn test_alert_infos_with_no_version() {
        let mut task = create_test_task();
        task.version = None;
        let service = AlertService::new(task, "Test".to_string(), "testuser".to_string());

        let (author, version) = service.alert_infos();
        assert_eq!(author, "testuser");
        assert_eq!(version, "");
    }

    #[test]
    fn test_compute_hmac_signature_is_deterministic() {
        let body = b"test body";
        let sig1 = AlertService::compute_hmac_signature("secret", body);
        let sig2 = AlertService::compute_hmac_signature("secret", body);
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_compute_hmac_signature_different_secrets() {
        let body = b"test body";
        let sig1 = AlertService::compute_hmac_signature("secret1", body);
        let sig2 = AlertService::compute_hmac_signature("secret2", body);
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_compute_hmac_signature_different_bodies() {
        let sig1 = AlertService::compute_hmac_signature("secret", b"body1");
        let sig2 = AlertService::compute_hmac_signature("secret", b"body2");
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_compute_hmac_signature_same_input_same_output() {
        let sig1 = AlertService::compute_hmac_signature("secret", b"body");
        let sig2 = AlertService::compute_hmac_signature("secret", b"body");
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_alert_task_info_with_version() {
        let task = Task {
            id: 1,
            project_id: 1,
            template_id: 1,
            status: TaskStatus::Success,
            version: Some("v1.2.3".to_string()),
            ..Task::default()
        };
        let service = AlertService::new(task, "Test Template".to_string(), "testuser".to_string());
        let (author, version) = service.alert_infos();
        assert_eq!(author, "testuser");
        assert_eq!(version, "v1.2.3");
    }

    #[test]
    fn test_alert_task_link_format() {
        let task = Task {
            id: 42,
            project_id: 10,
            template_id: 1,
            status: TaskStatus::Success,
            ..Task::default()
        };
        let service = AlertService::new(task, "Template".to_string(), "user".to_string());
        let link = service.task_link();
        assert!(link.contains("/project/10/"));
        assert!(link.contains("/tasks/42"));
    }

    #[test]
    fn test_alert_service_new() {
        let task = Task::default();
        let service = AlertService::new(task.clone(), "Tpl".to_string(), "usr".to_string());
        // Just verify creation doesn't panic
        let _ = service;
    }

    #[test]
    fn test_alert_color_for_starting_status() {
        let mut task = create_test_task();
        task.status = TaskStatus::Starting;
        let service = AlertService::new(task, "Test".to_string(), "user".to_string());
        assert_eq!(service.alert_color("generic"), "gray");
    }

    #[test]
    fn test_alert_color_for_stopped_teams() {
        let mut task = create_test_task();
        task.status = TaskStatus::Stopped;
        let service = AlertService::new(task, "Test".to_string(), "user".to_string());
        assert_eq!(service.alert_color("teams"), "FFC107");
    }

    #[test]
    fn test_alert_color_for_error_slack() {
        let mut task = create_test_task();
        task.status = TaskStatus::Error;
        let service = AlertService::new(task, "Test".to_string(), "user".to_string());
        assert_eq!(service.alert_color("slack"), "danger");
    }

    #[test]
    fn test_alert_task_struct_serializable() {
        let task = create_test_task();
        let service = AlertService::new(task, "Tpl".to_string(), "user".to_string());
        let alert = service.create_alert();
        let json = serde_json::to_string(&alert).unwrap();
        assert!(json.contains("\"name\":\"Tpl\""));
        assert!(json.contains("\"author\":\"user\""));
    }

    #[test]
    fn test_alert_chat_struct_default() {
        let alert = Alert {
            name: "test".to_string(),
            author: "user".to_string(),
            color: "green".to_string(),
            task: AlertTask {
                id: "1".to_string(),
                url: "http://example.com".to_string(),
                result: "success".to_string(),
                desc: "ok".to_string(),
                version: "1.0".to_string(),
            },
            chat: AlertChat { id: String::new() },
        };
        assert!(alert.chat.id.is_empty());
    }

    #[test]
    fn test_alert_task_url_not_empty() {
        let task = create_test_task();
        let service = AlertService::new(task, "Tpl".to_string(), "user".to_string());
        let alert = service.create_alert();
        assert!(!alert.task.url.is_empty());
    }

    #[test]
    fn test_alert_task_result_matches_status() {
        let mut task = create_test_task();
        task.status = TaskStatus::Error;
        let service = AlertService::new(task, "Tpl".to_string(), "user".to_string());
        let alert = service.create_alert();
        assert_eq!(alert.task.result, "error");
    }

    #[test]
    fn test_alert_desc_uses_task_message() {
        let task = create_test_task();
        let service = AlertService::new(task, "Tpl".to_string(), "user".to_string());
        let alert = service.create_alert();
        assert_eq!(alert.task.desc, "Test message");
    }

    #[test]
    fn test_alert_desc_empty_when_no_message() {
        let mut task = create_test_task();
        task.message = None;
        let service = AlertService::new(task, "Tpl".to_string(), "user".to_string());
        let alert = service.create_alert();
        assert!(alert.task.desc.is_empty());
    }

    #[test]
    fn test_alert_color_success_teams() {
        let task = create_test_task();
        let service = AlertService::new(task, "Test".to_string(), "user".to_string());
        assert_eq!(service.alert_color("teams"), "8BC34A");
    }

    #[test]
    fn test_alert_color_stopped_slack() {
        let mut task = create_test_task();
        task.status = TaskStatus::Stopped;
        let service = AlertService::new(task, "Test".to_string(), "user".to_string());
        assert_eq!(service.alert_color("slack"), "warning");
    }

    #[test]
    fn test_alert_color_error_generic() {
        let mut task = create_test_task();
        task.status = TaskStatus::Error;
        let service = AlertService::new(task, "Test".to_string(), "user".to_string());
        assert_eq!(service.alert_color("generic"), "red");
    }

    #[test]
    fn test_alert_struct_clone() {
        let alert = Alert {
            name: "n".to_string(),
            author: "a".to_string(),
            color: "c".to_string(),
            task: AlertTask {
                id: "1".to_string(),
                url: "u".to_string(),
                result: "r".to_string(),
                desc: "d".to_string(),
                version: "v".to_string(),
            },
            chat: AlertChat { id: "ch".to_string() },
        };
        let cloned = alert.clone();
        assert_eq!(cloned.name, alert.name);
        assert_eq!(cloned.task.id, alert.task.id);
        assert_eq!(cloned.chat.id, alert.chat.id);
    }

    #[test]
    fn test_alert_task_version_empty_when_none() {
        let mut task = create_test_task();
        task.version = None;
        let service = AlertService::new(task, "Tpl".to_string(), "user".to_string());
        let (author, version) = service.alert_infos();
        assert_eq!(author, "user");
        assert_eq!(version, "");
    }

    #[test]
    fn test_alert_task_id_matches_task_id() {
        let mut task = create_test_task();
        task.id = 42;
        let service = AlertService::new(task, "Tpl".to_string(), "user".to_string());
        let alert = service.create_alert();
        assert_eq!(alert.task.id, "42");
    }

    #[test]
    fn test_hmac_signature_empty_body() {
        let sig = AlertService::compute_hmac_signature("secret", b"");
        assert!(sig.starts_with("sha256="));
        assert_eq!(sig.len(), 71); // "sha256=" + 64 hex chars
    }

    #[test]
    fn test_hmac_signature_empty_secret() {
        let sig = AlertService::compute_hmac_signature("", b"body");
        assert!(sig.starts_with("sha256="));
        assert_eq!(sig.len(), 71);
    }

    #[test]
    fn test_hmac_signature_empty_secret_and_body() {
        let sig = AlertService::compute_hmac_signature("", b"");
        assert!(sig.starts_with("sha256="));
        assert_eq!(sig.len(), 71);
    }

    #[test]
    fn test_webhook_signature_different_timestamps() {
        let body = b"test";
        let s1 = AlertService::compute_webhook_request_signature("sec", "1000", body);
        let s2 = AlertService::compute_webhook_request_signature("sec", "2000", body);
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_webhook_signature_different_secrets_same_timestamp() {
        let body = b"test";
        let s1 = AlertService::compute_webhook_request_signature("sec1", "1000", body);
        let s2 = AlertService::compute_webhook_request_signature("sec2", "1000", body);
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_webhook_signature_different_bodies_same_timestamp() {
        let s1 = AlertService::compute_webhook_request_signature("sec", "1000", b"body1");
        let s2 = AlertService::compute_webhook_request_signature("sec", "1000", b"body2");
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_alert_task_serialization_roundtrip() {
        let task = create_test_task();
        let service = AlertService::new(task, "Template".to_string(), "user".to_string());
        let alert = service.create_alert();

        let json = serde_json::to_string(&alert).unwrap();
        let deserialized: Alert = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, alert.name);
        assert_eq!(deserialized.author, alert.author);
        assert_eq!(deserialized.task.id, alert.task.id);
        assert_eq!(deserialized.task.url, alert.task.url);
    }

    #[test]
    fn test_alert_color_success_all_channels() {
        let task = create_test_task();
        let service = AlertService::new(task, "T".to_string(), "u".to_string());

        assert_eq!(service.alert_color("telegram"), "✅");
        assert_eq!(service.alert_color("slack"), "good");
        assert_eq!(service.alert_color("teams"), "8BC34A");
        assert_eq!(service.alert_color("anything"), "green");
    }

    #[test]
    fn test_alert_color_error_all_channels() {
        let mut task = create_test_task();
        task.status = TaskStatus::Error;
        let service = AlertService::new(task, "T".to_string(), "u".to_string());

        assert_eq!(service.alert_color("telegram"), "❌");
        assert_eq!(service.alert_color("slack"), "danger");
        assert_eq!(service.alert_color("teams"), "F44336");
        assert_eq!(service.alert_color("other"), "red");
    }

    #[test]
    fn test_alert_color_stopped_all_channels() {
        let mut task = create_test_task();
        task.status = TaskStatus::Stopped;
        let service = AlertService::new(task, "T".to_string(), "u".to_string());

        assert_eq!(service.alert_color("telegram"), "⏹️");
        assert_eq!(service.alert_color("slack"), "warning");
        assert_eq!(service.alert_color("teams"), "FFC107");
        assert_eq!(service.alert_color("fallback"), "yellow");
    }

    #[test]
    fn test_alert_service_with_empty_template_name() {
        let task = create_test_task();
        let service = AlertService::new(task, "".to_string(), "user".to_string());
        assert_eq!(service.template_name, "");
        let alert = service.create_alert();
        assert_eq!(alert.name, "");
    }

    #[test]
    fn test_alert_service_with_empty_username() {
        let task = create_test_task();
        let service = AlertService::new(task, "Tpl".to_string(), "".to_string());
        let (author, _) = service.alert_infos();
        assert_eq!(author, "");
    }

    #[test]
    fn test_alert_task_url_contains_host() {
        let task = create_test_task();
        let service = AlertService::new(task, "Tpl".to_string(), "user".to_string());
        let link = service.task_link();
        // Contains the project path
        assert!(link.contains("/project/1/tasks/1"));
    }

    #[test]
    fn test_alert_clone_preserves_all_fields() {
        let alert = Alert {
            name: "Name".to_string(),
            author: "Author".to_string(),
            color: "Color".to_string(),
            task: AlertTask {
                id: "100".to_string(),
                url: "http://test.com".to_string(),
                result: "ok".to_string(),
                desc: "description".to_string(),
                version: "2.0".to_string(),
            },
            chat: AlertChat { id: "chat123".to_string() },
        };
        let cloned = alert.clone();
        assert_eq!(cloned.name, "Name");
        assert_eq!(cloned.author, "Author");
        assert_eq!(cloned.color, "Color");
        assert_eq!(cloned.task.id, "100");
        assert_eq!(cloned.task.version, "2.0");
        assert_eq!(cloned.chat.id, "chat123");
    }

    #[test]
    fn test_hmac_webhook_body_json_payload() {
        let body = br#"{"event":"task_result","task":{"id":"1"}}"#;
        let sig = AlertService::compute_hmac_signature("mysecret", body);
        assert!(sig.starts_with("sha256="));
        // Verify it's deterministic
        let sig2 = AlertService::compute_hmac_signature("mysecret", body);
        assert_eq!(sig, sig2);
    }

    #[test]
    fn test_alert_task_result_is_stringified_status() {
        for status in [TaskStatus::Success, TaskStatus::Error, TaskStatus::Stopped, TaskStatus::Waiting, TaskStatus::Running, TaskStatus::Starting] {
            let mut task = create_test_task();
            task.status = status;
            let service = AlertService::new(task, "T".to_string(), "u".to_string());
            let alert = service.create_alert();
            assert!(!alert.task.result.is_empty(), "Result should not be empty for {:?}", status);
        }
    }

    #[test]
    fn test_dingtalk_sign_encoding_chars() {
        // Verify that base64 chars +, /, = are properly URL-encoded
        use base64::Engine as _;
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let secret = "test_secret_with_special_chars";
        let timestamp = "1704067200000";
        let string_to_sign = format!("{}\n{}", timestamp, secret);
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(string_to_sign.as_bytes());
        let sign_bytes = mac.finalize().into_bytes();
        let sign = base64::engine::general_purpose::STANDARD.encode(sign_bytes);

        // Check that encoding produces characters that need escaping
        let encoded = sign.replace('+', "%2B").replace('/', "%2F").replace('=', "%3D");
        // Should not contain unescaped unsafe chars
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));
        assert!(!encoded.contains('='));
    }
}
