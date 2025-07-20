use crate::connection::DatabaseConnection;
use crate::models::UserProfile;
use ai_manager_shared::errors::SystemError;
use chrono::Utc;
use std::sync::Arc;

pub struct ConversationRepository {
    connection: Arc<dyn DatabaseConnection>,
}

impl ConversationRepository {
    pub fn new(connection: Arc<dyn DatabaseConnection>) -> Self {
        Self { connection }
    }

    pub async fn store_conversation(
        &self,
        user_id: &str,
        messages: &[ai_manager_shared::messages::Message],
    ) -> Result<(), SystemError> {
        let messages_json = serde_json::to_string(messages)
            .map_err(|e| SystemError::Database(format!("Failed to serialize messages: {}", e)))?;

        let now = Utc::now().to_rfc3339();

        // Check if conversation exists for this user
        let existing_query = format!(
            "SELECT id FROM conversations WHERE user_id = '{}' ORDER BY updated_at DESC LIMIT 1",
            user_id
        );

        let existing = self.connection.fetch_one_json(&existing_query).await?;

        if let Some(row) = existing {
            // Update existing conversation
            let conversation_id = row.get("id").and_then(|v| v.as_str()).ok_or_else(|| {
                SystemError::Database("Failed to get conversation ID".to_string())
            })?;

            let update_query = format!(
                "UPDATE conversations SET messages = '{}', updated_at = '{}' WHERE id = {}",
                messages_json.replace('\'', "''"), // Escape single quotes
                now,
                conversation_id
            );
            self.connection.execute(&update_query).await?;
        } else {
            // Create new conversation
            let insert_query = format!(
                "INSERT INTO conversations (user_id, messages, created_at, updated_at) VALUES ('{}', '{}', '{}', '{}')",
                user_id,
                messages_json.replace('\'', "''"), // Escape single quotes
                now,
                now
            );
            self.connection.execute(&insert_query).await?;
        }

        Ok(())
    }

    pub async fn get_conversation_history(
        &self,
        user_id: &str,
        limit: Option<i32>,
    ) -> Result<Vec<ai_manager_shared::messages::Message>, SystemError> {
        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
        let query = format!(
            "SELECT messages FROM conversations WHERE user_id = '{}' ORDER BY updated_at DESC{}",
            user_id, limit_clause
        );

        let rows = self.connection.fetch_all_json(&query).await?;

        let mut all_messages = Vec::new();

        for row in rows {
            if let Some(messages_str) = row.get("messages").and_then(|v| v.as_str()) {
                let messages: Vec<ai_manager_shared::messages::Message> =
                    serde_json::from_str(messages_str).map_err(|e| {
                        SystemError::Database(format!("Failed to deserialize messages: {}", e))
                    })?;
                all_messages.extend(messages);
            }
        }

        Ok(all_messages)
    }
}

pub struct UserProfileRepository {
    connection: Arc<dyn DatabaseConnection>,
}

impl UserProfileRepository {
    pub fn new(connection: Arc<dyn DatabaseConnection>) -> Self {
        Self { connection }
    }

