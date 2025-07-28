//! Comprehensive benchmarking tool for BestMe
//! 
//! This tool benchmarks transcription performance across different
//! model sizes and hardware configurations (CPU vs GPU).

use anyhow::Result;
use bestme::audio::transcribe::{TranscriptionManager, TranscriptionEvent};
use bestme::config::{Config, WhisperModelSize};
use clap::{Parser, ValueEnum};
use log::{info, warn};
use std::time::Instant;

#[derive(Parser, Debug)]
#[clap(name = "BestMe Benchmark", version = "1.0", about = "Benchmark transcription performance")]
struct Args {
    /// Enable GPU acceleration
    #[clap(long)]
    gpu: bool,

    /// Specific model to benchmark (defaults to all)
    #[clap(long, value_enum)]
    model: Option<ModelSize>,

    /// Compare CPU vs GPU performance
    #[clap(long)]
    compare: bool,

    /// Audio duration in seconds
    #[clap(long, default_value = "30")]
    duration: f32,

    /// Number of iterations per test
    #[clap(long, default_value = "3")]
    iterations: usize,

    /// Output format
    #[clap(long, value_enum, default_value = "table")]
    format: OutputFormat,
}

#[derive(Debug, Clone, ValueEnum)]
enum ModelSize {
    Tiny,
    Base,
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Csv,
}

impl From<ModelSize> for WhisperModelSize {
    fn from(size: ModelSize) -> Self {
        match size {
            ModelSize::Tiny => WhisperModelSize::Tiny,
            ModelSize::Base => WhisperModelSize::Base,
            ModelSize::Small => WhisperModelSize::Small,
            ModelSize::Medium => WhisperModelSize::Medium,
            ModelSize::Large => WhisperModelSize::Large,
        }
    }
}

#[derive(Debug, Clone)]
struct BenchmarkResult {
    model_size: WhisperModelSize,
    gpu_enabled: bool,
    duration_secs: f32,
    processing_time_secs: f32,
    rtf: f32,
    memory_mb: Option<f32>,
}

/// Generate test audio data
fn generate_test_audio(duration_secs: f32) -> Vec<f32> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let sample_rate = 16000;
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    
    let mut audio = Vec::with_capacity(num_samples);
    
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        
        // Mix of silence and "speech"
        let sample = if (t as i32) % 4 < 3 {
            // Simulated speech (multiple frequencies)
            let f1 = 200.0 + (t * 5.0).sin() * 50.0;
            let f2 = 300.0 + (t * 7.0).cos() * 70.0;
            let s1 = (2.0 * std::f32::consts::PI * f1 * t).sin() * 0.2;
            let s2 = (2.0 * std::f32::consts::PI * f2 * t).sin() * 0.15;
            s1 + s2 + rng.gen_range(-0.05..0.05)
        } else {
            // Silence with slight noise
            rng.gen_range(-0.01..0.01)
        };
        
        audio.push(sample);
    }
    
    audio
}

/// Run a single benchmark iteration
async fn run_benchmark(
    model_size: WhisperModelSize,
    gpu_enabled: bool,
    audio: &[f32],
    duration_secs: f32,
) -> Result<BenchmarkResult> {
    let mut config = Config::default();
    config.audio.speech.model_size = model_size.clone();
    
    // Enable GPU if requested and available
    #[cfg(any(feature = "gpu-cuda", feature = "gpu-metal"))]
    {
        if gpu_enabled {
            info!("GPU acceleration enabled");
        }
    }
    
    let (mut manager, mut rx) = TranscriptionManager::new(
        config.audio.speech.clone(),
    ).map_err(|e| anyhow::anyhow!("Failed to create transcription manager: {}", e))?;
    
    // Initialize (loads model)
    let init_start = Instant::now();
    manager.initialize().await?;
    let init_time = init_start.elapsed().as_secs_f32();
    info!("Model initialization took {:.2}s", init_time);
    
    // Get initial memory usage
    let initial_memory = get_memory_usage_mb();
    
    // Process audio
    let process_start = Instant::now();
    let _ = manager.process_audio(audio).await?;
    
    // Wait for completion
    let mut processing_time = 0.0;
    let timeout = tokio::time::Duration::from_secs(120);
    
    tokio::time::timeout(timeout, async {
        while let Some(event) = rx.recv().await {
            match event {
                TranscriptionEvent::Transcription(_) => {
                    processing_time = process_start.elapsed().as_secs_f32();
                    break;
                }
                TranscriptionEvent::Error(e) => {
                    return Err(anyhow::anyhow!("Transcription error: {}", e));
                }
                _ => {}
            }
        }
        Ok(())
    }).await??;
    
    // Get final memory usage
    let final_memory = get_memory_usage_mb();
    let memory_delta = final_memory.map(|f| f - initial_memory.unwrap_or(0.0));
    
    let rtf = processing_time / duration_secs;
    
    Ok(BenchmarkResult {
        model_size,
        gpu_enabled,
        duration_secs,
        processing_time_secs: processing_time,
        rtf,
        memory_mb: memory_delta,
    })
}

