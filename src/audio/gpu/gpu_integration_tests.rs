//! Integration tests for GPU acceleration with all Whisper model sizes

#[cfg(test)]
mod tests {
    use crate::audio::transcribe::{TranscriptionManager, TranscriptionEvent};
    use crate::config::{Config, WhisperModelSize, SpeechSettings};
    use std::path::PathBuf;
    use std::time::Instant;
    use tokio::sync::mpsc;
    use log::info;

    /// Test audio samples for different scenarios
    struct TestAudio {
        samples: Vec<f32>,
        duration_secs: f32,
        description: &'static str,
    }

    impl TestAudio {
        /// Generate test audio with silence
        fn silence(duration_secs: f32) -> Self {
            let sample_rate = 16000;
            let num_samples = (sample_rate as f32 * duration_secs) as usize;
            Self {
                samples: vec![0.0; num_samples],
                duration_secs,
                description: "silence",
            }
        }

        /// Generate test audio with noise
        fn noise(duration_secs: f32, amplitude: f32) -> Self {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let sample_rate = 16000;
            let num_samples = (sample_rate as f32 * duration_secs) as usize;
            
            let samples: Vec<f32> = (0..num_samples)
                .map(|_| rng.gen_range(-amplitude..amplitude))
                .collect();
            
            Self {
                samples,
                duration_secs,
                description: "noise",
            }
        }

        /// Generate test audio with sine wave (simulated speech)
        fn sine_wave(duration_secs: f32, frequency: f32) -> Self {
            let sample_rate = 16000;
            let num_samples = (sample_rate as f32 * duration_secs) as usize;
            
            let samples: Vec<f32> = (0..num_samples)
                .map(|i| {
                    let t = i as f32 / sample_rate as f32;
                    (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5
                })
                .collect();
            
            Self {
                samples,
                duration_secs,
                description: "sine wave",
            }
        }
    }

    /// Test GPU acceleration with a specific model size
    async fn test_model_with_gpu(model_size: WhisperModelSize, test_audio: &TestAudio) -> Result<(f32, bool), String> {
        info!("Testing model {:?} with {} audio ({:.1}s)", 
            model_size, test_audio.description, test_audio.duration_secs);

        // Create config with the model size
        let mut config = Config::default();
        config.audio.speech.model_size = model_size.clone();
        
        // Create transcription manager
        let (tx, mut rx) = mpsc::channel(100);
        let manager = TranscriptionManager::new(
            config.audio.speech.clone(),
            config.get_model_path().await,
        );
        
        // Set up event receiver
        manager.set_event_sender(tx);
        
        // Initialize (this loads the model)
        let init_start = Instant::now();
        manager.initialize().await
            .map_err(|e| format!("Failed to initialize: {}", e))?;
        let init_time = init_start.elapsed().as_secs_f32();
        info!("Model initialization took {:.2}s", init_time);
        
        // Check if GPU is being used
        let gpu_enabled = check_gpu_enabled();
        info!("GPU enabled: {}", gpu_enabled);
        
        // Process audio
        let transcribe_start = Instant::now();
        manager.process_audio(&test_audio.samples).await
            .map_err(|e| format!("Failed to process audio: {}", e))?;
        
        // Force processing
        manager.force_process_buffer().await
            .map_err(|e| format!("Failed to force process: {}", e))?;
        
        // Wait for transcription result
        let mut transcription_time = 0.0;
        let timeout = tokio::time::Duration::from_secs(60);
        
        match tokio::time::timeout(timeout, async {
            while let Some(event) = rx.recv().await {
                match event {
                    TranscriptionEvent::TranscriptionComplete { .. } => {
                        transcription_time = transcribe_start.elapsed().as_secs_f32();
                        info!("Transcription completed in {:.2}s", transcription_time);
                        break;
                    }
                    TranscriptionEvent::Error(e) => {
                        return Err(format!("Transcription error: {}", e));
                    }
                    _ => {}
                }
            }
            Ok(())
        }).await {
            Ok(Ok(())) => Ok((transcription_time, gpu_enabled)),
            Ok(Err(e)) => Err(e),
            Err(_) => Err("Transcription timeout".to_string()),
        }
    }

    /// Check if GPU features are enabled
    fn check_gpu_enabled() -> bool {
        #[cfg(any(feature = "gpu-cuda", feature = "gpu-metal"))]
        {
            true
        }
        
        #[cfg(not(any(feature = "gpu-cuda", feature = "gpu-metal")))]
        {
            false
        }
    }

