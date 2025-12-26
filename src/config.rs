use crate::error::{Error, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub github_token: String,
    pub anthropic_api_key: String,
    pub database_path: String,
    pub max_commits_per_repo: u32,
    pub include_forks: bool,
    pub concurrency_limit: usize,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let github_token = env::var("GITHUB_TOKEN")
            .map_err(|_| Error::Config("GITHUB_TOKEN environment variable not set".to_string()))?;

        let anthropic_api_key = env::var("ANTHROPIC_API_KEY")
            .map_err(|_| Error::Config("ANTHROPIC_API_KEY environment variable not set".to_string()))?;

        let database_path = env::var("DATABASE_PATH")
            .unwrap_or_else(|_| "gitanalyzer.db".to_string());

        let max_commits_per_repo = env::var("MAX_COMMITS_PER_REPO")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);

        let include_forks = env::var("INCLUDE_FORKS")
            .ok()
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);

        let concurrency_limit = env::var("CONCURRENCY_LIMIT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);

        Ok(Self {
            github_token,
            anthropic_api_key,
            database_path,
            max_commits_per_repo,
            include_forks,
            concurrency_limit,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub max_commits_per_repo: u32,
    pub include_forks: bool,
    pub concurrency_limit: usize,
}

impl From<&Config> for PipelineConfig {
    fn from(config: &Config) -> Self {
        Self {
            max_commits_per_repo: config.max_commits_per_repo,
            include_forks: config.include_forks,
            concurrency_limit: config.concurrency_limit,
        }
    }
}
