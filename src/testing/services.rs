//! Real service connections for testing workflow operations

use super::{TestService, ServiceConfig, TestServiceStatistics};
use async_nats::{Client, Subscriber};
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use futures::StreamExt;

/// NATS service connection for testing
pub struct NatsTestService {
    /// Service configuration
    config: ServiceConfig,
    /// NATS client
    client: Option<Client>,
    /// Active subscribers
    subscribers: Arc<RwLock<HashMap<String, Subscriber>>>,
    /// Connection start time
    connect_time: Option<Instant>,
    /// Operation statistics
    operations_count: AtomicU64,
    /// Error count
    errors_count: AtomicU64,
    /// Operation times for averaging
    operation_times: Arc<RwLock<Vec<Duration>>>,
}

/// HTTP client service for testing REST endpoints
pub struct HttpTestService {
    /// Service configuration
    config: ServiceConfig,
    /// HTTP client
    client: Option<reqwest::Client>,
    /// Connection start time
    connect_time: Option<Instant>,
    /// Operation statistics
    operations_count: AtomicU64,
    /// Error count
    errors_count: AtomicU64,
    /// Operation times for averaging
    operation_times: Arc<RwLock<Vec<Duration>>>,
}

/// Test message for NATS operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMessage {
    /// Message ID
    pub id: String,
    /// Message type
    pub message_type: String,
    /// Message payload
    pub payload: serde_json::Value,
    /// Message timestamp
    pub timestamp: SystemTime,
    /// Test metadata
    pub test_metadata: HashMap<String, String>,
}

