use crate::provider::{FinishReason, LLMProvider, LLMRequest, LLMResponse};
use ai_manager_shared::{Result, SystemError, TokenUsage};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, warn};

const OPENAI_API_BASE: &str = "https://api.openai.com/v1";
const DEFAULT_MODEL: &str = "gpt-3.5-turbo";
const DEFAULT_MAX_TOKENS: u32 = 2000;
const DEFAULT_TEMPERATURE: f32 = 0.7;

pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    base_url: String,
    default_model: String,
    max_tokens: u32,
    temperature: f32,
    total_usage: TokenUsage,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(ai_manager_shared::LLM_REQUEST_TIMEOUT))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            base_url: OPENAI_API_BASE.to_string(),
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

    fn build_messages(&self, request: &LLMRequest) -> Vec<OpenAIMessage> {
        let mut messages = Vec::new();

        // Add context messages if any
        for context in &request.context {
            messages.push(OpenAIMessage {
                role: "user".to_string(),
                content: context.clone(),
            });
        }

        // Add current prompt
        messages.push(OpenAIMessage {
            role: "user".to_string(),
            content: request.prompt.clone(),
        });

        messages
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn send_request(&self, request: LLMRequest) -> Result<LLMResponse> {
        debug!("Sending OpenAI request: {}", request.prompt);

        let messages = self.build_messages(&request);

        let openai_request = OpenAIRequest {
            model: if request.model.is_empty() {
                self.default_model.clone()
            } else {
                request.model.clone()
            },
            messages,
            max_tokens: request.max_tokens.or(Some(self.max_tokens)),
            temperature: request.temperature.or(Some(self.temperature)),
            stop: request.stop_sequences.clone(),
            stream: Some(request.stream),
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await
            .map_err(|e| SystemError::Network(format!("OpenAI request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("OpenAI API error {}: {}", status, error_text);

            return Err(SystemError::LLMApi {
                provider: "openai".to_string(),
                message: format!("HTTP {}: {}", status, error_text),
            });
        }

        let openai_response: OpenAIResponse = response.json().await.map_err(|e| {
            SystemError::Serialization(format!("Failed to parse OpenAI response: {}", e))
        })?;

        // Extract the response content
        let choice = openai_response
            .choices
            .first()
            .ok_or_else(|| SystemError::LLMApi {
                provider: "openai".to_string(),
                message: "No choices in OpenAI response".to_string(),
            })?;

        let content = choice.message.content.clone();
        let finish_reason = match choice.finish_reason.as_str() {
            "stop" => FinishReason::Stop,
            "length" => FinishReason::Length,
            "content_filter" => FinishReason::ContentFilter,
            other => {
                warn!("Unknown finish reason from OpenAI: {}", other);
                FinishReason::Stop
            }
        };

        // Update usage statistics
        let usage = TokenUsage {
            prompt_tokens: openai_response.usage.prompt_tokens,
            completion_tokens: openai_response.usage.completion_tokens,
            total_tokens: openai_response.usage.total_tokens,
        };

        debug!(
            "OpenAI request completed. Tokens used: {}",
            usage.total_tokens
        );

        Ok(LLMResponse {
            content,
            model: openai_response.model,
            usage,
            finish_reason,
            provider: "openai".to_string(),
        })
    }

    async fn get_usage(&self) -> TokenUsage {
        self.total_usage.clone()
    }

    fn provider_name(&self) -> &str {
        "openai"
    }

    async fn health_check(&self) -> Result<()> {
        debug!("Performing OpenAI health check");

        // Simple request to check if API is accessible
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| SystemError::Network(format!("OpenAI health check failed: {}", e)))?;

        if response.status().is_success() {
            debug!("OpenAI health check passed");
            Ok(())
        } else {
            let status = response.status();
            error!("OpenAI health check failed with status: {}", status);
            Err(SystemError::LLMApi {
                provider: "openai".to_string(),
                message: format!("Health check failed with HTTP {}", status),
            })
        }
    }
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAIChoice {
    index: u32,
    message: OpenAIMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a valid OpenAI API key to run
    // They are disabled by default to avoid unnecessary API calls

    #[tokio::test]
    #[ignore]
    async fn test_openai_provider() {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let provider = OpenAIProvider::new(api_key);

        let request = LLMRequest {
            prompt: "Hello, how are you?".to_string(),
            context: vec![],
            model: "gpt-3.5-turbo".to_string(),
            max_tokens: Some(50),
            temperature: Some(0.7),
            stop_sequences: None,
            stream: false,
        };

        let response = provider.send_request(request).await.unwrap();
        assert!(!response.content.is_empty());
        assert_eq!(response.provider, "openai");
    }

    #[tokio::test]
    #[ignore]
    async fn test_openai_health_check() {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let provider = OpenAIProvider::new(api_key);

        let result = provider.health_check().await;
        assert!(result.is_ok());
    }
}
