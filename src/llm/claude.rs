use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::llm::parser::parse_llm_response;
use crate::llm::prompts::{AnalysisRequest, SYSTEM_PROMPT};
use crate::llm::provider::LLMProvider;
use crate::models::analysis::LLMAnalysisResult;

pub struct ClaudeProvider {
    client: Client,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentBlock>,
    #[serde(default)]
    error: Option<ClaudeError>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct ClaudeError {
    message: String,
}

impl ClaudeProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            model: model.unwrap_or_else(|| "claude-sonnet-4-20250514".to_string()),
        }
    }
}

#[async_trait]
impl LLMProvider for ClaudeProvider {
    async fn analyze_commits(&self, request: AnalysisRequest) -> Result<LLMAnalysisResult> {
        let prompt = request.to_prompt();
        tracing::debug!("Sending {} tokens to Claude", request.estimate_tokens());

        let request_body = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            system: Some(SYSTEM_PROMPT.to_string()),
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: prompt,
            }],
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Error::LLMApi(format!("Failed to send request: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::LLMApi(format!(
                "Claude API error ({}): {}",
                status, body
            )));
        }

        let result: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| Error::LLMApi(format!("Failed to parse Claude response: {}", e)))?;

        if let Some(error) = result.error {
            return Err(Error::LLMApi(error.message));
        }

        let text = result
            .content
            .into_iter()
            .filter(|c| c.content_type == "text")
            .filter_map(|c| c.text)
            .collect::<Vec<_>>()
            .join("");

        if text.is_empty() {
            return Err(Error::LLMApi("Empty response from Claude".to_string()));
        }

        parse_llm_response(&text)
    }

    fn max_context_tokens(&self) -> usize {
        200_000 // Claude 3.5 Sonnet / Claude 4
    }

    fn name(&self) -> &str {
        "Claude"
    }
}
