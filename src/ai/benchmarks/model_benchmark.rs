use crate::ai::{Result, AIError, EnhancementOptions};
use crate::ai::services::model_service::{ModelService, AIModel};
use super::{BenchmarkConfig, BenchmarkResult, TestSample};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use serde_json;

/// Model benchmarking system
pub struct ModelBenchmark {
    config: BenchmarkConfig,
    model_service: Arc<ModelService>,
    results: Arc<RwLock<Vec<BenchmarkResult>>>,
}

impl ModelBenchmark {
    /// Create a new benchmark instance
    pub fn new(config: BenchmarkConfig, model_service: Arc<ModelService>) -> Self {
        Self {
            config,
            model_service,
            results: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Run benchmarks for a specific model
    pub async fn benchmark_model(&self, model_id: &str) -> Result<Vec<BenchmarkResult>> {
        log::info!("Starting benchmark for model: {}", model_id);
        
        // Load the model
        let model = self.model_service.load_model(model_id).await?;
        
        // Get initial memory usage
        let _initial_memory = self.get_memory_usage();
        
        // Run warmup
        self.warmup_model(&model).await?;
        
        // Run benchmarks for each test sample
        let mut all_results = Vec::new();
        
        for sample in &self.config.test_samples {
            let result = self.benchmark_sample(&model, model_id, sample).await?;
            all_results.push(result);
        }
        
        // Store results
        {
            let mut results = self.results.write().await;
            results.extend(all_results.clone());
        }
        
        Ok(all_results)
    }
    
    /// Run benchmarks for all available models
    pub async fn benchmark_all_models(&self) -> Result<Vec<BenchmarkResult>> {
        let registry = self.model_service.get_registry();
        let models = registry.list_available_models().await;
        
        let mut all_results = Vec::new();
        
        for metadata in models {
            match self.benchmark_model(&metadata.id).await {
                Ok(results) => all_results.extend(results),
                Err(e) => log::error!("Failed to benchmark {}: {}", metadata.id, e),
            }
        }
        
        Ok(all_results)
    }
    
    /// Warmup a model before benchmarking
    async fn warmup_model(&self, model: &Arc<dyn AIModel>) -> Result<()> {
        log::info!("Warming up model...");
        
        let options = EnhancementOptions {
            correct_grammar: true,
            improve_clarity: true,
            preserve_style: true,
            detect_intent: false,
            format_markdown: false,
            confidence_threshold: 0.5,
            improve_punctuation: true,
        };
        
        for _ in 0..self.config.warmup_iterations {
            let _ = model.enhance_text("This is a warmup text.", &options).await;
        }
        
        Ok(())
    }
    
    /// Benchmark a single test sample
    async fn benchmark_sample(
        &self,
        model: &Arc<dyn AIModel>,
        model_id: &str,
        sample: &TestSample,
    ) -> Result<BenchmarkResult> {
        log::info!("Benchmarking sample: {}", sample.name);
        
        let options = EnhancementOptions {
            correct_grammar: true,
            improve_clarity: true,
            preserve_style: true,
            detect_intent: false,
            format_markdown: false,
            confidence_threshold: 0.5,
            improve_punctuation: true,
        };
        
        let mut latencies = Vec::new();
        let mut memory_readings = Vec::new();
        let mut gpu_readings = Vec::new();
        
        // Run benchmark iterations
        for i in 0..self.config.benchmark_iterations {
            let start = Instant::now();
            
            // Run inference
            match model.enhance_text(&sample.text, &options).await {
                Ok(_) => {
                    let latency = start.elapsed().as_millis() as u64;
                    latencies.push(latency);
                    
                    // Collect memory usage
                    if self.config.profile_memory {
                        memory_readings.push(self.get_memory_usage());
                    }
                    
                    // Collect GPU usage
                    if self.config.benchmark_gpu {
                        if let Some(gpu_usage) = self.get_gpu_usage().await {
                            gpu_readings.push(gpu_usage);
                        }
                    }
                    
                    if i % 10 == 0 {
                        log::debug!("Iteration {}/{}: {}ms", i + 1, self.config.benchmark_iterations, latency);
                    }
                }
                Err(e) => {
                    log::warn!("Benchmark iteration {} failed: {}", i, e);
                }
            }
        }
        
        // Calculate statistics
        latencies.sort_unstable();
        
        let avg_latency = latencies.iter().sum::<u64>() as f64 / latencies.len() as f64;
        let min_latency = *latencies.first().unwrap_or(&0);
        let max_latency = *latencies.last().unwrap_or(&0);
        
        let p50 = percentile(&latencies, 50.0);
        let p90 = percentile(&latencies, 90.0);
        let p95 = percentile(&latencies, 95.0);
        let p99 = percentile(&latencies, 99.0);
        
        // Calculate throughput (tokens per second)
        let avg_latency_sec = avg_latency / 1000.0;
        let throughput = if avg_latency_sec > 0.0 {
            sample.expected_tokens as f64 / avg_latency_sec
        } else {
            0.0
        };
        
        // Average memory usage
        let avg_memory = if !memory_readings.is_empty() {
            memory_readings.iter().sum::<u64>() / memory_readings.len() as u64
        } else {
            0
        };
        
        // Average GPU usage
        let avg_gpu = if !gpu_readings.is_empty() {
            Some(gpu_readings.iter().sum::<f32>() / gpu_readings.len() as f32)
        } else {
            None
        };
        
        Ok(BenchmarkResult {
            model_id: model_id.to_string(),
            test_name: sample.name.clone(),
            iterations: self.config.benchmark_iterations,
            avg_latency_ms: avg_latency,
            min_latency_ms: min_latency,
            max_latency_ms: max_latency,
            p50_latency_ms: p50,
            p90_latency_ms: p90,
            p95_latency_ms: p95,
            p99_latency_ms: p99,
            throughput_tps: throughput,
            memory_usage_mb: avg_memory / (1024 * 1024),
            gpu_usage_percent: avg_gpu,
            timestamp: chrono::Utc::now(),
        })
    }
    
    /// Get current memory usage in bytes
    fn get_memory_usage(&self) -> u64 {
        // Simple implementation - in production, use proper memory profiling
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb * 1024; // Convert KB to bytes
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback estimate
        1024 * 1024 * 100 // 100MB
    }
    
    /// Get GPU usage percentage
    async fn get_gpu_usage(&self) -> Option<f32> {
        // This would integrate with GPU monitoring tools
        // For now, return a placeholder
        None
    }
    
    /// Export results in the configured format
    pub async fn export_results(&self, output_path: &str) -> Result<()> {
        let results = self.results.read().await;
        
        match self.config.output_format {
            super::OutputFormat::Json => {
                let json = serde_json::to_string_pretty(&*results)
                    .map_err(|e| AIError::ConfigError(format!("Failed to serialize results: {}", e)))?;
                std::fs::write(output_path, json)
                    .map_err(|e| AIError::ConfigError(format!("Failed to write results: {}", e)))?;
            }
            super::OutputFormat::Csv => {
                self.export_csv(&results, output_path)?;
            }
            super::OutputFormat::Markdown => {
                self.export_markdown(&results, output_path)?;
            }
            super::OutputFormat::Html => {
                self.export_html(&results, output_path)?;
            }
        }
        
        log::info!("Results exported to: {}", output_path);
        Ok(())
    }
    
    /// Export results as CSV
    fn export_csv(&self, results: &[BenchmarkResult], output_path: &str) -> Result<()> {
        use std::io::Write;
        
        let mut file = std::fs::File::create(output_path)
            .map_err(|e| AIError::ConfigError(format!("Failed to create file: {}", e)))?;
        
        // Write header
        writeln!(file, "model_id,test_name,iterations,avg_latency_ms,min_latency_ms,max_latency_ms,p50,p90,p95,p99,throughput_tps,memory_mb,gpu_percent,timestamp")
            .map_err(|e| AIError::ConfigError(format!("Failed to write header: {}", e)))?;
        
        // Write data
        for result in results {
            writeln!(
                file,
                "{},{},{},{:.2},{},{},{},{},{},{},{:.2},{},{},{}",
                result.model_id,
                result.test_name,
                result.iterations,
                result.avg_latency_ms,
                result.min_latency_ms,
                result.max_latency_ms,
                result.p50_latency_ms,
                result.p90_latency_ms,
                result.p95_latency_ms,
                result.p99_latency_ms,
                result.throughput_tps,
                result.memory_usage_mb,
                result.gpu_usage_percent.map(|g| g.to_string()).unwrap_or_else(|| "N/A".to_string()),
                result.timestamp
            ).map_err(|e| AIError::ConfigError(format!("Failed to write row: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Export results as Markdown
    fn export_markdown(&self, results: &[BenchmarkResult], output_path: &str) -> Result<()> {
        use std::io::Write;
        
        let mut file = std::fs::File::create(output_path)
            .map_err(|e| AIError::ConfigError(format!("Failed to create file: {}", e)))?;
        
        writeln!(file, "# AI Model Benchmark Results\n")
            .map_err(|e| AIError::ConfigError(format!("Failed to write header: {}", e)))?;
        
        writeln!(file, "Generated: {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))
            .map_err(|e| AIError::ConfigError(format!("Failed to write timestamp: {}", e)))?;
        
        // Group by model
        let mut model_results: std::collections::HashMap<String, Vec<&BenchmarkResult>> = std::collections::HashMap::new();
        for result in results {
            model_results.entry(result.model_id.clone()).or_default().push(result);
        }
        
        for (model_id, model_results) in model_results {
            writeln!(file, "## Model: {}\n", model_id)
                .map_err(|e| AIError::ConfigError(format!("Failed to write model header: {}", e)))?;
            
            writeln!(file, "| Test | Avg Latency (ms) | P95 (ms) | P99 (ms) | Throughput (tps) | Memory (MB) |")
                .map_err(|e| AIError::ConfigError(format!("Failed to write table header: {}", e)))?;
            writeln!(file, "|------|-----------------|----------|----------|-----------------|-------------|")
                .map_err(|e| AIError::ConfigError(format!("Failed to write table separator: {}", e)))?;
            
            for result in model_results {
                writeln!(
                    file,
                    "| {} | {:.2} | {} | {} | {:.2} | {} |",
                    result.test_name,
                    result.avg_latency_ms,
                    result.p95_latency_ms,
                    result.p99_latency_ms,
                    result.throughput_tps,
                    result.memory_usage_mb
                ).map_err(|e| AIError::ConfigError(format!("Failed to write table row: {}", e)))?;
            }
            
            writeln!(file, "")
                .map_err(|e| AIError::ConfigError(format!("Failed to write newline: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Export results as HTML
    fn export_html(&self, results: &[BenchmarkResult], output_path: &str) -> Result<()> {
        use std::io::Write;
        
        let mut file = std::fs::File::create(output_path)
            .map_err(|e| AIError::ConfigError(format!("Failed to create file: {}", e)))?;
        
        // Write HTML with embedded CSS and JavaScript for interactive charts
        let html = include_str!("benchmark_template.html")
            .replace("{{RESULTS_JSON}}", &serde_json::to_string(results).unwrap_or_default())
            .replace("{{TIMESTAMP}}", &chrono::Utc::now().to_string());
        
        file.write_all(html.as_bytes())
            .map_err(|e| AIError::ConfigError(format!("Failed to write HTML: {}", e)))?;
        
        Ok(())
    }
}

/// Calculate percentile from sorted data
fn percentile(sorted_data: &[u64], p: f64) -> u64 {
    if sorted_data.is_empty() {
        return 0;
    }
    
    let index = ((p / 100.0) * (sorted_data.len() - 1) as f64) as usize;
    sorted_data[index]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_percentile() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert_eq!(percentile(&data, 50.0), 5);
        assert_eq!(percentile(&data, 90.0), 9);
        assert_eq!(percentile(&data, 100.0), 10);
    }
}