    pub async fn get_profile(
        &self,
        user_id: &str,
    ) -> Result<Option<ai_manager_shared::messages::UserProfile>, SystemError> {
        let query = format!("SELECT * FROM user_profiles WHERE id = '{}'", user_id);

        let row = self.connection.fetch_one_json(&query).await?;

        if let Some(row) = row {
            let profile = UserProfile {
                id: row
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| SystemError::Database("Missing id field".to_string()))?
                    .to_string(),
                name: row
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                email: row
                    .get("email")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                preferences: row
                    .get("preferences")
                    .and_then(|v| v.as_str())
                    .unwrap_or("{}")
                    .to_string(),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    row.get("created_at")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            SystemError::Database("Missing created_at field".to_string())
                        })?,
                )
                .map_err(|e| SystemError::Database(format!("Invalid created_at format: {}", e)))?
                .with_timezone(&Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(
                    row.get("updated_at")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            SystemError::Database("Missing updated_at field".to_string())
                        })?,
                )
                .map_err(|e| SystemError::Database(format!("Invalid updated_at format: {}", e)))?
                .with_timezone(&Utc),
            };

            Ok(Some(profile.into()))
        } else {
            Ok(None)
        }
    }

    pub async fn create_profile(
        &self,
        profile: &ai_manager_shared::messages::UserProfile,
    ) -> Result<(), SystemError> {
        let preferences_json = serde_json::to_string(&profile.preferences).map_err(|e| {
            SystemError::Database(format!("Failed to serialize preferences: {}", e))
        })?;

        let query = format!(
            "INSERT INTO user_profiles (id, name, preferences, created_at, updated_at) VALUES ('{}', '{}', '{}', '{}', '{}')",
            profile.id,
            profile.name.as_deref().unwrap_or(""),
            preferences_json.replace('\'', "''"), // Escape single quotes
            profile.created_at.to_rfc3339(),
            profile.updated_at.to_rfc3339()
        );

        self.connection.execute(&query).await?;
        Ok(())
    }

    pub async fn update_profile(
        &self,
        profile: &ai_manager_shared::messages::UserProfile,
    ) -> Result<(), SystemError> {
        let preferences_json = serde_json::to_string(&profile.preferences).map_err(|e| {
            SystemError::Database(format!("Failed to serialize preferences: {}", e))
        })?;

        let query = format!(
            "UPDATE user_profiles SET name = '{}', preferences = '{}', updated_at = '{}' WHERE id = '{}'",
            profile.name.as_deref().unwrap_or(""),
            preferences_json.replace('\'', "''"), // Escape single quotes
            profile.updated_at.to_rfc3339(),
            profile.id
        );

        self.connection.execute(&query).await?;
        Ok(())
    }

    pub async fn delete_profile(&self, user_id: &str) -> Result<(), SystemError> {
        let query = format!("DELETE FROM user_profiles WHERE id = '{}'", user_id);
        self.connection.execute(&query).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::{create_connection, DatabaseType};
    use crate::migrations::run_migrations;
    use ai_manager_shared::messages::{Message, MessageRole, UserProfile};
    use chrono::Utc;
    use uuid::Uuid;

    async fn setup_test_db() -> Arc<dyn DatabaseConnection> {
        let connection = create_connection(DatabaseType::SQLite, ":memory:")
            .await
            .expect("Failed to create connection");

        run_migrations(&*connection)
            .await
            .expect("Failed to run migrations");

        connection
    }

    #[tokio::test]
    async fn test_conversation_repository() {
        let connection = setup_test_db().await;
        let repo = ConversationRepository::new(connection);

        let messages = vec![
            Message {
                id: Uuid::new_v4(),
                content: "Hello".to_string(),
                timestamp: Utc::now(),
                role: MessageRole::User,
                metadata: None,
            },
            Message {
                id: Uuid::new_v4(),
                content: "Hi there!".to_string(),
                timestamp: Utc::now(),
                role: MessageRole::Assistant,
                metadata: None,
            },
        ];

        // Store conversation
        let result = repo.store_conversation("test_user", &messages).await;
        assert!(result.is_ok());

        // Retrieve conversation
        let retrieved = repo.get_conversation_history("test_user", None).await;
        assert!(retrieved.is_ok());
        let retrieved_messages = retrieved.unwrap();
        assert_eq!(retrieved_messages.len(), 2);
    }

    #[tokio::test]
    async fn test_user_profile_repository() {
        let connection = setup_test_db().await;
        let repo = UserProfileRepository::new(connection);

        let profile = UserProfile {
            id: "test_user".to_string(),
            name: Some("Test User".to_string()),
            preferences: serde_json::json!({"theme": "dark"}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Create profile
        let result = repo.create_profile(&profile).await;
        assert!(result.is_ok());

        // Get profile
        let retrieved = repo.get_profile("test_user").await;
        assert!(retrieved.is_ok());
        let retrieved_profile = retrieved.unwrap();
        assert!(retrieved_profile.is_some());
        assert_eq!(retrieved_profile.unwrap().id, "test_user");
    }
}
