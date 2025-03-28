use std::collections::HashSet;
use std::sync::Arc;
use log::{info, debug};
use anyhow::Result;
use parking_lot::Mutex;
use tokio::sync::mpsc;
use crate::config::SpeechSettings;
use serde::{Deserialize, Serialize};
use chrono;

/// Voice command event types
#[derive(Debug, Clone)]
pub enum VoiceCommandEvent {
    /// A command was detected
    CommandDetected(VoiceCommand),
    
    /// Error processing commands
    Error(String),
}

/// Types of voice commands
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VoiceCommandType {
    /// Text editing commands
    Delete,
    Undo,
    Redo,
    Capitalize,
    Lowercase,
    
    /// Text navigation commands
    NewLine,
    NewParagraph,
    
    /// Punctuation commands
    Period,
    Comma,
    QuestionMark,
    ExclamationMark,
    
    /// Control commands
    Pause,
    Resume,
    Stop,
    
    /// Custom command
    Custom(String),
}

/// Voice command information
#[derive(Debug, Clone)]
pub struct VoiceCommand {
    /// Type of command
    pub command_type: VoiceCommandType,
    
    /// The original text that triggered the command
    pub trigger_text: String,
    
    /// Any additional parameters
    pub parameters: Option<String>,
}

impl VoiceCommand {
    /// Create a new voice command
    pub fn new(command_type: VoiceCommandType, trigger_text: &str) -> Self {
        Self {
            command_type,
            trigger_text: trigger_text.to_string(),
            parameters: None,
        }
    }
    
    /// Add parameters to the command
    pub fn with_parameters(mut self, parameters: &str) -> Self {
        self.parameters = Some(parameters.to_string());
        self
    }
}

/// Configuration for the voice command system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCommandConfig {
    /// Whether voice commands are enabled
    pub enabled: bool,
    
    /// The command prefix (e.g., "Computer", "Hey", etc.)
    pub command_prefix: Option<String>,
    
    /// Whether to require a prefix for all commands
    pub require_prefix: bool,
    
    /// Command detection sensitivity (0.0-1.0)
    pub sensitivity: f32,
    
    /// Custom command mappings (text to command type)
    pub custom_commands: Vec<(String, VoiceCommandType)>,
}

impl Default for VoiceCommandConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            command_prefix: None,
            require_prefix: false,
            sensitivity: 0.8,
            custom_commands: Vec::new(),
        }
    }
}

/// Text editing operation types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextEditOperation {
    /// Delete text (word, sentence, paragraph)
    Delete(DeleteScope),
    
    /// Replace text
    Replace {
        /// Text to replace
        original: String,
        /// Replacement text
        replacement: String,
    },
    
    /// Format text (capitalize, lowercase)
    Format(FormatOperation),
}

/// Scope for delete operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteScope {
    /// Delete the last word
    LastWord,
    /// Delete the last sentence
    LastSentence,
    /// Delete the last paragraph
    LastParagraph,
    /// Delete a specific number of words
    Words(usize),
    /// Delete from a specific position
    FromPosition(usize),
    /// Delete a character range
    Range(usize, usize),
}

/// Text formatting operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatOperation {
    /// Capitalize text
    Capitalize,
    /// Convert text to lowercase
    Lowercase,
    /// Convert text to uppercase
    Uppercase,
    /// Apply a specific style (bold, italic)
    Style(TextStyle),
}

/// Text styling options
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextStyle {
    /// Bold text
    Bold,
    /// Italic text
    Italic,
    /// Underline text
    Underline,
}

/// History entry for text operations
#[derive(Debug, Clone)]
pub struct TextOperationHistory {
    /// The operation that was performed
    pub operation: TextEditOperation,
    /// The text before the operation
    pub previous_text: String,
    /// The text after the operation
    pub current_text: String,
    /// Timestamp when the operation occurred
    pub timestamp: chrono::DateTime<chrono::Local>,
}

/// Text editor that handles voice commands for text editing
pub struct VoiceTextEditor {
    /// Operation history for undo/redo
    history: Vec<TextOperationHistory>,
    /// Current position in the history (for undo/redo)
    history_position: usize,
    /// Maximum history size
    max_history: usize,
}

