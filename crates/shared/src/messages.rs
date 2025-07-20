use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceMessage {
    // UI ↔ Core communication
    UserInput {
        content: String,
        timestamp: DateTime<Utc>,
        user_id: String,
    },
    SystemResponse {
        content: String,
        message_type: ResponseType,
        timestamp: DateTime<Utc>,
    },
    
    // Core ↔ LLM communication
    LLMRequest {
        prompt: String,
        context: Vec<String>,
        provider: String,
        request_id: Uuid,
    },
    LLMResponse {
        content: String,
        usage: TokenUsage,
        request_id: Uuid,
    },
    
    // Core ↔ External service communication
    CalendarSync {
        action: CalendarAction,
    },
    EmailProcess {
        emails: Vec<EmailData>,
    },
    
    // Core ↔ Data service communication
    StoreConversation {
        user_id: String,
        messages: Vec<Message>,
    },
    LoadUserProfile {
        user_id: String,
    },
    UserProfileResponse {
        profile: Option<UserProfile>,
    },
    
    // System management
    ServiceHealthCheck {
        service_id: String,
    },
    ServiceHealthResponse {
        service_id: String,
        status: ServiceHealth,
    },
    ShutdownService {
        service_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseType {
    Info,
    Success,
    Warning,
    Error,
    Thinking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CalendarAction {
    ListEvents {
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    },
    CreateEvent {
        title: String,
        description: Option<String>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    },
    UpdateEvent {
        event_id: String,
        title: Option<String>,
        description: Option<String>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    },
    DeleteEvent {
        event_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailData {
    pub id: String,
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub body: String,
    pub timestamp: DateTime<Utc>,
    pub is_read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub role: MessageRole,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub name: Option<String>,
    pub preferences: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceHealth {
    Healthy,
    Degraded { reason: String },
    Unhealthy { error: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    ServiceStarted { service_id: String },
    ServiceStopped { service_id: String },
    ServiceRestarted { service_id: String },
    ErrorOccurred { service_id: String, error: String },
    MessageReceived { from: String, to: String },
}