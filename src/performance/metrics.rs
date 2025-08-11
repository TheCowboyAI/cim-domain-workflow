//! Detailed performance metrics collection and analysis

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Detailed metrics collector for workflow operations
pub struct MetricsCollector {
    /// Time-series metrics storage
    time_series: Arc<RwLock<HashMap<String, TimeSeries>>>,
    /// Histogram data for latency analysis
    histograms: Arc<RwLock<HashMap<String, LatencyHistogram>>>,
    /// Counter metrics
    counters: Arc<RwLock<HashMap<String, Counter>>>,
    /// Gauge metrics
    gauges: Arc<RwLock<HashMap<String, Gauge>>>,
    /// Collection configuration
    config: MetricsConfig,
}

/// Configuration for metrics collection
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Maximum number of data points per time series
    pub max_data_points: usize,
    /// Retention period for metrics
    pub retention_period: Duration,
    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,
    /// Histogram bucket configuration
    pub histogram_buckets: Vec<f64>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            max_data_points: 10000,
            retention_period: Duration::from_secs(3600), // 1 hour
            sampling_rate: 1.0,
            histogram_buckets: vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0
            ],
        }
    }
}

/// Time series data for a metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeries {
    /// Metric name
    pub name: String,
    /// Data points
    pub points: VecDeque<DataPoint>,
    /// Last updated timestamp
    pub last_updated: SystemTime,
}

/// Individual data point in a time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// Timestamp
    pub timestamp: SystemTime,
    /// Value
    pub value: f64,
    /// Optional labels
    pub labels: HashMap<String, String>,
}

/// Latency histogram for analyzing performance distributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyHistogram {
    /// Metric name
    pub name: String,
    /// Histogram buckets
    pub buckets: Vec<HistogramBucket>,
    /// Total count
    pub count: u64,
    /// Sum of all values
    pub sum: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
}

/// Histogram bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    /// Upper bound of the bucket
    pub upper_bound: f64,
    /// Count of values in this bucket
    pub count: u64,
}

/// Counter metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counter {
    /// Counter name
    pub name: String,
    /// Current value
    pub value: u64,
    /// Labels
    pub labels: HashMap<String, String>,
    /// Last increment timestamp
    pub last_updated: SystemTime,
}

