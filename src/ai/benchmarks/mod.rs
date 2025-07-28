pub mod model_benchmark;
pub mod performance_tracker;
pub mod benchmark_suite;

use serde::{Deserialize, Serialize};

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Number of warmup iterations
    pub warmup_iterations: usize,
    /// Number of benchmark iterations
    pub benchmark_iterations: usize,
    /// Test samples for benchmarking
    pub test_samples: Vec<TestSample>,
    /// Enable GPU benchmarking
    pub benchmark_gpu: bool,
    /// Enable memory profiling
    pub profile_memory: bool,
    /// Output format
    pub output_format: OutputFormat,
}

/// Test sample for benchmarking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSample {
    pub name: String,
    pub text: String,
    pub expected_tokens: usize,
}

/// Output format for benchmark results
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OutputFormat {
    Json,
    Csv,
    Markdown,
    Html,
}

/// Benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub model_id: String,
    pub test_name: String,
    pub iterations: usize,
    pub avg_latency_ms: f64,
    pub min_latency_ms: u64,
    pub max_latency_ms: u64,
    pub p50_latency_ms: u64,
    pub p90_latency_ms: u64,
    pub p95_latency_ms: u64,
    pub p99_latency_ms: u64,
    pub throughput_tps: f64,
    pub memory_usage_mb: u64,
    pub gpu_usage_percent: Option<f32>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            warmup_iterations: 5,
            benchmark_iterations: 100,
            test_samples: Self::default_test_samples(),
            benchmark_gpu: true,
            profile_memory: true,
            output_format: OutputFormat::Json,
        }
    }
}

impl BenchmarkConfig {
    fn default_test_samples() -> Vec<TestSample> {
        vec![
            TestSample {
                name: "short_text".to_string(),
                text: "Fix the grammer in this sentense.".to_string(),
                expected_tokens: 10,
            },
            TestSample {
                name: "medium_text".to_string(),
                text: "The quick brown fox jumps over the lazy dog. This is a test sentence with some errors that need to be corrected for proper grammar and clarity.".to_string(),
                expected_tokens: 50,
            },
            TestSample {
                name: "long_text".to_string(),
                text: "In todays fast-paced world, effective communication is more important than ever. Whether your writing emails, reports, or social media posts, having clear and grammatically correct text is essential. This paragraph contains several intentional errors including missing apostrophes, incorrect verb forms, and punctuation mistakes that a good AI model should be able to identify and correct. The goal is to test the models ability to handle longer texts while maintaining context and producing coherent corrections throughout the entire passage.".to_string(),
                expected_tokens: 150,
            },
            TestSample {
                name: "code_comment".to_string(),
                text: "// This function calculate the fibonacci sequence recursivly and return the nth number".to_string(),
                expected_tokens: 20,
            },
            TestSample {
                name: "business_email".to_string(),
                text: "Dear Mr. Smith, I hope this email find you well. I wanted to followup on our conversation from last week regarding the new project proposal. Please let me know if you have any question or if their is anything else you need from me.".to_string(),
                expected_tokens: 60,
            },
        ]
    }
}