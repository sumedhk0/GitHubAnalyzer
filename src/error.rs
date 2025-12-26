use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("GitHub API error: {0}")]
    GitHubApi(String),

    #[error("Rate limit exceeded, retry after {0} seconds")]
    RateLimited(u64),

    #[error("LLM API error: {0}")]
    LLMApi(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Repository not found: {0}")]
    RepoNotFound(String),

    #[error("Invalid header value: {0}")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Error::RateLimited(_) | Error::Network(_))
    }
}
