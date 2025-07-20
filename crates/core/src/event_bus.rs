use ai_manager_shared::{
    Result, ServiceId, ServiceMessage, SystemError, SystemEvent, BROADCAST_CHANNEL_CAPACITY,
    MESSAGE_QUEUE_CAPACITY,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, error, info, warn};

pub type MessageSender = mpsc::Sender<ServiceMessage>;
pub type MessageReceiver = mpsc::Receiver<ServiceMessage>;
pub type EventSender = broadcast::Sender<SystemEvent>;
pub type EventReceiver = broadcast::Receiver<SystemEvent>;

#[derive(Debug)]
pub struct EventBus {
    // Service message senders
    service_senders: Arc<RwLock<HashMap<ServiceId, MessageSender>>>,

    // System event broadcaster
    event_broadcaster: EventSender,

    // Bus statistics
    stats: Arc<RwLock<EventBusStats>>,
}

#[derive(Debug, Default)]
pub struct EventBusStats {
    pub messages_routed: u64,
    pub events_broadcast: u64,
    pub routing_errors: u64,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);

        Self {
            service_senders: Arc::new(RwLock::new(HashMap::new())),
            event_broadcaster: event_tx,
            stats: Arc::new(RwLock::new(EventBusStats::default())),
        }
    }

    /// Register a service with the event bus
    pub async fn register_service(
        &self,
        service_id: ServiceId,
    ) -> Result<(MessageSender, MessageReceiver)> {
        let (tx, rx) = mpsc::channel(MESSAGE_QUEUE_CAPACITY);

        {
            let mut senders = self.service_senders.write().await;
            senders.insert(service_id.clone(), tx.clone());
        }

        info!("Service '{}' registered with event bus", service_id);

        // Broadcast service registration event
        let event = SystemEvent::ServiceStarted { service_id };
        self.broadcast_event(event).await;

        Ok((tx, rx))
    }

    /// Unregister a service from the event bus
    pub async fn unregister_service(&self, service_id: &ServiceId) -> Result<()> {
        {
            let mut senders = self.service_senders.write().await;
            senders.remove(service_id);
        }

        info!("Service '{}' unregistered from event bus", service_id);

        // Broadcast service stopped event
        let event = SystemEvent::ServiceStopped {
            service_id: service_id.clone(),
        };
        self.broadcast_event(event).await;

        Ok(())
    }

    /// Route a message to the appropriate service
    pub async fn route_message(
        &self,
        message: ServiceMessage,
        target_service: Option<ServiceId>,
    ) -> Result<()> {
        debug!("Routing message: {:?}", message);

        // Determine target service if not specified
        let target = match target_service {
            Some(service) => service,
            None => self.determine_target_service(&message)?,
        };

        // Get sender for target service
        let sender = {
            let senders = self.service_senders.read().await;
            senders.get(&target).cloned()
        };

        match sender {
            Some(tx) => {
                // Attempt to send message
                if let Err(e) = tx.send(message.clone()).await {
                    error!("Failed to route message to service '{}': {}", target, e);

                    // Update error stats
                    {
                        let mut stats = self.stats.write().await;
                        stats.routing_errors += 1;
                    }

                    return Err(SystemError::ServiceCommunication(format!(
                        "Failed to send message to service '{}': {}",
                        target, e
                    )));
                }

                // Update success stats
                {
                    let mut stats = self.stats.write().await;
                    stats.messages_routed += 1;
                }

                debug!("Message routed successfully to service '{}'", target);

                // Broadcast message received event
                let event = SystemEvent::MessageReceived {
                    from: "event_bus".to_string(),
                    to: target,
                };
                self.broadcast_event(event).await;

                Ok(())
            }
            None => {
                warn!("Target service '{}' not found", target);
                Err(SystemError::ServiceUnavailable { service: target })
            }
        }
    }

    /// Broadcast a system event to all subscribers
    pub async fn broadcast_event(&self, event: SystemEvent) {
        debug!("Broadcasting event: {:?}", event);

        if let Err(e) = self.event_broadcaster.send(event) {
            error!("Failed to broadcast event: {}", e);
        } else {
            // Update broadcast stats
            let mut stats = self.stats.write().await;
            stats.events_broadcast += 1;
        }
    }

    /// Subscribe to system events
    pub fn subscribe_to_events(&self) -> EventReceiver {
        self.event_broadcaster.subscribe()
    }

    /// Get event bus statistics
    pub async fn get_stats(&self) -> EventBusStats {
        self.stats.read().await.clone()
    }

    /// Get list of registered services
    pub async fn get_registered_services(&self) -> Vec<ServiceId> {
        let senders = self.service_senders.read().await;
        senders.keys().cloned().collect()
    }

    /// Determine the target service for a message based on message type
    fn determine_target_service(&self, message: &ServiceMessage) -> Result<ServiceId> {
        use ai_manager_shared::*;

        let target =
            match message {
                // Messages going to LLM service
                ServiceMessage::LLMRequest { .. } => LLM_SERVICE_ID,

                // Messages going to data service
                ServiceMessage::StoreConversation { .. }
                | ServiceMessage::LoadUserProfile { .. } => DATA_SERVICE_ID,

                // Messages going to external service
                ServiceMessage::CalendarSync { .. } | ServiceMessage::EmailProcess { .. } => {
                    EXTERNAL_SERVICE_ID
                }

                // Messages going to UI service
                ServiceMessage::SystemResponse { .. }
                | ServiceMessage::UserProfileResponse { .. } => UI_SERVICE_ID,

                // Messages going to core service
                ServiceMessage::UserInput { .. }
                | ServiceMessage::LLMResponse { .. }
                | ServiceMessage::ServiceHealthResponse { .. } => CORE_SERVICE_ID,

                // Health check messages - broadcast to all
                ServiceMessage::ServiceHealthCheck { .. } => {
                    return Err(SystemError::InvalidInput(
                        "Health check messages should be broadcast, not routed".to_string(),
                    ));
                }

                ServiceMessage::ShutdownService { service_id } => service_id,
            };

        Ok(target.to_string())
    }
}

impl Clone for EventBusStats {
    fn clone(&self) -> Self {
        Self {
            messages_routed: self.messages_routed,
            events_broadcast: self.events_broadcast,
            routing_errors: self.routing_errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_service_registration() {
        let bus = EventBus::new();
        let service_id = "test-service".to_string();

        let result = bus.register_service(service_id.clone()).await;
        assert!(result.is_ok());

        let services = bus.get_registered_services().await;
        assert!(services.contains(&service_id));
    }

    #[tokio::test]
    async fn test_message_routing() {
        let bus = EventBus::new();

        // Register a service
        let service_id = ai_manager_shared::CORE_SERVICE_ID.to_string();
        let (_tx, mut rx) = bus.register_service(service_id.clone()).await.unwrap();

        // Send a message
        let message = ServiceMessage::UserInput {
            content: "Hello".to_string(),
            timestamp: chrono::Utc::now(),
            user_id: "test-user".to_string(),
        };

        bus.route_message(message.clone(), Some(service_id))
            .await
            .unwrap();

        // Verify message was received
        let received = timeout(Duration::from_millis(100), rx.recv()).await;
        assert!(received.is_ok());
        assert!(received.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_event_broadcasting() {
        let bus = EventBus::new();
        let mut event_rx = bus.subscribe_to_events();

        let event = SystemEvent::ServiceStarted {
            service_id: "test-service".to_string(),
        };

        bus.broadcast_event(event.clone()).await;

        let received = timeout(Duration::from_millis(100), event_rx.recv()).await;
        assert!(received.is_ok());
    }
}
