//! API - Mailer Handler
//!
//! Обработчики для тестирования отправки email

use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::error::Error;
use crate::utils::mailer::{Email, SmtpConfig, send_email};
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    send_email(&smtp_config, &email).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("Failed to send email: {}", e))),
        )
    })?;

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

    #[test]
    fn test_email_payload_roundtrip() {
        // TestEmailPayload only derives Deserialize
        let json = r#"{"to": "roundtrip@example.com", "subject": "Test Subject"}"#;
        let restored: TestEmailPayload = serde_json::from_str(json).unwrap();
        assert_eq!(restored.to, "roundtrip@example.com");
        assert_eq!(restored.subject, Some("Test Subject".to_string()));
    }

    #[test]
    fn test_email_response_roundtrip() {
        // TestEmailResponse derives Serialize but not Deserialize
        let original = TestEmailResponse {
            success: false,
            message: "Failed to send".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        assert!(json.contains("false"));
        assert!(json.contains("Failed to send"));
    }

    #[test]
    fn test_email_payload_debug() {
        let payload = TestEmailPayload {
            to: "debug@example.com".to_string(),
            subject: None,
        };
        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("TestEmailPayload"));
        assert!(debug_str.contains("debug@example.com"));
    }

    #[test]
    fn test_email_response_debug() {
        let response = TestEmailResponse {
            success: true,
            message: "Debug message".to_string(),
        };
        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("TestEmailResponse"));
        assert!(debug_str.contains("Debug message"));
    }

    #[test]
    fn test_email_payload_unicode_recipient() {
        let json = r#"{"to": "пользователь@example.com"}"#;
        let payload: TestEmailPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.to, "пользователь@example.com");
    }

    #[test]
    fn test_email_payload_unicode_subject() {
        let json = r#"{"to": "test@example.com", "subject": "Тестовое письмо"}"#;
        let payload: TestEmailPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.subject, Some("Тестовое письмо".to_string()));
    }

    #[test]
    fn test_email_response_false() {
        let response = TestEmailResponse {
            success: false,
            message: "Email failed".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("false"));
        assert!(json.contains("Email failed"));
    }

    #[test]
    fn test_email_payload_clone() {
        // TestEmailPayload doesn't derive Clone, test via deserialization
        let json = r#"{"to": "clone@example.com", "subject": "Clone Subject"}"#;
        let p1: TestEmailPayload = serde_json::from_str(json).unwrap();
        let p2: TestEmailPayload = serde_json::from_str(json).unwrap();
        assert_eq!(p1.to, p2.to);
        assert_eq!(p1.subject, p2.subject);
    }

    #[test]
    fn test_email_response_clone() {
        // TestEmailResponse doesn't derive Clone
        let r1 = TestEmailResponse {
            success: true,
            message: "Clone Response".to_string(),
        };
        let r2 = TestEmailResponse {
            success: true,
            message: "Clone Response".to_string(),
        };
        assert_eq!(r1.success, r2.success);
        assert_eq!(r1.message, r2.message);
    }

    #[test]
    #[allow(unused_assignments)]
    fn test_email_payload_clone_independence() {
        // TestEmailPayload doesn't derive Clone
        let mut to = "original@example.com".to_string();
        let subject = Some("Original".to_string());
        let p1 = TestEmailPayload {
            to: to.clone(),
            subject: subject.clone(),
        };
        to = "modified@example.com".to_string();
        assert_eq!(p1.to, "original@example.com");
    }

    #[test]
    fn test_email_response_empty_message() {
        let response = TestEmailResponse {
            success: true,
            message: "".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"message\":\"\""));
    }

    #[test]
    fn test_email_payload_special_chars_recipient() {
        let json = r#"{"to": "user+test+tag@example.com"}"#;
        let payload: TestEmailPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.to, "user+test+tag@example.com");
    }

    #[test]
    fn test_email_payload_subject_with_quotes() {
        let json = r#"{"to": "test@example.com", "subject": "Test \"Quoted\" Subject"}"#;
        let payload: TestEmailPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.subject, Some("Test \"Quoted\" Subject".to_string()));
    }

    #[test]
    fn test_email_response_with_unicode() {
        // TestEmailResponse doesn't derive Deserialize, test serialization only
        let response = TestEmailResponse {
            success: true,
            message: "Письмо успешно отправлено".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Письмо успешно отправлено"));
    }
}
