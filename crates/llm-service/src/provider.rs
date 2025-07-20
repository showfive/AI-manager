use ai_manager_shared::{Result, SystemError, TokenUsage};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Send a request to the LLM provider
    async fn send_request(&self, request: LLMRequest) -> Result<LLMResponse>;

    /// Get usage statistics
    async fn get_usage(&self) -> TokenUsage;

    /// Get provider name
    fn provider_name(&self) -> &str;

    /// Check if provider is available
    async fn health_check(&self) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub prompt: String,
    pub context: Vec<String>,
    pub model: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub model: String,
    pub usage: TokenUsage,
    pub finish_reason: FinishReason,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    Error(String),
}

pub struct LLMService {
    providers: HashMap<String, Box<dyn LLMProvider>>,
    default_provider: String,
}

impl LLMService {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: "openai".to_string(),
        }
    }

    /// Add a provider to the service
    pub fn add_provider(&mut self, name: String, provider: Box<dyn LLMProvider>) {
        self.providers.insert(name, provider);
    }

    /// Set the default provider
    pub fn set_default_provider(&mut self, name: String) -> Result<()> {
        if self.providers.contains_key(&name) {
            self.default_provider = name;
            Ok(())
        } else {
            Err(SystemError::Configuration(format!(
                "Provider '{}' not found",
                name
            )))
        }
    }

    /// Send request using default provider
    pub async fn send_request(&self, request: LLMRequest) -> Result<LLMResponse> {
        self.send_request_with_provider(request, &self.default_provider)
            .await
    }

    /// Send request using specific provider
    pub async fn send_request_with_provider(
        &self,
        request: LLMRequest,
        provider_name: &str,
    ) -> Result<LLMResponse> {
        let provider = self.providers.get(provider_name).ok_or_else(|| {
            SystemError::Configuration(format!("Provider '{}' not found", provider_name))
        })?;

        provider.send_request(request).await
    }

    /// Get available providers
    pub fn get_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Get default provider name
    pub fn get_default_provider(&self) -> &str {
        &self.default_provider
    }

    /// Check health of all providers
    pub async fn health_check_all(&self) -> HashMap<String, Result<()>> {
        let mut results = HashMap::new();

        for (name, provider) in &self.providers {
            let result = provider.health_check().await;
            results.insert(name.clone(), result);
        }

        results
    }

    /// Get usage statistics for all providers
    pub async fn get_usage_all(&self) -> HashMap<String, TokenUsage> {
        let mut usage = HashMap::new();

        for (name, provider) in &self.providers {
            let provider_usage = provider.get_usage().await;
            usage.insert(name.clone(), provider_usage);
        }

        usage
    }
}

impl Default for LLMService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProvider {
        name: String,
    }

    #[async_trait]
    impl LLMProvider for MockProvider {
        async fn send_request(&self, request: LLMRequest) -> Result<LLMResponse> {
            Ok(LLMResponse {
                content: format!("Mock response to: {}", request.prompt),
                model: request.model,
                usage: TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                },
                finish_reason: FinishReason::Stop,
                provider: self.name.clone(),
            })
        }

        async fn get_usage(&self) -> TokenUsage {
            TokenUsage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            }
        }

        fn provider_name(&self) -> &str {
            &self.name
        }

        async fn health_check(&self) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_llm_service() {
        let mut service = LLMService::new();

        // Add mock provider
        let mock_provider = MockProvider {
            name: "mock".to_string(),
        };
        service.add_provider("mock".to_string(), Box::new(mock_provider));
        service.set_default_provider("mock".to_string()).unwrap();

        // Test request
        let request = LLMRequest {
            prompt: "Hello".to_string(),
            context: vec![],
            model: "mock-model".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.7),
            stop_sequences: None,
            stream: false,
        };

        let response = service.send_request(request).await.unwrap();
        assert!(response.content.contains("Hello"));
        assert_eq!(response.provider, "mock");
    }
}
