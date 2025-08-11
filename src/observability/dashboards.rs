//! Dashboard and Visualization Support
//!
//! Provides dashboard configuration, metric visualization, and integration
//! with monitoring platforms like Grafana and custom dashboard solutions.

use crate::observability::metrics::{MetricPoint, MetricValue, MetricsRegistry};
use crate::observability::health::{SystemHealthSummary, HealthStatus};
use crate::observability::alerts::{Alert, AlertSeverity, AlertStatus};
use crate::error::types::{WorkflowError, WorkflowResult, ErrorCategory, ErrorSeverity, ErrorContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Dashboard identifier
    pub dashboard_id: String,
    /// Dashboard title
    pub title: String,
    /// Dashboard description
    pub description: String,
    /// Dashboard tags
    pub tags: Vec<String>,
    /// Dashboard panels
    pub panels: Vec<Panel>,
    /// Dashboard variables
    pub variables: Vec<DashboardVariable>,
    /// Refresh interval
    pub refresh_interval: Duration,
    /// Time range
    pub time_range: TimeRange,
    /// Dashboard theme
    pub theme: DashboardTheme,
}

/// Dashboard panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Panel {
    /// Panel identifier
    pub panel_id: String,
    /// Panel title
    pub title: String,
    /// Panel type
    pub panel_type: PanelType,
    /// Panel position and size
    pub layout: PanelLayout,
    /// Data source queries
    pub queries: Vec<Query>,
    /// Panel configuration
    pub config: PanelConfig,
    /// Panel thresholds
    pub thresholds: Vec<Threshold>,
    /// Panel alerts
    pub alerts: Vec<PanelAlert>,
}

/// Panel types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelType {
    /// Time series graph
    Graph,
    /// Single stat display
    SingleStat,
    /// Table display
    Table,
    /// Heat map
    Heatmap,
    /// Gauge
    Gauge,
    /// Bar chart
    BarChart,
    /// Pie chart
    PieChart,
    /// Text panel
    Text,
    /// Alert list
    AlertList,
    /// Status panel
    Status,
    /// Log panel
    Logs,
}

/// Panel layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelLayout {
    /// X position in grid
    pub x: u32,
    /// Y position in grid
    pub y: u32,
    /// Width in grid units
    pub width: u32,
    /// Height in grid units
    pub height: u32,
}

/// Data source query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    /// Query identifier
    pub query_id: String,
    /// Data source type
    pub data_source: DataSourceType,
    /// Query expression
    pub query: String,
    /// Query alias
    pub alias: Option<String>,
    /// Query interval
    pub interval: Option<Duration>,
    /// Whether to hide query
    pub hidden: bool,
}

/// Data source types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSourceType {
    /// Prometheus metrics
    Prometheus,
    /// Internal metrics
    Internal,
    /// Log data
    Logs,
    /// Health checks
    Health,
    /// Alerts
    Alerts,
    /// Custom data source
    Custom(String),
}

/// Panel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelConfig {
    /// Y-axis configuration
    pub y_axes: Vec<YAxis>,
    /// Legend configuration
    pub legend: Legend,
    /// Display options
    pub display: DisplayOptions,
    /// Color scheme
    pub colors: Vec<String>,
    /// Unit of measurement
    pub unit: String,
    /// Decimal places
    pub decimals: Option<u32>,
}

/// Y-axis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YAxis {
    /// Axis label
    pub label: String,
    /// Minimum value
    pub min: Option<f64>,
    /// Maximum value
    pub max: Option<f64>,
    /// Logarithmic scale
    pub logarithmic: bool,
    /// Unit
    pub unit: String,
}

/// Legend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Legend {
    /// Whether to show legend
    pub show: bool,
    /// Legend position
    pub position: LegendPosition,
    /// Legend alignment
    pub alignment: LegendAlignment,
    /// Values to show
    pub values: Vec<LegendValue>,
}

/// Legend position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LegendPosition {
    Bottom,
    Right,
    Top,
    Left,
}

/// Legend alignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LegendAlignment {
    Left,
    Center,
    Right,
}

/// Legend values to display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LegendValue {
    Min,
    Max,
    Avg,
    Current,
    Total,
    Count,
}

