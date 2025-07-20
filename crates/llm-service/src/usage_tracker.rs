use ai_manager_shared::TokenUsage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub timestamp: DateTime<Utc>,
    pub provider: String,
    pub model: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub cost_estimate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub total_requests: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub by_provider: HashMap<String, ProviderStats>,
    pub by_model: HashMap<String, ModelStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStats {
    pub requests: u64,
    pub tokens: u64,
    pub cost: f64,
    pub average_tokens_per_request: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStats {
    pub requests: u64,
    pub tokens: u64,
    pub cost: f64,
    pub provider: String,
}

pub struct UsageTracker {
    records: Arc<RwLock<Vec<UsageRecord>>>,
    pricing: Arc<RwLock<HashMap<String, PricingInfo>>>,
}

#[derive(Debug, Clone)]
pub struct PricingInfo {
    pub prompt_price_per_1k: f64,
    pub completion_price_per_1k: f64,
}

impl UsageTracker {
    pub fn new() -> Self {
        let mut tracker = Self {
            records: Arc::new(RwLock::new(Vec::new())),
            pricing: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Set up default pricing (as of 2024 - these should be updated regularly)
        tracker.add_default_pricing();
        tracker
    }
    
    /// Record usage for a request
    pub async fn record_usage(
        &self,
        provider: &str,
        model: &str,
        usage: &TokenUsage,
    ) {
        let cost_estimate = self.calculate_cost(provider, model, usage).await;
        
        let record = UsageRecord {
            timestamp: Utc::now(),
            provider: provider.to_string(),
            model: model.to_string(),
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
            cost_estimate,
        };
        
        let mut records = self.records.write().await;
        records.push(record);
    }
    
    /// Calculate estimated cost for a request
    pub async fn calculate_cost(
        &self,
        provider: &str,
        model: &str,
        usage: &TokenUsage,
    ) -> Option<f64> {
        let pricing = self.pricing.read().await;
        let model_key = format!("{}:{}", provider, model);
        
        if let Some(pricing_info) = pricing.get(&model_key) {
            let prompt_cost = (usage.prompt_tokens as f64 / 1000.0) * pricing_info.prompt_price_per_1k;
            let completion_cost = (usage.completion_tokens as f64 / 1000.0) * pricing_info.completion_price_per_1k;
            Some(prompt_cost + completion_cost)
        } else {
            None
        }
    }
    
    /// Get usage statistics
    pub async fn get_stats(&self) -> UsageStats {
        let records = self.records.read().await;
        
        let mut stats = UsageStats {
            total_requests: 0,
            total_tokens: 0,
            total_cost: 0.0,
            by_provider: HashMap::new(),
            by_model: HashMap::new(),
        };
        
        for record in records.iter() {
            stats.total_requests += 1;
            stats.total_tokens += record.total_tokens as u64;
            if let Some(cost) = record.cost_estimate {
                stats.total_cost += cost;
            }
            
            // Update provider stats
            let provider_stats = stats.by_provider.entry(record.provider.clone()).or_insert(ProviderStats {
                requests: 0,
                tokens: 0,
                cost: 0.0,
                average_tokens_per_request: 0.0,
            });
            provider_stats.requests += 1;
            provider_stats.tokens += record.total_tokens as u64;
            if let Some(cost) = record.cost_estimate {
                provider_stats.cost += cost;
            }
            
            // Update model stats
            let model_stats = stats.by_model.entry(record.model.clone()).or_insert(ModelStats {
                requests: 0,
                tokens: 0,
                cost: 0.0,
                provider: record.provider.clone(),
            });
            model_stats.requests += 1;
            model_stats.tokens += record.total_tokens as u64;
            if let Some(cost) = record.cost_estimate {
                model_stats.cost += cost;
            }
        }
        
        // Calculate averages
        for provider_stats in stats.by_provider.values_mut() {
            if provider_stats.requests > 0 {
                provider_stats.average_tokens_per_request = provider_stats.tokens as f64 / provider_stats.requests as f64;
            }
        }
        
        stats
    }
    
    /// Get usage records within a time range
    pub async fn get_records_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<UsageRecord> {
        let records = self.records.read().await;
        records.iter()
            .filter(|record| record.timestamp >= start && record.timestamp <= end)
            .cloned()
            .collect()
    }
    
    /// Get recent usage records
    pub async fn get_recent_records(&self, limit: usize) -> Vec<UsageRecord> {
        let records = self.records.read().await;
        records.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
    
    /// Clear old records (keep only last N records)
    pub async fn cleanup_old_records(&self, keep_count: usize) {
        let mut records = self.records.write().await;
        if records.len() > keep_count {
            let drain_count = records.len() - keep_count;
            records.drain(0..drain_count);
        }
    }
    
    /// Add pricing information for a model
    pub async fn set_pricing(&self, provider: &str, model: &str, pricing: PricingInfo) {
        let mut pricing_map = self.pricing.write().await;
        let key = format!("{}:{}", provider, model);
        pricing_map.insert(key, pricing);
    }
    
    /// Export usage data as JSON
    pub async fn export_json(&self) -> serde_json::Result<String> {
        let records = self.records.read().await;
        serde_json::to_string_pretty(&*records)
    }
    
    /// Add default pricing information
    fn add_default_pricing(&mut self) {
        // Note: These prices are estimates and should be updated regularly
        // Prices are per 1000 tokens
        
        tokio::spawn({
            let pricing = self.pricing.clone();
            async move {
                let mut pricing_map = pricing.write().await;
                
                // OpenAI pricing (as of 2024)
                pricing_map.insert("openai:gpt-3.5-turbo".to_string(), PricingInfo {
                    prompt_price_per_1k: 0.0005,
                    completion_price_per_1k: 0.0015,
                });
                
                pricing_map.insert("openai:gpt-4".to_string(), PricingInfo {
                    prompt_price_per_1k: 0.03,
                    completion_price_per_1k: 0.06,
                });
                
                pricing_map.insert("openai:gpt-4-turbo".to_string(), PricingInfo {
                    prompt_price_per_1k: 0.01,
                    completion_price_per_1k: 0.03,
                });
                
                // Claude pricing (as of 2024)
                pricing_map.insert("claude:claude-3-haiku-20240307".to_string(), PricingInfo {
                    prompt_price_per_1k: 0.00025,
                    completion_price_per_1k: 0.00125,
                });
                
                pricing_map.insert("claude:claude-3-sonnet-20240229".to_string(), PricingInfo {
                    prompt_price_per_1k: 0.003,
                    completion_price_per_1k: 0.015,
                });
                
                pricing_map.insert("claude:claude-3-opus-20240229".to_string(), PricingInfo {
                    prompt_price_per_1k: 0.015,
                    completion_price_per_1k: 0.075,
                });
            }
        });
    }
}

impl Default for UsageTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    
    #[tokio::test]
    async fn test_usage_tracking() {
        let tracker = UsageTracker::new();
        
        // Record some usage
        let usage1 = TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };
        
        tracker.record_usage("openai", "gpt-3.5-turbo", &usage1).await;
        
        let usage2 = TokenUsage {
            prompt_tokens: 200,
            completion_tokens: 100,
            total_tokens: 300,
        };
        
        tracker.record_usage("claude", "claude-3-haiku-20240307", &usage2).await;
        
        // Wait a bit for async operations
        sleep(Duration::from_millis(100)).await;
        
        // Check stats
        let stats = tracker.get_stats().await;
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.total_tokens, 450);
        assert!(stats.by_provider.contains_key("openai"));
        assert!(stats.by_provider.contains_key("claude"));
    }
    
    #[tokio::test]
    async fn test_cost_calculation() {
        let tracker = UsageTracker::new();
        
        // Wait for default pricing to be set
        sleep(Duration::from_millis(100)).await;
        
        let usage = TokenUsage {
            prompt_tokens: 1000,
            completion_tokens: 500,
            total_tokens: 1500,
        };
        
        let cost = tracker.calculate_cost("openai", "gpt-3.5-turbo", &usage).await;
        assert!(cost.is_some());
        assert!(cost.unwrap() > 0.0);
    }
    
    #[tokio::test]
    async fn test_record_filtering() {
        let tracker = UsageTracker::new();
        
        let usage = TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };
        
        tracker.record_usage("openai", "gpt-3.5-turbo", &usage).await;
        
        let recent = tracker.get_recent_records(10).await;
        assert_eq!(recent.len(), 1);
        
        let now = Utc::now();
        let one_hour_ago = now - chrono::Duration::hours(1);
        let in_range = tracker.get_records_in_range(one_hour_ago, now).await;
        assert_eq!(in_range.len(), 1);
    }
}