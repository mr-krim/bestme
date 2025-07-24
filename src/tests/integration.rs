//! Integration tests for Phase 0 features

use anyhow::Result;
use bestme::{
    audio::{
        transcribe::{TranscriptionManager, TranscriptionEvent},
        voice_commands::{VoiceCommandProcessor, VoiceCommandEvent},
    },
    config::SpeechSettings,
};
use std::sync::Arc;
use parking_lot::Mutex;
use tokio::sync::mpsc;

#[cfg(feature = "storage")]
use bestme::storage::{StorageManager, DatabaseConfig, Transcript, ExportFormat};

/// Test voice command integration with transcription pipeline
#[tokio::test]
async fn test_voice_command_through_transcription() {
    // Setup transcription manager
    let settings = SpeechSettings::default();
    let (mut manager, mut receiver) = TranscriptionManager::new(settings).unwrap();
    
    // Setup voice command processor
    let voice_config = bestme::config::VoiceCommandConfig::default();
    let (processor, _cmd_receiver) = VoiceCommandProcessor::new(voice_config).unwrap();
    let processor = Arc::new(Mutex::new(processor));
    
    // Enable voice commands
    manager.enable_voice_commands(processor.clone());
    
    // Initialize manager
    manager.initialize().await.unwrap();
    manager.start().await.unwrap();
    
    // Process text that contains a command
    let command_text = "please delete the last word";
    let result = manager.process_for_commands(command_text).await;
    
    assert!(result.is_some());
    let (command_type, exec_result) = result.unwrap();
    assert_eq!(command_type, "Delete");
    assert!(exec_result.is_ok());
    
    // Verify event was sent
    if let Some(event) = receiver.recv().await {
        match event {
            TranscriptionEvent::CommandExecuted { command, result } => {
                assert_eq!(command, "Delete");
                assert!(result.is_ok());
            }
            _ => panic!("Expected CommandExecuted event"),
        }
    }
    
    manager.stop().await.unwrap();
}

/// Test storage integration with transcription
#[cfg(feature = "storage")]
#[tokio::test]
async fn test_transcript_auto_save() {
    use tempfile::tempdir;
    
    // Create temporary storage
    let temp_dir = tempdir().unwrap();
    let config = DatabaseConfig {
        path: temp_dir.path().join("test.db"),
        ..Default::default()
    };
    
    let storage_manager = Arc::new(StorageManager::new(config).await.unwrap());
    
    // Setup transcription manager with storage
    let mut settings = SpeechSettings::default();
    settings.save_transcription = true;
    
    let (mut manager, _receiver) = TranscriptionManager::new(settings).unwrap();
    manager.set_storage_manager(storage_manager.clone());
    
    // Start a session
    let session_id = storage_manager.start_session(Some("Test Device".to_string())).await.unwrap();
    
    // Simulate transcription
    manager.initialize().await.unwrap();
    manager.start().await.unwrap();
    
    // Process some audio (simulation will save a transcript)
    let audio_data = vec![0.0f32; 48000]; // 3 seconds of silence
    manager.process_audio(&audio_data).await.unwrap();
    
    // Wait for async save
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    // Verify transcript was saved
    let transcripts = storage_manager.list_transcripts(10, 0, Some(&session_id)).await.unwrap();
    assert!(!transcripts.is_empty(), "Transcript should have been saved");
    
    manager.stop().await.unwrap();
    storage_manager.end_session().await.unwrap();
}

/// Test export functionality
#[cfg(feature = "storage")]
#[tokio::test]
async fn test_export_formats() {
    use tempfile::tempdir;
    
    let temp_dir = tempdir().unwrap();
    let config = DatabaseConfig {
        path: temp_dir.path().join("test.db"),
        ..Default::default()
    };
    
    let storage_manager = StorageManager::new(config).await.unwrap();
    
    // Create test transcripts
    let transcript1 = Transcript::new(
        "test-session".to_string(),
        "This is the first test transcript.".to_string(),
    );
    let transcript2 = Transcript::new(
        "test-session".to_string(),
        "This is the second test transcript with more content.".to_string(),
    );
    
    let id1 = storage_manager.save_transcript(transcript1).await.unwrap();
    let id2 = storage_manager.save_transcript(transcript2).await.unwrap();
    
    // Test each export format
    let formats = vec![
        ExportFormat::Text,
        ExportFormat::Markdown,
        ExportFormat::Json,
        ExportFormat::Csv,
    ];
    
    for format in formats {
        let exported = storage_manager
            .export_transcripts(&[id1.clone(), id2.clone()], format)
            .await
            .unwrap();
        
        assert!(!exported.is_empty(), "Export for {:?} should not be empty", format);
        
        // Verify format-specific content
        match format {
            ExportFormat::Text => {
                assert!(exported.contains("first test transcript"));
                assert!(exported.contains("second test transcript"));
            }
            ExportFormat::Markdown => {
                assert!(exported.contains("# Transcripts Export"));
                assert!(exported.contains("**Words:**"));
            }
            ExportFormat::Json => {
                let parsed: Vec<Transcript> = serde_json::from_str(&exported).unwrap();
                assert_eq!(parsed.len(), 2);
            }
            ExportFormat::Csv => {
                assert!(exported.contains("ID,Session ID,Created At"));
                assert_eq!(exported.lines().count(), 3); // Header + 2 records
            }
        }
    }
}