impl NatsTestService {
    /// Create a new NATS test service
    pub fn new(config: ServiceConfig) -> Self {
        Self {
            config,
            client: None,
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            connect_time: None,
            operations_count: AtomicU64::new(0),
            errors_count: AtomicU64::new(0),
            operation_times: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Publish a test message
    pub async fn publish_test_message(
        &self,
        subject: String,
        message: TestMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let start = Instant::now();
        
        if let Some(ref client) = self.client {
            let payload = serde_json::to_vec(&message)?;
            client.publish(subject.clone(), payload.into()).await?;
            
            self.operations_count.fetch_add(1, Ordering::Relaxed);
            
            let mut times = self.operation_times.write().await;
            times.push(start.elapsed());
            // Keep only last 1000 measurements
            if times.len() > 1000 {
                let drain_count = times.len() - 1000;
                times.drain(0..drain_count);
            }
            
            Ok(())
        } else {
            self.errors_count.fetch_add(1, Ordering::Relaxed);
            Err("NATS client not connected".into())
        }
    }

    /// Subscribe to a subject and return received messages
    pub async fn subscribe_for_test(
        &mut self,
        subject: String,
    ) -> Result<Subscriber, Box<dyn std::error::Error>> {
        if let Some(ref client) = self.client {
            let subscriber = client.subscribe(subject).await?;
            self.operations_count.fetch_add(1, Ordering::Relaxed);
            Ok(subscriber)
        } else {
            self.errors_count.fetch_add(1, Ordering::Relaxed);
            Err("NATS client not connected".into())
        }
    }

    /// Wait for a specific number of messages on a subject with timeout
    pub async fn wait_for_messages(
        &self,
        subject: String,
        expected_count: usize,
        timeout: Duration,
    ) -> Result<Vec<TestMessage>, Box<dyn std::error::Error>> {
        if let Some(ref client) = self.client {
            let mut subscriber = client.subscribe(subject).await?;
            
            let mut messages = Vec::new();
            let deadline = Instant::now() + timeout;
            
            while messages.len() < expected_count && Instant::now() < deadline {
                let timeout_remaining = deadline.saturating_duration_since(Instant::now());
                
                match tokio::time::timeout(timeout_remaining, subscriber.next()).await {
                    Ok(Some(msg)) => {
                        if let Ok(test_msg) = serde_json::from_slice::<TestMessage>(&msg.payload) {
                            messages.push(test_msg);
                        }
                    }
                    Ok(None) => break,
                    Err(_) => break, // Timeout
                }
            }
            
            self.operations_count.fetch_add(messages.len() as u64, Ordering::Relaxed);
            Ok(messages)
        } else {
            self.errors_count.fetch_add(1, Ordering::Relaxed);
            Err("NATS client not connected".into())
        }
    }

    /// Clean up all test subjects (remove any persistent state)
    pub async fn cleanup_test_subjects(&self, _subjects: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        // For NATS, subjects are automatically cleaned up when connections close
        // No explicit cleanup needed for subjects
        Ok(())
    }
}

#[async_trait]
impl TestService for NatsTestService {
    async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.client.is_some() {
            return Ok(()); // Already connected
        }

        let client = async_nats::connect(&self.config.endpoint).await?;
        self.client = Some(client);
        self.connect_time = Some(Instant::now());
        
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Clear stored client connection
        self.client = None;
        self.connect_time = None;
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(ref client) = self.client {
            // Try to get server info to check connection
            match client.server_info() {
                info if !info.server_name.is_empty() => Ok(true),
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    fn endpoint(&self) -> String {
        self.config.endpoint.clone()
    }

    async fn statistics(&self) -> TestServiceStatistics {
        let operations_count = self.operations_count.load(Ordering::Relaxed);
        let errors_count = self.errors_count.load(Ordering::Relaxed);
        
        let avg_operation_time = {
            let times = self.operation_times.read().await;
            if times.is_empty() {
                Duration::from_nanos(0)
            } else {
                let total: Duration = times.iter().sum();
                total / times.len() as u32
            }
        };

        let uptime = self.connect_time
            .map(|start| start.elapsed())
            .unwrap_or(Duration::from_nanos(0));

        let mut custom_metrics = HashMap::new();
        custom_metrics.insert("active_subscriptions".to_string(), 
            serde_json::Value::Number((self.subscribers.read().await.len() as u64).into()));
        
        TestServiceStatistics {
            operations_count,
            errors_count,
            avg_operation_time,
            uptime,
            custom_metrics,
        }
    }

    async fn reset(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Unsubscribe from all subjects
        let mut subscribers = self.subscribers.write().await;
        for (_, mut subscriber) in subscribers.drain() {
            let _ = subscriber.unsubscribe().await;
        }
        drop(subscribers);

        // Reset statistics
        self.operations_count.store(0, Ordering::Relaxed);
        self.errors_count.store(0, Ordering::Relaxed);
        
        let mut times = self.operation_times.write().await;
        times.clear();
        
        Ok(())
    }
}

impl HttpTestService {
    /// Create a new HTTP test service
    pub fn new(config: ServiceConfig) -> Self {
        Self {
            config,
            client: None,
            connect_time: None,
            operations_count: AtomicU64::new(0),
            errors_count: AtomicU64::new(0),
            operation_times: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Make a GET request
    pub async fn get(&self, path: &str) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
        let start = Instant::now();
        
        if let Some(ref client) = self.client {
            let url = format!("{}{}", self.config.endpoint, path);
            let response = client.get(&url)
                .timeout(self.config.request_timeout)
                .send()
                .await?;
            
            self.operations_count.fetch_add(1, Ordering::Relaxed);
            
            let mut times = self.operation_times.write().await;
            times.push(start.elapsed());
            if times.len() > 1000 {
                let drain_count = times.len() - 1000;
                times.drain(0..drain_count);
            }
            
            Ok(response)
        } else {
            self.errors_count.fetch_add(1, Ordering::Relaxed);
            Err("HTTP client not initialized".into())
        }
    }

    /// Make a POST request
    pub async fn post<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
        let start = Instant::now();
        
        if let Some(ref client) = self.client {
            let url = format!("{}{}", self.config.endpoint, path);
            let response = client.post(&url)
                .json(body)
                .timeout(self.config.request_timeout)
                .send()
                .await?;
            
            self.operations_count.fetch_add(1, Ordering::Relaxed);
            
            let mut times = self.operation_times.write().await;
            times.push(start.elapsed());
            if times.len() > 1000 {
                let drain_count = times.len() - 1000;
                times.drain(0..drain_count);
            }
            
            Ok(response)
        } else {
            self.errors_count.fetch_add(1, Ordering::Relaxed);
            Err("HTTP client not initialized".into())
        }
    }
}

#[async_trait]
impl TestService for HttpTestService {
    async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.client.is_some() {
            return Ok(());
        }

        let client = reqwest::Client::builder()
            .timeout(self.config.connection_timeout)
            .build()?;
        
        self.client = Some(client);
        self.connect_time = Some(Instant::now());
        
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.client = None;
        self.connect_time = None;
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(ref client) = self.client {
            // Try a simple GET request to check connectivity
            let url = format!("{}/health", self.config.endpoint);
            match client.get(&url)
                .timeout(Duration::from_secs(5))
                .send()
                .await 
            {
                Ok(response) => Ok(response.status().is_success()),
                Err(_) => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    fn endpoint(&self) -> String {
        self.config.endpoint.clone()
    }

    async fn statistics(&self) -> TestServiceStatistics {
        let operations_count = self.operations_count.load(Ordering::Relaxed);
        let errors_count = self.errors_count.load(Ordering::Relaxed);
        
        let avg_operation_time = {
            let times = self.operation_times.read().await;
            if times.is_empty() {
                Duration::from_nanos(0)
            } else {
                let total: Duration = times.iter().sum();
                total / times.len() as u32
            }
        };

        let uptime = self.connect_time
            .map(|start| start.elapsed())
            .unwrap_or(Duration::from_nanos(0));

        TestServiceStatistics {
            operations_count,
            errors_count,
            avg_operation_time,
            uptime,
            custom_metrics: HashMap::new(),
        }
    }

    async fn reset(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.operations_count.store(0, Ordering::Relaxed);
        self.errors_count.store(0, Ordering::Relaxed);
        
        let mut times = self.operation_times.write().await;
        times.clear();
        
        Ok(())
    }
}

/// Test service factory for creating service instances
pub struct TestServiceFactory;

impl TestServiceFactory {
    /// Create a test service based on configuration
    pub fn create_service(config: ServiceConfig) -> Result<Box<dyn TestService>, Box<dyn std::error::Error>> {
        match config.service_type {
            crate::testing::ServiceType::Nats => {
                Ok(Box::new(NatsTestService::new(config)))
            }
            crate::testing::ServiceType::HttpRest => {
                Ok(Box::new(HttpTestService::new(config)))
            }
            crate::testing::ServiceType::Database => {
                Err("Database test service not implemented yet".into())
            }
            crate::testing::ServiceType::Custom(ref service_type) => {
                Err(format!("Custom service type '{}' not implemented", service_type).into())
            }
        }
    }

    /// Create default NATS test service
    pub fn create_nats_service(endpoint: Option<&str>) -> Box<dyn TestService> {
        let config = ServiceConfig {
            name: "nats".to_string(),
            service_type: crate::testing::ServiceType::Nats,
            endpoint: endpoint.unwrap_or("localhost:4222").to_string(),
            connection_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            config_params: HashMap::new(),
        };
        
        Box::new(NatsTestService::new(config))
    }

    /// Create default HTTP test service
    pub fn create_http_service(endpoint: &str) -> Box<dyn TestService> {
        let config = ServiceConfig {
            name: "http".to_string(),
            service_type: crate::testing::ServiceType::HttpRest,
            endpoint: endpoint.to_string(),
            connection_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            config_params: HashMap::new(),
        };
        
        Box::new(HttpTestService::new(config))
    }
}

/// Helper functions for common test scenarios
pub mod helpers {
    use super::*;

    /// Wait for NATS to be available
    pub async fn wait_for_nats(endpoint: &str, max_attempts: usize) -> Result<(), Box<dyn std::error::Error>> {
        for attempt in 1..=max_attempts {
            match async_nats::connect(endpoint).await {
                Ok(client) => {
                    drop(client);
                    return Ok(());
                }
                Err(e) if attempt == max_attempts => return Err(e.into()),
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    continue;
                }
            }
        }
        Err("Failed to connect to NATS after maximum attempts".into())
    }

    /// Clean up NATS test subjects
    pub async fn cleanup_nats_subjects(
        endpoint: &str,
        subjects: &[&str],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = async_nats::connect(endpoint).await?;
        
        // For each subject, we can't directly delete messages,
        // but we can drain any remaining messages
        for &subject in subjects {
            if let Ok(mut subscriber) = client.subscribe(subject.to_string()).await {
                // Drain messages with a short timeout
                let deadline = Instant::now() + Duration::from_millis(100);
                while Instant::now() < deadline {
                    match tokio::time::timeout(Duration::from_millis(10), subscriber.next()).await {
                        Ok(Some(_)) => continue,
                        _ => break,
                    }
                }
                let _ = subscriber.unsubscribe().await;
            }
        }
        
        drop(client);
        Ok(())
    }

    /// Create a test message with timestamp and metadata
    pub fn create_test_message(
        message_type: &str,
        payload: serde_json::Value,
        test_id: &str,
    ) -> TestMessage {
        let mut metadata = HashMap::new();
        metadata.insert("test_id".to_string(), test_id.to_string());
        metadata.insert("created_at".to_string(), 
            chrono::Utc::now().to_rfc3339());

        TestMessage {
            id: uuid::Uuid::new_v4().to_string(),
            message_type: message_type.to_string(),
            payload,
            timestamp: SystemTime::now(),
            test_metadata: metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::ServiceType;

    #[tokio::test]
    async fn test_nats_service_creation() {
        let config = ServiceConfig {
            name: "test_nats".to_string(),
            service_type: ServiceType::Nats,
            endpoint: "localhost:4222".to_string(),
            connection_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(10),
            config_params: HashMap::new(),
        };

        let service = NatsTestService::new(config);
        assert_eq!(service.endpoint(), "localhost:4222");
    }

    #[tokio::test]
    async fn test_service_factory() {
        let config = ServiceConfig {
            name: "test_nats".to_string(),
            service_type: ServiceType::Nats,
            endpoint: "localhost:4222".to_string(),
            connection_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(10),
            config_params: HashMap::new(),
        };

        let service = TestServiceFactory::create_service(config);
        assert!(service.is_ok());
    }

    #[test]
    fn test_message_creation() {
        let payload = serde_json::json!({"test": "data"});
        let message = helpers::create_test_message("test_message", payload, "test_case_1");
        
        assert_eq!(message.message_type, "test_message");
        assert!(message.test_metadata.contains_key("test_id"));
        assert_eq!(message.test_metadata["test_id"], "test_case_1");
    }

    // Note: Integration tests that require actual NATS connection
    // should be run separately with NATS available
}