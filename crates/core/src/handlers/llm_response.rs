use crate::event_bus::EventBus;
use ai_manager_shared::{
    Message, MessageRole, ResponseType, Result, ServiceMessage, SystemError, DATA_SERVICE_ID,
    UI_SERVICE_ID,
};
use chrono::Utc;
use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;

pub struct LLMResponseHandler {
    event_bus: Arc<EventBus>,
}

impl LLMResponseHandler {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self { event_bus }
    }

    /// Handle LLM response and route to UI and data services
    pub async fn handle_llm_response(&self, llm_response: ServiceMessage) -> Result<()> {
        if let ServiceMessage::LLMResponse {
            content,
            usage,
            request_id,
        } = llm_response
        {
            info!("Processing LLM response for request {}", request_id);
            debug!(
                "LLM response content: {} (tokens: {})",
                content, usage.total_tokens
            );

            // Create system response for UI
            let ui_response = ServiceMessage::SystemResponse {
                content: content.clone(),
                message_type: ResponseType::Success,
                timestamp: Utc::now(),
            };

            // Route response to UI
            self.event_bus
                .route_message(ui_response, Some(UI_SERVICE_ID.to_string()))
                .await?;

            // Create message for conversation storage
            let message = Message {
                id: Uuid::new_v4(),
                content,
                timestamp: Utc::now(),
                role: MessageRole::Assistant,
                metadata: Some(serde_json::json!({
                    "request_id": request_id,
                    "token_usage": usage,
                })),
            };

            // Store conversation in data service
            // Note: In a real implementation, we'd need to track the user_id from the original request
            let store_request = ServiceMessage::StoreConversation {
                user_id: "current_user".to_string(), // TODO: Get actual user ID
                messages: vec![message],
            };

            self.event_bus
                .route_message(store_request, Some(DATA_SERVICE_ID.to_string()))
                .await?;

            info!("LLM response processed and routed successfully");
            Ok(())
        } else {
            error!("Invalid message type for LLM response handler");
            Err(SystemError::InvalidInput(
                "Expected LLMResponse message".to_string(),
            ))
        }
    }

    /// Handle LLM errors
    pub async fn handle_llm_error(
        &self,
        provider: &str,
        error_message: &str,
        request_id: Uuid,
    ) -> Result<()> {
        error!(
            "LLM error from provider '{}' for request {}: {}",
            provider, request_id, error_message
        );

        // Create error response for UI
        let error_response = ServiceMessage::SystemResponse {
            content: format!(
                "Sorry, I encountered an error while processing your request: {}",
                error_message
            ),
            message_type: ResponseType::Error,
            timestamp: Utc::now(),
        };

        // Route error to UI
        self.event_bus
            .route_message(error_response, Some(UI_SERVICE_ID.to_string()))
            .await?;

        Ok(())
    }

    /// Handle streaming LLM responses (for future implementation)
    pub async fn handle_streaming_response(&self, _chunk: &str, _request_id: Uuid) -> Result<()> {
        // TODO: Implement streaming response handling
        // This would send partial responses to the UI as they arrive
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_bus::EventBus;
    use ai_manager_shared::TokenUsage;

    #[tokio::test]
    async fn test_llm_response_handler() {
        let event_bus = Arc::new(EventBus::new());
        let handler = LLMResponseHandler::new(event_bus.clone());

        // Register UI and data services to receive messages
        let _ui_service = event_bus
            .register_service(UI_SERVICE_ID.to_string())
            .await
            .unwrap();
        let _data_service = event_bus
            .register_service(DATA_SERVICE_ID.to_string())
            .await
            .unwrap();

        let llm_response = ServiceMessage::LLMResponse {
            content: "Hello! How can I help you today?".to_string(),
            usage: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 8,
                total_tokens: 18,
            },
            request_id: Uuid::new_v4(),
        };

        let result = handler.handle_llm_response(llm_response).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_llm_error_handling() {
        let event_bus = Arc::new(EventBus::new());
        let handler = LLMResponseHandler::new(event_bus.clone());

        // Register UI service to receive error messages
        let _ui_service = event_bus
            .register_service(UI_SERVICE_ID.to_string())
            .await
            .unwrap();

        let result = handler
            .handle_llm_error("openai", "API rate limit exceeded", Uuid::new_v4())
            .await;
        assert!(result.is_ok());
    }
}