/// Test error handling in transcription pipeline
#[tokio::test]
async fn test_transcription_error_handling() {
    let settings = SpeechSettings {
        model_path: Some("/invalid/path".to_string()),
        ..Default::default()
    };
    
    let (mut manager, mut receiver) = TranscriptionManager::new(settings).unwrap();
    
    // This should not panic even with invalid model path
    let init_result = manager.initialize().await;
    assert!(init_result.is_ok(), "Initialization should succeed even without model");
    
    manager.start().await.unwrap();
    
    // Process audio should work in simulation mode
    let audio_data = vec![0.0f32; 16000];
    let result = manager.process_audio(&audio_data).await;
    assert!(result.is_ok());
    
    // Should receive transcription event (simulated)
    tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        async {
            while let Some(event) = receiver.recv().await {
                if matches!(event, TranscriptionEvent::Transcription(_)) {
                    return Ok(());
                }
            }
            Err(anyhow::anyhow!("No transcription event received"))
        }
    )
    .await
    .unwrap()
    .unwrap();
    
    manager.stop().await.unwrap();
}

/// Test concurrent storage access
#[cfg(feature = "storage")]
#[tokio::test]
async fn test_concurrent_storage_access() {
    use tempfile::tempdir;
    
    let temp_dir = tempdir().unwrap();
    let config = DatabaseConfig {
        path: temp_dir.path().join("test.db"),
        ..Default::default()
    };
    
    let storage_manager = Arc::new(StorageManager::new(config).await.unwrap());
    
    // Spawn multiple tasks that access storage concurrently
    let mut handles = vec![];
    
    for i in 0..10 {
        let storage = storage_manager.clone();
        let handle = tokio::spawn(async move {
            let transcript = Transcript::new(
                format!("session-{}", i),
                format!("Concurrent transcript {}", i),
            );
            storage.save_transcript(transcript).await
        });
        handles.push(handle);
    }
    
    // Wait for all saves to complete
    let results: Vec<_> = futures::future::join_all(handles).await;
    
    // Verify all saves succeeded
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
    
    // Verify all transcripts were saved
    let all_transcripts = storage_manager.list_transcripts(20, 0, None).await.unwrap();
    assert_eq!(all_transcripts.len(), 10);
}

/// End-to-end test: Audio -> Transcription -> Command -> Storage
#[cfg(all(feature = "audio", feature = "transcribe", feature = "storage"))]
#[tokio::test]
async fn test_end_to_end_workflow() {
    use tempfile::tempdir;
    
    // Setup storage
    let temp_dir = tempdir().unwrap();
    let storage_config = DatabaseConfig {
        path: temp_dir.path().join("test.db"),
        ..Default::default()
    };
    let storage_manager = Arc::new(StorageManager::new(storage_config).await.unwrap());
    
    // Setup transcription with voice commands and storage
    let mut settings = SpeechSettings::default();
    settings.save_transcription = true;
    
    let (mut manager, mut receiver) = TranscriptionManager::new(settings).unwrap();
    
    // Enable voice commands
    let voice_config = bestme::config::VoiceCommandConfig::default();
    let (processor, _) = VoiceCommandProcessor::new(voice_config).unwrap();
    manager.enable_voice_commands(Arc::new(Mutex::new(processor)));
    
    // Set storage manager
    manager.set_storage_manager(storage_manager.clone());
    
    // Start session
    let session_id = storage_manager.start_session(Some("Test Device".to_string())).await.unwrap();
    
    // Initialize and start
    manager.initialize().await.unwrap();
    manager.start().await.unwrap();
    
    // Simulate audio processing that results in transcription
    let audio_data = vec![0.1f32; 48000]; // Non-silent audio
    
    // Process multiple chunks
    for _ in 0..3 {
        manager.process_audio(&audio_data).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    // Collect events
    let mut transcription_received = false;
    let mut command_received = false;
    
    let timeout = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        async {
            while let Some(event) = receiver.recv().await {
                match event {
                    TranscriptionEvent::Transcription(_) => transcription_received = true,
                    TranscriptionEvent::CommandExecuted { .. } => command_received = true,
                    _ => {}
                }
                
                if transcription_received {
                    break;
                }
            }
        }
    ).await;
    
    assert!(timeout.is_ok(), "Should receive events within timeout");
    assert!(transcription_received, "Should receive at least one transcription");
    
    // Stop and verify storage
    manager.stop().await.unwrap();
    storage_manager.end_session().await.unwrap();
    
    // Check that transcripts were saved
    let transcripts = storage_manager.list_transcripts(10, 0, Some(&session_id)).await.unwrap();
    assert!(!transcripts.is_empty(), "Should have saved transcripts");
    
    // Verify session was properly ended
    let sessions = storage_manager.list_transcripts(10, 0, None).await.unwrap();
    assert!(!sessions.is_empty());
}