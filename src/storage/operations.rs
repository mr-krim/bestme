//! Storage operations for transcripts

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::{debug, info, warn};
use rusqlite::{params, OptionalExtension, Row};
use std::sync::{Arc, Mutex};

use super::database::{Database, DatabaseConfig, StorageError};
use super::models::*;

/// Storage manager for transcript operations
pub struct StorageManager {
    db: Database,
    current_session: Arc<Mutex<Option<Session>>>,
}

impl StorageManager {
    /// Create a new storage manager
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let db = Database::new(config)?;
        
        Ok(Self {
            db,
            current_session: Arc::new(Mutex::new(None)),
        })
    }
    
    /// Start a new recording session
    pub async fn start_session(&self, device_name: Option<String>) -> Result<String> {
        let mut session = Session::new();
        session.device_name = device_name;
        
        // Save to database
        self.save_session(&session)?;
        
        let session_id = session.id.clone();
        
        // Set as current session
        {
            let mut current = self.current_session.lock().unwrap();
            *current = Some(session);
        }
        
        info!("Started new session: {}", session_id);
        Ok(session_id)
    }
    
    /// End the current session
    pub async fn end_session(&self) -> Result<()> {
        let mut current = self.current_session.lock().unwrap();
        
        if let Some(mut session) = current.take() {
            session.end();
            self.update_session(&session)?;
            info!("Ended session: {}", session.id);
        } else {
            warn!("No active session to end");
        }
        
        Ok(())
    }
    
    /// Get the current session ID
    pub fn current_session_id(&self) -> Option<String> {
        let current = self.current_session.lock().unwrap();
        current.as_ref().map(|s| s.id.clone())
    }
    
    /// Save a transcript
    pub async fn save_transcript(&self, mut transcript: Transcript) -> Result<String> {
        // Use current session if no session ID provided
        if transcript.session_id.is_empty() {
            if let Some(session_id) = self.current_session_id() {
                transcript.session_id = session_id;
            } else {
                // Auto-create a session if none exists
                let session_id = self.start_session(None).await?;
                transcript.session_id = session_id;
            }
        }
        
        let transcript_id = transcript.id.clone();
        
        let conn = self.db.connection();
        let conn = conn.lock().unwrap();
        
        // Convert metadata to JSON
        let metadata_json = serde_json::to_string(&transcript.metadata)?;
        let tags_str = transcript.tags.join(",");
        
        conn.execute(
            "INSERT INTO transcripts (
                id, session_id, text, language, model_size,
                duration_seconds, word_count, created_at, updated_at,
                metadata, tags
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                &transcript.id,
                &transcript.session_id,
                &transcript.text,
                &transcript.language,
                &transcript.model_size,
                &transcript.duration_seconds,
                transcript.word_count as i64,
                transcript.created_at.to_rfc3339(),
                transcript.updated_at.to_rfc3339(),
                &metadata_json,
                &tags_str,
            ],
        ).context("Failed to insert transcript")?;
        
        // Update session transcript count
        self.increment_session_transcript_count(&transcript.session_id)?;
        
        debug!("Saved transcript: {}", transcript_id);
        Ok(transcript_id)
    }
    
    /// Update an existing transcript
    pub async fn update_transcript(&self, transcript: &Transcript) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().unwrap();
        
        let metadata_json = serde_json::to_string(&transcript.metadata)?;
        let tags_str = transcript.tags.join(",");
        
        let rows_affected = conn.execute(
            "UPDATE transcripts SET 
                text = ?1, language = ?2, model_size = ?3,
                duration_seconds = ?4, word_count = ?5, 
                updated_at = ?6, metadata = ?7, tags = ?8
             WHERE id = ?9",
            params![
                &transcript.text,
                &transcript.language,
                &transcript.model_size,
                &transcript.duration_seconds,
                transcript.word_count as i64,
                transcript.updated_at.to_rfc3339(),
                &metadata_json,
                &tags_str,
                &transcript.id,
            ],
        ).context("Failed to update transcript")?;
        
        if rows_affected == 0 {
            return Err(StorageError::NotFound(format!("Transcript not found: {}", transcript.id)).into());
        }
        
        debug!("Updated transcript: {}", transcript.id);
        Ok(())
    }
    
    /// Get a transcript by ID
    pub async fn get_transcript(&self, id: &str) -> Result<Option<Transcript>> {
        let conn = self.db.connection();
        let conn = conn.lock().unwrap();
        
        let result = conn.query_row(
            "SELECT id, session_id, text, language, model_size,
                    duration_seconds, word_count, created_at, updated_at,
                    metadata, tags
             FROM transcripts WHERE id = ?1",
            params![id],
            |row| Self::transcript_from_row(row),
        ).optional()?;
        
        Ok(result)
    }
    
    /// List transcripts with pagination
    pub async fn list_transcripts(
        &self, 
        limit: usize, 
        offset: usize,
        session_id: Option<&str>,
    ) -> Result<Vec<Transcript>> {
        let conn = self.db.connection();
        let conn = conn.lock().unwrap();
        
        let mut query = "SELECT id, session_id, text, language, model_size,
                                duration_seconds, word_count, created_at, updated_at,
                                metadata, tags
                         FROM transcripts".to_string();
        
        let mut params_vec: Vec<String> = vec![];
        
        if let Some(sid) = session_id {
            query.push_str(" WHERE session_id = ?1");
            params_vec.push(sid.to_string());
        }
        
        query.push_str(" ORDER BY created_at DESC LIMIT ?2 OFFSET ?3");
        
        let mut stmt = conn.prepare(&query)?;
        
        let results: Result<Vec<_>, _> = if session_id.is_some() {
            stmt.query_map(
                params![&params_vec[0], limit as i64, offset as i64],
                |row| Self::transcript_from_row(row),
            )?.collect()
        } else {
            stmt.query_map(
                params![limit as i64, offset as i64],
                |row| Self::transcript_from_row(row),
            )?.collect()
        };
        
        Ok(results?)
    }
    
    /// Search transcripts using full-text search
    pub async fn search_transcripts(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let conn = self.db.connection();
        let conn = conn.lock().unwrap();
        
        let mut sql = String::from(
            "SELECT t.id, t.session_id, t.text, t.language, t.model_size,
                    t.duration_seconds, t.word_count, t.created_at, t.updated_at,
                    t.metadata, t.tags, 
                    bm25(transcripts_fts) as score,
                    snippet(transcripts_fts, 1, '<mark>', '</mark>', '...', 32) as snippet
             FROM transcripts t"
        );
        
        let mut conditions = vec![];
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];
        
        // Add FTS search if query text provided
        if !query.query.is_empty() {
            sql.push_str(" JOIN transcripts_fts ON t.id = transcripts_fts.id");
            conditions.push("transcripts_fts MATCH ?");
            params.push(Box::new(query.query.clone()));
        }
        
        // Add other filters
        if let Some(ref session_id) = query.session_id {
            conditions.push("t.session_id = ?");
            params.push(Box::new(session_id.clone()));
        }
        
        if let Some(ref language) = query.language {
            conditions.push("t.language = ?");
            params.push(Box::new(language.clone()));
        }
        
        if let Some(ref date_from) = query.date_from {
            conditions.push("t.created_at >= ?");
            params.push(Box::new(date_from.to_rfc3339()));
        }
        
        if let Some(ref date_to) = query.date_to {
            conditions.push("t.created_at <= ?");
            params.push(Box::new(date_to.to_rfc3339()));
        }
        
        // Add tag filters
        if !query.tags.is_empty() {
            for tag in &query.tags {
                conditions.push("t.tags LIKE ?");
                params.push(Box::new(format!("%{}%", tag)));
            }
        }
        
        // Build WHERE clause
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        
        // Add ordering
        match query.sort_by {
            SortOrder::DateAsc => sql.push_str(" ORDER BY t.created_at ASC"),
            SortOrder::DateDesc => sql.push_str(" ORDER BY t.created_at DESC"),
            SortOrder::Relevance if !query.query.is_empty() => {
                sql.push_str(" ORDER BY score DESC")
            },
            SortOrder::WordCount => sql.push_str(" ORDER BY t.word_count DESC"),
            _ => sql.push_str(" ORDER BY t.created_at DESC"),
        }
        
        // Add limit and offset
        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        
        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }
        
        debug!("Search SQL: {}", sql);
        
        let mut stmt = conn.prepare(&sql)?;
        
        // Execute query with dynamic parameters
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter()
            .map(|p| p.as_ref() as &dyn rusqlite::ToSql)
            .collect();
        
        let results = stmt.query_map(&param_refs[..], |row| {
            let transcript = Self::transcript_from_row(row)?;
            
            let score: Option<f64> = if !query.query.is_empty() {
                row.get(11).ok()
            } else {
                None
            };
            
            let snippet: Option<String> = if !query.query.is_empty() {
                row.get(12).ok()
            } else {
                None
            };
            
            Ok(SearchResult {
                transcript,
                score,
                snippet,
            })
        })?;
        
        let search_results: Result<Vec<_>, _> = results.collect();
        Ok(search_results?)
    }
    
    /// Delete a transcript
    pub async fn delete_transcript(&self, id: &str) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().unwrap();
        
        let rows_affected = conn.execute(
            "DELETE FROM transcripts WHERE id = ?1",
            params![id],
        )?;
        
        if rows_affected == 0 {
            return Err(StorageError::NotFound(format!("Transcript not found: {}", id)).into());
        }
        
        info!("Deleted transcript: {}", id);
        Ok(())
    }
    
    /// Export transcripts in the specified format
    pub async fn export_transcripts(
        &self,
        transcript_ids: &[String],
        format: ExportFormat,
    ) -> Result<String> {
        let mut transcripts = vec![];
        
        for id in transcript_ids {
            if let Some(transcript) = self.get_transcript(id).await? {
                transcripts.push(transcript);
            }
        }
        
        match format {
            ExportFormat::Text => self.export_as_text(&transcripts),
            ExportFormat::Markdown => self.export_as_markdown(&transcripts),
            ExportFormat::Json => self.export_as_json(&transcripts),
            ExportFormat::Csv => self.export_as_csv(&transcripts),
        }
    }
    
    // Helper methods
    
    fn transcript_from_row(row: &Row) -> rusqlite::Result<Transcript> {
        let metadata_json: String = row.get(9)?;
        let metadata: TranscriptMetadata = serde_json::from_str(&metadata_json)
            .unwrap_or_default();
        
        let tags_str: String = row.get(10)?;
        let tags: Vec<String> = if tags_str.is_empty() {
            vec![]
        } else {
            tags_str.split(',').map(|s| s.to_string()).collect()
        };
        
        Ok(Transcript {
            id: row.get(0)?,
            session_id: row.get(1)?,
            text: row.get(2)?,
            language: row.get(3)?,
            model_size: row.get(4)?,
            duration_seconds: row.get(5)?,
            word_count: row.get::<_, i64>(6)? as usize,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                .unwrap()
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                .unwrap()
                .with_timezone(&Utc),
            metadata,
            tags,
        })
    }
    
    fn save_session(&self, session: &Session) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().unwrap();
        
        conn.execute(
            "INSERT INTO sessions (
                id, name, device_name, started_at, ended_at,
                total_duration_seconds, transcript_count
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &session.id,
                &session.name,
                &session.device_name,
                session.started_at.to_rfc3339(),
                session.ended_at.as_ref().map(|d| d.to_rfc3339()),
                session.total_duration_seconds,
                session.transcript_count as i64,
            ],
        )?;
        
        Ok(())
    }
    
    fn update_session(&self, session: &Session) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().unwrap();
        
        conn.execute(
            "UPDATE sessions SET 
                name = ?1, ended_at = ?2, total_duration_seconds = ?3
             WHERE id = ?4",
            params![
                &session.name,
                session.ended_at.as_ref().map(|d| d.to_rfc3339()),
                session.total_duration_seconds,
                &session.id,
            ],
        )?;
        
        Ok(())
    }
    
    fn increment_session_transcript_count(&self, session_id: &str) -> Result<()> {
        let conn = self.db.connection();
        let conn = conn.lock().unwrap();
        
        conn.execute(
            "UPDATE sessions SET transcript_count = transcript_count + 1 WHERE id = ?1",
            params![session_id],
        )?;
        
        Ok(())
    }
    
    fn export_as_text(&self, transcripts: &[Transcript]) -> Result<String> {
        let mut output = String::new();
        
        for transcript in transcripts {
            output.push_str(&format!(
                "=== Transcript from {} ===\n{}\n\n",
                transcript.created_at.format("%Y-%m-%d %H:%M:%S"),
                transcript.text
            ));
        }
        
        Ok(output)
    }
    
    fn export_as_markdown(&self, transcripts: &[Transcript]) -> Result<String> {
        let mut output = String::new();
        
        output.push_str("# Transcripts Export\n\n");
        
        for transcript in transcripts {
            output.push_str(&format!(
                "## {}\n\n**Language:** {}\n**Model:** {}\n**Words:** {}\n\n{}\n\n---\n\n",
                transcript.created_at.format("%Y-%m-%d %H:%M:%S"),
                transcript.language.as_deref().unwrap_or("Unknown"),
                transcript.model_size.as_deref().unwrap_or("Unknown"),
                transcript.word_count,
                transcript.text
            ));
        }
        
        Ok(output)
    }
    
    fn export_as_json(&self, transcripts: &[Transcript]) -> Result<String> {
        Ok(serde_json::to_string_pretty(transcripts)?)
    }
    
    fn export_as_csv(&self, transcripts: &[Transcript]) -> Result<String> {
        let mut output = String::new();
        
        // Header
        output.push_str("ID,Session ID,Created At,Language,Model,Word Count,Text\n");
        
        // Data rows
        for transcript in transcripts {
            output.push_str(&format!(
                r#""{}","{}","{}","{}","{}",{},"{}"{}"#,
                transcript.id,
                transcript.session_id,
                transcript.created_at.to_rfc3339(),
                transcript.language.as_deref().unwrap_or(""),
                transcript.model_size.as_deref().unwrap_or(""),
                transcript.word_count,
                transcript.text.replace("\"", "\"\""),
                "\n"
            ));
        }
        
        Ok(output)
    }
}