/// Gauge metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gauge {
    /// Gauge name
    pub name: String,
    /// Current value
    pub value: f64,
    /// Labels
    pub labels: HashMap<String, String>,
    /// Last update timestamp
    pub last_updated: SystemTime,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            time_series: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Record a time series data point
    pub async fn record_time_series(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        if !self.should_sample() {
            return;
        }

        let mut series_map = self.time_series.write().await;
        let series = series_map.entry(name.to_string())
            .or_insert_with(|| TimeSeries {
                name: name.to_string(),
                points: VecDeque::new(),
                last_updated: SystemTime::now(),
            });

        let data_point = DataPoint {
            timestamp: SystemTime::now(),
            value,
            labels,
        };

        series.points.push_back(data_point);
        series.last_updated = SystemTime::now();

        // Maintain size limit
        while series.points.len() > self.config.max_data_points {
            series.points.pop_front();
        }

        // Clean old data points
        let cutoff = SystemTime::now() - self.config.retention_period;
        while let Some(front) = series.points.front() {
            if front.timestamp < cutoff {
                series.points.pop_front();
            } else {
                break;
            }
        }
    }

    /// Record a histogram value
    pub async fn record_histogram(&self, name: &str, value: f64) {
        if !self.should_sample() {
            return;
        }

        let mut histograms = self.histograms.write().await;
        let histogram = histograms.entry(name.to_string())
            .or_insert_with(|| LatencyHistogram {
                name: name.to_string(),
                buckets: self.config.histogram_buckets.iter()
                    .map(|&upper_bound| HistogramBucket { upper_bound, count: 0 })
                    .collect(),
                count: 0,
                sum: 0.0,
                min: f64::INFINITY,
                max: f64::NEG_INFINITY,
            });

        // Update histogram statistics
        histogram.count += 1;
        histogram.sum += value;
        histogram.min = histogram.min.min(value);
        histogram.max = histogram.max.max(value);

        // Update buckets
        for bucket in &mut histogram.buckets {
            if value <= bucket.upper_bound {
                bucket.count += 1;
            }
        }
    }

    /// Increment a counter
    pub async fn increment_counter(&self, name: &str, labels: HashMap<String, String>) {
        self.add_to_counter(name, 1, labels).await;
    }

    /// Add to a counter
    pub async fn add_to_counter(&self, name: &str, value: u64, labels: HashMap<String, String>) {
        let mut counters = self.counters.write().await;
        let counter = counters.entry(name.to_string())
            .or_insert_with(|| Counter {
                name: name.to_string(),
                value: 0,
                labels: labels.clone(),
                last_updated: SystemTime::now(),
            });

        counter.value += value;
        counter.last_updated = SystemTime::now();
        counter.labels = labels;
    }

    /// Set a gauge value
    pub async fn set_gauge(&self, name: &str, value: f64, labels: HashMap<String, String>) {
        let mut gauges = self.gauges.write().await;
        let gauge = gauges.entry(name.to_string())
            .or_insert_with(|| Gauge {
                name: name.to_string(),
                value: 0.0,
                labels: labels.clone(),
                last_updated: SystemTime::now(),
            });

        gauge.value = value;
        gauge.last_updated = SystemTime::now();
        gauge.labels = labels;
    }

    /// Get time series data
    pub async fn get_time_series(&self, name: &str) -> Option<TimeSeries> {
        let series_map = self.time_series.read().await;
        series_map.get(name).cloned()
    }

    /// Get histogram data
    pub async fn get_histogram(&self, name: &str) -> Option<LatencyHistogram> {
        let histograms = self.histograms.read().await;
        histograms.get(name).cloned()
    }

    /// Get counter value
    pub async fn get_counter(&self, name: &str) -> Option<Counter> {
        let counters = self.counters.read().await;
        counters.get(name).cloned()
    }

    /// Get gauge value
    pub async fn get_gauge(&self, name: &str) -> Option<Gauge> {
        let gauges = self.gauges.read().await;
        gauges.get(name).cloned()
    }

    /// Get all metrics
    pub async fn get_all_metrics(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            time_series: self.time_series.read().await.clone(),
            histograms: self.histograms.read().await.clone(),
            counters: self.counters.read().await.clone(),
            gauges: self.gauges.read().await.clone(),
            timestamp: SystemTime::now(),
        }
    }

    /// Calculate percentiles from histogram
    pub async fn calculate_percentiles(&self, histogram_name: &str, percentiles: &[f64]) -> Option<HashMap<String, f64>> {
        let histograms = self.histograms.read().await;
        let histogram = histograms.get(histogram_name)?;

        if histogram.count == 0 {
            return None;
        }

        let mut result = HashMap::new();
        
        for &percentile in percentiles {
            let target_count = (histogram.count as f64 * percentile / 100.0).ceil() as u64;
            let mut cumulative_count = 0;
            
            for bucket in &histogram.buckets {
                cumulative_count += bucket.count;
                if cumulative_count >= target_count {
                    result.insert(format!("p{}", percentile as u8), bucket.upper_bound);
                    break;
                }
            }
        }

        Some(result)
    }

    /// Calculate rate of change for time series
    pub async fn calculate_rate(&self, series_name: &str, window: Duration) -> Option<f64> {
        let series_map = self.time_series.read().await;
        let series = series_map.get(series_name)?;

        if series.points.len() < 2 {
            return None;
        }

        let now = SystemTime::now();
        let window_start = now - window;

        let recent_points: Vec<_> = series.points.iter()
            .filter(|point| point.timestamp >= window_start)
            .collect();

        if recent_points.len() < 2 {
            return None;
        }

        let first = recent_points.first()?;
        let last = recent_points.last()?;

        let value_change = last.value - first.value;
        let time_change = last.timestamp.duration_since(first.timestamp).ok()?.as_secs_f64();

        if time_change > 0.0 {
            Some(value_change / time_change)
        } else {
            None
        }
    }

    /// Check if we should sample this metric (based on sampling rate)
    fn should_sample(&self) -> bool {
        if self.config.sampling_rate >= 1.0 {
            true
        } else {
            rand::random::<f64>() < self.config.sampling_rate
        }
    }
}

/// Snapshot of all metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub time_series: HashMap<String, TimeSeries>,
    pub histograms: HashMap<String, LatencyHistogram>,
    pub counters: HashMap<String, Counter>,
    pub gauges: HashMap<String, Gauge>,
    pub timestamp: SystemTime,
}

/// Metrics analysis utilities
pub struct MetricsAnalyzer;

