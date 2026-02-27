//! Alert - система уведомлений
//!
//! Аналог services/tasks/alert.go из Go версии

use std::collections::HashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

use crate::error::{Error, Result};
use crate::models::{Task, User};
use crate::services::task_logger::TaskStatus;
use crate::services::task_logger::TaskLogger;

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
            chat: AlertChat {
                id: String::new(),
            },
        }
    }

    /// Отправляет email уведомление
    pub async fn send_email_alert(
        &self,
        users: Vec<User>,
    ) -> Result<()> {
        use crate::utils::mailer::{send_email, Email, SmtpConfig};
        
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

            info!("Attempting to send email alert to {}", user.email);

            let config = SmtpConfig::default(); // TODO: Загрузить из конфига
            let email = Email::new(
                crate::config::get_email_sender(),
                user.email,
                format!("Alert: {}", alert.name),
                body.clone(),
            );

            if let Err(e) = send_email(&config, &email).await {
                error!("Failed to send email to {}: {}", user.email, e);
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

        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            token
        );

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

        if let Some(secret_key) = secret {
            // TODO: Добавить подпись для безопасности
            payload["sign"] = serde_json::json!(secret_key);
        }

        let response = self.client.post(webhook_url).json(&payload).send().await?;

        if !response.status().is_success() {
            return Err(Error::Other(format!(
                "DingTalk webhook error: {}",
                response.text().await?
            )));
        }

        info!("DingTalk alert sent");
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
        Task {
            id: 1,
            created: Utc::now(),
            project_id: 1,
            template_id: 1,
            status: TaskStatus::Success,
            message: Some("Test message".to_string()),
            commit_hash: None,
            commit_message: None,
            version: Some("1.0.0".to_string()),
            inventory_id: None,
            repository_id: None,
            environment_id: None,
            arguments: None,
            params: String::new(),
            end: None,
        }
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
}
