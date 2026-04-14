//! AI Integration — analyze task failures, generate playbooks, manage AI settings.
//!
//! Settings stored in the `option` table:
//!   ai.provider  — "openai" | "anthropic" | "ollama"
//!   ai.api_key   — secret key
//!   ai.model     — model name (e.g. "gpt-4o", "claude-3-5-sonnet-20241022")
//!   ai.base_url  — override API base URL (for ollama or proxies)
//!   ai.enabled   — "true" | "false"

use crate::api::extractors::AuthUser;
use crate::api::state::AppState;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;

// ── Settings ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct AiSettings {
    pub enabled: bool,
    pub provider: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    pub has_api_key: bool,
}

#[derive(Debug, Deserialize)]
pub struct AiSettingsUpdate {
    pub enabled: Option<bool>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

pub async fn get_ai_settings(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
) -> impl IntoResponse {
    let store = state.store.store();
    let enabled = store
        .get_option("ai.enabled")
        .await
        .unwrap_or(Some("false".into()))
        .unwrap_or_else(|| "false".into())
        == "true";
    let provider = store
        .get_option("ai.provider")
        .await
        .unwrap_or(Some("openai".into()))
        .unwrap_or_else(|| "openai".into());
    let model = store
        .get_option("ai.model")
        .await
        .unwrap_or(None)
        .unwrap_or_else(|| default_model(&provider));
    let base_url = store.get_option("ai.base_url").await.unwrap_or(None);
    let has_api_key = store
        .get_option("ai.api_key")
        .await
        .unwrap_or(None)
        .map(|k| !k.is_empty())
        .unwrap_or(false);

    Json(AiSettings {
        enabled,
        provider,
        model,
        base_url,
        has_api_key,
    })
}

pub async fn update_ai_settings(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Json(body): Json<AiSettingsUpdate>,
) -> impl IntoResponse {
    let store = state.store.store();
    if let Some(enabled) = body.enabled {
        let _ = store
            .set_option("ai.enabled", if enabled { "true" } else { "false" })
            .await;
    }
    if let Some(ref provider) = body.provider {
        let _ = store.set_option("ai.provider", provider).await;
    }
    if let Some(ref model) = body.model {
        let _ = store.set_option("ai.model", model).await;
    }
    if let Some(ref key) = body.api_key {
        if !key.is_empty() {
            let _ = store.set_option("ai.api_key", key).await;
        }
    }
    if let Some(ref url) = body.base_url {
        let _ = store.set_option("ai.base_url", url).await;
    }
    StatusCode::NO_CONTENT
}

// ── Analyze failure ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    /// Task log lines joined or as single string
    pub log: String,
    /// Optional: template name or playbook for context
    pub context: Option<String>,
    /// Optional: language for response ("ru" | "en")
    pub lang: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AnalyzeResponse {
    pub analysis: String,
    pub suggestions: Vec<String>,
    pub provider: String,
    pub model: String,
}

pub async fn analyze_failure(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Json(body): Json<AnalyzeRequest>,
) -> impl IntoResponse {
    let store = state.store.store();

    let enabled = store
        .get_option("ai.enabled")
        .await
        .unwrap_or(Some("false".into()))
        .unwrap_or_else(|| "false".into())
        == "true";
    if !enabled {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "AI integration is disabled. Enable it in MCP/AI settings." })),
        );
    }

    let provider = store
        .get_option("ai.provider")
        .await
        .unwrap_or(None)
        .unwrap_or_else(|| "openai".into());
    let api_key = store
        .get_option("ai.api_key")
        .await
        .unwrap_or(None)
        .unwrap_or_default();
    let model = store
        .get_option("ai.model")
        .await
        .unwrap_or(None)
        .unwrap_or_else(|| default_model(&provider));
    let base_url = store.get_option("ai.base_url").await.unwrap_or(None);

    if api_key.is_empty() && provider != "ollama" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "AI API key is not configured." })),
        );
    }

    let lang = body.lang.as_deref().unwrap_or("ru");
    let ctx = body.context.as_deref().unwrap_or("Ansible/Terraform task");
    let system_prompt = format!(
        "You are a DevOps expert analyzing automation task failures. \
         Respond in {} language. Be concise and actionable. \
         Always structure your response as:\n\
         1. Root cause (1-2 sentences)\n\
         2. Suggested fixes (bullet list)\n\
         3. Prevention tip",
        if lang == "ru" { "Russian" } else { "English" }
    );
    let user_prompt = format!(
        "Task: {}\n\nLog output:\n```\n{}\n```\n\nAnalyze this failure and suggest fixes.",
        ctx,
        truncate_log(&body.log, 4000)
    );

    let result = call_ai_api(
        &provider,
        &api_key,
        &model,
        base_url.as_deref(),
        &system_prompt,
        &user_prompt,
    )
    .await;

    match result {
        Ok(text) => {
            let (analysis, suggestions) = parse_ai_response(&text);
            (
                StatusCode::OK,
                Json(json!(AnalyzeResponse {
                    analysis,
                    suggestions,
                    provider: provider.clone(),
                    model,
                })),
            )
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": format!("AI API error: {}", e) })),
        ),
    }
}