impl MetricsAnalyzer {
    /// Detect anomalies in time series data
    pub fn detect_anomalies(series: &TimeSeries, sensitivity: f64) -> Vec<AnomalyDetection> {
        let mut anomalies = Vec::new();
        
        if series.points.len() < 10 {
            return anomalies;
        }

        let values: Vec<f64> = series.points.iter().map(|p| p.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        
        let threshold = std_dev * sensitivity;

        for (i, point) in series.points.iter().enumerate() {
            if (point.value - mean).abs() > threshold {
                anomalies.push(AnomalyDetection {
                    timestamp: point.timestamp,
                    value: point.value,
                    expected_range: (mean - threshold, mean + threshold),
                    severity: if (point.value - mean).abs() > threshold * 2.0 {
                        AnomalySeverity::High
                    } else {
                        AnomalySeverity::Medium
                    },
                    index: i,
                });
            }
        }

        anomalies
    }

    /// Detect trends in time series data
    pub fn detect_trend(series: &TimeSeries, min_points: usize) -> Option<Trend> {
        if series.points.len() < min_points {
            return None;
        }

        let values: Vec<f64> = series.points.iter().map(|p| p.value).collect();
        let n = values.len();
        
        // Calculate linear regression
        let x_sum: f64 = (0..n).map(|i| i as f64).sum();
        let y_sum: f64 = values.iter().sum();
        let xy_sum: f64 = values.iter().enumerate()
            .map(|(i, &y)| i as f64 * y)
            .sum();
        let x_sq_sum: f64 = (0..n).map(|i| (i as f64).powi(2)).sum();

        let slope = (n as f64 * xy_sum - x_sum * y_sum) / (n as f64 * x_sq_sum - x_sum.powi(2));
        let intercept = (y_sum - slope * x_sum) / n as f64;

        let trend_type = if slope.abs() < 0.001 {
            TrendType::Stable
        } else if slope > 0.0 {
            TrendType::Increasing
        } else {
            TrendType::Decreasing
        };

        Some(Trend {
            trend_type,
            slope,
            intercept,
            correlation: Self::calculate_correlation(&values),
        })
    }

    /// Calculate correlation coefficient
    fn calculate_correlation(values: &[f64]) -> f64 {
        let n = values.len() as f64;
        let x_values: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();
        
        let x_mean = x_values.iter().sum::<f64>() / n;
        let y_mean = values.iter().sum::<f64>() / n;

        let numerator: f64 = x_values.iter().zip(values.iter())
            .map(|(&x, &y)| (x - x_mean) * (y - y_mean))
            .sum();

        let x_variance: f64 = x_values.iter()
            .map(|&x| (x - x_mean).powi(2))
            .sum();

        let y_variance: f64 = values.iter()
            .map(|&y| (y - y_mean).powi(2))
            .sum();

        let denominator = (x_variance * y_variance).sqrt();

        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetection {
    pub timestamp: SystemTime,
    pub value: f64,
    pub expected_range: (f64, f64),
    pub severity: AnomalySeverity,
    pub index: usize,
}

/// Anomaly severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
}

/// Trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trend {
    pub trend_type: TrendType,
    pub slope: f64,
    pub intercept: f64,
    pub correlation: f64,
}

/// Type of trend detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendType {
    Increasing,
    Decreasing,
    Stable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new(MetricsConfig::default());

        // Test time series
        collector.record_time_series("test_metric", 1.0, HashMap::new()).await;
        collector.record_time_series("test_metric", 2.0, HashMap::new()).await;

        let series = collector.get_time_series("test_metric").await;
        assert!(series.is_some());
        assert_eq!(series.unwrap().points.len(), 2);

        // Test histogram
        collector.record_histogram("test_latency", 0.1).await;
        collector.record_histogram("test_latency", 0.5).await;

        let histogram = collector.get_histogram("test_latency").await;
        assert!(histogram.is_some());
        assert_eq!(histogram.unwrap().count, 2);

        // Test counter
        collector.increment_counter("test_counter", HashMap::new()).await;
        collector.add_to_counter("test_counter", 5, HashMap::new()).await;

        let counter = collector.get_counter("test_counter").await;
        assert!(counter.is_some());
        assert_eq!(counter.unwrap().value, 6);

        // Test gauge
        collector.set_gauge("test_gauge", 42.0, HashMap::new()).await;

        let gauge = collector.get_gauge("test_gauge").await;
        assert!(gauge.is_some());
        assert_eq!(gauge.unwrap().value, 42.0);
    }

    #[tokio::test]
    async fn test_percentile_calculation() {
        let collector = MetricsCollector::new(MetricsConfig::default());

        // Record some histogram values
        for value in [0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0] {
            collector.record_histogram("test_latency", value).await;
        }

        let percentiles = collector.calculate_percentiles("test_latency", &[50.0, 90.0, 95.0, 99.0]).await;
        assert!(percentiles.is_some());

        let p = percentiles.unwrap();
        assert!(p.contains_key("p50"));
        assert!(p.contains_key("p90"));
        assert!(p.contains_key("p95"));
        assert!(p.contains_key("p99"));
    }
}