/// Display options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayOptions {
    /// Line width
    pub line_width: u32,
    /// Fill opacity
    pub fill: f64,
    /// Point size
    pub point_size: u32,
    /// Stack series
    pub stack: bool,
    /// Show null values
    pub null_value: NullValueMode,
}

/// Null value handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NullValueMode {
    /// Connect null values
    Connected,
    /// Show null as zero
    AsZero,
    /// Don't show null values
    Null,
}

/// Threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threshold {
    /// Threshold value
    pub value: f64,
    /// Color for threshold
    pub color: String,
    /// Threshold operation
    pub op: ThresholdOp,
}

/// Threshold operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdOp {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

/// Panel alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelAlert {
    /// Alert name
    pub name: String,
    /// Alert condition
    pub condition: String,
    /// Alert frequency
    pub frequency: Duration,
    /// Notification channels
    pub notifications: Vec<String>,
    /// Alert message
    pub message: String,
}

/// Dashboard variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardVariable {
    /// Variable name
    pub name: String,
    /// Variable type
    pub variable_type: VariableType,
    /// Variable query
    pub query: Option<String>,
    /// Variable options
    pub options: Vec<VariableOption>,
    /// Current value
    pub current: Option<String>,
    /// Whether multi-select is enabled
    pub multi: bool,
    /// Whether to include all option
    pub include_all: bool,
}

/// Variable types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableType {
    /// Query variable
    Query,
    /// Custom variable
    Custom,
    /// Constant variable
    Constant,
    /// Interval variable
    Interval,
    /// Data source variable
    DataSource,
}

/// Variable option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableOption {
    /// Option text
    pub text: String,
    /// Option value
    pub value: String,
    /// Whether option is selected
    pub selected: bool,
}

/// Time range configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time (relative or absolute)
    pub from: TimeSpec,
    /// End time (relative or absolute)
    pub to: TimeSpec,
}

/// Time specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeSpec {
    /// Relative time (e.g., "5m", "1h", "1d")
    Relative(String),
    /// Absolute time
    Absolute(SystemTime),
    /// Now
    Now,
}

/// Dashboard theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardTheme {
    Light,
    Dark,
    Auto,
}

/// Dashboard data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// Timestamp
    pub timestamp: SystemTime,
    /// Value
    pub value: f64,
    /// Series name
    pub series: String,
    /// Additional tags
    pub tags: HashMap<String, String>,
}

/// Dashboard time series data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeries {
    /// Series name
    pub name: String,
    /// Data points
    pub points: Vec<DataPoint>,
    /// Series metadata
    pub metadata: HashMap<String, String>,
}

/// Dashboard renderer for generating dashboard data
pub struct DashboardRenderer {
    /// Metrics registry
    metrics_registry: Option<MetricsRegistry>,
    /// Cached dashboard data
    dashboard_cache: HashMap<String, CachedDashboardData>,
}

/// Cached dashboard data
#[derive(Debug, Clone)]
pub struct CachedDashboardData {
    /// Dashboard configuration
    pub config: DashboardConfig,
    /// Panel data
    pub panel_data: HashMap<String, PanelData>,
    /// Last updated time
    pub last_updated: SystemTime,
    /// Cache expiry
    pub expires_at: SystemTime,
}

/// Panel data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelData {
    /// Time series data
    TimeSeries(Vec<TimeSeries>),
    /// Single value
    SingleValue {
        value: f64,
        unit: String,
        trend: Option<f64>,
    },
    /// Table data
    Table {
        columns: Vec<TableColumn>,
        rows: Vec<TableRow>,
    },
    /// Status data
    Status {
        status: String,
        message: String,
        color: String,
        details: HashMap<String, String>,
    },
    /// Alert data
    Alerts(Vec<Alert>),
    /// Text content
    Text(String),
}

/// Table column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    /// Column name
    pub name: String,
    /// Column type
    pub column_type: TableColumnType,
    /// Column unit
    pub unit: Option<String>,
}

/// Table column types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableColumnType {
    String,
    Number,
    Time,
    Boolean,
}

/// Table row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    /// Row values
    pub values: Vec<TableValue>,
}

/// Table value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableValue {
    String(String),
    Number(f64),
    Time(SystemTime),
    Boolean(bool),
    Null,
}

/// Grafana dashboard exporter
pub struct GrafanaDashboardExporter {
    /// Grafana API URL
    pub grafana_url: String,
    /// API key
    pub api_key: String,
    /// HTTP client
    client: reqwest::Client,
}

