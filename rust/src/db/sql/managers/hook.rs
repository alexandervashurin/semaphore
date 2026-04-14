//! HookManager - управление хуками

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::Hook;
use async_trait::async_trait;

#[async_trait]
impl HookManager for SqlStore {
    async fn get_hooks_by_template(&self, template_id: i32) -> Result<Vec<Hook>> {
        // Заглушка - возвращаем пустой список
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::models::hook::{Hook, HookType};

    #[test]
    fn test_hook_type_serialization() {
        assert_eq!(serde_json::to_string(&HookType::Http).unwrap(), "\"http\"");
        assert_eq!(serde_json::to_string(&HookType::Bash).unwrap(), "\"bash\"");
        assert_eq!(
            serde_json::to_string(&HookType::Python).unwrap(),
            "\"python\""
        );
    }

    #[test]
    fn test_hook_new_http() {
        let hook = Hook::new(10, 5, "Notify".to_string(), HookType::Http);
        assert_eq!(hook.id, 0);
        assert_eq!(hook.project_id, 10);
        assert_eq!(hook.template_id, 5);
        assert_eq!(hook.name, "Notify");
        assert_eq!(hook.r#type, HookType::Http);
    }

    #[test]
    fn test_hook_new_bash() {
        let hook = Hook::new(1, 1, "Cleanup".to_string(), HookType::Bash);
        assert_eq!(hook.r#type, HookType::Bash);
        assert!(hook.script.is_none());
    }

    #[test]
    fn test_hook_serialization() {
        let hook = Hook {
            id: 1,
            project_id: 10,
            template_id: 5,
            name: "Webhook".to_string(),
            r#type: HookType::Http,
            url: Some("https://hooks.slack.com/xxx".to_string()),
            script: None,
            http_method: Some("POST".to_string()),
            http_body: Some(r#"{"text":"done"}"#.to_string()),
            timeout_secs: Some(30),
        };
        let json = serde_json::to_string(&hook).unwrap();
        assert!(json.contains("\"name\":\"Webhook\""));
        assert!(json.contains("\"type\":\"http\""));
    }

    #[test]
    fn test_hook_clone() {
        let hook = Hook::new(1, 1, "Clone".to_string(), HookType::Python);
        let cloned = hook.clone();
        assert_eq!(cloned.name, hook.name);
        assert_eq!(cloned.r#type, hook.r#type);
    }

    #[test]
    fn test_hook_type_equality() {
        assert_eq!(HookType::Http, HookType::Http);
        assert_ne!(HookType::Http, HookType::Bash);
        assert_ne!(HookType::Bash, HookType::Python);
    }

    #[test]
    fn test_hook_deserialization() {
        let json = r#"{"id":5,"project_id":20,"template_id":10,"name":"Test Hook","type":"http","url":"https://test.com","script":null,"http_method":"POST","http_body":"{}","timeout_secs":5}"#;
        let hook: Hook = serde_json::from_str(json).unwrap();
        assert_eq!(hook.id, 5);
        assert_eq!(hook.r#type, HookType::Http);
    }

    #[test]
    fn test_hook_new_python() {
        let hook = Hook::new(1, 1, "Process Data".to_string(), HookType::Python);
        assert_eq!(hook.r#type, HookType::Python);
        assert_eq!(hook.id, 0);
    }
}
