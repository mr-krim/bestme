//! Storage plugin for managing transcripts with SQLite

use anyhow::{Context, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{
    plugin::Plugin,
    AppHandle, Manager, Runtime, State,
};
use tokio::sync::Mutex;

use bestme::storage::{
    StorageManager, DatabaseConfig, Transcript, SearchQuery, ExportFormat
};

/// Storage state for the Tauri plugin
pub struct StorageState {
    manager: Option<Arc<StorageManager>>,
}

impl StorageState {
    pub fn new() -> Self {
        Self {
            manager: None,
        }
    }
    
    pub async fn initialize<R: Runtime>(&mut self, app_handle: &AppHandle<R>) -> Result<()> {
        // Get app data directory
        let app_data_dir = app_handle.path()
            .app_data_dir()
            .context("Failed to get app data directory")?;
        
        let db_path = app_data_dir.join("transcripts.db");
        
        let config = DatabaseConfig {
            path: db_path,
            ..Default::default()
        };
        
        match StorageManager::new(config).await {
            Ok(manager) => {
                self.manager = Some(Arc::new(manager));
                info!("Storage system initialized successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to initialize storage: {}", e);
                Err(e)
            }
        }
    }
    
    pub fn get_manager(&self) -> Option<Arc<StorageManager>> {
        self.manager.clone()
    }
}

/// List item for saved transcripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedTranscriptListItem {
    pub id: String,
    pub title: String,
    pub date: String, // ISO 8601 format
    pub word_count: usize,
    pub language: Option<String>,
}

/// Full transcript content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedTranscriptContent {
    pub id: String,
    pub title: String,
    pub content: String,
    pub timestamp_ms: i64,
    pub word_count: usize,
    pub language: Option<String>,
    pub model_size: Option<String>,
    pub tags: Vec<String>,
}

/// List saved transcripts from SQLite storage
#[tauri::command]
pub async fn list_saved_transcripts_db(
    state: State<'_, Arc<Mutex<StorageState>>>,
    limit: Option<usize>,
    session_id: Option<String>,
) -> Result<Vec<SavedTranscriptListItem>, String> {
    let state = state.lock().await;
    
    if let Some(manager) = state.get_manager() {
        let transcripts = manager.list_transcripts(
            limit.unwrap_or(50),
            0,
            session_id.as_deref(),
        ).await.map_err(|e| e.to_string())?;
        
        let items: Vec<SavedTranscriptListItem> = transcripts.into_iter()
            .map(|t| SavedTranscriptListItem {
                id: t.id,
                title: t.text.lines().next().unwrap_or("Untitled").to_string(),
                date: t.created_at.to_rfc3339(),
                word_count: t.word_count,
                language: t.language,
            })
            .collect();
        
        Ok(items)
    } else {
        Err("Storage system not initialized".to_string())
    }
}

/// Get a specific saved transcript from SQLite
#[tauri::command]
pub async fn get_saved_transcript_db(
    state: State<'_, Arc<Mutex<StorageState>>>,
    id: String,
) -> Result<SavedTranscriptContent, String> {
    let state = state.lock().await;
    
    if let Some(manager) = state.get_manager() {
        let transcript = manager.get_transcript(&id).await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Transcript not found: {}", id))?;
        
        Ok(SavedTranscriptContent {
            id: transcript.id,
            title: transcript.text.lines().next().unwrap_or("Untitled").to_string(),
            content: transcript.text,
            timestamp_ms: transcript.created_at.timestamp_millis(),
            word_count: transcript.word_count,
            language: transcript.language,
            model_size: transcript.model_size,
            tags: transcript.tags,
        })
    } else {
        Err("Storage system not initialized".to_string())
    }
}

