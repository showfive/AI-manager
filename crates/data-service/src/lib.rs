pub mod connection;
mod migrations;
mod models;
pub mod repository;

use ai_manager_shared::{errors::SystemError, messages::ServiceMessage};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

pub use connection::{DatabaseConnection, DatabaseType};
pub use models::*;
pub use repository::{ConversationRepository, UserProfileRepository};

#[async_trait]
pub trait Service {
    async fn start(&mut self, mut rx: mpsc::Receiver<ServiceMessage>) -> Result<(), SystemError>;
    async fn handle_message(&mut self, msg: ServiceMessage) -> Result<(), SystemError>;
    async fn health_check(&self) -> ai_manager_shared::messages::ServiceHealth;
    async fn shutdown(&mut self) -> Result<(), SystemError>;
}

pub struct DataService {
    connection: Arc<dyn DatabaseConnection>,
    conversation_repo: ConversationRepository,
    profile_repo: UserProfileRepository,
    tx: Option<mpsc::Sender<ServiceMessage>>,
}

impl DataService {
    pub async fn new(
        db_type: DatabaseType,
        database_url: &str,
        tx: mpsc::Sender<ServiceMessage>,
    ) -> Result<Self, SystemError> {
        let connection = connection::create_connection(db_type, database_url).await?;

        // Run migrations
        migrations::run_migrations(&*connection).await?;

        let conversation_repo = ConversationRepository::new(connection.clone());
        let profile_repo = UserProfileRepository::new(connection.clone());

        Ok(Self {
            connection,
            conversation_repo,
            profile_repo,
            tx: Some(tx),
        })
    }

    async fn handle_store_conversation(
        &mut self,
        user_id: String,
        messages: Vec<ai_manager_shared::messages::Message>,
    ) -> Result<(), SystemError> {
        self.conversation_repo
            .store_conversation(&user_id, &messages)
            .await?;
        info!("Stored conversation for user: {}", user_id);
        Ok(())
    }

    async fn handle_load_user_profile(&mut self, user_id: String) -> Result<(), SystemError> {
        let profile = self.profile_repo.get_profile(&user_id).await?;

        if let Some(tx) = &self.tx {
            let response = ServiceMessage::UserProfileResponse { profile };
            tx.send(response).await.map_err(|e| {
                SystemError::ServiceCommunication(format!("Failed to send profile response: {}", e))
            })?;
        }

        Ok(())
    }
}

#[async_trait]
impl Service for DataService {
    async fn start(&mut self, mut rx: mpsc::Receiver<ServiceMessage>) -> Result<(), SystemError> {
        info!("Data Service starting...");

        while let Some(message) = rx.recv().await {
            if let Err(e) = self.handle_message(message).await {
                error!("Error handling message: {}", e);
            }
        }

        warn!("Data Service message receiver closed");
        Ok(())
    }

    async fn handle_message(&mut self, msg: ServiceMessage) -> Result<(), SystemError> {
        match msg {
            ServiceMessage::StoreConversation { user_id, messages } => {
                self.handle_store_conversation(user_id, messages).await
            }
            ServiceMessage::LoadUserProfile { user_id } => {
                self.handle_load_user_profile(user_id).await
            }
            ServiceMessage::ServiceHealthCheck { service_id: _ } => {
                if let Some(tx) = &self.tx {
                    let health = self.health_check().await;
                    let response = ServiceMessage::ServiceHealthResponse {
                        service_id: "data-service".to_string(),
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
                warn!("Data Service received unhandled message: {:?}", msg);
                Ok(())
            }
        }
    }

    async fn health_check(&self) -> ai_manager_shared::messages::ServiceHealth {
        match self.connection.health_check().await {
            Ok(_) => ai_manager_shared::messages::ServiceHealth::Healthy,
            Err(e) => ai_manager_shared::messages::ServiceHealth::Unhealthy {
                error: e.to_string(),
            },
        }
    }

    async fn shutdown(&mut self) -> Result<(), SystemError> {
        info!("Data Service shutting down...");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_data_service_creation() {
        let (tx, _rx) = mpsc::channel(100);
        let result = DataService::new(DatabaseType::SQLite, ":memory:", tx).await;

        assert!(result.is_ok());
    }
}
