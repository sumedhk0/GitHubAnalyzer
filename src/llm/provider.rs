use async_trait::async_trait;
use crate::error::Result;
use crate::llm::prompts::AnalysisRequest;
use crate::models::analysis::LLMAnalysisResult;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn analyze_commits(&self, request: AnalysisRequest) -> Result<LLMAnalysisResult>;
    fn max_context_tokens(&self) -> usize;
    fn name(&self) -> &str;
}
