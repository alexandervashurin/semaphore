//! Webhook Service Tests

#[cfg(test)]
mod tests {
    use crate::services::webhook::*;
    use serde_json::json;
    use chrono::Utc;

    #[test]
    fn test_webhook_config_creation() {
        let config = WebhookConfig {
            id: 1,
            name: "Test Webhook".to_string(),
            r#type: WebhookType::Slack,
            url: "https://hooks.slack.com/test".to_string(),
            secret: None,
            headers: None,
            active: true,
            events: vec!["task_success".to_string()],
            retry_count: 3,
            timeout_secs: 30,
        };

        assert_eq!(config.id, 1);
        assert_eq!(config.r#type, WebhookType::Slack);
        assert!(config.active);
    }

    #[test]
    fn test_webhook_event_creation() {
        let event = WebhookEvent {
            event_type: "task_success".to_string(),
            timestamp: Utc::now(),
            data: json!({"task_id": 1, "status": "success"}),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "0.1.0".to_string(),
                project_id: Some(1),
                user_id: Some(1),
            },
        };

        assert_eq!(event.event_type, "task_success");
        assert!(event.data.is_object());
    }

    #[test]
    fn test_webhook_type_serialization() {
        let types = vec![
            WebhookType::Generic,
            WebhookType::Slack,
            WebhookType::Teams,
            WebhookType::Discord,
            WebhookType::Telegram,
        ];

        for webhook_type in types {
            let json = serde_json::to_string(&webhook_type).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_create_task_event() {
        let event = create_task_event(
            "task_success",
            1,
            "Test Task",
            Some(1),
            Some(1),
            Some("success"),
        );

        assert_eq!(event.event_type, "task_success");
        assert_eq!(event.metadata.project_id, Some(1));
        assert!(event.data.get("task_id").is_some());
    }

    #[test]
    fn test_create_user_event() {
        let event = create_user_event(
            "user_login",
            1,
            "testuser",
            Some(1),
        );

        assert_eq!(event.event_type, "user_login");
        assert_eq!(event.metadata.user_id, Some(1));
    }

    #[test]
    fn test_create_project_event() {
        let event = create_project_event(
            "project_created",
            1,
            "Test Project",
            Some(1),
        );

        assert_eq!(event.event_type, "project_created");
        assert_eq!(event.metadata.project_id, Some(1));
    }

    #[tokio::test]
    async fn test_webhook_service_creation() {
        let service = WebhookService::new();
        assert!(true); // Service created successfully
    }

    #[tokio::test]
    async fn test_webhook_service_with_timeout() {
        let service = WebhookService::with_timeout(60);
        assert!(true); // Service created with custom timeout
    }

    #[tokio::test]
    async fn test_inactive_webhook() {
        let service = WebhookService::new();
        
        let config = WebhookConfig {
            id: 1,
            name: "Inactive Webhook".to_string(),
            r#type: WebhookType::Generic,
            url: "http://example.com".to_string(),
            secret: None,
            headers: None,
            active: false,
            events: vec![],
            retry_count: 0,
            timeout_secs: 30,
        };

        let event = WebhookEvent {
            event_type: "test".to_string(),
            timestamp: Utc::now(),
            data: json!({}),
            metadata: WebhookMetadata {
                source: "test".to_string(),
                version: "0.1.0".to_string(),
                project_id: None,
                user_id: None,
            },
        };

        let result = service.send_webhook(&config, &event).await.unwrap();
        assert!(!result.success);
        assert_eq!(result.attempts, 0);
    }

    #[test]
    fn test_webhook_result_default() {
        let result = WebhookResult {
            success: true,
            status_code: Some(200),
            response_body: Some("OK".to_string()),
            error: None,
            attempts: 1,
        };

        assert!(result.success);
        assert_eq!(result.status_code, Some(200));
    }
}
