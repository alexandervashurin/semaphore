//! CLI - Healthcheck Command
//!
//! Используется в scratch-образе, где нет curl или wget для Docker healthcheck.

use crate::cli::CliResult;
use anyhow::{anyhow, Context};
use clap::Args;
use std::time::Duration;

/// Команда healthcheck
#[derive(Debug, Args)]
pub struct HealthcheckCommand {
    /// URL для проверки
    #[arg(long, default_value = "http://127.0.0.1:3000/healthz")]
    pub url: String,

    /// Таймаут запроса в секундах
    #[arg(long, default_value_t = 5)]
    pub timeout_secs: u64,
}

impl HealthcheckCommand {
    /// Выполняет команду
    pub fn run(&self) -> CliResult<()> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(self.timeout_secs))
            .build()
            .context("failed to create healthcheck client")?;

        let response = client
            .get(&self.url)
            .send()
            .with_context(|| format!("healthcheck request failed for {}", self.url))?;

        if response.status().is_success() {
            return Ok(());
        }

        Err(anyhow!(
            "healthcheck failed with status {} for {}",
            response.status(),
            self.url
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthcheck_command_defaults() {
        let cmd = HealthcheckCommand {
            url: "http://127.0.0.1:3000/healthz".to_string(),
            timeout_secs: 5,
        };

        assert_eq!(cmd.url, "http://127.0.0.1:3000/healthz");
        assert_eq!(cmd.timeout_secs, 5);
    }
}