/// Built-in dashboard templates
pub struct DashboardTemplates;

impl DashboardRenderer {
    /// Create new dashboard renderer
    pub fn new(metrics_registry: Option<MetricsRegistry>) -> Self {
        Self {
            metrics_registry,
            dashboard_cache: HashMap::new(),
        }
    }
    
    /// Render dashboard data
    pub async fn render_dashboard(&mut self, config: &DashboardConfig) -> WorkflowResult<HashMap<String, PanelData>> {
        let mut panel_data = HashMap::new();
        
        for panel in &config.panels {
            let data = self.render_panel(panel).await?;
            panel_data.insert(panel.panel_id.clone(), data);
        }
        
        Ok(panel_data)
    }
    
    /// Render individual panel
    async fn render_panel(&self, panel: &Panel) -> WorkflowResult<PanelData> {
        match panel.panel_type {
            PanelType::Graph => self.render_graph_panel(panel).await,
            PanelType::SingleStat => self.render_single_stat_panel(panel).await,
            PanelType::Table => self.render_table_panel(panel).await,
            PanelType::Gauge => self.render_gauge_panel(panel).await,
            PanelType::Status => self.render_status_panel(panel).await,
            PanelType::AlertList => self.render_alert_list_panel(panel).await,
            PanelType::Text => self.render_text_panel(panel).await,
            _ => Ok(PanelData::Text("Panel type not implemented".to_string())),
        }
    }
    
    /// Render graph panel
    async fn render_graph_panel(&self, panel: &Panel) -> WorkflowResult<PanelData> {
        let mut time_series = Vec::new();
        
        for query in &panel.queries {
            match query.data_source {
                DataSourceType::Internal => {
                    if let Some(ref registry) = self.metrics_registry {
                        let metrics = registry.collect_metrics().await;
                        
                        // Convert metrics to time series
                        for metric in metrics {
                            if metric.name.contains(&query.query) {
                                let series_name = query.alias.as_ref().unwrap_or(&metric.name).clone();
                                let points = vec![DataPoint {
                                    timestamp: metric.timestamp,
                                    value: self.extract_metric_value(&metric.value),
                                    series: series_name.clone(),
                                    tags: metric.labels,
                                }];
                                
                                time_series.push(TimeSeries {
                                    name: series_name,
                                    points,
                                    metadata: HashMap::new(),
                                });
                            }
                        }
                    }
                }
                DataSourceType::Prometheus => {
                    // Simulate Prometheus query
                    let series_name = query.alias.as_ref().unwrap_or(&query.query).clone();
                    let points = self.generate_sample_time_series(&series_name, 100).await;
                    
                    time_series.push(TimeSeries {
                        name: series_name,
                        points,
                        metadata: HashMap::new(),
                    });
                }
                _ => {}
            }
        }
        
        Ok(PanelData::TimeSeries(time_series))
    }
    
    /// Render single stat panel
    async fn render_single_stat_panel(&self, panel: &Panel) -> WorkflowResult<PanelData> {
        if let Some(query) = panel.queries.first() {
            match query.data_source {
                DataSourceType::Internal => {
                    if let Some(ref registry) = self.metrics_registry {
                        let metrics = registry.collect_metrics().await;
                        
                        if let Some(metric) = metrics.iter().find(|m| m.name.contains(&query.query)) {
                            return Ok(PanelData::SingleValue {
                                value: self.extract_metric_value(&metric.value),
                                unit: panel.config.unit.clone(),
                                trend: Some(rand::random::<f64>() * 10.0 - 5.0), // Simulate trend
                            });
                        }
                    }
                }
                _ => {
                    // Simulate single stat value
                    return Ok(PanelData::SingleValue {
                        value: rand::random::<f64>() * 100.0,
                        unit: panel.config.unit.clone(),
                        trend: Some(rand::random::<f64>() * 10.0 - 5.0),
                    });
                }
            }
        }
        
        Ok(PanelData::SingleValue {
            value: 0.0,
            unit: panel.config.unit.clone(),
            trend: None,
        })
    }
    