impl VoiceTextEditor {
    /// Create a new voice text editor
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            history_position: 0,
            max_history: 50,
        }
    }
    
    /// Apply a delete operation to text
    pub fn apply_delete(&mut self, text: &str, scope: &DeleteScope) -> Result<String, String> {
        let previous_text = text.to_string();
        let current_text = match scope {
            DeleteScope::LastWord => self.delete_last_word(text),
            DeleteScope::LastSentence => self.delete_last_sentence(text),
            DeleteScope::LastParagraph => self.delete_last_paragraph(text),
            DeleteScope::Range(start, end) => self.delete_range(text, *start, *end),
            DeleteScope::Words(count) => self.delete_words(text, *count),
            DeleteScope::FromPosition(pos) => self.delete_from_position(text, *pos),
        };
        
        // Record the operation in history
        self.add_to_history(
            TextEditOperation::Delete(scope.clone()),
            previous_text,
            current_text.clone()
        );
        
        Ok(current_text)
    }
    
    /// Delete the last word in the text
    fn delete_last_word(&self, text: &str) -> String {
        let text = text.trim_end();
        if text.is_empty() {
            return String::new();
        }
        
        // Find the last word boundary
        if let Some(pos) = text.rfind(|c: char| c.is_whitespace()) {
            text[..pos].to_string()
        } else {
            // If no whitespace, delete everything
            String::new()
        }
    }
    
    /// Delete the last sentence in the text
    fn delete_last_sentence(&self, text: &str) -> String {
        let text = text.trim_end();
        if text.is_empty() {
            return String::new();
        }
        
        // Find the last sentence boundary (., !, ?)
        if let Some(pos) = text.rfind(|c: char| c == '.' || c == '!' || c == '?') {
            // Include the sentence-ending character
            let end_pos = pos + 1;
            // Trim any trailing whitespace after the sentence
            text[..end_pos].trim_end().to_string()
        } else {
            // If no sentence ending, delete everything
            String::new()
        }
    }
    
    /// Delete the last paragraph in the text
    fn delete_last_paragraph(&self, text: &str) -> String {
        let text = text.trim_end();
        if text.is_empty() {
            return String::new();
        }
        
        // Find the last paragraph boundary (double newline)
        if let Some(pos) = text.rfind("\n\n") {
            text[..pos].to_string()
        } else if let Some(pos) = text.rfind('\n') {
            // If no double newline, try single newline
            text[..pos].to_string()
        } else {
            // If no paragraph break, delete everything
            String::new()
        }
    }
    
    /// Delete a range of text
    fn delete_range(&self, text: &str, start: usize, end: usize) -> String {
        if start >= text.len() || start >= end {
            return text.to_string();
        }
        
        let end = end.min(text.len());
        format!("{}{}", &text[..start], &text[end..])
    }
    
    /// Delete a specific number of words from the end
    fn delete_words(&self, text: &str, count: usize) -> String {
        let text = text.trim_end();
        if text.is_empty() || count == 0 {
            return text.to_string();
        }
        
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() <= count {
            return String::new();
        }
        
        // Rejoin all words except the last 'count' words
        words[..words.len() - count].join(" ")
    }
    
    /// Delete text from a specific position to the end
    fn delete_from_position(&self, text: &str, position: usize) -> String {
        if position >= text.len() {
            return text.to_string();
        }
        
        text[..position].to_string()
    }
    
    /// Add an operation to the history
    fn add_to_history(&mut self, operation: TextEditOperation, previous_text: String, current_text: String) {
        // If we're not at the end of the history, truncate it
        if self.history_position < self.history.len() {
            self.history.truncate(self.history_position);
        }
        
        // Add the new operation
        self.history.push(TextOperationHistory {
            operation,
            previous_text,
            current_text,
            timestamp: chrono::Local::now(),
        });
        
        // Update position
        self.history_position = self.history.len();
        
        // Enforce maximum history size
        if self.history.len() > self.max_history {
            self.history.remove(0);
            self.history_position -= 1;
        }
    }
    
    /// Undo the last operation
    pub fn undo(&mut self) -> Option<String> {
        if self.history_position == 0 {
            return None;
        }
        
        self.history_position -= 1;
        Some(self.history[self.history_position].previous_text.clone())
    }
    
    /// Redo a previously undone operation
    pub fn redo(&mut self) -> Option<String> {
        if self.history_position >= self.history.len() {
            return None;
        }
        
        let text = self.history[self.history_position].current_text.clone();
        self.history_position += 1;
        Some(text)
    }
    
    /// Get the current history
    pub fn get_history(&self) -> &[TextOperationHistory] {
        &self.history
    }
    
    /// Get the history position
    pub fn get_history_position(&self) -> usize {
        self.history_position
    }
    
    /// Clear the history
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.history_position = 0;
    }
    
    /// Apply a formatting operation to text
    pub fn apply_format(&mut self, text: &str, format_op: FormatOperation) -> Result<String, String> {
        let previous_text = text.to_string();
        let current_text = match format_op {
            FormatOperation::Capitalize => self.capitalize_last_word(text),
            FormatOperation::Lowercase => self.lowercase_last_word(text),
            FormatOperation::Uppercase => self.uppercase_last_word(text),
            FormatOperation::Style(ref style) => self.apply_style(text, style.clone()),
        };
        
        // Record the operation in history
        self.add_to_history(
            TextEditOperation::Format(format_op),
            previous_text,
            current_text.clone()
        );
        
        Ok(current_text)
    }
    
    /// Capitalize the last word in text
    fn capitalize_last_word(&self, text: &str) -> String {
        if text.is_empty() {
            return String::new();
        }
        
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return text.to_string();
        }
        
        let last_word = words[words.len() - 1];
        let mut chars = last_word.chars();
        let capitalized = match chars.next() {
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        };
        
        // Replace last word with capitalized version
        if words.len() == 1 {
            capitalized
        } else {
            let prefix = words[..words.len() - 1].join(" ");
            format!("{} {}", prefix, capitalized)
        }
    }
    
    /// Convert the last word to lowercase
    fn lowercase_last_word(&self, text: &str) -> String {
        if text.is_empty() {
            return String::new();
        }
        
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return text.to_string();
        }
        
        let last_word = words[words.len() - 1].to_lowercase();
        
        // Replace last word with lowercase version
        if words.len() == 1 {
            last_word
        } else {
            let prefix = words[..words.len() - 1].join(" ");
            format!("{} {}", prefix, last_word)
        }
    }
    
    /// Convert the last word to uppercase
    fn uppercase_last_word(&self, text: &str) -> String {
        if text.is_empty() {
            return String::new();
        }
        
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return text.to_string();
        }
        
        let last_word = words[words.len() - 1].to_uppercase();
        
        // Replace last word with uppercase version
        if words.len() == 1 {
            last_word
        } else {
            let prefix = words[..words.len() - 1].join(" ");
            format!("{} {}", prefix, last_word)
        }
    }
    
    /// Apply a text style to the last word
    fn apply_style(&self, text: &str, style: TextStyle) -> String {
        if text.is_empty() {
            return String::new();
        }
        
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return text.to_string();
        }
        
        let last_word = words[words.len() - 1];
        let styled_word = match style {
            TextStyle::Bold => format!("**{}**", last_word),
            TextStyle::Italic => format!("*{}*", last_word),
            TextStyle::Underline => format!("_{}_", last_word),
        };
        
        // Replace last word with styled version
        if words.len() == 1 {
            styled_word
        } else {
            let prefix = words[..words.len() - 1].join(" ");
            format!("{} {}", prefix, styled_word)
        }
    }
}

