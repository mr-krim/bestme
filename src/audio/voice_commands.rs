use anyhow::Result;
use log::{error, info, debug};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use parking_lot::Mutex;
use tokio::sync::mpsc;

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
}

impl VoiceCommandManager {
    /// Create a new voice command manager
    pub fn new(config: VoiceCommandConfig) -> Result<(Self, mpsc::Receiver<VoiceCommandEvent>)> {
        // Create channel for events
        let (sender, receiver) = mpsc::channel(100);
        
        // Default command detectors
        let default_detectors = vec![
            CommandDetector::new("delete", VoiceCommandType::Delete),
            CommandDetector::new("undo", VoiceCommandType::Undo),
            CommandDetector::new("redo", VoiceCommandType::Redo),
            CommandDetector::new("capitalize", VoiceCommandType::Capitalize),
            CommandDetector::new("lowercase", VoiceCommandType::Lowercase),
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
    pub fn process_transcription(&self, text: &str) -> Result<Vec<VoiceCommand>> {
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
                    // Log the detected command
                    info!("Detected voice command: {:?} from text: {}", command.command_type, search_text);
                    
                    // Store a clone of the command in our results
                    let command_clone = command.clone();
                    detected_commands.push(command_clone);
                    
                    // Send event for the command - clone the event_sender for each command
                    let command_event = VoiceCommandEvent::CommandDetected(command);
                    let event_sender = self.event_sender.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = event_sender.send(command_event).await {
                            error!("Failed to send command event: {}", e);
                        }
                    });
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
