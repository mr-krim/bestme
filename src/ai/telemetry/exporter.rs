use crate::ai::{Result, AIError};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tokio::sync::RwLock;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Telemetry export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Export format
    pub format: ExportFormat,
    /// Export destination
    pub destination: ExportDestination,
    /// Export interval
    pub interval: Duration,
    /// Buffer size before export
    pub buffer_size: usize,
    /// Enable compression
    pub compress: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Prometheus,
    OpenTelemetry,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportDestination {
    File(String),
    Http(String),
    Stdout,
    Memory,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            destination: ExportDestination::File("telemetry.json".to_string()),
            interval: Duration::from_secs(60),
            buffer_size: 1000,
            compress: false,
        }
    }
}

/// Telemetry data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryData {
    pub timestamp: u64,
    pub metric_type: MetricType,
    pub name: String,
    pub value: MetricValue,
    pub labels: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricValue {
    Int(i64),
    Float(f64),
    Histogram {
        sum: f64,
        count: u64,
        buckets: Vec<(f64, u64)>,
    },
    Summary {
        sum: f64,
        count: u64,
        quantiles: Vec<(f64, f64)>,
    },
}

/// Telemetry exporter
pub struct TelemetryExporter {
    config: ExportConfig,
    buffer: Arc<RwLock<Vec<TelemetryData>>>,
}

impl TelemetryExporter {
    /// Create a new exporter
    pub fn new(config: ExportConfig) -> Self {
        Self {
            config,
            buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Add data to the buffer
    pub async fn record(&self, data: TelemetryData) -> Result<()> {
        let mut buffer = self.buffer.write().await;
        buffer.push(data);
        
        // Export if buffer is full
        if buffer.len() >= self.config.buffer_size {
            drop(buffer);
            self.export().await?;
        }
        
        Ok(())
    }
    
    /// Force export of buffered data
    pub async fn export(&self) -> Result<()> {
        let mut buffer = self.buffer.write().await;
        if buffer.is_empty() {
            return Ok(());
        }
        
        let data = std::mem::take(&mut *buffer);
        drop(buffer);
        
        match &self.config.destination {
            ExportDestination::File(path) => self.export_to_file(&data, path).await?,
            ExportDestination::Http(url) => self.export_to_http(&data, url).await?,
            ExportDestination::Stdout => self.export_to_stdout(&data).await?,
            ExportDestination::Memory => {
                // Data is already in memory, nothing to do
            }
        }
        
        Ok(())
    }
    
    /// Export to file
    async fn export_to_file(&self, data: &[TelemetryData], path: &str) -> Result<()> {
        let content = match self.config.format {
            ExportFormat::Json => self.format_json(data)?,
            ExportFormat::Prometheus => self.format_prometheus(data)?,
            _ => return Err(AIError::ConfigError("Unsupported format for file export".to_string())),
        };
        
        let content = if self.config.compress {
            self.compress_data(&content)?
        } else {
            content.into_bytes()
        };
        
        // Append to file with timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let file_path = if path.contains("{}") {
            path.replace("{}", &timestamp.to_string())
        } else {
            path.to_string()
        };
        
        fs::write(&file_path, content).await
            .map_err(|e| AIError::ConfigError(format!("Failed to write telemetry: {}", e)))?;
        
        log::debug!("Exported {} telemetry records to {}", data.len(), file_path);
        
        Ok(())
    }
    
    /// Export to HTTP endpoint
    async fn export_to_http(&self, data: &[TelemetryData], url: &str) -> Result<()> {
        let body = match self.config.format {
            ExportFormat::Json => self.format_json(data)?,
            ExportFormat::OpenTelemetry => self.format_otlp(data)?,
            _ => return Err(AIError::ConfigError("Unsupported format for HTTP export".to_string())),
        };
        
        // In a real implementation, use an HTTP client like reqwest
        log::info!("Would export {} records to {}", data.len(), url);
        
        Ok(())
    }
    
    /// Export to stdout
    async fn export_to_stdout(&self, data: &[TelemetryData]) -> Result<()> {
        let output = match self.config.format {
            ExportFormat::Json => self.format_json(data)?,
            ExportFormat::Prometheus => self.format_prometheus(data)?,
            _ => self.format_json(data)?,
        };
        
        println!("{}", output);
        
        Ok(())
    }
    
    /// Format as JSON
    fn format_json(&self, data: &[TelemetryData]) -> Result<String> {
        serde_json::to_string_pretty(data)
            .map_err(|e| AIError::ConfigError(format!("Failed to serialize JSON: {}", e)))
    }
    
    /// Format as Prometheus text format
    fn format_prometheus(&self, data: &[TelemetryData]) -> Result<String> {
        let mut output = String::new();
        
        for point in data {
            // Format labels
            let labels = if point.labels.is_empty() {
                String::new()
            } else {
                let label_str = point.labels.iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("{{{}}}", label_str)
            };
            
            // Format value based on type
            match &point.value {
                MetricValue::Int(v) => {
                    output.push_str(&format!("{}{} {} {}\n", 
                        point.name, labels, v, point.timestamp));
                }
                MetricValue::Float(v) => {
                    output.push_str(&format!("{}{} {} {}\n", 
                        point.name, labels, v, point.timestamp));
                }
                MetricValue::Histogram { sum, count, buckets } => {
                    // Histogram format
                    for (le, count) in buckets {
                        output.push_str(&format!("{}_bucket{{le=\"{}\",{}}} {} {}\n", 
                            point.name, le, labels.trim_matches(|c| c == '{' || c == '}'), 
                            count, point.timestamp));
                    }
                    output.push_str(&format!("{}_sum{} {} {}\n", 
                        point.name, labels, sum, point.timestamp));
                    output.push_str(&format!("{}_count{} {} {}\n", 
                        point.name, labels, count, point.timestamp));
                }
                MetricValue::Summary { sum, count, quantiles } => {
                    // Summary format
                    for (q, v) in quantiles {
                        output.push_str(&format!("{}{{quantile=\"{}\",{}}} {} {}\n", 
                            point.name, q, labels.trim_matches(|c| c == '{' || c == '}'), 
                            v, point.timestamp));
                    }
                    output.push_str(&format!("{}_sum{} {} {}\n", 
                        point.name, labels, sum, point.timestamp));
                    output.push_str(&format!("{}_count{} {} {}\n", 
                        point.name, labels, count, point.timestamp));
                }
            }
        }
        
        Ok(output)
    }
    
    /// Format as OTLP (placeholder)
    fn format_otlp(&self, data: &[TelemetryData]) -> Result<String> {
        // In a real implementation, this would format as OTLP protobuf
        self.format_json(data)
    }
    
    /// Compress data
    fn compress_data(&self, data: &str) -> Result<Vec<u8>> {
        // In a real implementation, use flate2 or similar
        Ok(data.as_bytes().to_vec())
    }
    
    /// Start periodic export
    pub fn start_periodic_export(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.config.interval);
            
            loop {
                interval.tick().await;
                
                if let Err(e) = self.export().await {
                    log::error!("Failed to export telemetry: {}", e);
                }
            }
        })
    }
}