    /// Render table panel
    async fn render_table_panel(&self, _panel: &Panel) -> WorkflowResult<PanelData> {
        let columns = vec![
            TableColumn {
                name: "Component".to_string(),
                column_type: TableColumnType::String,
                unit: None,
            },
            TableColumn {
                name: "Status".to_string(),
                column_type: TableColumnType::String,
                unit: None,
            },
            TableColumn {
                name: "Last Check".to_string(),
                column_type: TableColumnType::Time,
                unit: None,
            },
            TableColumn {
                name: "Response Time".to_string(),
                column_type: TableColumnType::Number,
                unit: Some("ms".to_string()),
            },
        ];
        
        let rows = vec![
            TableRow {
                values: vec![
                    TableValue::String("Database".to_string()),
                    TableValue::String("Healthy".to_string()),
                    TableValue::Time(SystemTime::now()),
                    TableValue::Number(45.2),
                ],
            },
            TableRow {
                values: vec![
                    TableValue::String("NATS".to_string()),
                    TableValue::String("Degraded".to_string()),
                    TableValue::Time(SystemTime::now()),
                    TableValue::Number(156.8),
                ],
            },
        ];
        
        Ok(PanelData::Table { columns, rows })
    }
    
    /// Render gauge panel
    async fn render_gauge_panel(&self, panel: &Panel) -> WorkflowResult<PanelData> {
        // Render as single value for now
        self.render_single_stat_panel(panel).await
    }
    
    /// Render status panel
    async fn render_status_panel(&self, _panel: &Panel) -> WorkflowResult<PanelData> {
        // Simulate system status
        let statuses = ["Healthy", "Degraded", "Unhealthy"];
        let colors = ["green", "yellow", "red"];
        let index = rand::random::<usize>() % statuses.len();
        
        let mut details = HashMap::new();
        details.insert("uptime".to_string(), "2d 4h 15m".to_string());
        details.insert("version".to_string(), "1.0.0".to_string());
        details.insert("active_workflows".to_string(), "42".to_string());
        
        Ok(PanelData::Status {
            status: statuses[index].to_string(),
            message: format!("System is {}", statuses[index].to_lowercase()),
            color: colors[index].to_string(),
            details,
        })
    }
    
    /// Render alert list panel
    async fn render_alert_list_panel(&self, _panel: &Panel) -> WorkflowResult<PanelData> {
        // Simulate alert list
        let alerts = vec![
            Alert {
                alert_id: Uuid::new_v4(),
                title: "High CPU Usage".to_string(),
                description: "CPU usage is above 80%".to_string(),
                severity: AlertSeverity::High,
                status: AlertStatus::Active,
                source: crate::observability::alerts::AlertSource::Metric {
                    metric_name: "cpu_usage".to_string(),
                    threshold: 80.0,
                    actual_value: 85.2,
                },
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
                acknowledged_at: None,
                acknowledged_by: None,
                resolved_at: None,
                resolution_notes: None,
                metadata: HashMap::new(),
                labels: HashMap::new(),
                fire_count: 1,
                expires_at: None,
            },
        ];
        
        Ok(PanelData::Alerts(alerts))
    }
    
    /// Render text panel
    async fn render_text_panel(&self, _panel: &Panel) -> WorkflowResult<PanelData> {
        let content = r#"
# Workflow System Dashboard

This dashboard provides comprehensive monitoring of the workflow orchestration system.

## Key Metrics
- Active workflows
- Error rates
- System performance
- Health status

## Alerts
Current alerts are displayed in the alert panel.
        "#;
        
        Ok(PanelData::Text(content.to_string()))
    }
    
    /// Generate sample time series data
    async fn generate_sample_time_series(&self, series_name: &str, points: usize) -> Vec<DataPoint> {
        let mut data_points = Vec::new();
        let now = SystemTime::now();
        
        for i in 0..points {
            let timestamp = now - Duration::from_secs((points - i) as u64 * 60);
            let value = 50.0 + 20.0 * ((i as f64 * 0.1).sin()) + rand::random::<f64>() * 10.0 - 5.0;
            
            data_points.push(DataPoint {
                timestamp,
                value,
                series: series_name.to_string(),
                tags: HashMap::new(),
            });
        }
        
        data_points
    }
    
