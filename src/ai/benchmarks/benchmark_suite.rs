use crate::ai::{Result, AIError};
use crate::ai::services::model_service::ModelService;
use super::{BenchmarkConfig, BenchmarkResult};
use super::model_benchmark::ModelBenchmark;
use std::sync::Arc;
use std::path::Path;

/// Comprehensive benchmark suite for AI models
pub struct BenchmarkSuite {
    config: BenchmarkConfig,
    model_service: Arc<ModelService>,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite
    pub async fn new(config: BenchmarkConfig) -> Result<Self> {
        let model_service = Arc::new(ModelService::new().await?);
        
        Ok(Self {
            config,
            model_service,
        })
    }
    
    /// Run the complete benchmark suite
    pub async fn run_suite(&self) -> Result<SuiteResults> {
        log::info!("Starting comprehensive benchmark suite");
        let start_time = std::time::Instant::now();
        
        // Create benchmark instance
        let benchmark = ModelBenchmark::new(self.config.clone(), self.model_service.clone());
        
        // Run benchmarks for all models
        let results = benchmark.benchmark_all_models().await?;
        
        // Generate report
        let output_dir = Path::new("benchmark_results");
        std::fs::create_dir_all(&output_dir)
            .map_err(|e| AIError::ConfigError(format!("Failed to create output directory: {}", e)))?;
        
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        
        // Export in all formats
        let json_path = output_dir.join(format!("results_{}.json", timestamp));
        let csv_path = output_dir.join(format!("results_{}.csv", timestamp));
        let md_path = output_dir.join(format!("results_{}.md", timestamp));
        let html_path = output_dir.join(format!("results_{}.html", timestamp));
        
        // Export JSON
        self.config.output_format = super::OutputFormat::Json;
        benchmark.export_results(json_path.to_str().unwrap()).await?;
        
        // Export CSV
        self.config.output_format = super::OutputFormat::Csv;
        benchmark.export_results(csv_path.to_str().unwrap()).await?;
        
        // Export Markdown
        self.config.output_format = super::OutputFormat::Markdown;
        benchmark.export_results(md_path.to_str().unwrap()).await?;
        
        // Export HTML
        self.config.output_format = super::OutputFormat::Html;
        benchmark.export_results(html_path.to_str().unwrap()).await?;
        
        let total_time = start_time.elapsed();
        
        Ok(SuiteResults {
            results,
            total_time_seconds: total_time.as_secs(),
            output_paths: vec![
                json_path.to_string_lossy().to_string(),
                csv_path.to_string_lossy().to_string(),
                md_path.to_string_lossy().to_string(),
                html_path.to_string_lossy().to_string(),
            ],
        })
    }
    
    /// Run a quick benchmark on a subset of models
    pub async fn run_quick_test(&self, model_ids: Vec<String>) -> Result<Vec<BenchmarkResult>> {
        log::info!("Running quick benchmark test");
        
        let benchmark = ModelBenchmark::new(self.config.clone(), self.model_service.clone());
        let mut all_results = Vec::new();
        
        for model_id in model_ids {
            match benchmark.benchmark_model(&model_id).await {
                Ok(results) => all_results.extend(results),
                Err(e) => log::error!("Failed to benchmark {}: {}", model_id, e),
            }
        }
        
        Ok(all_results)
    }
    
    /// Compare two models head-to-head
    pub async fn compare_models(
        &self,
        model_a: &str,
        model_b: &str,
    ) -> Result<ComparisonResult> {
        log::info!("Comparing {} vs {}", model_a, model_b);
        
        let benchmark = ModelBenchmark::new(self.config.clone(), self.model_service.clone());
        
        // Benchmark both models
        let results_a = benchmark.benchmark_model(model_a).await?;
        let results_b = benchmark.benchmark_model(model_b).await?;
        
        // Calculate comparison metrics
        let avg_latency_a = results_a.iter().map(|r| r.avg_latency_ms).sum::<f64>() / results_a.len() as f64;
        let avg_latency_b = results_b.iter().map(|r| r.avg_latency_ms).sum::<f64>() / results_b.len() as f64;
        
        let avg_throughput_a = results_a.iter().map(|r| r.throughput_tps).sum::<f64>() / results_a.len() as f64;
        let avg_throughput_b = results_b.iter().map(|r| r.throughput_tps).sum::<f64>() / results_b.len() as f64;
        
        let latency_improvement = ((avg_latency_a - avg_latency_b) / avg_latency_a) * 100.0;
        let throughput_improvement = ((avg_throughput_b - avg_throughput_a) / avg_throughput_a) * 100.0;
        
        Ok(ComparisonResult {
            model_a: model_a.to_string(),
            model_b: model_b.to_string(),
            results_a,
            results_b,
            latency_improvement_percent: latency_improvement,
            throughput_improvement_percent: throughput_improvement,
            winner: if latency_improvement > 0.0 { model_b.to_string() } else { model_a.to_string() },
        })
    }
}

/// Results from a complete benchmark suite run
#[derive(Debug, Clone)]
pub struct SuiteResults {
    pub results: Vec<BenchmarkResult>,
    pub total_time_seconds: u64,
    pub output_paths: Vec<String>,
}

/// Results from comparing two models
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    pub model_a: String,
    pub model_b: String,
    pub results_a: Vec<BenchmarkResult>,
    pub results_b: Vec<BenchmarkResult>,
    pub latency_improvement_percent: f64,
    pub throughput_improvement_percent: f64,
    pub winner: String,
}

impl ComparisonResult {
    /// Generate a comparison report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("# Model Comparison: {} vs {}\n\n", self.model_a, self.model_b));
        report.push_str(&format!("**Winner**: {}\n\n", self.winner));
        
        report.push_str("## Summary\n\n");
        report.push_str(&format!("- Latency improvement: {:.1}%\n", self.latency_improvement_percent));
        report.push_str(&format!("- Throughput improvement: {:.1}%\n\n", self.throughput_improvement_percent));
        
        report.push_str("## Detailed Results\n\n");
        
        // Create comparison table
        report.push_str("| Test | Model A Latency | Model B Latency | Improvement |\n");
        report.push_str("|------|----------------|----------------|-------------|\n");
        
        for (a, b) in self.results_a.iter().zip(self.results_b.iter()) {
            let improvement = ((a.avg_latency_ms - b.avg_latency_ms) / a.avg_latency_ms) * 100.0;
            report.push_str(&format!(
                "| {} | {:.2}ms | {:.2}ms | {:.1}% |\n",
                a.test_name,
                a.avg_latency_ms,
                b.avg_latency_ms,
                improvement
            ));
        }
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_benchmark_suite_creation() {
        let config = BenchmarkConfig::default();
        let suite = BenchmarkSuite::new(config).await;
        assert!(suite.is_ok());
    }
}