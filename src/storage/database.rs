//! Database connection and initialization

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use log::{info, debug};

/// Storage-specific error types
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Path to the database file
    pub path: PathBuf,
    
    /// Enable Write-Ahead Logging for better performance
    pub enable_wal: bool,
    
    /// Enable foreign key constraints
    pub enable_foreign_keys: bool,
    
    /// Maximum database size in MB (0 = unlimited)
    pub max_size_mb: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("transcripts.db"),
            enable_wal: true,
            enable_foreign_keys: true,
            max_size_mb: 1024, // 1GB default
        }
    }
}

/// Database connection wrapper
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    config: DatabaseConfig,
}

impl Database {
    /// Create a new database connection
    pub fn new(config: DatabaseConfig) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = config.path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create database directory")?;
        }
        
        // Open connection
        let conn = Connection::open(&config.path)
            .context("Failed to open database")?;
        
        // Configure database
        if config.enable_wal {
            conn.pragma_update(None, "journal_mode", "WAL")
                .context("Failed to enable WAL mode")?;
        }
        
        if config.enable_foreign_keys {
            conn.pragma_update(None, "foreign_keys", "ON")
                .context("Failed to enable foreign keys")?;
        }
        
        // Set other pragmas for performance
        conn.pragma_update(None, "synchronous", "NORMAL")
            .context("Failed to set synchronous mode")?;
        conn.pragma_update(None, "cache_size", "-64000")  // 64MB cache
            .context("Failed to set cache size")?;
        conn.pragma_update(None, "temp_store", "MEMORY")
            .context("Failed to set temp store")?;
        
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            config: config.clone(),
        };
        
        // Initialize schema
        db.initialize_schema()?;
        
        info!("Database initialized at: {:?}", config.path);
        
        Ok(db)
    }
    
    /// Initialize database schema
    fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        // Create transcripts table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS transcripts (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                text TEXT NOT NULL,
                language TEXT,
                model_size TEXT,
                duration_seconds REAL,
                word_count INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                metadata TEXT, -- JSON field for additional metadata
                tags TEXT -- Comma-separated tags
            )",
            [],
        ).context("Failed to create transcripts table")?;
        
        // Create indices for common queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transcripts_session_id 
             ON transcripts(session_id)",
            [],
        ).context("Failed to create session_id index")?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_transcripts_created_at 
             ON transcripts(created_at DESC)",
            [],
        ).context("Failed to create created_at index")?;
        
        // Create FTS5 virtual table for full-text search
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS transcripts_fts USING fts5(
                id UNINDEXED,
                text,
                tags,
                content=transcripts,
                content_rowid=rowid
            )",
            [],
        ).context("Failed to create FTS table")?;
        
        // Create triggers to keep FTS in sync
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS transcripts_ai AFTER INSERT ON transcripts
            BEGIN
                INSERT INTO transcripts_fts(id, text, tags) 
                VALUES (new.id, new.text, new.tags);
            END",
            [],
        ).context("Failed to create insert trigger")?;
        
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS transcripts_ad AFTER DELETE ON transcripts
            BEGIN
                DELETE FROM transcripts_fts WHERE id = old.id;
            END",
            [],
        ).context("Failed to create delete trigger")?;
        
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS transcripts_au AFTER UPDATE ON transcripts
            BEGIN
                UPDATE transcripts_fts 
                SET text = new.text, tags = new.tags 
                WHERE id = new.id;
            END",
            [],
        ).context("Failed to create update trigger")?;
        
        // Create sessions table for grouping transcripts
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                name TEXT,
                device_name TEXT,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                total_duration_seconds REAL,
                transcript_count INTEGER DEFAULT 0
            )",
            [],
        ).context("Failed to create sessions table")?;
        
        debug!("Database schema initialized");
        
        Ok(())
    }
    
    /// Get database connection for operations
    pub fn connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }
    
    /// Run database optimization
    pub fn optimize(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        // Optimize the database
        conn.pragma_update(None, "optimize", "")
            .context("Failed to optimize database")?;
        
        // Vacuum if needed (this can be slow for large databases)
        if self.should_vacuum()? {
            info!("Running database vacuum...");
            conn.execute("VACUUM", [])
                .context("Failed to vacuum database")?;
        }
        
        Ok(())
    }
    
    /// Check if vacuum is needed
    fn should_vacuum(&self) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        
        // Get page count and freelist count
        let page_count: i64 = conn.pragma_query_value(None, "page_count", |row| row.get(0))?;
        
        let freelist_count: i64 = conn.pragma_query_value(None, "freelist_count", |row| row.get(0))?;
        
        // Vacuum if more than 20% of pages are free
        let should_vacuum = freelist_count as f64 / page_count as f64 > 0.2;
        
        debug!("Page count: {}, Freelist count: {}, Should vacuum: {}", 
               page_count, freelist_count, should_vacuum);
        
        Ok(should_vacuum)
    }
    
    /// Get database statistics
    pub fn get_stats(&self) -> Result<DatabaseStats> {
        let conn = self.conn.lock().unwrap();
        
        let transcript_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transcripts",
            [],
            |row| row.get(0),
        )?;
        
        let session_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sessions",
            [],
            |row| row.get(0),
        )?;
        
        let total_words: i64 = conn.query_row(
            "SELECT COALESCE(SUM(word_count), 0) FROM transcripts",
            [],
            |row| row.get(0),
        )?;
        
        let db_size = std::fs::metadata(&self.config.path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        Ok(DatabaseStats {
            transcript_count,
            session_count,
            total_words,
            database_size_bytes: db_size,
        })
    }
}

/// Database statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct DatabaseStats {
    pub transcript_count: i64,
    pub session_count: i64,
    pub total_words: i64,
    pub database_size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_database_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let config = DatabaseConfig {
            path: db_path.clone(),
            ..Default::default()
        };
        
        let db = Database::new(config).unwrap();
        assert!(db_path.exists());
        
        // Test stats
        let stats = db.get_stats().unwrap();
        assert_eq!(stats.transcript_count, 0);
        assert_eq!(stats.session_count, 0);
    }
    
    #[test]
    fn test_schema_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let config = DatabaseConfig {
            path: db_path,
            ..Default::default()
        };
        
        let db = Database::new(config).unwrap();
        let conn = db.connection();
        let conn = conn.lock().unwrap();
        
        // Check if tables exist
        let table_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master 
             WHERE type='table' AND name IN ('transcripts', 'sessions')",
            [],
            |row| row.get(0),
        ).unwrap();
        
        assert_eq!(table_count, 2);
    }
}