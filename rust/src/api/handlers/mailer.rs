//! API - Mailer Handler
//!
//! Обработчики для тестирования отправки email

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::api::state::AppState;
use crate::error::Error;
use crate::api::middleware::ErrorResponse;
use crate::utils::mailer::{send_email, Email, SmtpConfig};

/// Отправляет тестовое email уведомление
///
/// POST /api/admin/mail/test
pub async fn send_test_email(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TestEmailPayload>,
) -> std::result::Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Проверяем, что пользователь админ (требуется в middleware)
    // В реальной реализации нужно проверить права пользователя

    // Получаем конфигурацию SMTP из конфига
    let smtp_config = SmtpConfig {
        host: state.config.mailer_host.clone(),
        port: state.config.mailer_port.clone(),
        username: state.config.mailer_username.clone(),
        password: state.config.mailer_password.clone(),
        use_tls: state.config.mailer_use_tls,
        secure: state.config.mailer_secure,
        from: state.config.mailer_from.clone(),
    };

    // Формируем тестовое сообщение
    let subject = "🔔 Test Email from Velum UI";
    let body = format!(
        r#"
        <html>
            <body>
                <h1>Test Email from Velum UI</h1>
                <p>This is a test email to verify SMTP configuration.</p>
                <p><strong>Recipient:</strong> {}</p>
                <p><strong>Time:</strong> {}</p>
                <hr>
                <p style="color: gray; font-size: 12px;">Sent by Velum UI</p>
            </body>
        </html>
        "#,
        payload.to,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );

    let email = Email::new(
        smtp_config.from.clone(),
        payload.to.clone(),
        subject.to_string(),
        body,
    );

    // Отправляем email
    send_email(&smtp_config, &email)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to send email: {}", e)))
        ))?;

    tracing::info!("Test email sent to {}", payload.to);

    Ok(StatusCode::OK)
}

// ============================================================================
// Types
// ============================================================================

/// Payload для отправки тестового email
#[derive(Debug, Deserialize)]
pub struct TestEmailPayload {
    /// Email получателя
    pub to: String,
    /// Тема (опционально, используется по умолчанию)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
}

/// Response отправки email
#[derive(Debug, Serialize)]
pub struct TestEmailResponse {
    pub success: bool,
    pub message: String,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_payload_deserialize_minimal() {
        let json = r#"{"to": "test@example.com"}"#;
        let payload: TestEmailPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.to, "test@example.com");
        assert_eq!(payload.subject, None);
    }

    #[test]
    fn test_email_payload_deserialize_full() {
        let json = r#"{"to": "test@example.com", "subject": "Custom Subject"}"#;
        let payload: TestEmailPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.to, "test@example.com");
        assert_eq!(payload.subject, Some("Custom Subject".to_string()));
    }

    #[test]
    fn test_email_response_serialize() {
        let response = TestEmailResponse {
            success: true,
            message: "Email sent successfully".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("true"));
        assert!(json.contains("Email sent successfully"));
    }
}
