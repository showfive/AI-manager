use crate::event_bus::EventBus;
use ai_manager_shared::{ResponseType, Result, ServiceMessage, SystemError, LLM_SERVICE_ID};

#[cfg(test)]
use ai_manager_shared::UI_SERVICE_ID;
use chrono::Utc;
use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;

pub struct UserInputHandler {
    event_bus: Arc<EventBus>,
}

impl UserInputHandler {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self { event_bus }
    }

    /// Handle user input and route to appropriate services
    pub async fn handle_user_input(&self, user_input: ServiceMessage) -> Result<()> {
        if let ServiceMessage::UserInput {
            content,
            timestamp: _,
            user_id,
        } = user_input
        {
            info!("Processing user input from user '{}': {}", user_id, content);

            // Basic input validation
            if content.trim().is_empty() {
                let response = ServiceMessage::SystemResponse {
                    content: "Please provide a non-empty message.".to_string(),
                    message_type: ResponseType::Warning,
                    timestamp: Utc::now(),
                };

                return self.event_bus.route_message(response, None).await;
            }

            // Check for system commands
            if content.starts_with('/') {
                return self.handle_system_command(&content, &user_id).await;
            }

            // Send thinking response
            let thinking_response = ServiceMessage::SystemResponse {
                content: "Thinking...".to_string(),
                message_type: ResponseType::Thinking,
                timestamp: Utc::now(),
            };
            self.event_bus
                .route_message(thinking_response, None)
                .await?;

            // Create LLM request
            let llm_request = ServiceMessage::LLMRequest {
                prompt: content,
                context: vec![],                // TODO: Add conversation context
                provider: "openai".to_string(), // TODO: Get from config
                request_id: Uuid::new_v4(),
            };

            // Route to LLM service
            self.event_bus
                .route_message(llm_request, Some(LLM_SERVICE_ID.to_string()))
                .await?;

            debug!("User input routed to LLM service");
            Ok(())
        } else {
            error!("Invalid message type for user input handler");
            Err(SystemError::InvalidInput(
                "Expected UserInput message".to_string(),
            ))
        }
    }

    /// Handle system commands (commands starting with /)
    async fn handle_system_command(&self, command: &str, _user_id: &str) -> Result<()> {
        debug!("Processing system command: {}", command);

        let response_content = match command {
            "/help" => {
                "Available commands:\n/help - Show this help\n/status - Show system status\n/clear - Clear conversation history".to_string()
            }
            "/status" => {
                self.get_system_status().await
            }
            "/clear" => {
                // TODO: Implement conversation clearing
                "Conversation history cleared.".to_string()
            }
            _ => {
                format!("Unknown command: {}. Type /help for available commands.", command)
            }
        };

        let response = ServiceMessage::SystemResponse {
            content: response_content,
            message_type: ResponseType::Info,
            timestamp: Utc::now(),
        };

        self.event_bus.route_message(response, None).await
    }

    /// Get system status information
    async fn get_system_status(&self) -> String {
        let services = self.event_bus.get_registered_services().await;
        let stats = self.event_bus.get_stats().await;

        format!(
            "System Status:\n• Registered services: {}\n• Messages routed: {}\n• Events broadcast: {}\n• Routing errors: {}",
            services.len(),
            stats.messages_routed,
            stats.events_broadcast,
            stats.routing_errors
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_bus::EventBus;

    #[tokio::test]
    async fn test_user_input_handler() {
        let event_bus = Arc::new(EventBus::new());
        let handler = UserInputHandler::new(event_bus.clone());

        // Register both LLM and UI services to receive messages
        let _llm_service = event_bus
            .register_service(LLM_SERVICE_ID.to_string())
            .await
            .unwrap();
        let _ui_service = event_bus
            .register_service(UI_SERVICE_ID.to_string())
            .await
            .unwrap();

        let user_input = ServiceMessage::UserInput {
            content: "Hello, AI!".to_string(),
            timestamp: Utc::now(),
            user_id: "test-user".to_string(),
        };

        let result = handler.handle_user_input(user_input).await;
        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_system_commands() {
        let event_bus = Arc::new(EventBus::new());
        let handler = UserInputHandler::new(event_bus.clone());

        // Register UI service to receive system responses
        let _ui_service = event_bus
            .register_service(UI_SERVICE_ID.to_string())
            .await
            .unwrap();

        let help_command = ServiceMessage::UserInput {
            content: "/help".to_string(),
            timestamp: Utc::now(),
            user_id: "test-user".to_string(),
        };

        let result = handler.handle_user_input(help_command).await;
        assert!(result.is_ok());
    }
}
