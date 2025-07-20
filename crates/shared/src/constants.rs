// Service identifiers
pub const CORE_SERVICE_ID: &str = "core";
pub const LLM_SERVICE_ID: &str = "llm";
pub const DATA_SERVICE_ID: &str = "data";
pub const EXTERNAL_SERVICE_ID: &str = "external";
pub const UI_SERVICE_ID: &str = "ui";

// Configuration file paths
pub const DEFAULT_CONFIG_PATH: &str = "config/default.toml";
pub const USER_CONFIG_PATH: &str = "config/user.toml";

// Database constants
pub const DEFAULT_SQLITE_PATH: &str = "data/ai_manager.db";
pub const MAX_MESSAGE_HISTORY: usize = 1000;
pub const CONVERSATION_CLEANUP_INTERVAL_HOURS: u64 = 24;

// LLM provider constants
pub const DEFAULT_LLM_PROVIDER: &str = "openai";
pub const MAX_PROMPT_LENGTH: usize = 32000;
pub const DEFAULT_MAX_TOKENS: u32 = 2000;
pub const DEFAULT_TEMPERATURE: f32 = 0.7;

// HTTP timeouts (in seconds)
pub const DEFAULT_REQUEST_TIMEOUT: u64 = 30;
pub const LLM_REQUEST_TIMEOUT: u64 = 60;
pub const CALENDAR_REQUEST_TIMEOUT: u64 = 30;
pub const EMAIL_REQUEST_TIMEOUT: u64 = 30;

// Retry configuration
pub const MAX_RETRY_ATTEMPTS: u32 = 3;
pub const RETRY_DELAY_MS: u64 = 1000;
pub const BACKOFF_MULTIPLIER: f64 = 2.0;

// Health check intervals
pub const HEALTH_CHECK_INTERVAL_SECONDS: u64 = 30;
pub const SERVICE_RESTART_COOLDOWN_SECONDS: u64 = 5;

// Message processing
pub const MESSAGE_QUEUE_CAPACITY: usize = 1000;
pub const BROADCAST_CHANNEL_CAPACITY: usize = 100;

// File paths
pub const LOG_FILE_PATH: &str = "logs/ai_manager.log";
pub const CREDENTIALS_PATH: &str = "credentials";

// UI constants
pub const DEFAULT_WINDOW_WIDTH: u32 = 1200;
pub const DEFAULT_WINDOW_HEIGHT: u32 = 800;
pub const MIN_WINDOW_WIDTH: u32 = 800;
pub const MIN_WINDOW_HEIGHT: u32 = 600;
