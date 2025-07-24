use crate::ai::{Result, AIError, EnhancedText, Correction};
use std::collections::HashMap;

pub struct TextEnhancer {
    grammar_rules: HashMap<String, String>,
    common_mistakes: HashMap<String, String>,
}

impl TextEnhancer {
    pub fn new() -> Self {
        let mut grammar_rules = HashMap::new();
        let mut common_mistakes = HashMap::new();
        
        // Common grammar rules
        grammar_rules.insert("i".to_string(), "I".to_string());
        grammar_rules.insert("dont".to_string(), "don't".to_string());
        grammar_rules.insert("wont".to_string(), "won't".to_string());
        grammar_rules.insert("cant".to_string(), "can't".to_string());
        grammar_rules.insert("shouldnt".to_string(), "shouldn't".to_string());
        grammar_rules.insert("wouldnt".to_string(), "wouldn't".to_string());
        
        // Common mistakes
        common_mistakes.insert("teh".to_string(), "the".to_string());
        common_mistakes.insert("recieve".to_string(), "receive".to_string());
        common_mistakes.insert("beleive".to_string(), "believe".to_string());
        common_mistakes.insert("occured".to_string(), "occurred".to_string());
        common_mistakes.insert("seperate".to_string(), "separate".to_string());
        
        Self {
            grammar_rules,
            common_mistakes,
        }
    }

    pub fn apply_basic_corrections(&self, text: &str) -> (String, Vec<Correction>) {
        let mut result = text.to_string();
        let mut corrections = Vec::new();
        
        // Apply common mistake corrections
        for (mistake, correct) in &self.common_mistakes {
            if let Some(pos) = result.find(mistake) {
                corrections.push(Correction {
                    start: pos,
                    end: pos + mistake.len(),
                    original: mistake.clone(),
                    corrected: correct.clone(),
                    reason: "Common spelling mistake".to_string(),
                });
                result = result.replace(mistake, correct);
            }
        }
        
        // Apply grammar rules
        let words: Vec<&str> = result.split_whitespace().collect();
        let mut new_words = Vec::new();
        let mut word_start = 0;
        
        for word in words {
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
            let prefix = &word[..word.len() - clean_word.len()];
            let suffix = &word[word.len() - clean_word.len() + clean_word.len()..];
            
            if let Some(replacement) = self.grammar_rules.get(clean_word) {
                corrections.push(Correction {
                    start: word_start + prefix.len(),
                    end: word_start + prefix.len() + clean_word.len(),
                    original: clean_word.to_string(),
                    corrected: replacement.clone(),
                    reason: "Grammar correction".to_string(),
                });
                new_words.push(format!("{}{}{}", prefix, replacement, suffix));
            } else {
                new_words.push(word.to_string());
            }
            
            word_start += word.len() + 1; // +1 for space
        }
        
        result = new_words.join(" ");
        
        (result, corrections)
    }

    pub fn improve_punctuation(&self, text: &str) -> String {
        let mut result = text.trim().to_string();
        
        // Basic sentence detection and punctuation
        if !result.is_empty() {
            // Ensure sentence ends with punctuation
            let last_char = result.chars().last().unwrap();
            if !".!?".contains(last_char) {
                // Simple heuristic: add period for statements, question mark for questions
                if result.to_lowercase().starts_with("what") ||
                   result.to_lowercase().starts_with("when") ||
                   result.to_lowercase().starts_with("where") ||
                   result.to_lowercase().starts_with("who") ||
                   result.to_lowercase().starts_with("why") ||
                   result.to_lowercase().starts_with("how") ||
                   result.to_lowercase().starts_with("is") ||
                   result.to_lowercase().starts_with("are") ||
                   result.to_lowercase().starts_with("can") ||
                   result.to_lowercase().starts_with("could") ||
                   result.to_lowercase().starts_with("would") ||
                   result.to_lowercase().starts_with("should") {
                    result.push('?');
                } else {
                    result.push('.');
                }
            }
            
            // Capitalize first letter
            if let Some(first_char) = result.chars().next() {
                if first_char.is_lowercase() {
                    let mut chars = result.chars();
                    chars.next();
                    result = first_char.to_uppercase().collect::<String>() + chars.as_str();
                }
            }
            
            // Capitalize after sentence endings
            let mut chars: Vec<char> = result.chars().collect();
            let mut capitalize_next = false;
            
            for i in 0..chars.len() {
                if capitalize_next && chars[i].is_alphabetic() {
                    chars[i] = chars[i].to_uppercase().next().unwrap();
                    capitalize_next = false;
                }
                if ".!?".contains(chars[i]) && i + 1 < chars.len() {
                    capitalize_next = true;
                }
            }
            
            result = chars.into_iter().collect();
        }
        
        result
    }
}

pub struct ContextualEnhancer {
    context_window: Vec<String>,
    max_context_size: usize,
}

impl ContextualEnhancer {
    pub fn new(max_context_size: usize) -> Self {
        Self {
            context_window: Vec::new(),
            max_context_size,
        }
    }

    pub fn add_to_context(&mut self, text: &str) {
        self.context_window.push(text.to_string());
        if self.context_window.len() > self.max_context_size {
            self.context_window.remove(0);
        }
    }

    pub fn enhance_with_context(&self, text: &str) -> String {
        // This would use context to improve pronoun resolution,
        // maintain consistent terminology, etc.
        // For now, just return the text as-is
        text.to_string()
    }

    pub fn get_context_prompt(&self) -> String {
        if self.context_window.is_empty() {
            String::new()
        } else {
            format!("Previous context: {}\n", self.context_window.join(" "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_corrections() {
        let enhancer = TextEnhancer::new();
        let (result, corrections) = enhancer.apply_basic_corrections("i cant beleive teh weather");
        
        assert_eq!(result, "I can't believe the weather");
        assert_eq!(corrections.len(), 4);
    }

    #[test]
    fn test_punctuation_improvement() {
        let enhancer = TextEnhancer::new();
        
        assert_eq!(
            enhancer.improve_punctuation("hello world"),
            "Hello world."
        );
        
        assert_eq!(
            enhancer.improve_punctuation("what is your name"),
            "What is your name?"
        );
        
        assert_eq!(
            enhancer.improve_punctuation("this is great!"),
            "This is great!"
        );
    }

    #[test]
    fn test_context_window() {
        let mut enhancer = ContextualEnhancer::new(3);
        
        enhancer.add_to_context("First sentence.");
        enhancer.add_to_context("Second sentence.");
        enhancer.add_to_context("Third sentence.");
        enhancer.add_to_context("Fourth sentence.");
        
        // Should only keep last 3
        assert_eq!(enhancer.context_window.len(), 3);
        assert_eq!(enhancer.context_window[0], "Second sentence.");
    }
}