/// Create exporters for common scenarios
pub struct ExporterFactory;

impl ExporterFactory {
    /// Create a file exporter
    pub fn file_exporter(path: &str, format: ExportFormat) -> TelemetryExporter {
        TelemetryExporter::new(ExportConfig {
            format,
            destination: ExportDestination::File(path.to_string()),
            ..Default::default()
        })
    }
    
    /// Create an HTTP exporter
    pub fn http_exporter(url: &str, format: ExportFormat) -> TelemetryExporter {
        TelemetryExporter::new(ExportConfig {
            format,
            destination: ExportDestination::Http(url.to_string()),
            ..Default::default()
        })
    }
    
    /// Create a development exporter (stdout)
    pub fn dev_exporter() -> TelemetryExporter {
        TelemetryExporter::new(ExportConfig {
            format: ExportFormat::Json,
            destination: ExportDestination::Stdout,
            interval: Duration::from_secs(10),
            buffer_size: 10,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_telemetry_exporter() {
        let exporter = ExporterFactory::dev_exporter();
        
        let data = TelemetryData {
            timestamp: 1234567890,
            metric_type: MetricType::Counter,
            name: "test_counter".to_string(),
            value: MetricValue::Int(42),
            labels: vec![("env".to_string(), "test".to_string())],
        };
        
        exporter.record(data).await.unwrap();
    }
    
    #[test]
    fn test_prometheus_format() {
        let exporter = TelemetryExporter::new(ExportConfig::default());
        
        let data = vec![
            TelemetryData {
                timestamp: 1234567890,
                metric_type: MetricType::Counter,
                name: "requests_total".to_string(),
                value: MetricValue::Int(100),
                labels: vec![("method".to_string(), "GET".to_string())],
            },
        ];
        
        let formatted = exporter.format_prometheus(&data).unwrap();
        assert!(formatted.contains("requests_total{method=\"GET\"} 100"));
    }
}