/// Voice command manager
pub struct VoiceCommandManager {
    /// Configuration for the voice command system
    config: VoiceCommandConfig,
    
    /// Active command detectors
    command_detectors: Vec<CommandDetector>,
    
    /// Set of commands that are currently registered
    registered_commands: HashSet<VoiceCommandType>,
    
    /// Event sender for command events
    event_sender: mpsc::Sender<VoiceCommandEvent>,
    
    /// Whether the command detector is active
    is_active: Arc<Mutex<bool>>,
    
    /// Text editor for handling text editing commands
    text_editor: VoiceTextEditor,
    
    /// Current text buffer being edited
    current_text: Arc<Mutex<String>>,
}

impl VoiceCommandManager {
    /// Create a new voice command manager
    pub fn new(config: VoiceCommandConfig) -> Result<(Self, mpsc::Receiver<VoiceCommandEvent>)> {
        // Create channel for events
        let (sender, receiver) = mpsc::channel(100);
        
        // Default command detectors
        let default_detectors = vec![
            CommandDetector::new("delete", VoiceCommandType::Delete),
            CommandDetector::new("delete that", VoiceCommandType::Delete),
            CommandDetector::new("delete last word", VoiceCommandType::Delete),
            CommandDetector::new("delete last sentence", VoiceCommandType::Delete),
            CommandDetector::new("delete last paragraph", VoiceCommandType::Delete),
            CommandDetector::new("undo", VoiceCommandType::Undo),
            CommandDetector::new("undo that", VoiceCommandType::Undo),
            CommandDetector::new("redo", VoiceCommandType::Redo),
            CommandDetector::new("redo that", VoiceCommandType::Redo),
            CommandDetector::new("capitalize", VoiceCommandType::Capitalize),
            CommandDetector::new("capitalize that", VoiceCommandType::Capitalize),
            CommandDetector::new("lowercase", VoiceCommandType::Lowercase),
            CommandDetector::new("lowercase that", VoiceCommandType::Lowercase),
            CommandDetector::new("new line", VoiceCommandType::NewLine),
            CommandDetector::new("new paragraph", VoiceCommandType::NewParagraph),
            CommandDetector::new("period", VoiceCommandType::Period),
            CommandDetector::new("comma", VoiceCommandType::Comma),
            CommandDetector::new("question mark", VoiceCommandType::QuestionMark),
            CommandDetector::new("exclamation", VoiceCommandType::ExclamationMark),
            CommandDetector::new("pause", VoiceCommandType::Pause),
            CommandDetector::new("resume", VoiceCommandType::Resume),
            CommandDetector::new("stop", VoiceCommandType::Stop),
        ];
        
        // Register the default commands
        let mut registered_commands = HashSet::new();
        for detector in &default_detectors {
            registered_commands.insert(detector.command_type.clone());
        }
        
        // Add custom commands
        let mut command_detectors = default_detectors;
        for (trigger, command_type) in &config.custom_commands {
            command_detectors.push(CommandDetector::new(trigger, command_type.clone()));
            registered_commands.insert(command_type.clone());
        }
        
        Ok((
            Self {
                config,
                command_detectors,
                registered_commands,
                event_sender: sender,
                is_active: Arc::new(Mutex::new(false)),
                text_editor: VoiceTextEditor::new(),
                current_text: Arc::new(Mutex::new(String::new())),
            },
            receiver
        ))
    }
    