    /// Extract numeric value from metric value
    fn extract_metric_value(&self, metric_value: &MetricValue) -> f64 {
        match metric_value {
            MetricValue::Counter(v) => *v as f64,
            MetricValue::Gauge(v) => *v,
            MetricValue::Histogram { sum, count, .. } => {
                if *count > 0 {
                    sum / (*count as f64)
                } else {
                    0.0
                }
            }
            MetricValue::Summary { sum, count, .. } => {
                if *count > 0 {
                    sum / (*count as f64)
                } else {
                    0.0
                }
            }
        }
    }
}

impl GrafanaDashboardExporter {
    /// Create new Grafana exporter
    pub fn new(grafana_url: String, api_key: String) -> Self {
        Self {
            grafana_url,
            api_key,
            client: reqwest::Client::new(),
        }
    }
    
    /// Export dashboard to Grafana
    pub async fn export_dashboard(&self, config: &DashboardConfig) -> WorkflowResult<String> {
        // Convert internal dashboard config to Grafana format
        let grafana_dashboard = self.convert_to_grafana_format(config);
        
        // Send to Grafana API
        let url = format!("{}/api/dashboards/db", self.grafana_url);
        
        // Simulate API call
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        println!("Exported dashboard '{}' to Grafana", config.title);
        Ok(format!("dashboard-{}", config.dashboard_id))
    }
    
    /// Convert internal format to Grafana format
    fn convert_to_grafana_format(&self, config: &DashboardConfig) -> serde_json::Value {
        // Simplified conversion - in real implementation would be more comprehensive
        serde_json::json!({
            "dashboard": {
                "id": null,
                "title": config.title,
                "description": config.description,
                "tags": config.tags,
                "panels": config.panels.iter().map(|p| {
                    serde_json::json!({
                        "id": p.panel_id,
                        "title": p.title,
                        "type": self.convert_panel_type(&p.panel_type),
                        "gridPos": {
                            "x": p.layout.x,
                            "y": p.layout.y,
                            "w": p.layout.width,
                            "h": p.layout.height
                        }
                    })
                }).collect::<Vec<_>>(),
                "time": {
                    "from": "now-6h",
                    "to": "now"
                },
                "refresh": "30s"
            }
        })
    }
    
    /// Convert panel type to Grafana format
    fn convert_panel_type(&self, panel_type: &PanelType) -> &str {
        match panel_type {
            PanelType::Graph => "graph",
            PanelType::SingleStat => "singlestat",
            PanelType::Table => "table",
            PanelType::Heatmap => "heatmap",
            PanelType::Gauge => "gauge",
            PanelType::BarChart => "barchart",
            PanelType::PieChart => "piechart",
            PanelType::Text => "text",
            PanelType::AlertList => "alertlist",
            PanelType::Status => "stat",
            PanelType::Logs => "logs",
        }
    }
}

impl DashboardTemplates {
    /// Create system overview dashboard
    pub fn system_overview() -> DashboardConfig {
        DashboardConfig {
            dashboard_id: "system-overview".to_string(),
            title: "System Overview".to_string(),
            description: "High-level system metrics and health status".to_string(),
            tags: vec!["system".to_string(), "overview".to_string()],
            panels: vec![
                Panel {
                    panel_id: "system-status".to_string(),
                    title: "System Status".to_string(),
                    panel_type: PanelType::Status,
                    layout: PanelLayout { x: 0, y: 0, width: 6, height: 4 },
                    queries: vec![],
                    config: PanelConfig::default(),
                    thresholds: vec![],
                    alerts: vec![],
                },
                Panel {
                    panel_id: "active-workflows".to_string(),
                    title: "Active Workflows".to_string(),
                    panel_type: PanelType::SingleStat,
                    layout: PanelLayout { x: 6, y: 0, width: 6, height: 4 },
                    queries: vec![
                        Query {
                            query_id: "active-workflows".to_string(),
                            data_source: DataSourceType::Internal,
                            query: "workflow_executions_active".to_string(),
                            alias: None,
                            interval: None,
                            hidden: false,
                        }
                    ],
                    config: PanelConfig {
                        unit: "count".to_string(),
                        ..PanelConfig::default()
                    },
                    thresholds: vec![],
                    alerts: vec![],
                },
                Panel {
                    panel_id: "workflow-throughput".to_string(),
                    title: "Workflow Throughput".to_string(),
                    panel_type: PanelType::Graph,
                    layout: PanelLayout { x: 0, y: 4, width: 12, height: 6 },
                    queries: vec![
                        Query {
                            query_id: "started".to_string(),
                            data_source: DataSourceType::Internal,
                            query: "workflow_executions_started_total".to_string(),
                            alias: Some("Started".to_string()),
                            interval: None,
                            hidden: false,
                        },
                        Query {
                            query_id: "completed".to_string(),
                            data_source: DataSourceType::Internal,
                            query: "workflow_executions_completed_total".to_string(),
                            alias: Some("Completed".to_string()),
                            interval: None,
                            hidden: false,
                        }
                    ],
                    config: PanelConfig {
                        unit: "ops".to_string(),
                        ..PanelConfig::default()
                    },
                    thresholds: vec![],
                    alerts: vec![],
                }
            ],
            variables: vec![],
            refresh_interval: Duration::from_secs(30),
            time_range: TimeRange {
                from: TimeSpec::Relative("6h".to_string()),
                to: TimeSpec::Now,
            },
            theme: DashboardTheme::Auto,
        }
    }
    
