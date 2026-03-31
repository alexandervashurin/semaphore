/// Velum REST API HTTP client.
///
/// Configuration via environment variables:
///   VELUM_URL          — base URL (required), e.g. http://localhost:8088
///   VELUM_API_TOKEN    — API token from User Settings → API Tokens (required)
///   VELUM_TIMEOUT_SECS — request timeout in seconds (default: 30)
use anyhow::{anyhow, Context, Result};
use reqwest::{Client, StatusCode};
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;

#[derive(Clone)]
pub struct VelumClient {
    http: Client,
    base: String,
}

impl VelumClient {
    pub fn from_env() -> Result<Self> {
        let base = std::env::var("VELUM_URL")
            .context("VELUM_URL environment variable is required")?
            .trim_end_matches('/')
            .to_string();
        let token = std::env::var("VELUM_API_TOKEN")
            .context("VELUM_API_TOKEN environment variable is required")?;
        let timeout = std::env::var("VELUM_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30u64);

        let http = Client::builder()
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(
                    reqwest::header::AUTHORIZATION,
                    format!("Bearer {token}").parse()?,
                );
                h.insert(reqwest::header::CONTENT_TYPE, "application/json".parse()?);
                h.insert(reqwest::header::ACCEPT, "application/json".parse()?);
                h
            })
            .timeout(Duration::from_secs(timeout))
            .build()?;

        Ok(Self { http, base })
    }

    fn url(&self, path: &str) -> String {
        format!("{}/api{}", self.base, path)
    }

    pub async fn get(&self, path: &str) -> Result<Value> {
        let resp = self.http.get(self.url(path)).send().await?;
        self.handle(resp).await
    }

    pub async fn get_query(&self, path: &str, query: &[(&str, &str)]) -> Result<Value> {
        let resp = self.http.get(self.url(path)).query(query).send().await?;
        self.handle(resp).await
    }

    pub async fn post<B: Serialize>(&self, path: &str, body: &B) -> Result<Value> {
        let resp = self.http.post(self.url(path)).json(body).send().await?;
        self.handle(resp).await
    }

    pub async fn post_empty(&self, path: &str) -> Result<Value> {
        let resp = self.http.post(self.url(path)).json(&serde_json::json!({})).send().await?;
        self.handle(resp).await
    }

    pub async fn put<B: Serialize>(&self, path: &str, body: &B) -> Result<Value> {
        let resp = self.http.put(self.url(path)).json(body).send().await?;
        self.handle(resp).await
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        let resp = self.http.delete(self.url(path)).send().await?;
        if resp.status().is_success() || resp.status() == StatusCode::NOT_FOUND {
            return Ok(());
        }
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(anyhow!("DELETE {path} failed {status}: {text}"))
    }

    async fn handle(&self, resp: reqwest::Response) -> Result<Value> {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if status.is_success() {
            if text.is_empty() {
                return Ok(Value::Null);
            }
            serde_json::from_str(&text)
                .with_context(|| format!("Failed to parse JSON response: {text}"))
        } else {
            Err(anyhow!("API error {status}: {text}"))
        }
    }
}
