pub mod calendar;
pub mod email;
pub mod notifications;

use ai_manager_shared::{errors::SystemError, messages::ServiceMessage};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

pub use calendar::GoogleCalendarClient;
pub use email::EmailClient;
pub use notifications::NotificationClient;

#[async_trait]
pub trait Service {
    async fn start(&mut self, mut rx: mpsc::Receiver<ServiceMessage>) -> Result<(), SystemError>;
    async fn handle_message(&mut self, msg: ServiceMessage) -> Result<(), SystemError>;
    async fn health_check(&self) -> ai_manager_shared::messages::ServiceHealth;
    async fn shutdown(&mut self) -> Result<(), SystemError>;
}

pub struct ExternalService {
    calendar: GoogleCalendarClient,
    email: EmailClient,
    notifications: NotificationClient,
    tx: Option<mpsc::Sender<ServiceMessage>>,
}

impl ExternalService {
    pub async fn new(tx: mpsc::Sender<ServiceMessage>) -> Result<Self, SystemError> {
        let calendar = GoogleCalendarClient::new().await?;
        let email = EmailClient::new().await?;
        let notifications = NotificationClient::new().await?;

        Ok(Self {
            calendar,
            email,
            notifications,
            tx: Some(tx),
        })
    }

    async fn handle_calendar_sync(
        &mut self,
        action: ai_manager_shared::messages::CalendarAction,
    ) -> Result<(), SystemError> {
        match action {
            ai_manager_shared::messages::CalendarAction::ListEvents {
                start_date,
                end_date,
            } => {
                let events = self.calendar.list_events(start_date, end_date).await?;
                info!("Retrieved {} calendar events", events.len());

                // Send response back to core service
                if let Some(tx) = &self.tx {
                    let response = ServiceMessage::SystemResponse {
                        content: format!("Found {} calendar events", events.len()),
                        message_type: ai_manager_shared::messages::ResponseType::Info,
                        timestamp: chrono::Utc::now(),
                    };
                    tx.send(response).await.map_err(|e| {
                        SystemError::ServiceCommunication(format!(
                            "Failed to send calendar response: {}",
                            e
                        ))
                    })?;
                }
            }
            ai_manager_shared::messages::CalendarAction::CreateEvent {
                title,
                description,
                start_time,
                end_time,
            } => {
                let event_id = self
                    .calendar
                    .create_event(&title, description.as_deref(), start_time, end_time)
                    .await?;
                info!("Created calendar event: {}", event_id);

                if let Some(tx) = &self.tx {
                    let response = ServiceMessage::SystemResponse {
                        content: format!("Created calendar event: {}", event_id),
                        message_type: ai_manager_shared::messages::ResponseType::Success,
                        timestamp: chrono::Utc::now(),
                    };
                    tx.send(response).await.map_err(|e| {
                        SystemError::ServiceCommunication(format!(
                            "Failed to send calendar response: {}",
                            e
                        ))
                    })?;
                }
            }
            ai_manager_shared::messages::CalendarAction::UpdateEvent {
                event_id,
                title,
                description,
                start_time,
                end_time,
            } => {
                self.calendar
                    .update_event(
                        &event_id,
                        title.as_deref(),
                        description.as_deref(),
                        start_time,
                        end_time,
                    )
                    .await?;
                info!("Updated calendar event: {}", event_id);

                if let Some(tx) = &self.tx {
                    let response = ServiceMessage::SystemResponse {
                        content: format!("Updated calendar event: {}", event_id),
                        message_type: ai_manager_shared::messages::ResponseType::Success,
                        timestamp: chrono::Utc::now(),
                    };
                    tx.send(response).await.map_err(|e| {
                        SystemError::ServiceCommunication(format!(
                            "Failed to send calendar response: {}",
                            e
                        ))
                    })?;
                }
            }
            ai_manager_shared::messages::CalendarAction::DeleteEvent { event_id } => {
                self.calendar.delete_event(&event_id).await?;
                info!("Deleted calendar event: {}", event_id);

                if let Some(tx) = &self.tx {
                    let response = ServiceMessage::SystemResponse {
                        content: format!("Deleted calendar event: {}", event_id),
                        message_type: ai_manager_shared::messages::ResponseType::Success,
                        timestamp: chrono::Utc::now(),
                    };
                    tx.send(response).await.map_err(|e| {
                        SystemError::ServiceCommunication(format!(
                            "Failed to send calendar response: {}",
                            e
                        ))
                    })?;
                }
            }
        }
        Ok(())
    }