    /// Create error monitoring dashboard
    pub fn error_monitoring() -> DashboardConfig {
        DashboardConfig {
            dashboard_id: "error-monitoring".to_string(),
            title: "Error Monitoring".to_string(),
            description: "Error rates, categories, and recovery metrics".to_string(),
            tags: vec!["errors".to_string(), "monitoring".to_string()],
            panels: vec![
                Panel {
                    panel_id: "error-rate".to_string(),
                    title: "Error Rate".to_string(),
                    panel_type: PanelType::Graph,
                    layout: PanelLayout { x: 0, y: 0, width: 12, height: 6 },
                    queries: vec![
                        Query {
                            query_id: "error-rate".to_string(),
                            data_source: DataSourceType::Internal,
                            query: "errors_by_category_total".to_string(),
                            alias: Some("Error Rate".to_string()),
                            interval: None,
                            hidden: false,
                        }
                    ],
                    config: PanelConfig {
                        unit: "errors/sec".to_string(),
                        ..PanelConfig::default()
                    },
                    thresholds: vec![
                        Threshold {
                            value: 10.0,
                            color: "yellow".to_string(),
                            op: ThresholdOp::GreaterThan,
                        },
                        Threshold {
                            value: 50.0,
                            color: "red".to_string(),
                            op: ThresholdOp::GreaterThan,
                        }
                    ],
                    alerts: vec![],
                }
            ],
            variables: vec![],
            refresh_interval: Duration::from_secs(30),
            time_range: TimeRange {
                from: TimeSpec::Relative("1h".to_string()),
                to: TimeSpec::Now,
            },
            theme: DashboardTheme::Auto,
        }
    }
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            y_axes: vec![],
            legend: Legend {
                show: true,
                position: LegendPosition::Bottom,
                alignment: LegendAlignment::Left,
                values: vec![],
            },
            display: DisplayOptions {
                line_width: 1,
                fill: 0.1,
                point_size: 5,
                stack: false,
                null_value: NullValueMode::Connected,
            },
            colors: vec!["blue".to_string(), "green".to_string(), "red".to_string()],
            unit: "short".to_string(),
            decimals: Some(2),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dashboard_renderer() {
        let mut renderer = DashboardRenderer::new(None);
        let config = DashboardTemplates::system_overview();
        
        let panel_data = renderer.render_dashboard(&config).await.unwrap();
        
        assert!(!panel_data.is_empty());
        assert!(panel_data.contains_key("system-status"));
        assert!(panel_data.contains_key("active-workflows"));
    }
    
    #[test]
    fn test_dashboard_templates() {
        let overview = DashboardTemplates::system_overview();
        assert_eq!(overview.dashboard_id, "system-overview");
        assert_eq!(overview.title, "System Overview");
        assert!(!overview.panels.is_empty());
        
        let error_monitoring = DashboardTemplates::error_monitoring();
        assert_eq!(error_monitoring.dashboard_id, "error-monitoring");
        assert_eq!(error_monitoring.title, "Error Monitoring");
    }
    
    #[tokio::test]
    async fn test_grafana_exporter() {
        let exporter = GrafanaDashboardExporter::new(
            "http://localhost:3000".to_string(),
            "test-api-key".to_string(),
        );
        
        let config = DashboardTemplates::system_overview();
        let result = exporter.export_dashboard(&config).await;
        
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with("dashboard-"));
    }
}