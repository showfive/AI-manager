use ai_manager_shared::{ServiceHealth, SystemError, Result};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub service_id: String,
    pub status: ServiceHealth,
    #[serde(skip, default = "Instant::now")]
    pub last_check: Instant,
    #[serde(with = "duration_serde")]
    pub uptime: Duration,
    pub metrics: HealthMetrics,
}

mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub message_queue_length: usize,
    pub error_count: u64,
    pub last_error: Option<String>,
}

impl Default for HealthMetrics {
    fn default() -> Self {
        Self {
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            message_queue_length: 0,
            error_count: 0,
            last_error: None,
        }
    }
}

pub struct HealthChecker {
    start_time: Instant,
    last_check: Option<Instant>,
    error_count: u64,
    last_error: Option<String>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            last_check: None,
            error_count: 0,
            last_error: None,
        }
    }
    
    /// Perform a health check
    pub async fn check_health(&mut self, service_id: &str) -> Result<HealthReport> {
        let now = Instant::now();
        self.last_check = Some(now);
        
        // Get system metrics
        let metrics = self.collect_metrics().await?;
        
        // Determine overall health status
        let status = self.determine_health_status(&metrics);
        
        Ok(HealthReport {
            service_id: service_id.to_string(),
            status,
            last_check: now,
            uptime: now.duration_since(self.start_time),
            metrics,
        })
    }
    
    /// Record an error
    pub fn record_error(&mut self, error: &str) {
        self.error_count += 1;
        self.last_error = Some(error.to_string());
    }
    
    /// Get uptime duration
    pub fn uptime(&self) -> Duration {
        Instant::now().duration_since(self.start_time)
    }
    
    /// Collect system metrics
    async fn collect_metrics(&self) -> Result<HealthMetrics> {
        // In a real implementation, we would collect actual system metrics
        // For now, we'll return mock data
        
        Ok(HealthMetrics {
            memory_usage_mb: self.get_memory_usage(),
            cpu_usage_percent: self.get_cpu_usage(),
            message_queue_length: 0, // Would be set by the service
            error_count: self.error_count,
            last_error: self.last_error.clone(),
        })
    }
    
    /// Determine health status based on metrics
    fn determine_health_status(&self, metrics: &HealthMetrics) -> ServiceHealth {
        // High error rate
        if metrics.error_count > 10 {
            return ServiceHealth::Unhealthy {
                error: format!("High error count: {}", metrics.error_count)
            };
        }
        
        // High memory usage
        if metrics.memory_usage_mb > 500.0 {
            return ServiceHealth::Degraded {
                reason: format!("High memory usage: {:.1} MB", metrics.memory_usage_mb)
            };
        }
        
        // High CPU usage
        if metrics.cpu_usage_percent > 80.0 {
            return ServiceHealth::Degraded {
                reason: format!("High CPU usage: {:.1}%", metrics.cpu_usage_percent)
            };
        }
        
        // Large message queue
        if metrics.message_queue_length > 100 {
            return ServiceHealth::Degraded {
                reason: format!("Large message queue: {}", metrics.message_queue_length)
            };
        }
        
        ServiceHealth::Healthy
    }
    
    /// Get current memory usage (mock implementation)
    fn get_memory_usage(&self) -> f64 {
        // In a real implementation, this would use system APIs
        50.0 // Mock 50MB usage
    }
    
    /// Get current CPU usage (mock implementation)
    fn get_cpu_usage(&self) -> f64 {
        // In a real implementation, this would use system APIs
        5.0 // Mock 5% CPU usage
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_health_check() {
        let mut checker = HealthChecker::new();
        let report = checker.check_health("test-service").await.unwrap();
        
        assert_eq!(report.service_id, "test-service");
        assert!(matches!(report.status, ServiceHealth::Healthy));
        assert!(report.uptime.as_millis() > 0);
    }
    
    #[tokio::test]
    async fn test_error_recording() {
        let mut checker = HealthChecker::new();
        
        // Record some errors
        for i in 0..15 {
            checker.record_error(&format!("Test error {}", i));
        }
        
        let report = checker.check_health("test-service").await.unwrap();
        
        // Should be unhealthy due to high error count
        assert!(matches!(report.status, ServiceHealth::Unhealthy { .. }));
        assert_eq!(report.metrics.error_count, 15);
    }
}