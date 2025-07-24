use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::ai::{AIProvider, EnhancementOptions};
use super::voice_commands::{VoiceCommand, VoiceCommandType, VoiceCommandManager};

/// AI-enhanced voice command processor
pub struct AIVoiceCommandProcessor {
    /// Base voice command manager
    base_processor: Arc<RwLock<VoiceCommandManager>>,
    
    /// AI provider for natural language understanding
    ai_provider: Option<Arc<dyn AIProvider>>,
    
    /// Configuration
    config: AIVoiceCommandConfig,
    
    /// Command understanding cache
    command_cache: Arc<RwLock<lru::LruCache<String, InterpretedCommand>>>,
}

/// Configuration for AI voice commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIVoiceCommandConfig {
    /// Enable AI command interpretation
    pub enabled: bool,
    
    /// Confidence threshold for command recognition (0.0-1.0)
    pub confidence_threshold: f32,
    
    /// Enable natural language understanding
    pub enable_nlu: bool,
    
    /// Enable context awareness
    pub enable_context: bool,
    
    /// Maximum context length to consider
    pub max_context_length: usize,
    
    /// Enable command suggestions
    pub enable_suggestions: bool,
    
    /// Custom command patterns
    pub custom_patterns: Vec<CommandPattern>,
}

/// A pattern for recognizing custom commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPattern {
    /// Pattern name
    pub name: String,
    
    /// Example phrases that match this pattern
    pub examples: Vec<String>,
    
    /// The command type this pattern maps to
    pub command_type: VoiceCommandType,
    
    /// Whether this pattern supports parameters
    pub supports_parameters: bool,
}

/// Interpreted command from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpretedCommand {
    /// Original text
    pub original_text: String,
    
    /// Detected intent
    pub intent: CommandIntent,
    
    /// Confidence score
    pub confidence: f32,
    
    /// Extracted parameters
    pub parameters: Option<CommandParameters>,
    
    /// Alternative interpretations
    pub alternatives: Vec<AlternativeInterpretation>,
}

/// Command intent categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandIntent {
    /// Text editing (delete, undo, redo, etc.)
    TextEditing(TextEditingIntent),
    
    /// Text formatting (capitalize, lowercase, etc.)
    TextFormatting(TextFormattingIntent),
    
    /// Navigation (go to, scroll, etc.)
    Navigation(NavigationIntent),
    
    /// Punctuation insertion
    Punctuation(PunctuationType),
    
    /// Control (pause, resume, stop)
    Control(ControlIntent),
    
    /// Complex action requiring AI
    ComplexAction(String),
    
    /// Unknown intent
    Unknown,
}

/// Text editing intents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextEditingIntent {
    Delete { scope: DeleteScope },
    Undo { count: Option<usize> },
    Redo { count: Option<usize> },
    Replace { target: String, replacement: String },
    Insert { text: String, position: InsertPosition },
}

/// Delete scope options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeleteScope {
    LastWord,
    LastSentence,
    LastParagraph,
    Words(usize),
    Characters(usize),
    Selection,
    All,
}

/// Insert position options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsertPosition {
    Current,
    Beginning,
    End,
    After(String),
    Before(String),
}

/// Text formatting intents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextFormattingIntent {
    Capitalize { scope: FormatScope },
    Lowercase { scope: FormatScope },
    Uppercase { scope: FormatScope },
    TitleCase { scope: FormatScope },
}

/// Format scope options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormatScope {
    Word,
    Sentence,
    Paragraph,
    Selection,
    All,
}

/// Navigation intents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NavigationIntent {
    GoToBeginning,
    GoToEnd,
    NextWord,
    PreviousWord,
    NextSentence,
    PreviousSentence,
    NextParagraph,
    PreviousParagraph,
}

/// Punctuation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PunctuationType {
    Period,
    Comma,
    QuestionMark,
    ExclamationMark,
    Colon,
    Semicolon,
    Dash,
    OpenQuote,
    CloseQuote,
    OpenParenthesis,
    CloseParenthesis,
}

/// Control intents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlIntent {
    Pause,
    Resume,
    Stop,
    Save,
    Clear,
}