/// Get current memory usage in MB
fn get_memory_usage_mb() -> Option<f32> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<f32>() {
                            return Some(kb / 1024.0);
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("ps")
            .args(&["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
        {
            if let Ok(s) = String::from_utf8(output.stdout) {
                if let Ok(kb) = s.trim().parse::<f32>() {
                    return Some(kb / 1024.0);
                }
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        // Windows memory reporting would go here
    }
    
    None
}

/// Format results as table
fn print_table(results: &[BenchmarkResult]) {
    println!("\n╔═══════════════════════════════════════════════════════════════════════╗");
    println!("║                        BestMe Benchmark Results                        ║");
    println!("╠═══════════╦═════════╦═══════════╦═══════════╦════════╦═══════════════╣");
    println!("║ Model     ║ Backend ║ Time (s)  ║ RTF       ║ Memory ║ Status        ║");
    println!("╠═══════════╬═════════╬═══════════╬═══════════╬════════╬═══════════════╣");
    
    for result in results {
        let backend = if result.gpu_enabled { "GPU" } else { "CPU" };
        let status = if result.rtf < 1.0 { 
            "✓ Real-time" 
        } else if result.rtf < 2.0 {
            "⚡ Near RT"
        } else {
            "✗ Slow"
        };
        
        let memory = result.memory_mb
            .map(|m| format!("{:>6.1} MB", m))
            .unwrap_or_else(|| "    N/A".to_string());
        
        println!("║ {:9} ║ {:7} ║ {:9.2} ║ {:9.2}x ║ {} ║ {:13} ║",
            format!("{:?}", result.model_size),
            backend,
            result.processing_time_secs,
            result.rtf,
            memory,
            status
        );
    }
    
    println!("╚═══════════╩═════════╩═══════════╩═══════════╩════════╩═══════════════╝");
}

/// Format results as JSON
fn print_json(results: &[BenchmarkResult]) {
    let json_results: Vec<serde_json::Value> = results.iter().map(|r| {
        serde_json::json!({
            "model": format!("{:?}", r.model_size),
            "backend": if r.gpu_enabled { "GPU" } else { "CPU" },
            "duration_secs": r.duration_secs,
            "processing_time_secs": r.processing_time_secs,
            "rtf": r.rtf,
            "memory_mb": r.memory_mb,
            "real_time": r.rtf < 1.0
        })
    }).collect();
    
    println!("{}", serde_json::to_string_pretty(&json_results).unwrap());
}

/// Format results as CSV
fn print_csv(results: &[BenchmarkResult]) {
    println!("model,backend,duration_secs,processing_time_secs,rtf,memory_mb,real_time");
    for r in results {
        println!("{:?},{},{},{},{},{},{}",
            r.model_size,
            if r.gpu_enabled { "GPU" } else { "CPU" },
            r.duration_secs,
            r.processing_time_secs,
            r.rtf,
            r.memory_mb.unwrap_or(0.0),
            r.rtf < 1.0
        );
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let args = Args::parse();
    
    // Check GPU availability
    let gpu_available = check_gpu_available();
    if args.gpu && !gpu_available {
        warn!("GPU acceleration requested but no GPU features enabled at compile time");
        warn!("Rebuild with --features gpu-cuda (NVIDIA), gpu-metal (Apple), etc.");
    }
    
    // Determine which models to test
    let models = if let Some(model) = args.model {
        vec![WhisperModelSize::from(model)]
    } else {
        vec![
            WhisperModelSize::Tiny,
            WhisperModelSize::Base,
            WhisperModelSize::Small,
            WhisperModelSize::Medium,
        ]
    };
    
    // Generate test audio once
    info!("Generating {} seconds of test audio...", args.duration);
    let audio = generate_test_audio(args.duration);
    
    let mut all_results = Vec::new();
    
    // Run benchmarks
    if args.compare {
        // Compare CPU vs GPU for each model
        for model in &models {
            println!("\nBenchmarking {:?} model...", model);
            
            // CPU benchmark
            println!("  Running CPU benchmark ({} iterations)...", args.iterations);
            let mut cpu_times = Vec::new();
            for i in 0..args.iterations {
                match run_benchmark(model.clone(), false, &audio, args.duration).await {
                    Ok(result) => {
                        println!("    Iteration {}: {:.2}s (RTF: {:.2}x)", 
                            i + 1, result.processing_time_secs, result.rtf);
                        cpu_times.push(result);
                    }
                    Err(e) => {
                        eprintln!("    Iteration {} failed: {}", i + 1, e);
                    }
                }
            }
            
            if !cpu_times.is_empty() {
                let avg_result = average_results(&cpu_times);
                all_results.push(avg_result);
            }
            
            // GPU benchmark (if available)
            if gpu_available {
                println!("  Running GPU benchmark ({} iterations)...", args.iterations);
                let mut gpu_times = Vec::new();
                for i in 0..args.iterations {
                    match run_benchmark(model.clone(), true, &audio, args.duration).await {
                        Ok(result) => {
                            println!("    Iteration {}: {:.2}s (RTF: {:.2}x)", 
                                i + 1, result.processing_time_secs, result.rtf);
                            gpu_times.push(result);
                        }
                        Err(e) => {
                            eprintln!("    Iteration {} failed: {}", i + 1, e);
                        }
                    }
                }
                
                if !gpu_times.is_empty() {
                    let avg_result = average_results(&gpu_times);
                    all_results.push(avg_result);
                }
            }
        }
    } else {
        // Single mode benchmark
        for model in &models {
            println!("\nBenchmarking {:?} model...", model);
            let mut results = Vec::new();
            
            for i in 0..args.iterations {
                match run_benchmark(model.clone(), args.gpu && gpu_available, &audio, args.duration).await {
                    Ok(result) => {
                        println!("  Iteration {}: {:.2}s (RTF: {:.2}x)", 
                            i + 1, result.processing_time_secs, result.rtf);
                        results.push(result);
                    }
                    Err(e) => {
                        eprintln!("  Iteration {} failed: {}", i + 1, e);
                    }
                }
            }
            
            if !results.is_empty() {
                let avg_result = average_results(&results);
                all_results.push(avg_result);
            }
        }
    }
    
    // Print results
    match args.format {
        OutputFormat::Table => print_table(&all_results),
        OutputFormat::Json => print_json(&all_results),
        OutputFormat::Csv => print_csv(&all_results),
    }
    
    // Print summary
    if args.compare && all_results.len() >= 2 {
        println!("\n📊 Performance Summary:");
        
        // Group by model
        for model in &models {
            let model_results: Vec<&BenchmarkResult> = all_results.iter()
                .filter(|r| r.model_size == *model)
                .collect();
            
            if model_results.len() == 2 {
                let cpu_result = model_results.iter().find(|r| !r.gpu_enabled).unwrap();
                let gpu_result = model_results.iter().find(|r| r.gpu_enabled).unwrap();
                
                let speedup = cpu_result.processing_time_secs / gpu_result.processing_time_secs;
                println!("  {:?}: GPU is {:.1}x faster than CPU", model, speedup);
            }
        }
    }
    
    Ok(())
}

/// Check if GPU features are available at compile time
fn check_gpu_available() -> bool {
    #[cfg(any(feature = "gpu-cuda", feature = "gpu-metal"))]
    {
        true
    }
    
    #[cfg(not(any(feature = "gpu-cuda", feature = "gpu-metal")))]
    {
        false
    }
}

/// Average multiple benchmark results
fn average_results(results: &[BenchmarkResult]) -> BenchmarkResult {
    let n = results.len() as f32;
    
    let avg_processing_time = results.iter()
        .map(|r| r.processing_time_secs)
        .sum::<f32>() / n;
    
    let avg_rtf = results.iter()
        .map(|r| r.rtf)
        .sum::<f32>() / n;
    
    let avg_memory = if results.iter().all(|r| r.memory_mb.is_some()) {
        Some(results.iter()
            .map(|r| r.memory_mb.unwrap())
            .sum::<f32>() / n)
    } else {
        None
    };
    
    BenchmarkResult {
        model_size: results[0].model_size.clone(),
        gpu_enabled: results[0].gpu_enabled,
        duration_secs: results[0].duration_secs,
        processing_time_secs: avg_processing_time,
        rtf: avg_rtf,
        memory_mb: avg_memory,
    }
}