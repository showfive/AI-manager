use ai_manager_shared::{AppConfig, SystemError, Result};
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::Path;
use tracing::{info, warn, debug};

const DEFAULT_CONFIG_FILE: &str = "config/default.toml";
const USER_CONFIG_FILE: &str = "config/user.toml";
const ENV_PREFIX: &str = "AI_MANAGER";

pub struct ConfigManager {
    config: Config,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Result<Self> {
        let mut config = Config::builder();
        
        // Load default configuration
        if Path::new(DEFAULT_CONFIG_FILE).exists() {
            debug!("Loading default config from: {}", DEFAULT_CONFIG_FILE);
            config = config.add_source(File::with_name(DEFAULT_CONFIG_FILE));
        } else {
            warn!("Default config file not found: {}", DEFAULT_CONFIG_FILE);
        }
        
        // Load user configuration (optional)
        if Path::new(USER_CONFIG_FILE).exists() {
            debug!("Loading user config from: {}", USER_CONFIG_FILE);
            config = config.add_source(File::with_name(USER_CONFIG_FILE));
        } else {
            info!("User config file not found: {} (this is optional)", USER_CONFIG_FILE);
        }
        
        // Load environment variables
        config = config.add_source(
            Environment::with_prefix(ENV_PREFIX)
                .prefix_separator("_")
                .separator("__")
        );
        
        let config = config.build()
            .map_err(|e| SystemError::Configuration(format!("Failed to build config: {}", e)))?;
        
        Ok(Self { config })
    }
    
    /// Load configuration from a specific file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(SystemError::Configuration(
                format!("Config file not found: {}", path.display())
            ));
        }
        
        let config = Config::builder()
            .add_source(File::from(path))
            .add_source(
                Environment::with_prefix(ENV_PREFIX)
                    .prefix_separator("_")
                    .separator("__")
            )
            .build()
            .map_err(|e| SystemError::Configuration(format!("Failed to load config: {}", e)))?;
        
        Ok(Self { config })
    }
    
    /// Get the full application configuration
    pub fn get_app_config(&self) -> Result<AppConfig> {
        self.config.clone().try_deserialize()
            .map_err(|e| SystemError::Configuration(format!("Failed to deserialize config: {}", e)))
    }
    
    /// Get a specific configuration value
    pub fn get<T>(&self, key: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.config.get(key)
            .map_err(|e| SystemError::Configuration(format!("Failed to get config value '{}': {}", key, e)))
    }
    
    /// Get a configuration value with a default
    pub fn get_or_default<T>(&self, key: &str, default: T) -> T
    where
        T: for<'de> Deserialize<'de>,
    {
        self.config.get(key).unwrap_or(default)
    }
    
    /// Check if a configuration key exists
    pub fn has_key(&self, key: &str) -> bool {
        self.config.get::<Option<serde_json::Value>>(key).is_ok()
    }
    
    /// Get database connection string
    pub fn get_database_url(&self) -> Result<String> {
        self.get("database.connection_string")
            .or_else(|_| {
                warn!("Database connection string not configured, using default SQLite");
                Ok("sqlite:data/ai_manager.db".to_string())
            })
    }
    
    /// Get LLM API key for a provider
    pub fn get_llm_api_key(&self, provider: &str) -> Result<String> {
        let key = format!("llm.providers.{}.api_key", provider);
        self.get(&key)
            .map_err(|_| SystemError::Configuration(
                format!("LLM API key not found for provider: {}", provider)
            ))
    }
    
    /// Get LLM configuration for a provider
    pub fn get_llm_config(&self, provider: &str) -> Result<ai_manager_shared::LLMProviderConfig> {
        let key = format!("llm.providers.{}", provider);
        self.get(&key)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        let _config = self.get_app_config()?;
        
        // Validate required fields
        if !self.has_key("llm.default_provider") {
            return Err(SystemError::Configuration(
                "Missing required config: llm.default_provider".to_string()
            ));
        }
        
        // Validate database configuration
        let _db_url = self.get_database_url()?;
        
        info!("Configuration validation passed");
        Ok(())
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default config manager")
    }
}

/// Create a default configuration template
pub fn create_default_config() -> AppConfig {
    use ai_manager_shared::*;
    use std::collections::HashMap;
    
    let mut llm_providers = HashMap::new();
    llm_providers.insert("openai".to_string(), LLMProviderConfig {
        api_key: "your-openai-api-key".to_string(),
        base_url: None,
        model: "gpt-3.5-turbo".to_string(),
        max_tokens: Some(2000),
        temperature: Some(0.7),
    });
    
    AppConfig {
        llm: LLMConfig {
            default_provider: "openai".to_string(),
            providers: llm_providers,
        },
        database: DatabaseConfig {
            database_type: DatabaseType::SQLite,
            connection_string: "sqlite:data/ai_manager.db".to_string(),
            max_connections: Some(10),
            enable_logging: false,
        },
        external_services: ExternalServicesConfig {
            google_calendar: None,
            email: None,
            notifications: NotificationConfig {
                enable_desktop: true,
                enable_sound: true,
            },
        },
        ui: UIConfig {
            theme: "dark".to_string(),
            window_size: WindowSize {
                width: DEFAULT_WINDOW_WIDTH,
                height: DEFAULT_WINDOW_HEIGHT,
            },
            enable_system_tray: true,
        },
        logging: LoggingConfig {
            level: "info".to_string(),
            file_logging: true,
            log_file_path: Some("logs/ai_manager.log".to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    
    #[test]
    fn test_config_creation() {
        let config = create_default_config();
        assert_eq!(config.llm.default_provider, "openai");
        assert!(config.llm.providers.contains_key("openai"));
    }
    
    #[test]
    fn test_config_from_file() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test_config.toml");
        
        let config_content = r#"
[llm]
default_provider = "openai"

[llm.providers.openai]
api_key = "test-key"
model = "gpt-4"

[database]
connection_string = "sqlite::memory:"
        "#;
        
        fs::write(&config_path, config_content).unwrap();
        
        let config_manager = ConfigManager::from_file(&config_path).unwrap();
        let provider: String = config_manager.get("llm.default_provider").unwrap();
        assert_eq!(provider, "openai");
        
        let api_key: String = config_manager.get("llm.providers.openai.api_key").unwrap();
        assert_eq!(api_key, "test-key");
    }
}