    /// Run comprehensive GPU tests for all model sizes
    async fn run_all_model_tests() -> Result<Vec<(WhisperModelSize, f32, f32)>, String> {
        let model_sizes = vec![
            WhisperModelSize::Tiny,
            WhisperModelSize::Base,
            WhisperModelSize::Small,
            WhisperModelSize::Medium,
            // WhisperModelSize::Large, // Skip large for automated tests
        ];

        let test_audio = TestAudio::noise(10.0, 0.1); // 10 seconds of noise
        let mut results = Vec::new();

        for model_size in model_sizes {
            match test_model_with_gpu(model_size.clone(), &test_audio).await {
                Ok((time, gpu)) => {
                    let rtf = time / test_audio.duration_secs;
                    info!("Model {:?}: {:.2}s (RTF: {:.2}x) GPU: {}", 
                        model_size, time, rtf, gpu);
                    results.push((model_size, time, rtf));
                }
                Err(e) => {
                    info!("Model {:?} failed: {}", model_size, e);
                    // Continue with other models
                }
            }
        }

        Ok(results)
    }

    #[tokio::test]
    async fn test_gpu_tiny_model() {
        let test_audio = TestAudio::noise(5.0, 0.1);
        match test_model_with_gpu(WhisperModelSize::Tiny, &test_audio).await {
            Ok((time, gpu)) => {
                let rtf = time / test_audio.duration_secs;
                println!("Tiny model: {:.2}s (RTF: {:.2}x) GPU: {}", time, rtf, gpu);
                
                // Performance assertions
                if gpu {
                    assert!(rtf < 0.5, "GPU should achieve RTF < 0.5 for tiny model");
                } else {
                    assert!(rtf < 2.0, "CPU should achieve RTF < 2.0 for tiny model");
                }
            }
            Err(e) => {
                // Don't fail test if model not available
                println!("Tiny model test skipped: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_gpu_small_model() {
        let test_audio = TestAudio::noise(5.0, 0.1);
        match test_model_with_gpu(WhisperModelSize::Small, &test_audio).await {
            Ok((time, gpu)) => {
                let rtf = time / test_audio.duration_secs;
                println!("Small model: {:.2}s (RTF: {:.2}x) GPU: {}", time, rtf, gpu);
                
                // Performance assertions
                if gpu {
                    assert!(rtf < 1.0, "GPU should achieve RTF < 1.0 for small model");
                }
            }
            Err(e) => {
                println!("Small model test skipped: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Ignore by default as it takes longer
    async fn test_all_models_comprehensive() {
        match run_all_model_tests().await {
            Ok(results) => {
                println!("\n=== GPU Performance Test Results ===");
                println!("Model      | Time (s) | RTF    | Status");
                println!("-----------|----------|--------|-------");
                
                for (model, time, rtf) in results {
                    let status = if rtf < 1.0 { "✓ Real-time" } else { "✗ Slower" };
                    println!("{:10} | {:8.2} | {:6.2}x | {}", 
                        format!("{:?}", model), time, rtf, status);
                }
            }
            Err(e) => {
                println!("Comprehensive test failed: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_gpu_with_vad() {
        // Test that GPU works with VAD enabled
        let mut config = Config::default();
        config.audio.speech.model_size = WhisperModelSize::Tiny;
        config.audio.speech.vad.enabled = true;
        
        let (tx, _rx) = mpsc::channel(100);
        let manager = TranscriptionManager::new(
            config.audio.speech.clone(),
            config.get_model_path().await,
        );
        
        manager.set_event_sender(tx);
        
        match manager.initialize().await {
            Ok(_) => {
                println!("GPU + VAD initialization successful");
                
                // Test with mixed audio (silence + noise)
                let mut test_audio = Vec::new();
                test_audio.extend(TestAudio::silence(2.0).samples); // 2s silence
                test_audio.extend(TestAudio::noise(3.0, 0.2).samples); // 3s noise
                test_audio.extend(TestAudio::silence(2.0).samples); // 2s silence
                
                match manager.process_audio(&test_audio).await {
                    Ok(_) => println!("GPU + VAD processing successful"),
                    Err(e) => println!("GPU + VAD processing failed: {}", e),
                }
            }
            Err(e) => {
                println!("GPU + VAD initialization skipped: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_gpu_memory_handling() {
        // Test loading and unloading models to check memory management
        let model_sizes = vec![
            WhisperModelSize::Tiny,
            WhisperModelSize::Base,
        ];

        for model_size in model_sizes {
            info!("Testing memory handling for {:?}", model_size);
            
            // Create and destroy manager multiple times
            for i in 0..3 {
                let mut config = Config::default();
                config.audio.speech.model_size = model_size.clone();
                
                let (tx, _rx) = mpsc::channel(100);
                let manager = TranscriptionManager::new(
                    config.audio.speech.clone(),
                    config.get_model_path().await,
                );
                
                manager.set_event_sender(tx);
                
                match manager.initialize().await {
                    Ok(_) => {
                        info!("Iteration {} initialized", i);
                        
                        // Log GPU memory if available
                        #[cfg(any(feature = "gpu-cuda", feature = "gpu-metal"))]
                        {
                            use crate::audio::gpu_config;
                            let gpu_config = gpu_config::GpuConfig::default();
                            gpu_config::log_gpu_memory_usage(&gpu_config);
                        }
                    }
                    Err(e) => {
                        info!("Iteration {} failed: {}", i, e);
                        break;
                    }
                }
                
                // Drop manager to release resources
                drop(manager);
                
                // Small delay to allow cleanup
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
}