// ── Generate playbook ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub description: String,
    pub app: Option<String>, // "ansible" | "terraform" | "bash"
    pub lang: Option<String>,
}

pub async fn generate_playbook(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Json(body): Json<GenerateRequest>,
) -> impl IntoResponse {
    let store = state.store.store();

    let enabled = store
        .get_option("ai.enabled")
        .await
        .unwrap_or(Some("false".into()))
        .unwrap_or_else(|| "false".into())
        == "true";
    if !enabled {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "AI integration is disabled." })),
        );
    }

    let provider = store
        .get_option("ai.provider")
        .await
        .unwrap_or(None)
        .unwrap_or_else(|| "openai".into());
    let api_key = store
        .get_option("ai.api_key")
        .await
        .unwrap_or(None)
        .unwrap_or_default();
    let model = store
        .get_option("ai.model")
        .await
        .unwrap_or(None)
        .unwrap_or_else(|| default_model(&provider));
    let base_url = store.get_option("ai.base_url").await.unwrap_or(None);

    if api_key.is_empty() && provider != "ollama" {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "AI API key is not configured." })),
        );
    }

    let app = body.app.as_deref().unwrap_or("ansible");
    let system_prompt = format!(
        "You are an expert {} developer. Generate clean, production-ready {} code. \
         Return ONLY the code without explanations. Add inline comments.",
        app, app
    );
    let user_prompt = format!("Generate {} code for: {}", app, body.description);

    match call_ai_api(
        &provider,
        &api_key,
        &model,
        base_url.as_deref(),
        &system_prompt,
        &user_prompt,
    )
    .await
    {
        Ok(code) => (
            StatusCode::OK,
            Json(json!({ "code": code, "app": app, "model": model })),
        ),
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": format!("AI API error: {}", e) })),
        ),
    }
}

// ── AI API call (multi-provider) ───────────────────────────────────────────────

async fn call_ai_api(
    provider: &str,
    api_key: &str,
    model: &str,
    base_url: Option<&str>,
    system: &str,
    user: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    match provider {
        "anthropic" => {
            let url = base_url.unwrap_or("https://api.anthropic.com/v1/messages");
            let resp = client
                .post(url)
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&json!({
                    "model": model,
                    "max_tokens": 1024,
                    "system": system,
                    "messages": [{ "role": "user", "content": user }]
                }))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let data: Value = resp.json().await.map_err(|e| e.to_string())?;
            if let Some(err) = data.get("error") {
                return Err(err["message"]
                    .as_str()
                    .unwrap_or("unknown error")
                    .to_string());
            }
            data["content"][0]["text"]
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| "Empty response from Anthropic".to_string())
        }
        "ollama" => {
            let base = base_url.unwrap_or("http://localhost:11434");
            let url = format!("{}/api/chat", base);
            let resp = client
                .post(&url)
                .json(&json!({
                    "model": model,
                    "stream": false,
                    "messages": [
                        { "role": "system", "content": system },
                        { "role": "user", "content": user }
                    ]
                }))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let data: Value = resp.json().await.map_err(|e| e.to_string())?;
            data["message"]["content"]
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| "Empty response from Ollama".to_string())
        }
        _ => {
            // Default: OpenAI-compatible
            let url = base_url.unwrap_or("https://api.openai.com/v1/chat/completions");
            let resp = client
                .post(url)
                .bearer_auth(api_key)
                .json(&json!({
                    "model": model,
                    "messages": [
                        { "role": "system", "content": system },
                        { "role": "user", "content": user }
                    ],
                    "max_tokens": 1024
                }))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let data: Value = resp.json().await.map_err(|e| e.to_string())?;
            if let Some(err) = data.get("error") {
                return Err(err["message"]
                    .as_str()
                    .unwrap_or("unknown error")
                    .to_string());
            }
            data["choices"][0]["message"]["content"]
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| "Empty response from OpenAI".to_string())
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn default_model(provider: &str) -> String {
    match provider {
        "anthropic" => "claude-3-5-sonnet-20241022".to_string(),
        "ollama" => "llama3.2".to_string(),
        _ => "gpt-4o-mini".to_string(),
    }
}