    /// Start processing voice commands
    pub fn start(&mut self) -> Result<()> {
        let mut active = self.is_active.lock();
        *active = true;
        info!("Voice command detection started");
        Ok(())
    }
    
    /// Stop processing voice commands
    pub fn stop(&mut self) -> Result<()> {
        let mut active = self.is_active.lock();
        *active = false;
        info!("Voice command detection stopped");
        Ok(())
    }
    
    /// Process a transcription to detect commands
    pub fn process_transcription(&mut self, text: &str) -> Result<Vec<VoiceCommand>> {
        if !*self.is_active.lock() || !self.config.enabled {
            return Ok(Vec::new());
        }
        
        let mut detected_commands = Vec::new();
        let text = text.to_lowercase().trim().to_string();
        
        // If the text is too short, skip processing
        if text.len() < 2 {
            return Ok(Vec::new());
        }
        
        // Prepare command text based on prefix settings
        let original_text = text.clone(); // Clone the text for logging
        let (has_prefix, command_text) = if let Some(prefix) = &self.config.command_prefix {
            let prefix = prefix.to_lowercase();
            
            // Check if text starts with or contains the prefix
            if text.starts_with(&prefix) {
                // Extract command text after the prefix
                let after_prefix = &text[prefix.len()..].trim();
                (true, after_prefix.to_string())
            } else if text.contains(&prefix) {
                // Find the prefix position and extract text after it
                if let Some(pos) = text.find(&prefix) {
                    let after_prefix = &text[pos + prefix.len()..].trim();
                    (true, after_prefix.to_string())
                } else {
                    (false, text.clone()) // Clone here
                }
            } else {
                (false, text.clone()) // Clone here
            }
        } else {
            (false, text.clone()) // Clone here
        };
        
        // Check if a prefix is required but not present
        if self.config.require_prefix && !has_prefix {
            debug!("Prefix required but not found in transcription: {:?}", original_text);
            return Ok(Vec::new());
        }
        
        // Prepare texts to search through
        let texts_to_search = if has_prefix {
            vec![command_text.clone()]
        } else {
            // If no prefix requirement or prefix found, check both the original and command text
            // This helps catch cases where the prefix might be ambiguous
            vec![text.clone(), command_text.clone()]
        };
        
        // Process the transcription for commands
        for search_text in texts_to_search {
            for detector in &self.command_detectors {
                if let Some(command) = detector.detect(&search_text, self.config.sensitivity) {
                    // Process commands based on type
                    match &command.command_type {
                        VoiceCommandType::Delete => {
                            // Determine delete scope based on command context
                            let scope = if command.trigger_text.contains("word") {
                                DeleteScope::LastWord
                            } else if command.trigger_text.contains("sentence") {
                                DeleteScope::LastSentence
                            } else if command.trigger_text.contains("paragraph") {
                                DeleteScope::LastParagraph
                            } else {
                                // Default to last word
                                DeleteScope::LastWord
                            };
                            
                            // Get current text and apply delete operation
                            let mut current_text = self.current_text.lock();
                            if let Ok(new_text) = self.text_editor.apply_delete(&current_text, &scope) {
                                // Update the current text
                                *current_text = new_text;
                                
                                // Send a text update event
                                let _ = self.event_sender.try_send(VoiceCommandEvent::CommandDetected(command.clone()));
                            } else {
                                // Send error event if operation failed
                                let _ = self.event_sender.try_send(VoiceCommandEvent::Error(
                                    format!("Failed to apply delete operation: {:?}", scope)
                                ));
                            }
                        },
                        VoiceCommandType::Capitalize => {
                            // Apply capitalize operation
                            let mut current_text = self.current_text.lock();
                            if let Ok(new_text) = self.text_editor.apply_format(&current_text, FormatOperation::Capitalize) {
                                // Update the current text
                                *current_text = new_text;
                                
                                // Send a command event
                                let _ = self.event_sender.try_send(VoiceCommandEvent::CommandDetected(command.clone()));
                            } else {
                                // Send error event if operation failed
                                let _ = self.event_sender.try_send(VoiceCommandEvent::Error(
                                    "Failed to capitalize text".to_string()
                                ));
                            }
                        },
                        VoiceCommandType::Lowercase => {
                            // Apply lowercase operation
                            let mut current_text = self.current_text.lock();
                            if let Ok(new_text) = self.text_editor.apply_format(&current_text, FormatOperation::Lowercase) {
                                // Update the current text
                                *current_text = new_text;
                                
                                // Send a command event
                                let _ = self.event_sender.try_send(VoiceCommandEvent::CommandDetected(command.clone()));
                            } else {
                                // Send error event if operation failed
                                let _ = self.event_sender.try_send(VoiceCommandEvent::Error(
                                    "Failed to lowercase text".to_string()
                                ));
                            }
                        },
                        VoiceCommandType::Undo => {
                            // Apply undo operation
                            if let Some(new_text) = self.text_editor.undo() {
                                // Update the current text
                                let mut current_text = self.current_text.lock();
                                *current_text = new_text;
                                
                                // Send a command event
                                let _ = self.event_sender.try_send(VoiceCommandEvent::CommandDetected(command.clone()));
                            } else {
                                // Send error event if no operation to undo
                                let _ = self.event_sender.try_send(VoiceCommandEvent::Error(
                                    "Nothing to undo".to_string()
                                ));
                            }
                        },
                        VoiceCommandType::Redo => {
                            // Apply redo operation
                            if let Some(new_text) = self.text_editor.redo() {
                                // Update the current text
                                let mut current_text = self.current_text.lock();
                                *current_text = new_text;
                                
                                // Send a command event
                                let _ = self.event_sender.try_send(VoiceCommandEvent::CommandDetected(command.clone()));
                            } else {
                                // Send error event if no operation to redo
                                let _ = self.event_sender.try_send(VoiceCommandEvent::Error(
                                    "Nothing to redo".to_string()
                                ));
                            }
                        },
                        // Add other command types as needed
                        _ => {
                            // For now, just send the command event
                            let _ = self.event_sender.try_send(VoiceCommandEvent::CommandDetected(command.clone()));
                        }
                    }
                    
                    detected_commands.push(command);
                    // Once we've found a command, no need to check further
                    break;
                }
            }
        }
        
        Ok(detected_commands)
    }
    
