use crate::provider::{FinishReason, LLMProvider, LLMRequest, LLMResponse};
use ai_manager_shared::{Result, SystemError, TokenUsage};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, warn};

const CLAUDE_API_BASE: &str = "https://api.anthropic.com/v1";
const DEFAULT_MODEL: &str = "claude-3-haiku-20240307";
const DEFAULT_MAX_TOKENS: u32 = 2000;
const DEFAULT_TEMPERATURE: f32 = 0.7;

pub struct ClaudeProvider {
    client: Client,
    api_key: String,
    base_url: String,
    default_model: String,
    max_tokens: u32,
    temperature: f32,
    total_usage: TokenUsage,
}

impl ClaudeProvider {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(ai_manager_shared::LLM_REQUEST_TIMEOUT))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            base_url: CLAUDE_API_BASE.to_string(),
            default_model: DEFAULT_MODEL.to_string(),
            max_tokens: DEFAULT_MAX_TOKENS,
            temperature: DEFAULT_TEMPERATURE,
            total_usage: TokenUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
        }
    }

    pub fn with_config(
        api_key: String,
        base_url: Option<String>,
        model: Option<String>,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Self {
        let mut provider = Self::new(api_key);

        if let Some(url) = base_url {
            provider.base_url = url;
        }
        if let Some(model) = model {
            provider.default_model = model;
        }
        if let Some(tokens) = max_tokens {
            provider.max_tokens = tokens;
        }
        if let Some(temp) = temperature {
            provider.temperature = temp;
        }

        provider
    }

    fn build_messages(&self, request: &LLMRequest) -> Vec<ClaudeMessage> {
        let mut messages = Vec::new();

        // Add context messages if any
        for context in &request.context {
            messages.push(ClaudeMessage {
                role: "user".to_string(),
                content: context.clone(),
            });
        }

        // Add current prompt
        messages.push(ClaudeMessage {
            role: "user".to_string(),
            content: request.prompt.clone(),
        });

        messages
    }
}

#[async_trait]
impl LLMProvider for ClaudeProvider {
    async fn send_request(&self, request: LLMRequest) -> Result<LLMResponse> {
        debug!("Sending Claude request: {}", request.prompt);

        let messages = self.build_messages(&request);

        let claude_request = ClaudeRequest {
            model: if request.model.is_empty() {
                self.default_model.clone()
            } else {
                request.model.clone()
            },
            max_tokens: request.max_tokens.unwrap_or(self.max_tokens),
            messages,
            temperature: request.temperature.or(Some(self.temperature)),
            stop_sequences: request.stop_sequences.clone(),
            stream: Some(request.stream),
        };

        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&claude_request)
            .send()
            .await
            .map_err(|e| SystemError::Network(format!("Claude request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Claude API error {}: {}", status, error_text);

            return Err(SystemError::LLMApi {
                provider: "claude".to_string(),
                message: format!("HTTP {}: {}", status, error_text),
            });
        }

        let claude_response: ClaudeResponse = response.json().await.map_err(|e| {
            SystemError::Serialization(format!("Failed to parse Claude response: {}", e))
        })?;

        // Extract the response content
        let content_block = claude_response
            .content
            .first()
            .ok_or_else(|| SystemError::LLMApi {
                provider: "claude".to_string(),
                message: "No content in Claude response".to_string(),
            })?;

        let content = match content_block {
            ClaudeContent::Text { text } => text.clone(),
        };

        let finish_reason = match claude_response.stop_reason.as_str() {
            "end_turn" => FinishReason::Stop,
            "max_tokens" => FinishReason::Length,
            "stop_sequence" => FinishReason::Stop,
            other => {
                warn!("Unknown stop reason from Claude: {}", other);
                FinishReason::Stop
            }
        };

        // Build usage statistics
        let usage = TokenUsage {
            prompt_tokens: claude_response.usage.input_tokens,
            completion_tokens: claude_response.usage.output_tokens,
            total_tokens: claude_response.usage.input_tokens + claude_response.usage.output_tokens,
        };

        debug!(
            "Claude request completed. Tokens used: {}",
            usage.total_tokens
        );

        Ok(LLMResponse {
            content,
            model: claude_response.model,
            usage,
            finish_reason,
            provider: "claude".to_string(),
        })
    }

    async fn get_usage(&self) -> TokenUsage {
        self.total_usage.clone()
    }

    fn provider_name(&self) -> &str {
        "claude"
    }

    async fn health_check(&self) -> Result<()> {
        debug!("Performing Claude health check");

        // Claude doesn't have a simple health check endpoint, so we'll do a minimal request
        let test_request = ClaudeRequest {
            model: self.default_model.clone(),
            max_tokens: 1,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: "Hi".to_string(),
            }],
            temperature: Some(0.0),
            stop_sequences: None,
            stream: Some(false),
        };

        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&test_request)
            .send()
            .await
            .map_err(|e| SystemError::Network(format!("Claude health check failed: {}", e)))?;

        if response.status().is_success() {
            debug!("Claude health check passed");
            Ok(())
        } else {
            let status = response.status();
            error!("Claude health check failed with status: {}", status);
            Err(SystemError::LLMApi {
                provider: "claude".to_string(),
                message: format!("Health check failed with HTTP {}", status),
            })
        }
    }
}

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ClaudeResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<ClaudeContent>,
    model: String,
    stop_reason: String,
    stop_sequence: Option<String>,
    usage: ClaudeUsage,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClaudeContent {
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a valid Claude API key to run
    // They are disabled by default to avoid unnecessary API calls

    #[tokio::test]
    #[ignore]
    async fn test_claude_provider() {
        let api_key = std::env::var("CLAUDE_API_KEY").expect("CLAUDE_API_KEY not set");
        let provider = ClaudeProvider::new(api_key);

        let request = LLMRequest {
            prompt: "Hello, how are you?".to_string(),
            context: vec![],
            model: "claude-3-haiku-20240307".to_string(),
            max_tokens: Some(50),
            temperature: Some(0.7),
            stop_sequences: None,
            stream: false,
        };

        let response = provider.send_request(request).await.unwrap();
        assert!(!response.content.is_empty());
        assert_eq!(response.provider, "claude");
    }

    #[tokio::test]
    #[ignore]
    async fn test_claude_health_check() {
        let api_key = std::env::var("CLAUDE_API_KEY").expect("CLAUDE_API_KEY not set");
        let provider = ClaudeProvider::new(api_key);

        let result = provider.health_check().await;
        assert!(result.is_ok());
    }
}
