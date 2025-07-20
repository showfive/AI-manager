use ai_manager_core::{
    config::ConfigManager,
    event_bus::EventBus,
    handlers::{LLMResponseHandler, SystemEventHandler, UserInputHandler},
    service_manager::{RestartPolicy, ServiceManager},
};
use ai_manager_shared::{Result, ServiceMessage, CORE_SERVICE_ID};
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{debug, error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    info!("🚀 Starting AI Manager Core Service");

    // Load configuration
    let config_manager = ConfigManager::new().map_err(|e| {
        error!("Failed to load configuration: {}", e);
        e
    })?;

    // Validate configuration
    if let Err(e) = config_manager.validate() {
        error!("Configuration validation failed: {}", e);
        return Err(e);
    }

    info!("✓ Configuration loaded and validated");

    // Create event bus
    let event_bus = Arc::new(EventBus::new());
    info!("✓ Event bus initialized");

    // Create service manager with restart policy
    let restart_policy = RestartPolicy {
        max_restart_attempts: 5,
        restart_delay: Duration::from_secs(2),
        backoff_multiplier: 1.5,
        max_restart_delay: Duration::from_secs(60),
    };

    let mut service_manager =
        ServiceManager::new(event_bus.clone()).with_restart_policy(restart_policy);

    info!("✓ Service manager initialized");

    // Start core service
    let event_bus_clone = event_bus.clone();
    let core_service_task = move || {
        let event_bus = event_bus_clone;
        async move {
            let mut core_service = CoreService::new(event_bus, config_manager);
            core_service.start().await
        }
    };

    match service_manager
        .start_service(CORE_SERVICE_ID.to_string(), core_service_task)
        .await
    {
        Ok(_) => info!("✓ Core service started successfully"),
        Err(e) => {
            error!("Failed to start core service: {}", e);
            return Err(e);
        }
    }

    // Start health monitoring
    service_manager.start_health_monitoring().await;
    info!("✓ Health monitoring started");

    // Handle shutdown gracefully
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            info!("📴 Shutdown signal received");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }

    // Shutdown all services
    info!("🔄 Shutting down services...");
    service_manager.shutdown_all().await?;
    info!("✓ All services shut down successfully");

    info!("👋 AI Manager Core Service stopped");
    Ok(())
}

fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ai_manager_core=debug,ai_manager_shared=info".into()),
        )
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
}

#[allow(dead_code)]
struct CoreService {
    event_bus: Arc<EventBus>,
    config_manager: ConfigManager,
    user_input_handler: UserInputHandler,
    llm_response_handler: LLMResponseHandler,
    system_event_handler: SystemEventHandler,
}

impl CoreService {
    fn new(event_bus: Arc<EventBus>, config_manager: ConfigManager) -> Self {
        let user_input_handler = UserInputHandler::new(event_bus.clone());
        let llm_response_handler = LLMResponseHandler::new(event_bus.clone());
        let system_event_handler = SystemEventHandler::new(event_bus.clone());

        Self {
            event_bus,
            config_manager,
            user_input_handler,
            llm_response_handler,
            system_event_handler,
        }
    }

    async fn start(&mut self) -> Result<()> {
        info!("Starting core service components");

        // Register core service with event bus
        let (_tx, mut rx) = self
            .event_bus
            .register_service(CORE_SERVICE_ID.to_string())
            .await?;

        // Start system event handler
        self.system_event_handler.start().await?;
        info!("✓ System event handler started");

        // Create references to handlers
        let event_bus = self.event_bus.clone();
        let user_input_handler = UserInputHandler::new(event_bus.clone());
        let llm_response_handler = LLMResponseHandler::new(event_bus.clone());

        // Start message processing loop
        info!("📨 Core service message loop started");

        while let Some(message) = rx.recv().await {
            debug!("Core service received message: {:?}", message);

            let result = match &message {
                ServiceMessage::UserInput { .. } => {
                    user_input_handler.handle_user_input(message.clone()).await
                }
                ServiceMessage::LLMResponse { .. } => {
                    llm_response_handler
                        .handle_llm_response(message.clone())
                        .await
                }
                ServiceMessage::ServiceHealthCheck { service_id } => {
                    Self::handle_health_check(service_id, &event_bus).await
                }
                ServiceMessage::ShutdownService { service_id } => {
                    info!("Shutdown request for service: {}", service_id);
                    break; // Exit the loop to shutdown
                }
                _ => {
                    warn!("Unhandled message type in core service: {:?}", message);
                    Ok(())
                }
            };

            if let Err(e) = result {
                error!("Error processing message in core service: {}", e);
            }
        }

        info!("📪 Core service message loop ended");
        Ok(())
    }

    async fn handle_health_check(service_id: &str, event_bus: &EventBus) -> Result<()> {
        debug!("Processing health check for service: {}", service_id);

        // TODO: Implement actual health check logic
        let health_response = ServiceMessage::ServiceHealthResponse {
            service_id: service_id.to_string(),
            status: ai_manager_shared::ServiceHealth::Healthy,
        };

        event_bus.route_message(health_response, None).await
    }
}

impl Drop for CoreService {
    fn drop(&mut self) {
        info!("Core service dropping");
    }
}
