use crate::event_bus::EventBus;
use ai_manager_shared::{Result, SystemEvent};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

pub struct SystemEventHandler {
    event_bus: Arc<EventBus>,
    handler_task: Option<JoinHandle<()>>,
}

impl SystemEventHandler {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            handler_task: None,
        }
    }

    /// Start listening for system events
    pub async fn start(&mut self) -> Result<()> {
        if self.handler_task.is_some() {
            warn!("System event handler already running");
            return Ok(());
        }

        let mut event_receiver = self.event_bus.subscribe_to_events();
        let event_bus = self.event_bus.clone();

        let handle = tokio::spawn(async move {
            info!("System event handler started");

            loop {
                match event_receiver.recv().await {
                    Ok(event) => {
                        if let Err(e) = Self::handle_event(event, &event_bus).await {
                            error!("Error handling system event: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Error receiving system event: {}", e);
                        break;
                    }
                }
            }

            info!("System event handler stopped");
        });

        self.handler_task = Some(handle);
        Ok(())
    }

    /// Stop the event handler
    pub async fn stop(&mut self) {
        if let Some(handle) = self.handler_task.take() {
            handle.abort();
            info!("System event handler stopped");
        }
    }

    /// Handle a single system event
    async fn handle_event(event: SystemEvent, event_bus: &EventBus) -> Result<()> {
        debug!("Handling system event: {:?}", event);

        match event {
            SystemEvent::ServiceStarted { service_id } => {
                info!("Service '{}' started", service_id);
                Self::on_service_started(&service_id, event_bus).await?;
            }

            SystemEvent::ServiceStopped { service_id } => {
                info!("Service '{}' stopped", service_id);
                Self::on_service_stopped(&service_id, event_bus).await?;
            }

            SystemEvent::ServiceRestarted { service_id } => {
                info!("Service '{}' restarted", service_id);
                Self::on_service_restarted(&service_id, event_bus).await?;
            }

            SystemEvent::ErrorOccurred { service_id, error } => {
                error!("Error in service '{}': {}", service_id, error);
                Self::on_service_error(&service_id, &error, event_bus).await?;
            }

            SystemEvent::MessageReceived { from, to } => {
                debug!("Message routed from '{}' to '{}'", from, to);
                Self::on_message_received(&from, &to, event_bus).await?;
            }
        }

        Ok(())
    }

    /// Handle service started event
    async fn on_service_started(service_id: &str, _event_bus: &EventBus) -> Result<()> {
        // Log service startup
        info!("✓ Service '{}' is now online", service_id);

        // TODO: Additional startup actions could be added here
        // - Update service registry
        // - Send notifications
        // - Initialize service-specific resources

        Ok(())
    }

    /// Handle service stopped event
    async fn on_service_stopped(service_id: &str, _event_bus: &EventBus) -> Result<()> {
        // Log service shutdown
        info!("✗ Service '{}' is now offline", service_id);

        // TODO: Additional shutdown actions could be added here
        // - Clean up resources
        // - Update service registry
        // - Send notifications

        Ok(())
    }

    /// Handle service restarted event
    async fn on_service_restarted(service_id: &str, _event_bus: &EventBus) -> Result<()> {
        // Log service restart
        info!("🔄 Service '{}' has been restarted", service_id);

        // TODO: Additional restart actions could be added here
        // - Reset error counters
        // - Re-initialize connections
        // - Send restart notifications

        Ok(())
    }

    /// Handle service error event
    async fn on_service_error(service_id: &str, error: &str, _event_bus: &EventBus) -> Result<()> {
        // Log and categorize error
        error!("⚠️  Service '{}' encountered error: {}", service_id, error);

        // TODO: Additional error handling could be added here
        // - Increment error counters
        // - Trigger alerts
        // - Attempt automatic recovery
        // - Update service health status

        Ok(())
    }

    /// Handle message received event
    async fn on_message_received(from: &str, to: &str, _event_bus: &EventBus) -> Result<()> {
        // Log message routing (debug level to avoid spam)
        debug!("📨 Message routed from '{}' to '{}'", from, to);

        // TODO: Additional message tracking could be added here
        // - Update message statistics
        // - Monitor message flow patterns
        // - Detect potential bottlenecks

        Ok(())
    }

    /// Get event handler statistics
    pub fn is_running(&self) -> bool {
        self.handler_task.is_some()
    }
}

impl Drop for SystemEventHandler {
    fn drop(&mut self) {
        if let Some(handle) = self.handler_task.take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_bus::EventBus;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_event_handler_lifecycle() {
        let event_bus = Arc::new(EventBus::new());
        let mut handler = SystemEventHandler::new(event_bus.clone());

        // Start handler
        let result = handler.start().await;
        assert!(result.is_ok());
        assert!(handler.is_running());

        // Broadcast an event
        let event = SystemEvent::ServiceStarted {
            service_id: "test-service".to_string(),
        };
        event_bus.broadcast_event(event).await;

        // Give handler time to process
        sleep(Duration::from_millis(10)).await;

        // Stop handler
        handler.stop().await;
        assert!(!handler.is_running());
    }

    #[tokio::test]
    async fn test_multiple_events() {
        let event_bus = Arc::new(EventBus::new());
        let mut handler = SystemEventHandler::new(event_bus.clone());

        handler.start().await.unwrap();

        // Send multiple events
        let events = vec![
            SystemEvent::ServiceStarted {
                service_id: "service1".to_string(),
            },
            SystemEvent::ServiceStopped {
                service_id: "service2".to_string(),
            },
            SystemEvent::ErrorOccurred {
                service_id: "service3".to_string(),
                error: "Test error".to_string(),
            },
        ];

        for event in events {
            event_bus.broadcast_event(event).await;
        }

        // Give handler time to process all events
        sleep(Duration::from_millis(50)).await;

        handler.stop().await;
    }
}