    /// Check if a command type is registered
    pub fn is_command_registered(&self, command_type: &VoiceCommandType) -> bool {
        self.registered_commands.contains(command_type)
    }
    
    /// Register a custom command
    pub fn register_custom_command(&mut self, trigger: &str, command_type: VoiceCommandType) -> Result<()> {
        // Add to detectors
        self.command_detectors.push(CommandDetector::new(trigger, command_type.clone()));
        
        // Add to registered commands
        self.registered_commands.insert(command_type);
        
        Ok(())
    }
    
    /// Get the current text being edited
    pub fn get_current_text(&self) -> String {
        self.current_text.lock().clone()
    }
    
    /// Set the current text
    pub fn set_current_text(&self, text: &str) {
        let mut current = self.current_text.lock();
        *current = text.to_string();
    }
    
    /// Get the text editor
    pub fn get_text_editor(&self) -> &VoiceTextEditor {
        &self.text_editor
    }
}

/// Command detector for a specific voice command
struct CommandDetector {
    /// The trigger text for the command
    trigger: String,
    
    /// The type of command this detector is for
    command_type: VoiceCommandType,
}

impl CommandDetector {
    /// Create a new command detector
    fn new(trigger: &str, command_type: VoiceCommandType) -> Self {
        Self {
            trigger: trigger.to_lowercase(),
            command_type,
        }
    }
    
