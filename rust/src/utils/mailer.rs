//! Модуль отправки email через SMTP
//!
//! Предоставляет функциональность для отправки email уведомлений
//! через SMTP с поддержкой TLS/SSL и различных методов аутентификации.

use chrono::Utc;
use lettre::{
    message::{header::ContentType, Mailbox, Message},
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
    Address, AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use std::error::Error;
use thiserror::Error;

/// Типы ошибок mailer
#[derive(Debug, Error)]
pub enum MailerError {
    #[error("Ошибка создания сообщения: {0}")]
    MessageCreation(String),

    #[error("Ошибка отправки email: {0}")]
    SendError(String),

    #[error("Неподдерживаемая версия TLS: {0}")]
    UnsupportedTlsVersion(String),

    #[error("Некорректный email адрес: {0}")]
    InvalidEmail(String),

    #[error("Ошибка подключения: {0}")]
    ConnectionError(String),
}

impl From<lettre::error::Error> for MailerError {
    fn from(err: lettre::error::Error) -> Self {
        MailerError::SendError(err.to_string())
    }
}

impl From<lettre::address::AddressError> for MailerError {
    fn from(err: lettre::address::AddressError) -> Self {
        MailerError::InvalidEmail(err.to_string())
    }
}

impl From<lettre::transport::smtp::Error> for MailerError {
    fn from(err: lettre::transport::smtp::Error) -> Self {
        MailerError::SendError(err.to_string())
    }
}

/// Конфигурация SMTP сервера
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    /// Хост SMTP сервера
    pub host: String,
    /// Порт SMTP сервера
    pub port: String,
    /// Имя пользователя
    pub username: Option<String>,
    /// Пароль
    pub password: Option<String>,
    /// Использовать TLS
    pub use_tls: bool,
    /// Использовать безопасное соединение (TLS с начала)
    pub secure: bool,
    /// От кого
    pub from: String,
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            host: String::from("localhost"),
            port: String::from("25"),
            username: None,
            password: None,
            use_tls: false,
            secure: false,
            from: String::from("noreply@localhost"),
        }
    }
}

/// Email сообщение
#[derive(Debug, Clone)]
pub struct Email {
    /// От кого
    pub from: String,
    /// Кому
    pub to: String,
    /// Тема
    pub subject: String,
    /// Тело сообщения (HTML)
    pub body: String,
}

impl Email {
    /// Создаёт новое email сообщение
    pub fn new(from: String, to: String, subject: String, body: String) -> Self {
        Self {
            from,
            to,
            subject,
            body,
        }
    }

    /// Создаёт сообщение с защитой от header injection
    fn create_message(&self) -> Result<Message, MailerError> {
        // Очищаем заголовки от опасных символов (header injection protection)
        let from_clean = sanitize_header(&self.from);
        let to_clean = sanitize_header(&self.to);
        let subject_clean = sanitize_header(&self.subject);

        // Парсим адреса (Address::new требует имя и домен)
        let from_addr = Mailbox::new(
            None,
            from_clean.parse().map_err(|e: lettre::address::AddressError| {
                MailerError::InvalidEmail(e.to_string())
            })?,
        );
        let to_addr = Mailbox::new(
            None,
            to_clean.parse().map_err(|e: lettre::address::AddressError| {
                MailerError::InvalidEmail(e.to_string())
            })?,
        );

        // Создаём сообщение
        Message::builder()
            .from(from_addr)
            .to(to_addr)
            .subject(subject_clean)
            .header(ContentType::TEXT_HTML)
            .body(self.body.clone())
            .map_err(|e| MailerError::MessageCreation(e.to_string()))
    }
}

/// Очищает строку от опасных символов для предотвращения header injection
fn sanitize_header(s: &str) -> String {
    s.chars()
        .filter(|c| !matches!(*c, '\r' | '\n' | '%'))
        .collect()
}

/// Отправляет email через SMTP
pub async fn send_email(config: &SmtpConfig, email: &Email) -> Result<(), MailerError> {
    let message = email.create_message()?;

    if config.secure {
        if config.use_tls {
            send_with_tls(config, message).await
        } else {
            send_with_plain_auth(config, message).await
        }
    } else {
        send_anonymous(config, message).await
    }
}

/// Отправка с TLS
async fn send_with_tls(
    config: &SmtpConfig,
    message: Message,
) -> Result<(), MailerError> {
    // Создаём TLS параметры
    let tls_params = TlsParameters::new(config.host.clone())
        .map_err(|e| MailerError::ConnectionError(e.to_string()))?;

    // Создаём транспорт с TLS
    let mut builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
        .port(config.port.parse().unwrap_or(465))
        .tls(Tls::Required(tls_params));

    // Добавляем аутентификацию если есть credentials
    if let (Some(username), Some(password)) = (&config.username, &config.password) {
        let creds = Credentials::new(username.clone(), password.clone());
        builder = builder.credentials(creds);
    }

    let mailer = builder.build();

    // Отправляем
    mailer.send(message).await?;

    Ok(())
}

/// Отправка с PLAIN/LOGIN аутентификацией без TLS
async fn send_with_plain_auth(
    config: &SmtpConfig,
    message: Message,
) -> Result<(), MailerError> {
    let mut builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
        .port(config.port.parse().unwrap_or(587));

    // Добавляем аутентификацию
    if let (Some(username), Some(password)) = (&config.username, &config.password) {
        let creds = Credentials::new(username.clone(), password.clone());
        builder = builder.credentials(creds);
    }

    let mailer = builder.build();

    // Отправляем
    mailer.send(message).await?;

    Ok(())
}

/// Анонимная отправка (без аутентификации)
async fn send_anonymous(
    config: &SmtpConfig,
    message: Message,
) -> Result<(), MailerError> {
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
        .port(config.port.parse().unwrap_or(25))
        .build();

    // Отправляем
    mailer.send(message).await?;

    Ok(())
}

/// Утилита для проверки email на валидность
pub fn is_valid_email(email: &str) -> bool {
    email.parse::<Address>().is_ok()
}

// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_header() {
        assert_eq!(sanitize_header("test\r\ninjection"), "testinjection");
        assert_eq!(sanitize_header("test\ninjection"), "testinjection");
        assert_eq!(sanitize_header("test%injection"), "testinjection");
        assert_eq!(sanitize_header("normal@example.com"), "normal@example.com");
    }

    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name@domain.org"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("test@"));
    }

    #[test]
    fn test_email_creation() {
        let email = Email::new(
            "from@example.com".to_string(),
            "to@example.com".to_string(),
            "Test Subject".to_string(),
            "<p>Test Body</p>".to_string(),
        );

        assert_eq!(email.from, "from@example.com");
        assert_eq!(email.to, "to@example.com");
        assert_eq!(email.subject, "Test Subject");
        assert_eq!(email.body, "<p>Test Body</p>");
    }

    #[test]
    fn test_email_creation_with_injection_protection() {
        // Проверяем что sanitize_header удаляет опасные символы
        assert_eq!(sanitize_header("test\r\ninjection"), "testinjection");
        assert_eq!(sanitize_header("Subject\r\nInjection"), "SubjectInjection");
        
        let email = Email::new(
            "from\r\n@example.com".to_string(),
            "to@example.com".to_string(),
            "Subject\r\nInjection".to_string(),
            "Body".to_string(),
        );

        // Проверяем, что сообщение создаётся без ошибок
        let result = email.create_message();
        assert!(result.is_ok());
    }

    #[test]
    fn test_smtp_config_default() {
        let config = SmtpConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, "25");
        assert!(!config.use_tls);
        assert!(!config.secure);
    }
}
