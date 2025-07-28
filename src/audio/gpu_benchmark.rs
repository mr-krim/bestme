//! GPU performance benchmarking for Whisper transcription

use anyhow::{Context, Result};
use log::{info, warn};
use std::time::{Duration, Instant};
use std::path::Path;

use crate::config::WhisperModelSize;
use crate::audio::transcribe::TranscriptionManager;

/// Benchmark result for a single test
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub model_size: WhisperModelSize,
    pub audio_duration: Duration,
    pub transcription_time: Duration,
    pub real_time_factor: f32,
    pub gpu_enabled: bool,
    pub backend: Option<String>,
    pub memory_used_mb: Option<usize>,
}

/// GPU benchmark configuration
pub struct BenchmarkConfig {
    pub model_sizes: Vec<WhisperModelSize>,
    pub test_audio_path: Option<String>,
    pub test_duration_seconds: u32,
    pub warmup_runs: u32,
    pub benchmark_runs: u32,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            model_sizes: vec![
                WhisperModelSize::Tiny,
                WhisperModelSize::Base,
                WhisperModelSize::Small,
                WhisperModelSize::Medium,
            ],
            test_audio_path: None,
            test_duration_seconds: 30,
            warmup_runs: 1,
            benchmark_runs: 3,
        }
    }
}

/// GPU performance benchmark runner
pub struct GpuBenchmark {
    config: BenchmarkConfig,
    results: Vec<BenchmarkResult>,
}

