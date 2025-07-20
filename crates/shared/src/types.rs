use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ServiceId = String;
pub type UserId = String;
pub type MessageId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub llm: LLMConfig,
    pub database: DatabaseConfig,
    pub external_services: ExternalServicesConfig,
    pub ui: UIConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub default_provider: String,
    pub providers: HashMap<String, LLMProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMProviderConfig {
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub database_type: DatabaseType,
    pub connection_string: String,
    pub max_connections: Option<u32>,
    pub enable_logging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseType {
    SQLite,
    PostgreSQL,
    External { provider: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalServicesConfig {
    pub google_calendar: Option<GoogleCalendarConfig>,
    pub email: Option<EmailConfig>,
    pub notifications: NotificationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleCalendarConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub calendar_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub accounts: Vec<EmailAccountConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAccountConfig {
    pub name: String,
    pub email: String,
    pub imap_server: String,
    pub imap_port: u16,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub enable_desktop: bool,
    pub enable_sound: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub theme: String,
    pub window_size: WindowSize,
    pub enable_system_tray: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_logging: bool,
    pub log_file_path: Option<String>,
}