/// Trait for storage operations (for testing/mocking)
pub trait StorageOperations {
    fn save_transcript(&self, transcript: Transcript) -> impl std::future::Future<Output = Result<String>> + Send;
    fn get_transcript(&self, id: &str) -> impl std::future::Future<Output = Result<Option<Transcript>>> + Send;
    fn search_transcripts(&self, query: &SearchQuery) -> impl std::future::Future<Output = Result<Vec<SearchResult>>> + Send;
    fn delete_transcript(&self, id: &str) -> impl std::future::Future<Output = Result<()>> + Send;
}

impl StorageOperations for StorageManager {
    async fn save_transcript(&self, transcript: Transcript) -> Result<String> {
        self.save_transcript(transcript).await
    }
    
    async fn get_transcript(&self, id: &str) -> Result<Option<Transcript>> {
        self.get_transcript(id).await
    }
    
    async fn search_transcripts(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        self.search_transcripts(query).await
    }
    
    async fn delete_transcript(&self, id: &str) -> Result<()> {
        self.delete_transcript(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    async fn create_test_manager() -> Result<StorageManager> {
        let temp_dir = tempdir()?;
        let config = DatabaseConfig {
            path: temp_dir.path().join("test.db"),
            ..Default::default()
        };
        
        StorageManager::new(config).await
    }
    
    #[tokio::test]
    async fn test_session_lifecycle() {
        let manager = create_test_manager().await.unwrap();
        
        // Start session
        let session_id = manager.start_session(Some("TestDevice".to_string())).await.unwrap();
        assert!(!session_id.is_empty());
        assert_eq!(manager.current_session_id(), Some(session_id.clone()));
        
        // End session
        manager.end_session().await.unwrap();
        assert_eq!(manager.current_session_id(), None);
    }
    
    #[tokio::test]
    async fn test_transcript_crud() {
        let manager = create_test_manager().await.unwrap();
        
        // Create transcript
        let mut transcript = Transcript::new(
            "test-session".to_string(),
            "This is a test transcript.".to_string(),
        );
        transcript.language = Some("en".to_string());
        
        let id = manager.save_transcript(transcript.clone()).await.unwrap();
        assert_eq!(id, transcript.id);
        
        // Read transcript
        let loaded = manager.get_transcript(&id).await.unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.text, transcript.text);
        assert_eq!(loaded.word_count, 5);
        
        // Update transcript
        transcript.update_text("This is an updated test transcript.".to_string());
        manager.update_transcript(&transcript).await.unwrap();
        
        let updated = manager.get_transcript(&id).await.unwrap().unwrap();
        assert_eq!(updated.text, transcript.text);
        assert_eq!(updated.word_count, 6);
        
        // Delete transcript
        manager.delete_transcript(&id).await.unwrap();
        let deleted = manager.get_transcript(&id).await.unwrap();
        assert!(deleted.is_none());
    }
    
    #[tokio::test]
    async fn test_search() {
        let manager = create_test_manager().await.unwrap();
        
        // Create test transcripts
        let transcripts = vec![
            Transcript::new("session1".to_string(), "Hello world from Rust".to_string()),
            Transcript::new("session1".to_string(), "Testing the search functionality".to_string()),
            Transcript::new("session2".to_string(), "Another session with Rust code".to_string()),
        ];
        
        for transcript in &transcripts {
            manager.save_transcript(transcript.clone()).await.unwrap();
        }
        
        // Search for "Rust"
        let query = SearchQuery {
            query: "Rust".to_string(),
            ..Default::default()
        };
        
        let results = manager.search_transcripts(&query).await.unwrap();
        assert_eq!(results.len(), 2);
        
        // Search by session
        let query = SearchQuery {
            session_id: Some("session1".to_string()),
            ..Default::default()
        };
        
        let results = manager.search_transcripts(&query).await.unwrap();
        assert_eq!(results.len(), 2);
    }
}