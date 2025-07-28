//! Data models for storage

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Transcript model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    /// Unique identifier
    pub id: String,
    
    /// Session this transcript belongs to
    pub session_id: String,
    
    /// The transcribed text
    pub text: String,
    
    /// Language of the transcript
    pub language: Option<String>,
    
    /// Whisper model size used
    pub model_size: Option<String>,
    
    /// Duration of the audio in seconds
    pub duration_seconds: Option<f64>,
    
    /// Word count
    pub word_count: usize,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Additional metadata
    pub metadata: TranscriptMetadata,
    
    /// Tags for categorization
    pub tags: Vec<String>,
}

impl Transcript {
    /// Create a new transcript
    pub fn new(session_id: String, text: String) -> Self {
        let word_count = text.split_whitespace().count();
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4().to_string(),
            session_id,
            text,
            language: None,
            model_size: None,
            duration_seconds: None,
            word_count,
            created_at: now,
            updated_at: now,
            metadata: TranscriptMetadata::default(),
            tags: Vec::new(),
        }
    }
    
    /// Update the transcript text
    pub fn update_text(&mut self, text: String) {
        self.text = text;
        self.word_count = self.text.split_whitespace().count();
        self.updated_at = Utc::now();
    }
    
    /// Add a tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }
    
    /// Remove a tag
    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.updated_at = Utc::now();
        }
    }
}

/// Additional metadata for transcripts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranscriptMetadata {
    /// Device used for recording
    pub device_name: Option<String>,
    
    /// Audio sample rate
    pub sample_rate: Option<u32>,
    
    /// Whether translation was enabled
    pub translated: bool,
    
    /// Whether auto-punctuation was enabled
    pub auto_punctuated: bool,
    
    /// Custom user notes
    pub notes: Option<String>,
    
    /// Voice commands detected count
    pub voice_commands_count: usize,
    
    /// Any errors during transcription
    pub errors: Vec<String>,
}

/// Recording session model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique identifier
    pub id: String,
    
    /// Optional session name
    pub name: Option<String>,
    
    /// Device used for the session
    pub device_name: Option<String>,
    
    /// Session start time
    pub started_at: DateTime<Utc>,
    
    /// Session end time
    pub ended_at: Option<DateTime<Utc>>,
    
    /// Total duration in seconds
    pub total_duration_seconds: f64,
    
    /// Number of transcripts in this session
    pub transcript_count: usize,
}

impl Session {
    /// Create a new session
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: None,
            device_name: None,
            started_at: Utc::now(),
            ended_at: None,
            total_duration_seconds: 0.0,
            transcript_count: 0,
        }
    }
    
    /// End the session
    pub fn end(&mut self) {
        self.ended_at = Some(Utc::now());
        if let Some(ended) = self.ended_at {
            self.total_duration_seconds = (ended - self.started_at).num_seconds() as f64;
        }
    }
    
    /// Check if session is active
    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        self.ended_at.is_none()
    }
}

/// Search query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Text to search for
    pub query: String,
    
    /// Filter by session ID
    pub session_id: Option<String>,
    
    /// Filter by language
    pub language: Option<String>,
    
    /// Filter by tags
    pub tags: Vec<String>,
    
    /// Date range start
    pub date_from: Option<DateTime<Utc>>,
    
    /// Date range end
    pub date_to: Option<DateTime<Utc>>,
    
    /// Maximum number of results
    pub limit: Option<usize>,
    
    /// Offset for pagination
    pub offset: Option<usize>,
    
    /// Sort order
    pub sort_by: SortOrder,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            session_id: None,
            language: None,
            tags: Vec::new(),
            date_from: None,
            date_to: None,
            limit: Some(50),
            offset: None,
            sort_by: SortOrder::DateDesc,
        }
    }
}

/// Sort order for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    DateAsc,
    DateDesc,
    Relevance,
    WordCount,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The transcript
    pub transcript: Transcript,
    
    /// Relevance score (for full-text search)
    pub score: Option<f64>,
    
    /// Highlighted snippet
    pub snippet: Option<String>,
}

/// Export format options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExportFormat {
    /// Plain text
    Text,
    
    /// Markdown
    Markdown,
    
    /// JSON
    Json,
    
    /// CSV
    Csv,
}

impl ExportFormat {
    /// Get file extension for the format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Text => "txt",
            Self::Markdown => "md",
            Self::Json => "json",
            Self::Csv => "csv",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transcript_creation() {
        let transcript = Transcript::new(
            "session-123".to_string(),
            "Hello world, this is a test.".to_string(),
        );
        
        assert_eq!(transcript.word_count, 6);
        assert_eq!(transcript.session_id, "session-123");
        assert!(transcript.tags.is_empty());
    }
    
    #[test]
    fn test_transcript_update() {
        let mut transcript = Transcript::new(
            "session-123".to_string(),
            "Hello world.".to_string(),
        );
        
        let original_updated = transcript.updated_at;
        
        // Sleep briefly to ensure time difference
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        transcript.update_text("Hello world, how are you?".to_string());
        
        assert_eq!(transcript.word_count, 5);
        assert!(transcript.updated_at > original_updated);
    }
    
    #[test]
    fn test_session_lifecycle() {
        let mut session = Session::new();
        assert!(session.is_active());
        
        session.end();
        assert!(!session.is_active());
        assert!(session.ended_at.is_some());
        assert!(session.total_duration_seconds >= 0.0);
    }
    
    #[test]
    fn test_tag_management() {
        let mut transcript = Transcript::new(
            "session-123".to_string(),
            "Test transcript".to_string(),
        );
        
        transcript.add_tag("important".to_string());
        transcript.add_tag("meeting".to_string());
        transcript.add_tag("important".to_string()); // Duplicate
        
        assert_eq!(transcript.tags.len(), 2);
        assert!(transcript.tags.contains(&"important".to_string()));
        
        transcript.remove_tag("meeting");
        assert_eq!(transcript.tags.len(), 1);
        assert!(!transcript.tags.contains(&"meeting".to_string()));
    }
}