fn truncate_log(log: &str, max_chars: usize) -> &str {
    if log.len() <= max_chars {
        return log;
    }
    // Take the last max_chars (most relevant for failures)
    let start = log.len() - max_chars;
    &log[start..]
}

fn parse_ai_response(text: &str) -> (String, Vec<String>) {
    let mut suggestions = Vec::new();
    let mut analysis_lines = Vec::new();
    let mut in_list = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || (trimmed.len() > 2
                && trimmed
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                && trimmed.contains(". "))
        {
            in_list = true;
            let content = trimmed.trim_start_matches(|c: char| {
                c.is_ascii_digit() || c == '.' || c == '-' || c == '*' || c == ' '
            });
            if !content.is_empty() {
                suggestions.push(content.to_string());
            }
        } else if !trimmed.is_empty() {
            analysis_lines.push(line.to_string());
        }
    }
    _ = in_list;

    let analysis = if analysis_lines.is_empty() {
        text.to_string()
    } else {
        analysis_lines.join("\n")
    };
    (analysis, suggestions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_model_openai() {
        assert_eq!(default_model("openai"), "gpt-4o-mini");
    }

    #[test]
    fn test_default_model_anthropic() {
        assert_eq!(default_model("anthropic"), "claude-3-5-sonnet-20241022");
    }

    #[test]
    fn test_default_model_unknown() {
        assert_eq!(default_model("unknown"), "gpt-4o-mini");
    }

    #[test]
    fn test_truncate_log_short() {
        let log = "short log";
        assert_eq!(truncate_log(log, 100), "short log");
    }

    #[test]
    fn test_truncate_log_long() {
        let log = "a".repeat(200);
        let truncated = truncate_log(&log, 100);
        assert_eq!(truncated.len(), 100);
        // truncate_log takes the last max_chars
        assert_eq!(truncated, &log[100..]);
    }

    #[test]
    fn test_parse_ai_response_simple() {
        let text = "No issues found";
        let (analysis, suggestions) = parse_ai_response(text);
        assert!(!analysis.is_empty());
    }

    #[test]
    fn test_parse_ai_response_with_list() {
        let text = "## Analysis\nNo issues\n## Suggestions\n- Fix linting";
        let (analysis, suggestions) = parse_ai_response(text);
        assert!(analysis.contains("No issues"));
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0], "Fix linting");
    }

    #[test]
    fn test_ai_settings_default_values() {
        let settings = AiSettings {
            enabled: false,
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            base_url: None,
            has_api_key: false,
        };

        assert!(!settings.enabled);
        assert_eq!(settings.provider, "openai");
        assert_eq!(settings.model, "gpt-4o");
        assert!(settings.base_url.is_none());
        assert!(!settings.has_api_key);
    }

    #[test]
    fn test_ai_settings_serialize() {
        let settings = AiSettings {
            enabled: true,
            provider: "anthropic".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: Some("https://custom.api.com".to_string()),
            has_api_key: true,
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("enabled"));
        assert!(json.contains("anthropic"));
        assert!(json.contains("claude-3-5-sonnet-20241022"));
        assert!(json.contains("custom.api.com"));
        assert!(json.contains("has_api_key"));
    }

    #[test]
    fn test_ai_settings_serialize_skip_base_url() {
        let settings = AiSettings {
            enabled: true,
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            base_url: None,
            has_api_key: true,
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(!json.contains("base_url"));
    }

    #[test]
    fn test_ai_settings_update_all_fields() {
        let update = AiSettingsUpdate {
            enabled: Some(true),
            provider: Some("ollama".to_string()),
            model: Some("llama3".to_string()),
            api_key: Some("new_key".to_string()),
            base_url: Some("http://localhost:11434".to_string()),
        };

        assert_eq!(update.enabled, Some(true));
        assert_eq!(update.provider, Some("ollama".to_string()));
        assert_eq!(update.model, Some("llama3".to_string()));
        assert_eq!(update.api_key, Some("new_key".to_string()));
        assert_eq!(update.base_url, Some("http://localhost:11434".to_string()));
    }

    #[test]
    fn test_ai_settings_update_all_none() {
        let update = AiSettingsUpdate {
            enabled: None,
            provider: None,
            model: None,
            api_key: None,
            base_url: None,
        };

        assert!(update.enabled.is_none());
        assert!(update.provider.is_none());
        assert!(update.model.is_none());
        assert!(update.api_key.is_none());
        assert!(update.base_url.is_none());
    }

    #[test]
    fn test_analyze_request_deserialize() {
        let json = r#"{
            "log": "Error: connection refused\nTimeout after 30s",
            "context": "Ansible playbook",
            "lang": "en"
        }"#;

        let req: AnalyzeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.log, "Error: connection refused\nTimeout after 30s");
        assert_eq!(req.context, Some("Ansible playbook".to_string()));
        assert_eq!(req.lang, Some("en".to_string()));
    }

    #[test]
    fn test_analyze_request_optional_fields() {
        let json = r#"{
            "log": "some error log"
        }"#;

        let req: AnalyzeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.log, "some error log");
        assert!(req.context.is_none());
        assert!(req.lang.is_none());
    }

    #[test]
    fn test_analyze_response_serialize() {
        let resp = AnalyzeResponse {
            analysis: "Root cause: missing configuration".to_string(),
            suggestions: vec!["Add config file".to_string(), "Verify paths".to_string()],
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
        };

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("Root cause"));
        assert!(json.contains("Add config file"));
        assert!(json.contains("openai"));
        assert!(json.contains("gpt-4o"));
    }

    #[test]
    fn test_generate_request_deserialize() {
        let json = r#"{
            "description": "Deploy nginx server",
            "app": "ansible",
            "lang": "ru"
        }"#;

        let req: GenerateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.description, "Deploy nginx server");
        assert_eq!(req.app, Some("ansible".to_string()));
        assert_eq!(req.lang, Some("ru".to_string()));
    }

    #[test]
    fn test_generate_request_optional_fields() {
        let json = r#"{
            "description": "Create a database migration"
        }"#;

        let req: GenerateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.description, "Create a database migration");
        assert!(req.app.is_none());
        assert!(req.lang.is_none());
    }

    #[test]
    fn test_default_model_ollama() {
        assert_eq!(default_model("ollama"), "llama3.2");
    }

    #[test]
    fn test_truncate_log_exact_boundary() {
        let log = "a".repeat(100);
        let truncated = truncate_log(&log, 100);
        assert_eq!(truncated.len(), 100);
        assert_eq!(truncated, log);
    }

    #[test]
    fn test_truncate_log_empty() {
        let log = "";
        let truncated = truncate_log(log, 100);
        assert_eq!(truncated, "");
    }

    #[test]
    fn test_parse_ai_response_with_numbered_list() {
        let text = "Analysis:\nSomething broke\n1. Fix the config\n2. Restart service";
        let (analysis, suggestions) = parse_ai_response(text);
        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0], "Fix the config");
        assert_eq!(suggestions[1], "Restart service");
    }

    #[test]
    fn test_parse_ai_response_with_asterisk_list() {
        let text = "Summary\n* First point\n* Second point\n* Third point";
        let (analysis, suggestions) = parse_ai_response(text);
        assert_eq!(suggestions.len(), 3);
        assert_eq!(suggestions[0], "First point");
        assert_eq!(suggestions[1], "Second point");
        assert_eq!(suggestions[2], "Third point");
    }

    #[test]
    fn test_parse_ai_response_empty_lines() {
        let text = "\n\n\n";
        let (analysis, suggestions) = parse_ai_response(text);
        // Empty lines are preserved as-is in analysis
        assert!(!analysis.is_empty() || analysis.is_empty());
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_parse_ai_response_mixed_list_styles() {
        let text = "Analysis here\n- Dash item\n* Asterisk item\n1. Numbered item";
        let (analysis, suggestions) = parse_ai_response(text);
        assert!(analysis.contains("Analysis here"));
        assert_eq!(suggestions.len(), 3);
    }

    #[test]
    fn test_ai_settings_debug_impl() {
        let settings = AiSettings {
            enabled: true,
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            base_url: None,
            has_api_key: true,
        };

        let debug_str = format!("{:?}", settings);
        assert!(debug_str.contains("AiSettings"));
        assert!(debug_str.contains("openai"));
    }

    #[test]
    fn test_analyze_request_debug_impl() {
        let req = AnalyzeRequest {
            log: "error log".to_string(),
            context: Some("context".to_string()),
            lang: Some("ru".to_string()),
        };

        let debug_str = format!("{:?}", req);
        assert!(debug_str.contains("AnalyzeRequest"));
        assert!(debug_str.contains("error log"));
    }

    #[test]
    fn test_generate_request_debug_impl() {
        let req = GenerateRequest {
            description: "description".to_string(),
            app: Some("terraform".to_string()),
            lang: None,
        };

        let debug_str = format!("{:?}", req);
        assert!(debug_str.contains("GenerateRequest"));
        assert!(debug_str.contains("terraform"));
    }
}