/// Save a new transcript to SQLite
#[tauri::command]
pub async fn save_transcript_db(
    state: State<'_, Arc<Mutex<StorageState>>>,
    title: String,
    content: String,
) -> Result<String, String> {
    let state = state.lock().await;
    
    if let Some(manager) = state.get_manager() {
        // Start a new session if needed
        let session_id = manager.current_session_id()
            .unwrap_or_else(|| {
                // Create a new session
                let rt = tokio::runtime::Handle::current();
                rt.block_on(async {
                    manager.start_session(None).await.unwrap_or_default()
                })
            });
        
        // Create transcript
        let mut transcript = Transcript::new(session_id, content.clone());
        
        // Use title as the first line if provided
        if !title.is_empty() {
            transcript.text = format!("{}\n\n{}", title, content);
        }
        
        // Save transcript
        let id = manager.save_transcript(transcript).await
            .map_err(|e| e.to_string())?;
        
        info!("Saved transcript: {}", id);
        Ok(id)
    } else {
        Err("Storage system not initialized".to_string())
    }
}

/// Delete a saved transcript from SQLite
#[tauri::command]
pub async fn delete_saved_transcript_db(
    state: State<'_, Arc<Mutex<StorageState>>>,
    id: String,
) -> Result<(), String> {
    let state = state.lock().await;
    
    if let Some(manager) = state.get_manager() {
        manager.delete_transcript(&id).await
            .map_err(|e| e.to_string())?;
        
        info!("Deleted transcript: {}", id);
        Ok(())
    } else {
        Err("Storage system not initialized".to_string())
    }
}

/// Search transcripts in SQLite
#[tauri::command]
pub async fn search_transcripts_db(
    state: State<'_, Arc<Mutex<StorageState>>>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<SavedTranscriptListItem>, String> {
    let state = state.lock().await;
    
    if let Some(manager) = state.get_manager() {
        let search_query = SearchQuery {
            query,
            limit,
            ..Default::default()
        };
        
        let results = manager.search_transcripts(&search_query).await
            .map_err(|e| e.to_string())?;
        
        let items: Vec<SavedTranscriptListItem> = results.into_iter()
            .map(|r| SavedTranscriptListItem {
                id: r.transcript.id,
                title: r.transcript.text.lines().next().unwrap_or("Untitled").to_string(),
                date: r.transcript.created_at.to_rfc3339(),
                word_count: r.transcript.word_count,
                language: r.transcript.language,
            })
            .collect();
        
        Ok(items)
    } else {
        Err("Storage system not initialized".to_string())
    }
}

/// Export transcripts from SQLite
#[tauri::command]
pub async fn export_transcripts_db(
    state: State<'_, Arc<Mutex<StorageState>>>,
    ids: Vec<String>,
    format: String,
) -> Result<String, String> {
    let state = state.lock().await;
    
    if let Some(manager) = state.get_manager() {
        let export_format = match format.as_str() {
            "text" => ExportFormat::Text,
            "markdown" => ExportFormat::Markdown,
            "json" => ExportFormat::Json,
            "csv" => ExportFormat::Csv,
            _ => return Err("Invalid export format".to_string()),
        };
        
        let content = manager.export_transcripts(&ids, export_format).await
            .map_err(|e| e.to_string())?;
        
        Ok(content)
    } else {
        Err("Storage system not initialized".to_string())
    }
}

/// Storage plugin for Tauri 2.0
#[derive(Default)]
pub struct StoragePlugin<R: Runtime> {
    _phantom: std::marker::PhantomData<fn() -> R>,
}

impl<R: Runtime> StoragePlugin<R> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<R: Runtime> Plugin<R> for StoragePlugin<R> {
    fn name(&self) -> &'static str {
        "storage"
    }
    
    fn initialize(&mut self, app: &AppHandle<R>, _: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        let storage_state = Arc::new(Mutex::new(StorageState::new()));
        
        // Initialize storage in background
        let app_handle = app.clone();
        let state_clone = storage_state.clone();
        tauri::async_runtime::spawn(async move {
            let mut state = state_clone.lock().await;
            if let Err(e) = state.initialize(&app_handle).await {
                error!("Failed to initialize storage plugin: {}", e);
            } else {
                info!("Storage plugin initialized successfully");
            }
        });
        
        app.manage(storage_state);
        
        Ok(())
    }
}