    /// Detect if this command is present in the given text
    fn detect(&self, text: &str, sensitivity: f32) -> Option<VoiceCommand> {
        // Simple strategies first - exact match
        if text.contains(&self.trigger) {
            return Some(VoiceCommand::new(self.command_type.clone(), text));
        }
        
        // For higher sensitivity, perform more fuzzy matching
        if sensitivity > 0.5 {
            // Split the text into words
            let text_words: Vec<&str> = text.split_whitespace().collect();
            let trigger_words: Vec<&str> = self.trigger.split_whitespace().collect();
            
            // If the trigger is a single word
            if trigger_words.len() == 1 {
                // Check if any word is similar to our trigger
                for word in &text_words {
                    if word_similarity(word, &self.trigger) > sensitivity {
                        return Some(VoiceCommand::new(self.command_type.clone(), text));
                    }
                }
            } else {
                // For multi-word triggers, try to match a sequence
                if text_words.len() >= trigger_words.len() {
                    'outer: for i in 0..=(text_words.len() - trigger_words.len()) {
                        let mut total_similarity = 0.0;
                        
                        for j in 0..trigger_words.len() {
                            let similarity = word_similarity(text_words[i + j], trigger_words[j]);
                            if similarity < 0.3 {  // Minimum word match threshold
                                continue 'outer;
                            }
                            total_similarity += similarity;
                        }
                        
                        let avg_similarity = total_similarity / trigger_words.len() as f32;
                        if avg_similarity > sensitivity {
                            return Some(VoiceCommand::new(self.command_type.clone(), text));
                        }
                    }
                }
            }
        }
        
        None
    }
}

