//! Storage module for managing transcript persistence
//! 
//! This module provides functionality for:
//! - Saving transcripts with metadata
//! - Retrieving transcripts by various criteria
//! - Full-text search across transcripts
//! - Export functionality

use anyhow::Result;

#[cfg(feature = "storage")]
mod database;
#[cfg(feature = "storage")]
mod models;
#[cfg(feature = "storage")]
mod operations;

#[cfg(feature = "storage")]
pub use database::{Database, DatabaseConfig};
#[cfg(feature = "storage")]
pub use models::{Transcript, TranscriptMetadata, SearchQuery, SearchResult};
#[cfg(feature = "storage")]
pub use operations::{StorageManager, StorageOperations};

// Re-export error types
#[cfg(feature = "storage")]
pub use database::StorageError;

/// Initialize the storage system
#[cfg(feature = "storage")]
pub async fn initialize_storage(config: DatabaseConfig) -> Result<StorageManager> {
    StorageManager::new(config).await
}

#[cfg(not(feature = "storage"))]
pub async fn initialize_storage(_config: ()) -> Result<()> {
    log::warn!("Storage feature is not enabled");
    Ok(())
}