/// Command parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandParameters {
    /// Numeric parameters
    pub numbers: Vec<i32>,
    
    /// Text parameters
    pub text: Vec<String>,
    
    /// Boolean flags
    pub flags: Vec<(String, bool)>,
}

/// Alternative interpretation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeInterpretation {
    pub intent: CommandIntent,
    pub confidence: f32,
}

impl Default for AIVoiceCommandConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            confidence_threshold: 0.7,
            enable_nlu: true,
            enable_context: true,
            max_context_length: 1000,
            enable_suggestions: true,
            custom_patterns: Self::default_patterns(),
        }
    }
}

impl AIVoiceCommandConfig {
    fn default_patterns() -> Vec<CommandPattern> {
        vec![
            // Delete patterns
            CommandPattern {
                name: "delete_word".to_string(),
                examples: vec![
                    "delete the last word".to_string(),
                    "remove the previous word".to_string(),
                    "erase that word".to_string(),
                ],
                command_type: VoiceCommandType::Delete,
                supports_parameters: true,
            },
            // Formatting patterns
            CommandPattern {
                name: "capitalize".to_string(),
                examples: vec![
                    "capitalize that".to_string(),
                    "make it uppercase".to_string(),
                    "capital letter".to_string(),
                ],
                command_type: VoiceCommandType::Capitalize,
                supports_parameters: true,
            },
            // Navigation patterns
            CommandPattern {
                name: "new_line".to_string(),
                examples: vec![
                    "new line".to_string(),
                    "next line".to_string(),
                    "press enter".to_string(),
                ],
                command_type: VoiceCommandType::NewLine,
                supports_parameters: false,
            },
        ]
    }
}