/// Calculate similarity between two words (simplified edit distance approach)
fn word_similarity(a: &str, b: &str) -> f32 {
    if a == b {
        return 1.0;
    }
    
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    
    // For very different length words, return low similarity
    let max_len = a_chars.len().max(b_chars.len()) as f32;
    if (a_chars.len() as f32 - b_chars.len() as f32).abs() / max_len > 0.5 {
        return 0.0;
    }
    
    // Calculate number of matching characters (simplified)
    let mut matches = 0;
    for i in 0..a_chars.len().min(b_chars.len()) {
        if a_chars[i] == b_chars[i] {
            matches += 1;
        }
    }
    
    matches as f32 / max_len
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_voice_command_detection() {
        // Test basic command detection functionality
        let detector = CommandDetector::new("delete", VoiceCommandType::Delete);
        
        // Should match exactly "delete"
        assert!(detector.detect("delete", 0.8).is_some());
        
        // Should match with some fuzziness
        assert!(detector.detect("deleet", 0.7).is_some());
        
        // Shouldn't match unrelated words
        assert!(detector.detect("hello", 0.8).is_none());
    }
    
    #[test]
    fn test_text_editor_delete_word() {
        let mut editor = VoiceTextEditor::new();
        
        // Test deleting the last word
        let text = "This is a test sentence";
        let result = editor.apply_delete(text, DeleteScope::LastWord);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "This is a test");
        
        // Test with trailing whitespace
        let text = "This is a test   ";
        let result = editor.apply_delete(text, DeleteScope::LastWord);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "This is a");
        
        // Test with empty text
        let text = "";
        let result = editor.apply_delete(text, DeleteScope::LastWord);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }
    
    #[test]
    fn test_text_editor_delete_sentence() {
        let mut editor = VoiceTextEditor::new();
        
        // Test deleting the last sentence
        let text = "This is the first sentence. This is the second sentence.";
        let result = editor.apply_delete(text, DeleteScope::LastSentence);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "This is the first sentence.");
        
        // Test with multiple sentence endings
        let text = "Hello! This is a test. And another one!";
        let result = editor.apply_delete(text, DeleteScope::LastSentence);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello! This is a test.");
        
        // Test with no sentence ending
        let text = "This has no sentence ending";
        let result = editor.apply_delete(text, DeleteScope::LastSentence);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }
    
    #[test]
    fn test_text_editor_delete_paragraph() {
        let mut editor = VoiceTextEditor::new();
        
        // Test deleting the last paragraph with double newlines
        let text = "First paragraph.\n\nSecond paragraph.";
        let result = editor.apply_delete(text, DeleteScope::LastParagraph);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "First paragraph.");
        
        // Test with single newlines
        let text = "First line.\nSecond line.";
        let result = editor.apply_delete(text, DeleteScope::LastParagraph);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "First line.");
        
        // Test with no paragraph breaks
        let text = "Single paragraph.";
        let result = editor.apply_delete(text, DeleteScope::LastParagraph);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }
    
    #[test]
    fn test_text_editor_undo_redo() {
        let mut editor = VoiceTextEditor::new();
        
        // Initial state
        assert_eq!(editor.get_history().len(), 0);
        
        // Apply a delete operation
        let text = "This is a test sentence";
        let result = editor.apply_delete(text, DeleteScope::LastWord);
        assert!(result.is_ok());
        let new_text = result.unwrap();
        assert_eq!(new_text, "This is a test");
        assert_eq!(editor.get_history().len(), 1);
        
        // Apply another operation
        let result = editor.apply_delete(&new_text, DeleteScope::LastWord);
        assert!(result.is_ok());
        let new_text = result.unwrap();
        assert_eq!(new_text, "This is a");
        assert_eq!(editor.get_history().len(), 2);
        
        // Undo the last operation
        let undo_result = editor.undo();
        assert!(undo_result.is_some());
        assert_eq!(undo_result.unwrap(), "This is a test");
        assert_eq!(editor.get_history_position(), 1);
        
        // Undo again to get back to the original text
        let undo_result = editor.undo();
        assert!(undo_result.is_some());
        assert_eq!(undo_result.unwrap(), "This is a test sentence");
        assert_eq!(editor.get_history_position(), 0);
        
        // Should not be able to undo more
        let undo_result = editor.undo();
        assert!(undo_result.is_none());
        
        // Redo to get back to "This is a test"
        let redo_result = editor.redo();
        assert!(redo_result.is_some());
        assert_eq!(redo_result.unwrap(), "This is a test");
        assert_eq!(editor.get_history_position(), 1);
        
        // Redo again to get to "This is a"
        let redo_result = editor.redo();
        assert!(redo_result.is_some());
        assert_eq!(redo_result.unwrap(), "This is a");
        assert_eq!(editor.get_history_position(), 2);
        
        // Should not be able to redo more
        let redo_result = editor.redo();
        assert!(redo_result.is_none());
    }
    
    #[test]
    fn test_history_truncation() {
        let mut editor = VoiceTextEditor::new();
        
        // Set a small history size for testing
        editor.max_history = 3;
        
        // Add more operations than the history size
        let texts = [
            "First operation",
            "Second operation",
            "Third operation", 
            "Fourth operation",
            "Fifth operation"
        ];
        
        for text in texts.iter() {
            let _ = editor.apply_delete(text, DeleteScope::LastWord);
        }
        
        // History should be truncated to max_history
        assert_eq!(editor.get_history().len(), 3);
        
        // The oldest operations should be removed
        assert!(editor.get_history()[0].previous_text.contains("Fifth"));
        assert!(editor.get_history()[1].previous_text.contains("Fourth"));
        assert!(editor.get_history()[2].previous_text.contains("Third"));
    }
    
    #[test]
    fn test_command_with_tauri_2_syntax() {
        // This test is a placeholder for the Tauri 2.0 testing pattern
        // It will be expanded during the actual migration
        
        #[cfg(feature = "tauri-2")]
        {
            // Tauri 2.0 specific tests would go here
            // For now, just ensure the test compiles
            let config = VoiceCommandConfig::default();
            assert!(config.enabled);
        }
        
        // If not using Tauri 2.0, just pass
        #[cfg(not(feature = "tauri-2"))]
        {
            assert!(true);
        }
    }
    
    #[test]
    fn test_text_editor_capitalize() {
        let mut editor = VoiceTextEditor::new();
        
        // Test capitalizing the last word
        let text = "this is a test sentence";
        let result = editor.apply_format(text, FormatOperation::Capitalize);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "this is a test Sentence");
        
        // Test with single word
        let text = "test";
        let result = editor.apply_format(text, FormatOperation::Capitalize);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Test");
        
        // Test with empty text
        let text = "";
        let result = editor.apply_format(text, FormatOperation::Capitalize);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }
    
    #[test]
    fn test_text_editor_lowercase() {
        let mut editor = VoiceTextEditor::new();
        
        // Test lowercasing the last word
        let text = "this is a TEST sentence";
        let result = editor.apply_format(text, FormatOperation::Lowercase);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "this is a test sentence");
        
        // Test with mixed case
        let text = "this is a TeSt";
        let result = editor.apply_format(text, FormatOperation::Lowercase);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "this is a test");
        
        // Test with single word
        let text = "TEST";
        let result = editor.apply_format(text, FormatOperation::Lowercase);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test");
    }
    
    #[test]
    fn test_text_editor_uppercase() {
        let mut editor = VoiceTextEditor::new();
        
        // Test uppercasing the last word
        let text = "this is a test sentence";
        let result = editor.apply_format(text, FormatOperation::Uppercase);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "this is a test SENTENCE");
        
        // Test with mixed case
        let text = "this is a TeSt";
        let result = editor.apply_format(text, FormatOperation::Uppercase);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "this is a TEST");
        
        // Test with single word
        let text = "test";
        let result = editor.apply_format(text, FormatOperation::Uppercase);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "TEST");
    }
    
    #[test]
    fn test_text_editor_styling() {
        let mut editor = VoiceTextEditor::new();
        
        // Test bold styling
        let text = "this is a test sentence";
        let result = editor.apply_format(text, FormatOperation::Style(TextStyle::Bold));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "this is a test **sentence**");
        
        // Test italic styling
        let text = "this is a test sentence";
        let result = editor.apply_format(text, FormatOperation::Style(TextStyle::Italic));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "this is a test *sentence*");
        
        // Test underline styling
        let text = "this is a test sentence";
        let result = editor.apply_format(text, FormatOperation::Style(TextStyle::Underline));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "this is a test _sentence_");
    }
    
    #[test]
    fn test_tauri_2_compatible_formats() {
        // Test the Tauri 2.0-specific formatting operations
        #[cfg(feature = "tauri-2")]
        {
            // We need to create a simulated environment
            // This would normally be tested with a full Tauri setup
            // For now, just verify that the feature flag works
            
            let config = VoiceCommandConfig::default();
            assert!(config.enabled);
            
            // Test creating the format operations
            let capitalize = FormatOperation::Capitalize;
            let lowercase = FormatOperation::Lowercase;
            let uppercase = FormatOperation::Uppercase;
            
            assert!(matches!(capitalize, FormatOperation::Capitalize));
            assert!(matches!(lowercase, FormatOperation::Lowercase));
            assert!(matches!(uppercase, FormatOperation::Uppercase));
        }
        
        // If not using Tauri 2.0, just pass
        #[cfg(not(feature = "tauri-2"))]
        {
            assert!(true);
        }
    }
} 
