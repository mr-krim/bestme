use crate::ai::{Result, AIError};
use crate::ai::context::Role;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Template for conversation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTemplate {
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// System prompt template
    pub system_prompt: String,
    /// User message template
    pub user_template: String,
    /// Assistant message template
    pub assistant_template: String,
    /// Variables that can be filled in
    pub variables: Vec<TemplateVariable>,
    /// Template metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}

/// Template manager for different conversation styles
pub struct TemplateManager {
    templates: HashMap<String, ConversationTemplate>,
}

impl TemplateManager {
    /// Create a new template manager with default templates
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        
        // Add default templates
        templates.insert("default".to_string(), Self::default_template());
        templates.insert("professional".to_string(), Self::professional_template());
        templates.insert("technical".to_string(), Self::technical_template());
        templates.insert("creative".to_string(), Self::creative_template());
        templates.insert("educational".to_string(), Self::educational_template());
        
        Self { templates }
    }
    
    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&ConversationTemplate> {
        self.templates.get(name)
    }
    
    /// Add a custom template
    pub fn add_template(&mut self, template: ConversationTemplate) {
        self.templates.insert(template.name.clone(), template);
    }
    
    /// List all available templates
    pub fn list_templates(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }
    
    /// Apply a template with variables
    pub fn apply_template(
        &self,
        template_name: &str,
        variables: HashMap<String, String>,
    ) -> Result<AppliedTemplate> {
        let template = self.templates.get(template_name)
            .ok_or_else(|| AIError::ConfigError(format!("Template {} not found", template_name)))?;
        
        // Check required variables
        for var in &template.variables {
            if var.required && !variables.contains_key(&var.name) {
                if var.default_value.is_none() {
                    return Err(AIError::ConfigError(
                        format!("Required variable {} not provided", var.name)
                    ));
                }
            }
        }
        
        // Apply variables to templates
        let system_prompt = self.apply_variables(&template.system_prompt, &variables, &template.variables);
        let user_template = self.apply_variables(&template.user_template, &variables, &template.variables);
        let assistant_template = self.apply_variables(&template.assistant_template, &variables, &template.variables);
        
        Ok(AppliedTemplate {
            system_prompt,
            user_template,
            assistant_template,
        })
    }
    
    /// Apply variables to a template string
    fn apply_variables(
        &self,
        template: &str,
        variables: &HashMap<String, String>,
        template_vars: &[TemplateVariable],
    ) -> String {
        let mut result = template.to_string();
        
        for var in template_vars {
            let value = variables.get(&var.name)
                .or(var.default_value.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("");
            
            result = result.replace(&format!("{{{{{}}}}}", var.name), value);
        }
        
        result
    }
    
    /// Default conversation template
    fn default_template() -> ConversationTemplate {
        ConversationTemplate {
            name: "default".to_string(),
            description: "Standard conversational template".to_string(),
            system_prompt: "You are a helpful AI assistant. Be concise and accurate in your responses.".to_string(),
            user_template: "User: {{message}}".to_string(),
            assistant_template: "Assistant: {{response}}".to_string(),
            variables: vec![
                TemplateVariable {
                    name: "message".to_string(),
                    description: "User's message".to_string(),
                    required: true,
                    default_value: None,
                },
                TemplateVariable {
                    name: "response".to_string(),
                    description: "Assistant's response".to_string(),
                    required: true,
                    default_value: None,
                },
            ],
            metadata: HashMap::new(),
        }
    }
    
    /// Professional writing template
    fn professional_template() -> ConversationTemplate {
        ConversationTemplate {
            name: "professional".to_string(),
            description: "Professional writing and communication".to_string(),
            system_prompt: "You are a professional writing assistant. Help improve clarity, tone, and professionalism in communications. Focus on {{industry}} standards and {{audience}} expectations.".to_string(),
            user_template: "Please improve this {{document_type}}: {{message}}".to_string(),
            assistant_template: "Here's the enhanced version: {{response}}".to_string(),
            variables: vec![
                TemplateVariable {
                    name: "industry".to_string(),
                    description: "Industry or field".to_string(),
                    required: false,
                    default_value: Some("business".to_string()),
                },
                TemplateVariable {
                    name: "audience".to_string(),
                    description: "Target audience".to_string(),
                    required: false,
                    default_value: Some("professional".to_string()),
                },
                TemplateVariable {
                    name: "document_type".to_string(),
                    description: "Type of document".to_string(),
                    required: false,
                    default_value: Some("text".to_string()),
                },
                TemplateVariable {
                    name: "message".to_string(),
                    description: "Text to improve".to_string(),
                    required: true,
                    default_value: None,
                },
                TemplateVariable {
                    name: "response".to_string(),
                    description: "Enhanced text".to_string(),
                    required: true,
                    default_value: None,
                },
            ],
            metadata: HashMap::new(),
        }
    }
    
    /// Technical documentation template
    fn technical_template() -> ConversationTemplate {
        ConversationTemplate {
            name: "technical".to_string(),
            description: "Technical documentation and code comments".to_string(),
            system_prompt: "You are a technical documentation expert. Help write clear, accurate technical content for {{technology}} in {{language}}. Use appropriate technical terminology while maintaining clarity.".to_string(),
            user_template: "Document this {{content_type}}: {{message}}".to_string(),
            assistant_template: "```{{language}}\n{{response}}\n```".to_string(),
            variables: vec![
                TemplateVariable {
                    name: "technology".to_string(),
                    description: "Technology stack".to_string(),
                    required: false,
                    default_value: Some("software".to_string()),
                },
                TemplateVariable {
                    name: "language".to_string(),
                    description: "Programming language".to_string(),
                    required: false,
                    default_value: Some("text".to_string()),
                },
                TemplateVariable {
                    name: "content_type".to_string(),
                    description: "Type of content".to_string(),
                    required: false,
                    default_value: Some("code".to_string()),
                },
                TemplateVariable {
                    name: "message".to_string(),
                    description: "Content to document".to_string(),
                    required: true,
                    default_value: None,
                },
                TemplateVariable {
                    name: "response".to_string(),
                    description: "Documentation".to_string(),
                    required: true,
                    default_value: None,
                },
            ],
            metadata: HashMap::new(),
        }
    }
    
    /// Creative writing template
    fn creative_template() -> ConversationTemplate {
        ConversationTemplate {
            name: "creative".to_string(),
            description: "Creative writing and storytelling".to_string(),
            system_prompt: "You are a creative writing assistant specializing in {{genre}}. Help craft engaging, imaginative content with {{style}} style and {{tone}} tone.".to_string(),
            user_template: "Help me write: {{message}}".to_string(),
            assistant_template: "{{response}}".to_string(),
            variables: vec![
                TemplateVariable {
                    name: "genre".to_string(),
                    description: "Writing genre".to_string(),
                    required: false,
                    default_value: Some("general".to_string()),
                },
                TemplateVariable {
                    name: "style".to_string(),
                    description: "Writing style".to_string(),
                    required: false,
                    default_value: Some("descriptive".to_string()),
                },
                TemplateVariable {
                    name: "tone".to_string(),
                    description: "Writing tone".to_string(),
                    required: false,
                    default_value: Some("engaging".to_string()),
                },
                TemplateVariable {
                    name: "message".to_string(),
                    description: "Writing prompt".to_string(),
                    required: true,
                    default_value: None,
                },
                TemplateVariable {
                    name: "response".to_string(),
                    description: "Creative output".to_string(),
                    required: true,
                    default_value: None,
                },
            ],
            metadata: HashMap::new(),
        }
    }
    
    /// Educational template
    fn educational_template() -> ConversationTemplate {
        ConversationTemplate {
            name: "educational".to_string(),
            description: "Educational content and explanations".to_string(),
            system_prompt: "You are an educational assistant teaching {{subject}} at {{level}} level. Explain concepts clearly with examples. Adapt your explanations to the student's understanding.".to_string(),
            user_template: "Explain this concept: {{message}}".to_string(),
            assistant_template: "{{response}}\n\nKey points:\n- {{key_points}}".to_string(),
            variables: vec![
                TemplateVariable {
                    name: "subject".to_string(),
                    description: "Subject matter".to_string(),
                    required: false,
                    default_value: Some("general".to_string()),
                },
                TemplateVariable {
                    name: "level".to_string(),
                    description: "Education level".to_string(),
                    required: false,
                    default_value: Some("intermediate".to_string()),
                },
                TemplateVariable {
                    name: "message".to_string(),
                    description: "Concept to explain".to_string(),
                    required: true,
                    default_value: None,
                },
                TemplateVariable {
                    name: "response".to_string(),
                    description: "Explanation".to_string(),
                    required: true,
                    default_value: None,
                },
                TemplateVariable {
                    name: "key_points".to_string(),
                    description: "Summary points".to_string(),
                    required: false,
                    default_value: Some("To be summarized".to_string()),
                },
            ],
            metadata: HashMap::new(),
        }
    }
}

/// Applied template with variables filled in
#[derive(Debug, Clone)]
pub struct AppliedTemplate {
    pub system_prompt: String,
    pub user_template: String,
    pub assistant_template: String,
}

impl AppliedTemplate {
    /// Format a user message
    pub fn format_user_message(&self, message: &str) -> String {
        self.user_template.replace("{{message}}", message)
    }
    
    /// Format an assistant response
    pub fn format_assistant_response(&self, response: &str) -> String {
        self.assistant_template.replace("{{response}}", response)
    }
}

/// Template-based context builder
pub struct TemplateContextBuilder {
    template_manager: TemplateManager,
}

impl TemplateContextBuilder {
    /// Create a new template context builder
    pub fn new() -> Self {
        Self {
            template_manager: TemplateManager::new(),
        }
    }
    
    /// Build context using a template
    pub fn build_context(
        &self,
        template_name: &str,
        messages: Vec<(Role, String)>,
        variables: HashMap<String, String>,
    ) -> Result<String> {
        let applied = self.template_manager.apply_template(template_name, variables)?;
        let mut context = String::new();
        
        // Add system prompt
        context.push_str(&format!("System: {}\n\n", applied.system_prompt));
        
        // Add messages
        for (role, content) in messages {
            match role {
                Role::User => {
                    context.push_str(&applied.format_user_message(&content));
                }
                Role::Assistant => {
                    context.push_str(&applied.format_assistant_response(&content));
                }
                Role::System => {
                    context.push_str(&format!("System: {}", content));
                }
            }
            context.push_str("\n\n");
        }
        
        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_template_manager() {
        let manager = TemplateManager::new();
        
        // Test default template exists
        assert!(manager.get_template("default").is_some());
        
        // Test listing templates
        let templates = manager.list_templates();
        assert!(templates.contains(&"default"));
        assert!(templates.contains(&"professional"));
    }
    
    #[test]
    fn test_template_application() {
        let manager = TemplateManager::new();
        
        let mut variables = HashMap::new();
        variables.insert("message".to_string(), "Hello world".to_string());
        variables.insert("response".to_string(), "Hi there!".to_string());
        
        let applied = manager.apply_template("default", variables).unwrap();
        
        assert_eq!(applied.format_user_message("Test"), "User: Test");
        assert_eq!(applied.format_assistant_response("Reply"), "Assistant: Reply");
    }
    
    #[test]
    fn test_professional_template() {
        let manager = TemplateManager::new();
        
        let mut variables = HashMap::new();
        variables.insert("message".to_string(), "Please review".to_string());
        variables.insert("response".to_string(), "Reviewed".to_string());
        variables.insert("industry".to_string(), "tech".to_string());
        
        let applied = manager.apply_template("professional", variables).unwrap();
        
        assert!(applied.system_prompt.contains("tech"));
    }
}