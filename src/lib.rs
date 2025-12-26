pub mod config;
pub mod error;
pub mod models;
pub mod github;
pub mod llm;
pub mod taxonomy;
pub mod analysis;
pub mod storage;

pub use config::{Config, PipelineConfig};
pub use error::{Error, Result};
pub use github::GitHubClient;
pub use llm::{ClaudeProvider, LLMProvider};
pub use analysis::AnalysisPipeline;
pub use storage::Storage;