impl GpuBenchmark {
    /// Create a new benchmark runner
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
        }
    }
    
    /// Run the benchmark suite
    pub async fn run(&mut self) -> Result<Vec<BenchmarkResult>> {
        info!("Starting GPU benchmark suite");
        
        // Get GPU status
        #[cfg(any(feature = "gpu-cuda", feature = "gpu-metal"))]
        let gpu_info = {
            use crate::audio::gpu_config;
            gpu_config::get_gpu_status()
        };
        
        #[cfg(not(any(feature = "gpu-cuda", feature = "gpu-metal")))]
        let gpu_info = crate::audio::gpu_config::GpuStatus {
            gpu_available: false,
            backend: None,
            device_name: None,
            memory_mb: None,
            driver_version: None,
        };
        
        info!("GPU Status: Available={}, Backend={:?}", 
            gpu_info.gpu_available, 
            gpu_info.backend
        );
        
        // Generate or load test audio
        let test_audio = self.get_test_audio()?;
        let audio_duration = Duration::from_secs(self.config.test_duration_seconds as u64);
        
        // Run benchmarks for each model size
        for model_size in &self.config.model_sizes {
            info!("Benchmarking model: {:?}", model_size);
            
            match self.benchmark_model(model_size.clone(), &test_audio, audio_duration, &gpu_info).await {
                Ok(result) => {
                    info!("Model {:?} - Time: {:.2}s, RTF: {:.2}x", 
                        model_size,
                        result.transcription_time.as_secs_f32(),
                        result.real_time_factor
                    );
                    self.results.push(result);
                }
                Err(e) => {
                    warn!("Failed to benchmark model {:?}: {}", model_size, e);
                }
            }
        }
        
        Ok(self.results.clone())
    }
    
    /// Benchmark a specific model
    async fn benchmark_model(
        &self,
        model_size: WhisperModelSize,
        audio: &[f32],
        audio_duration: Duration,
        gpu_info: &crate::audio::gpu_config::GpuStatus,
    ) -> Result<BenchmarkResult> {
        // Create transcription manager with the model
        let mut config = crate::config::Config::default();
        config.audio.speech.model_size = model_size.clone();
        
        let manager = TranscriptionManager::new(
            config.audio.speech.clone(),
            config.get_model_path().await,
        );
        
        // Initialize the model
        manager.initialize().await
            .context("Failed to initialize transcription manager")?;
        
        // Warmup runs
        for i in 0..self.config.warmup_runs {
            info!("Warmup run {}/{}", i + 1, self.config.warmup_runs);
            let _ = manager.transcribe_audio(audio).await;
        }
        
        // Benchmark runs
        let mut total_time = Duration::ZERO;
        
        for i in 0..self.config.benchmark_runs {
            info!("Benchmark run {}/{}", i + 1, self.config.benchmark_runs);
            
            let start = Instant::now();
            let _ = manager.transcribe_audio(audio).await
                .context("Transcription failed")?;
            let elapsed = start.elapsed();
            
            total_time += elapsed;
            
            // Log memory usage if available
            #[cfg(any(feature = "gpu-cuda", feature = "gpu-metal"))]
            {
                use crate::audio::gpu_config;
                gpu_config::log_gpu_memory_usage(&gpu_config::GpuConfig::default());
            }
        }
        
        let avg_time = total_time / self.config.benchmark_runs;
        let real_time_factor = avg_time.as_secs_f32() / audio_duration.as_secs_f32();
        
        Ok(BenchmarkResult {
            model_size,
            audio_duration,
            transcription_time: avg_time,
            real_time_factor,
            gpu_enabled: gpu_info.gpu_available,
            backend: gpu_info.backend.clone(),
            memory_used_mb: None, // TODO: Capture actual memory usage
        })
    }
    
    /// Get test audio data
    fn get_test_audio(&self) -> Result<Vec<f32>> {
        if let Some(path) = &self.config.test_audio_path {
            // Load audio from file
            self.load_audio_file(Path::new(path))
        } else {
            // Generate synthetic test audio (silence with some noise)
            Ok(self.generate_test_audio())
        }
    }
    
    /// Load audio from file
    fn load_audio_file(&self, path: &Path) -> Result<Vec<f32>> {
        use hound;
        
        let reader = hound::WavReader::open(path)
            .context("Failed to open audio file")?;
        
        let spec = reader.spec();
        let samples: Vec<f32> = reader.into_samples::<i16>()
            .filter_map(Result::ok)
            .map(|s| s as f32 / i16::MAX as f32)
            .collect();
        
        // Resample to 16kHz if needed
        if spec.sample_rate != 16000 {
            warn!("Audio file has sample rate {}Hz, resampling to 16kHz", spec.sample_rate);
            // Simple downsampling - in production use a proper resampler
            let ratio = spec.sample_rate as f32 / 16000.0;
            let resampled: Vec<f32> = samples.iter()
                .step_by(ratio as usize)
                .copied()
                .collect();
            Ok(resampled)
        } else {
            Ok(samples)
        }
    }
    
    /// Generate synthetic test audio
    fn generate_test_audio(&self) -> Vec<f32> {
        let sample_rate = 16000;
        let duration = self.config.test_duration_seconds;
        let num_samples = sample_rate * duration;
        
        // Generate low-level noise to simulate speech
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        (0..num_samples)
            .map(|_| rng.gen_range(-0.1..0.1))
            .collect()
    }
    
    /// Generate a summary report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# GPU Benchmark Report\n\n");
        
        // GPU info
        #[cfg(any(feature = "gpu-cuda", feature = "gpu-metal"))]
        {
            use crate::audio::gpu_config;
            let gpu_info = gpu_config::get_gpu_status();
            
            report.push_str(&format!("## GPU Information\n"));
            report.push_str(&format!("- Available: {}\n", gpu_info.gpu_available));
            if let Some(backend) = &gpu_info.backend {
                report.push_str(&format!("- Backend: {}\n", backend));
            }
            if let Some(device) = &gpu_info.device_name {
                report.push_str(&format!("- Device: {}\n", device));
            }
            report.push_str("\n");
        }
        
        // Results table
        report.push_str("## Benchmark Results\n\n");
        report.push_str("| Model | Audio Duration | Transcription Time | Real-time Factor | Speedup |\n");
        report.push_str("|-------|----------------|-------------------|------------------|----------|\n");
        
        for result in &self.results {
            let speedup = if result.real_time_factor < 1.0 {
                format!("{:.1}x faster", 1.0 / result.real_time_factor)
            } else {
                format!("{:.1}x slower", result.real_time_factor)
            };
            
            report.push_str(&format!(
                "| {:?} | {:.1}s | {:.2}s | {:.2}x | {} |\n",
                result.model_size,
                result.audio_duration.as_secs_f32(),
                result.transcription_time.as_secs_f32(),
                result.real_time_factor,
                speedup
            ));
        }
        
        report
    }
}

/// Run a quick GPU benchmark
pub async fn run_quick_benchmark() -> Result<String> {
    let config = BenchmarkConfig {
        model_sizes: vec![WhisperModelSize::Tiny, WhisperModelSize::Small],
        test_duration_seconds: 10,
        warmup_runs: 0,
        benchmark_runs: 1,
        ..Default::default()
    };
    
    let mut benchmark = GpuBenchmark::new(config);
    benchmark.run().await?;
    
    Ok(benchmark.generate_report())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_benchmark_config() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.model_sizes.len(), 4);
        assert_eq!(config.test_duration_seconds, 30);
        assert_eq!(config.benchmark_runs, 3);
    }
    
    #[tokio::test]
    async fn test_synthetic_audio_generation() {
        let config = BenchmarkConfig {
            test_duration_seconds: 1,
            ..Default::default()
        };
        
        let benchmark = GpuBenchmark::new(config);
        let audio = benchmark.generate_test_audio();
        
        assert_eq!(audio.len(), 16000); // 1 second at 16kHz
    }
}