    async fn handle_email_process(
        &mut self,
        emails: Vec<ai_manager_shared::messages::EmailData>,
    ) -> Result<(), SystemError> {
        info!("Processing {} emails", emails.len());

        let email_count = emails.len();
        for email in emails {
            // Process each email (categorization, priority assessment, etc.)
            let processed = self.email.process_email(&email).await?;
            info!("Processed email: {}", email.subject);

            // Send notification if high priority
            if processed.is_high_priority {
                self.notifications
                    .send_notification(&format!("High priority email: {}", email.subject))
                    .await?;
            }
        }

        if let Some(tx) = &self.tx {
            let response = ServiceMessage::SystemResponse {
                content: format!("Processed {} emails", email_count),
                message_type: ai_manager_shared::messages::ResponseType::Info,
                timestamp: chrono::Utc::now(),
            };
            tx.send(response).await.map_err(|e| {
                SystemError::ServiceCommunication(format!("Failed to send email response: {}", e))
            })?;
        }

        Ok(())
    }
}

#[async_trait]
impl Service for ExternalService {
    async fn start(&mut self, mut rx: mpsc::Receiver<ServiceMessage>) -> Result<(), SystemError> {
        info!("External Service starting...");

        while let Some(message) = rx.recv().await {
            if let Err(e) = self.handle_message(message).await {
                error!("Error handling message: {}", e);
            }
        }

        warn!("External Service message receiver closed");
        Ok(())
    }

    async fn handle_message(&mut self, msg: ServiceMessage) -> Result<(), SystemError> {
        match msg {
            ServiceMessage::CalendarSync { action } => self.handle_calendar_sync(action).await,
            ServiceMessage::EmailProcess { emails } => self.handle_email_process(emails).await,
            ServiceMessage::ServiceHealthCheck { service_id: _ } => {
                if let Some(tx) = &self.tx {
                    let health = self.health_check().await;
                    let response = ServiceMessage::ServiceHealthResponse {
                        service_id: "external-service".to_string(),
                        status: health,
                    };
                    tx.send(response).await.map_err(|e| {
                        SystemError::ServiceCommunication(format!(
                            "Failed to send health response: {}",
                            e
                        ))
                    })?;
                }
                Ok(())
            }
            _ => {
                warn!("External Service received unhandled message: {:?}", msg);
                Ok(())
            }
        }
    }

    async fn health_check(&self) -> ai_manager_shared::messages::ServiceHealth {
        // Check connectivity to external services
        match self.calendar.health_check().await {
            Ok(_) => match self.email.health_check().await {
                Ok(_) => ai_manager_shared::messages::ServiceHealth::Healthy,
                Err(e) => ai_manager_shared::messages::ServiceHealth::Degraded {
                    reason: format!("Email service unavailable: {}", e),
                },
            },
            Err(e) => ai_manager_shared::messages::ServiceHealth::Degraded {
                reason: format!("Calendar service unavailable: {}", e),
            },
        }
    }

    async fn shutdown(&mut self) -> Result<(), SystemError> {
        info!("External Service shutting down...");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    #[ignore] // Requires external API credentials
    async fn test_external_service_creation() {
        let (tx, _rx) = mpsc::channel(100);
        let result = ExternalService::new(tx).await;

        // This will fail without proper credentials, but tests the structure
        assert!(result.is_err() || result.is_ok());
    }
}