impl AIVoiceCommandProcessor {
    /// Create a new AI voice command processor
    pub fn new(
        base_processor: Arc<RwLock<VoiceCommandManager>>,
        ai_provider: Option<Arc<dyn AIProvider>>,
        config: AIVoiceCommandConfig,
    ) -> Self {
        let cache_size = 1000;
        let command_cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(cache_size).unwrap()
        )));
        
        Self {
            base_processor,
            ai_provider,
            config,
            command_cache,
        }
    }
    
    /// Process a voice command with AI enhancement
    pub async fn process_command(&self, text: &str, context: Option<&str>) -> Result<Vec<VoiceCommand>> {
        // First, try the base processor for simple commands
        let base_commands = self.base_processor.write().await
            .process_transcription(text)?;
        
        // If base processor found commands with high confidence, use them
        if !base_commands.is_empty() && !self.config.enable_nlu {
            return Ok(base_commands);
        }
        
        // If AI is not available or disabled, return base results
        if self.ai_provider.is_none() || !self.config.enabled {
            return Ok(base_commands);
        }
        
        // Check cache first
        let cache_key = format!("{}:{}", text, context.unwrap_or(""));
        if let Some(cached) = self.command_cache.read().await.peek(&cache_key) {
            return self.interpreted_to_commands(cached.clone()).await;
        }
        
        // Use AI to interpret the command
        let interpreted = self.interpret_with_ai(text, context).await?;
        
        // Cache the interpretation
        self.command_cache.write().await.put(cache_key, interpreted.clone());
        
        // Convert interpretation to commands
        self.interpreted_to_commands(interpreted).await
    }
    
    /// Interpret command using AI
    async fn interpret_with_ai(
        &self,
        text: &str,
        context: Option<&str>,
    ) -> Result<InterpretedCommand> {
        let ai_provider = self.ai_provider.as_ref().unwrap();
        
        // Build prompt for AI
        let prompt = self.build_interpretation_prompt(text, context);
        
        // Use AI to understand the command
        let options = EnhancementOptions {
            correct_grammar: false,
            improve_punctuation: false,
            detect_intent: true,
            preserve_style: true,
            confidence_threshold: self.config.confidence_threshold,
        };
        
        let enhanced = ai_provider.enhance_text(&prompt, &options).await?;
        
        // Parse AI response into interpreted command
        self.parse_ai_response(text, &enhanced.enhanced)
    }
    
    /// Build interpretation prompt
    fn build_interpretation_prompt(&self, text: &str, context: Option<&str>) -> String {
        let mut prompt = format!(
            "Interpret the following voice command and identify the user's intent:\n\
            Command: \"{}\"\n",
            text
        );
        
        if let Some(ctx) = context {
            prompt.push_str(&format!("Context: \"{}\"\n", ctx));
        }
        
        prompt.push_str(
            "\nIdentify if this is:\n\
            1. A text editing command (delete, undo, replace)\n\
            2. A formatting command (capitalize, lowercase)\n\
            3. A navigation command (go to, move cursor)\n\
            4. A punctuation command (period, comma)\n\
            5. A control command (pause, stop, save)\n\
            6. A complex action requiring further processing\n\
            \nRespond with the intent category and any parameters."
        );
        
        prompt
    }
    
    /// Parse AI response into interpreted command
    fn parse_ai_response(&self, original_text: &str, ai_response: &str) -> Result<InterpretedCommand> {
        // This is a simplified parser - in production, you'd want more robust parsing
        let response_lower = ai_response.to_lowercase();
        
        let intent = if response_lower.contains("delete") || response_lower.contains("remove") {
            let scope = if response_lower.contains("word") {
                DeleteScope::LastWord
            } else if response_lower.contains("sentence") {
                DeleteScope::LastSentence
            } else if response_lower.contains("paragraph") {
                DeleteScope::LastParagraph
            } else {
                DeleteScope::LastWord
            };
            CommandIntent::TextEditing(TextEditingIntent::Delete { scope })
        } else if response_lower.contains("capitalize") || response_lower.contains("uppercase") {
            CommandIntent::TextFormatting(TextFormattingIntent::Capitalize {
                scope: FormatScope::Word
            })
        } else if response_lower.contains("period") || response_lower.contains("full stop") {
            CommandIntent::Punctuation(PunctuationType::Period)
        } else if response_lower.contains("comma") {
            CommandIntent::Punctuation(PunctuationType::Comma)
        } else if response_lower.contains("new line") || response_lower.contains("next line") {
            CommandIntent::Navigation(NavigationIntent::NextSentence)
        } else if response_lower.contains("pause") {
            CommandIntent::Control(ControlIntent::Pause)
        } else if response_lower.contains("stop") {
            CommandIntent::Control(ControlIntent::Stop)
        } else {
            CommandIntent::Unknown
        };
        
        Ok(InterpretedCommand {
            original_text: original_text.to_string(),
            intent,
            confidence: 0.85, // This would come from the AI in a real implementation
            parameters: None,
            alternatives: vec![],
        })
    }
    
    /// Convert interpreted command to voice commands
    async fn interpreted_to_commands(&self, interpreted: InterpretedCommand) -> Result<Vec<VoiceCommand>> {
        let mut commands = Vec::new();
        
        match interpreted.intent {
            CommandIntent::TextEditing(editing) => {
                match editing {
                    TextEditingIntent::Delete { .. } => {
                        commands.push(VoiceCommand::new(
                            VoiceCommandType::Delete,
                            &interpreted.original_text
                        ));
                    }
                    TextEditingIntent::Undo { count } => {
                        let count = count.unwrap_or(1);
                        for _ in 0..count {
                            commands.push(VoiceCommand::new(
                                VoiceCommandType::Undo,
                                &interpreted.original_text
                            ));
                        }
                    }
                    _ => {}
                }
            }
            CommandIntent::TextFormatting(formatting) => {
                match formatting {
                    TextFormattingIntent::Capitalize { .. } => {
                        commands.push(VoiceCommand::new(
                            VoiceCommandType::Capitalize,
                            &interpreted.original_text
                        ));
                    }
                    TextFormattingIntent::Lowercase { .. } => {
                        commands.push(VoiceCommand::new(
                            VoiceCommandType::Lowercase,
                            &interpreted.original_text
                        ));
                    }
                    _ => {}
                }
            }
            CommandIntent::Punctuation(punct) => {
                let cmd_type = match punct {
                    PunctuationType::Period => VoiceCommandType::Period,
                    PunctuationType::Comma => VoiceCommandType::Comma,
                    PunctuationType::QuestionMark => VoiceCommandType::QuestionMark,
                    PunctuationType::ExclamationMark => VoiceCommandType::ExclamationMark,
                    _ => VoiceCommandType::Custom("punctuation".to_string()),
                };
                commands.push(VoiceCommand::new(cmd_type, &interpreted.original_text));
            }
            CommandIntent::Control(control) => {
                let cmd_type = match control {
                    ControlIntent::Pause => VoiceCommandType::Pause,
                    ControlIntent::Resume => VoiceCommandType::Resume,
                    ControlIntent::Stop => VoiceCommandType::Stop,
                    _ => VoiceCommandType::Custom("control".to_string()),
                };
                commands.push(VoiceCommand::new(cmd_type, &interpreted.original_text));
            }
            CommandIntent::Navigation(_) => {
                commands.push(VoiceCommand::new(
                    VoiceCommandType::NewLine,
                    &interpreted.original_text
                ));
            }
            CommandIntent::ComplexAction(action) => {
                commands.push(VoiceCommand::new(
                    VoiceCommandType::Custom(action),
                    &interpreted.original_text
                ));
            }
            CommandIntent::Unknown => {
                // If confidence is still high enough, create a custom command
                if interpreted.confidence >= self.config.confidence_threshold {
                    commands.push(VoiceCommand::new(
                        VoiceCommandType::Custom("unknown".to_string()),
                        &interpreted.original_text
                    ));
                }
            }
        }
        
        Ok(commands)
    }
    
    /// Get command suggestions based on partial input
    pub async fn get_suggestions(&self, partial_text: &str) -> Result<Vec<String>> {
        if !self.config.enable_suggestions {
            return Ok(vec![]);
        }
        
        let mut suggestions = Vec::new();
        
        // Add suggestions from custom patterns
        for pattern in &self.config.custom_patterns {
            for example in &pattern.examples {
                if example.starts_with(partial_text) {
                    suggestions.push(example.clone());
                }
            }
        }
        
        // Add common command suggestions
        let common_commands = vec![
            "delete last word",
            "new line",
            "capitalize that",
            "period",
            "comma",
            "undo",
            "redo",
            "pause recording",
            "stop recording",
        ];
        
        for cmd in common_commands {
            if cmd.starts_with(partial_text) {
                suggestions.push(cmd.to_string());
            }
        }
        
        // Limit suggestions
        suggestions.truncate(10);
        Ok(suggestions)
    }
    
    /// Train custom patterns from user examples
    pub async fn train_pattern(
        &mut self,
        example_text: &str,
        command_type: VoiceCommandType,
    ) -> Result<()> {
        // Find or create pattern for this command type
        let pattern_name = format!("custom_{:?}", command_type);
        
        if let Some(pattern) = self.config.custom_patterns.iter_mut()
            .find(|p| p.command_type == command_type)
        {
            // Add example to existing pattern
            if !pattern.examples.contains(&example_text.to_string()) {
                pattern.examples.push(example_text.to_string());
            }
        } else {
            // Create new pattern
            self.config.custom_patterns.push(CommandPattern {
                name: pattern_name,
                examples: vec![example_text.to_string()],
                command_type,
                supports_parameters: false,
            });
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_interpret_delete_command() {
        let config = AIVoiceCommandConfig::default();
        let base_processor = Arc::new(RwLock::new(
            VoiceCommandManager::new(Default::default())
        ));
        
        let processor = AIVoiceCommandProcessor::new(
            base_processor,
            None, // No AI provider for testing
            config,
        );
        
        // Test interpretation
        let interpreted = processor.parse_ai_response(
            "delete the last word",
            "This is a text editing command to delete the last word"
        ).unwrap();
        
        match interpreted.intent {
            CommandIntent::TextEditing(TextEditingIntent::Delete { scope }) => {
                assert!(matches!(scope, DeleteScope::LastWord));
            }
            _ => panic!("Expected delete command"),
        }
    }
    
    #[tokio::test]
    async fn test_get_suggestions() {
        let config = AIVoiceCommandConfig::default();
        let base_processor = Arc::new(RwLock::new(
            VoiceCommandManager::new(Default::default())
        ));
        
        let processor = AIVoiceCommandProcessor::new(
            base_processor,
            None,
            config,
        );
        
        let suggestions = processor.get_suggestions("del").await.unwrap();
        assert!(suggestions.contains(&"delete last word".to_string()));
    }
}