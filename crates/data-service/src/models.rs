use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: i64,
    pub user_id: String,
    pub messages: String, // JSON serialized messages
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub preferences: String, // JSON serialized preferences
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub id: Uuid,
    pub conversation_id: i64,
    pub content: String,
    pub role: MessageRole,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<String>, // JSON serialized metadata
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl From<ai_manager_shared::messages::MessageRole> for MessageRole {
    fn from(role: ai_manager_shared::messages::MessageRole) -> Self {
        match role {
            ai_manager_shared::messages::MessageRole::User => MessageRole::User,
            ai_manager_shared::messages::MessageRole::Assistant => MessageRole::Assistant,
            ai_manager_shared::messages::MessageRole::System => MessageRole::System,
        }
    }
}

impl From<MessageRole> for ai_manager_shared::messages::MessageRole {
    fn from(role: MessageRole) -> Self {
        match role {
            MessageRole::User => ai_manager_shared::messages::MessageRole::User,
            MessageRole::Assistant => ai_manager_shared::messages::MessageRole::Assistant,
            MessageRole::System => ai_manager_shared::messages::MessageRole::System,
        }
    }
}

impl From<UserProfile> for ai_manager_shared::messages::UserProfile {
    fn from(profile: UserProfile) -> Self {
        ai_manager_shared::messages::UserProfile {
            id: profile.id,
            name: profile.name,
            preferences: serde_json::from_str(&profile.preferences)
                .unwrap_or(serde_json::Value::Null),
            created_at: profile.created_at,
            updated_at: profile.updated_at,
        }
    }
}

impl TryFrom<ai_manager_shared::messages::UserProfile> for UserProfile {
    type Error = serde_json::Error;

    fn try_from(profile: ai_manager_shared::messages::UserProfile) -> Result<Self, Self::Error> {
        Ok(UserProfile {
            id: profile.id,
            name: profile.name,
            email: None, // Not present in shared UserProfile
            preferences: serde_json::to_string(&profile.preferences)?,
            created_at: profile.created_at,
            updated_at: profile.updated_at,
        })
    }
}
