use thiserror::Error;

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("Service communication error: {0}")]
    ServiceCommunication(String),

    #[error("LLM API error: {provider} - {message}")]
    LLMApi { provider: String, message: String },

    #[error("Database error: {0}")]
    Database(String),

    #[error("External service error: {service} - {message}")]
    ExternalService { service: String, message: String },

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Request timeout")]
    Timeout,

    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: String },

    #[error("Rate limit exceeded for service: {service}")]
    RateLimitExceeded { service: String },

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl SystemError {
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            SystemError::ServiceCommunication(_)
                | SystemError::Network(_)
                | SystemError::Timeout
                | SystemError::ServiceUnavailable { .. }
                | SystemError::RateLimitExceeded { .. }
        )
    }

    pub fn should_retry(&self) -> bool {
        matches!(
            self,
            SystemError::Network(_) | SystemError::Timeout | SystemError::ServiceUnavailable { .. }
        )
    }
}

pub type Result<T> = std::result::Result<T, SystemError>;
