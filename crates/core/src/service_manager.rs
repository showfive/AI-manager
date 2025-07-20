use crate::event_bus::EventBus;
use ai_manager_shared::{Result, ServiceId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration, Instant};
use tracing::{debug, error, info, warn};

#[derive(Debug)]
pub struct ServiceManager {
    services: Arc<RwLock<HashMap<ServiceId, ServiceInfo>>>,
    event_bus: Arc<EventBus>,
    restart_policy: RestartPolicy,
    health_monitor_handle: Option<JoinHandle<()>>,
}

#[derive(Debug)]
struct ServiceInfo {
    handle: JoinHandle<()>,
    last_health_check: Instant,
    restart_count: u32,
    status: ServiceStatus,
}

#[derive(Debug, Clone)]
pub enum ServiceStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed { error: String },
    Restarting,
}

#[derive(Debug, Clone)]
pub struct RestartPolicy {
    pub max_restart_attempts: u32,
    pub restart_delay: Duration,
    pub backoff_multiplier: f64,
    pub max_restart_delay: Duration,
}

impl Default for RestartPolicy {
    fn default() -> Self {
        Self {
            max_restart_attempts: 3,
            restart_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
            max_restart_delay: Duration::from_secs(30),
        }
    }
}

impl ServiceManager {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            event_bus,
            restart_policy: RestartPolicy::default(),
            health_monitor_handle: None,
        }
    }

    pub fn with_restart_policy(mut self, policy: RestartPolicy) -> Self {
        self.restart_policy = policy;
        self
    }

    /// Start a service with a provided task function
    pub async fn start_service<F, Fut>(&mut self, service_id: ServiceId, task: F) -> Result<()>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        info!("Starting service: {}", service_id);

        // Register with event bus
        let (_tx, _rx) = self.event_bus.register_service(service_id.clone()).await?;

        // Start the service task
        let service_id_clone = service_id.clone();
        let handle = tokio::spawn(async move {
            if let Err(e) = task().await {
                error!("Service '{}' failed: {}", service_id_clone, e);
            }
        });

        // Register service info
        let service_info = ServiceInfo {
            handle,
            last_health_check: Instant::now(),
            restart_count: 0,
            status: ServiceStatus::Starting,
        };

        {
            let mut services = self.services.write().await;
            services.insert(service_id.clone(), service_info);
        }

        info!("Service '{}' started successfully", service_id);
        Ok(())
    }

    /// Stop a specific service
    pub async fn stop_service(&mut self, service_id: &ServiceId) -> Result<()> {
        info!("Stopping service: {}", service_id);

        let service_info = {
            let mut services = self.services.write().await;
            services.remove(service_id)
        };

        if let Some(mut info) = service_info {
            info.status = ServiceStatus::Stopping;
            info.handle.abort();

            // Wait for service to stop
            if let Err(e) = info.handle.await {
                if !e.is_cancelled() {
                    error!("Error stopping service '{}': {}", service_id, e);
                }
            }
        }

        // Unregister from event bus
        self.event_bus.unregister_service(service_id).await?;

        info!("Service '{}' stopped", service_id);
        Ok(())
    }

    /// Restart a service
    pub async fn restart_service(&mut self, service_id: &ServiceId) -> Result<()> {
        info!("Restarting service: {}", service_id);

        // Update status
        {
            let mut services = self.services.write().await;
            if let Some(service_info) = services.get_mut(service_id) {
                service_info.status = ServiceStatus::Restarting;
                service_info.restart_count += 1;
            }
        }

        // Stop the service
        self.stop_service(service_id).await?;

        // Wait for restart delay
        let restart_count = {
            let services = self.services.read().await;
            services
                .get(service_id)
                .map(|s| s.restart_count)
                .unwrap_or(0)
        };

        let delay = self.calculate_restart_delay(restart_count);
        sleep(delay).await;

        // Note: In a real implementation, we'd need to store the service
        // factory/constructor to recreate the service here
        warn!(
            "Service restart not fully implemented - would restart '{}' here",
            service_id
        );

        Ok(())
    }

    /// Get the status of all services
    pub async fn get_service_statuses(&self) -> HashMap<ServiceId, ServiceStatus> {
        let services = self.services.read().await;
        services
            .iter()
            .map(|(id, info)| (id.clone(), info.status.clone()))
            .collect()
    }

    /// Get the status of a specific service
    pub async fn get_service_status(&self, service_id: &ServiceId) -> Option<ServiceStatus> {
        let services = self.services.read().await;
        services.get(service_id).map(|info| info.status.clone())
    }

    /// Start health monitoring for all services
    pub async fn start_health_monitoring(&mut self) {
        if self.health_monitor_handle.is_some() {
            warn!("Health monitoring already running");
            return;
        }

        let services = self.services.clone();
        let interval = Duration::from_secs(ai_manager_shared::HEALTH_CHECK_INTERVAL_SECONDS);

        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                debug!("Running health checks");

                let service_ids: Vec<ServiceId> = {
                    let services_read = services.read().await;
                    services_read.keys().cloned().collect()
                };

                for service_id in service_ids {
                    // In a real implementation, we'd check service health here
                    debug!("Health check for service: {}", service_id);

                    // Update last health check time
                    {
                        let mut services_write = services.write().await;
                        if let Some(service_info) = services_write.get_mut(&service_id) {
                            service_info.last_health_check = Instant::now();
                        }
                    }
                }
            }
        });

        self.health_monitor_handle = Some(handle);
        info!("Health monitoring started");
    }

    /// Stop health monitoring
    pub async fn stop_health_monitoring(&mut self) {
        if let Some(handle) = self.health_monitor_handle.take() {
            handle.abort();
            info!("Health monitoring stopped");
        }
    }

    /// Shutdown all services
    pub async fn shutdown_all(&mut self) -> Result<()> {
        info!("Shutting down all services");

        // Stop health monitoring
        self.stop_health_monitoring().await;

        // Get all service IDs
        let service_ids: Vec<ServiceId> = {
            let services = self.services.read().await;
            services.keys().cloned().collect()
        };

        // Stop all services
        for service_id in service_ids {
            if let Err(e) = self.stop_service(&service_id).await {
                error!("Error stopping service '{}': {}", service_id, e);
            }
        }

        info!("All services shut down");
        Ok(())
    }

    /// Calculate restart delay with exponential backoff
    fn calculate_restart_delay(&self, restart_count: u32) -> Duration {
        let base_delay = self.restart_policy.restart_delay.as_secs_f64();
        let multiplier = self
            .restart_policy
            .backoff_multiplier
            .powi(restart_count as i32);
        let delay_secs = base_delay * multiplier;

        let max_delay_secs = self.restart_policy.max_restart_delay.as_secs_f64();
        let final_delay_secs = delay_secs.min(max_delay_secs);

        Duration::from_secs_f64(final_delay_secs)
    }

    /// Check if a service should be restarted
    pub fn should_restart_service(&self, _service_id: &ServiceId, restart_count: u32) -> bool {
        restart_count < self.restart_policy.max_restart_attempts
    }
}

impl Drop for ServiceManager {
    fn drop(&mut self) {
        // Clean shutdown in destructor
        if let Some(handle) = self.health_monitor_handle.take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_bus::EventBus;

    #[tokio::test]
    async fn test_service_lifecycle() {
        let event_bus = Arc::new(EventBus::new());
        let mut manager = ServiceManager::new(event_bus);

        // Start service with a simple task
        let result = manager
            .start_service("test-service".to_string(), || async {
                // Simulate service running
                sleep(Duration::from_millis(100)).await;
                Ok(())
            })
            .await;
        assert!(result.is_ok());

        // Check service is running
        let status = manager
            .get_service_status(&"test-service".to_string())
            .await;
        assert!(status.is_some());

        // Stop service
        let result = manager.stop_service(&"test-service".to_string()).await;
        assert!(result.is_ok());

        // Check service is stopped
        let status = manager
            .get_service_status(&"test-service".to_string())
            .await;
        assert!(status.is_none());
    }
}
