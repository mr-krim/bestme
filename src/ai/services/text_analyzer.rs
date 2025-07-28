use std::collections::HashMap;
use regex::Regex;
use once_cell::sync::Lazy;

/// Analyzes text characteristics to help with model selection
#[derive(Debug, Clone)]
pub struct TextAnalyzer;

/// Text characteristics used for model selection
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TextCharacteristics {
    /// Total length in characters
    pub length: usize,
    /// Number of words
    pub word_count: usize,
    /// Average word length
    pub avg_word_length: f32,
    /// Number of sentences
    pub sentence_count: usize,
    /// Average sentence length (in words)
    pub avg_sentence_length: f32,
    /// Text complexity score (0-1)
    pub complexity_score: f32,
    /// Detected language (ISO 639-1 code)
    pub language: String,
    /// Detected domain/topic
    pub domain: TextDomain,
    /// Whether the text contains code snippets
    pub contains_code: bool,
    /// Whether the text contains mathematical expressions
    pub contains_math: bool,
    /// Whether the text contains URLs
    pub contains_urls: bool,
    /// Estimated reading level (grade level)
    pub reading_level: f32,
    /// Vocabulary diversity (unique words / total words)
    pub vocabulary_diversity: f32,
}

/// Text domain/topic categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TextDomain {
    General,
    Technical,
    Medical,
    Legal,
    Academic,
    Creative,
    Business,
    Conversational,
    Code,
    Scientific,
}

// Lazy static regexes for performance
static CODE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(function|class|def|import|export|const|let|var|if|else|for|while|return|public|private|protected|\{|\}|\[|\]|=>|->|::)").unwrap()
});

static MATH_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\+\-\*/=<>≤≥∑∏∫∂∇±√∞∈∉⊂⊃∪∩]|[a-zA-Z]\^[0-9]|\b(sin|cos|tan|log|exp|sqrt)\b").unwrap()
});

static URL_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https?://[^\s]+|www\.[^\s]+").unwrap()
});

static SENTENCE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[.!?]+\s+|\n\n+").unwrap()
});

impl TextAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    /// Analyze text and extract characteristics
    pub fn analyze(&self, text: &str) -> TextCharacteristics {
        let length = text.len();
        let words: Vec<&str> = text.split_whitespace().collect();
        let word_count = words.len();
        
        let avg_word_length = if word_count > 0 {
            words.iter().map(|w| w.len()).sum::<usize>() as f32 / word_count as f32
        } else {
            0.0
        };
        
        let sentences = SENTENCE_PATTERN.split(text)
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<_>>();
        let sentence_count = sentences.len().max(1);
        
        let avg_sentence_length = word_count as f32 / sentence_count as f32;
        
        // Calculate vocabulary diversity
        let unique_words: std::collections::HashSet<String> = words.iter()
            .map(|w| w.to_lowercase())
            .collect();
        let vocabulary_diversity = if word_count > 0 {
            unique_words.len() as f32 / word_count as f32
        } else {
            0.0
        };
        
        // Detect patterns
        let contains_code = CODE_PATTERN.is_match(text);
        let contains_math = MATH_PATTERN.is_match(text);
        let contains_urls = URL_PATTERN.is_match(text);
        
        // Calculate complexity score
        let complexity_score = self.calculate_complexity(
            avg_word_length,
            avg_sentence_length,
            vocabulary_diversity,
            contains_code,
            contains_math,
        );
        
        // Detect language (simplified - in production, use a proper language detection library)
        let language = self.detect_language(text);
        
        // Detect domain
        let domain = self.detect_domain(text, contains_code, contains_math);
        
        // Calculate reading level (Flesch-Kincaid Grade Level approximation)
        let reading_level = self.calculate_reading_level(avg_word_length, avg_sentence_length);
        
        TextCharacteristics {
            length,
            word_count,
            avg_word_length,
            sentence_count,
            avg_sentence_length,
            complexity_score,
            language,
            domain,
            contains_code,
            contains_math,
            contains_urls,
            reading_level,
            vocabulary_diversity,
        }
    }
    
    /// Calculate text complexity score (0-1)
    fn calculate_complexity(
        &self,
        avg_word_length: f32,
        avg_sentence_length: f32,
        vocabulary_diversity: f32,
        contains_code: bool,
        contains_math: bool,
    ) -> f32 {
        let mut score = 0.0;
        
        // Word length contribution (longer words = more complex)
        score += (avg_word_length - 3.0).max(0.0).min(5.0) / 5.0 * 0.3;
        
        // Sentence length contribution
        score += (avg_sentence_length - 10.0).max(0.0).min(30.0) / 30.0 * 0.3;
        
        // Vocabulary diversity contribution
        score += vocabulary_diversity * 0.2;
        
        // Special content contribution
        if contains_code {
            score += 0.1;
        }
        if contains_math {
            score += 0.1;
        }
        
        score.min(1.0)
    }
    
    /// Simple language detection (in production, use a proper library)
    fn detect_language(&self, text: &str) -> String {
        // Check for common English patterns
        let english_patterns = ["the", "is", "are", "and", "or", "but", "in", "on", "at", "to"];
        let english_count = english_patterns.iter()
            .filter(|&&word| text.to_lowercase().contains(word))
            .count();
        
        if english_count >= 3 {
            "en".to_string()
        } else {
            "unknown".to_string()
        }
    }
    
    /// Detect the domain/topic of the text
    fn detect_domain(&self, text: &str, contains_code: bool, contains_math: bool) -> TextDomain {
        let text_lower = text.to_lowercase();
        
        // Domain-specific keywords
        let domain_keywords: HashMap<TextDomain, Vec<&str>> = [
            (TextDomain::Medical, vec!["patient", "diagnosis", "treatment", "symptom", "medical", "health", "disease", "therapy"]),
            (TextDomain::Legal, vec!["law", "legal", "court", "judge", "attorney", "contract", "agreement", "clause"]),
            (TextDomain::Technical, vec!["system", "software", "hardware", "network", "database", "algorithm", "protocol"]),
            (TextDomain::Academic, vec!["research", "study", "hypothesis", "methodology", "analysis", "conclusion", "abstract"]),
            (TextDomain::Business, vec!["business", "revenue", "profit", "market", "strategy", "customer", "sales", "finance"]),
            (TextDomain::Scientific, vec!["experiment", "theory", "hypothesis", "data", "observation", "measurement", "variable"]),
        ].iter().cloned().collect();
        
        // Count keyword matches for each domain
        let mut domain_scores: HashMap<TextDomain, usize> = HashMap::new();
        
        for (domain, keywords) in domain_keywords {
            let score = keywords.iter()
                .filter(|&&keyword| text_lower.contains(keyword))
                .count();
            if score > 0 {
                domain_scores.insert(domain, score);
            }
        }
        
        // Special cases
        if contains_code {
            return TextDomain::Code;
        }
        
        if contains_math && domain_scores.get(&TextDomain::Scientific).unwrap_or(&0) > &0 {
            return TextDomain::Scientific;
        }
        
        // Return domain with highest score
        domain_scores.into_iter()
            .max_by_key(|(_, score)| *score)
            .map(|(domain, _)| domain)
            .unwrap_or(TextDomain::General)
    }
    
    /// Calculate reading level using Flesch-Kincaid Grade Level formula
    fn calculate_reading_level(&self, avg_word_length: f32, avg_sentence_length: f32) -> f32 {
        // Simplified version - assumes avg syllables per word correlates with word length
        let avg_syllables_per_word = (avg_word_length / 3.0).max(1.0);
        let grade_level = 0.39 * avg_sentence_length + 11.8 * avg_syllables_per_word - 15.59;
        grade_level.max(0.0).min(20.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_analysis() {
        let analyzer = TextAnalyzer::new();
        let text = "The quick brown fox jumps over the lazy dog. This is a simple test.";
        let characteristics = analyzer.analyze(text);
        
        assert_eq!(characteristics.word_count, 13);
        assert_eq!(characteristics.sentence_count, 2);
        assert!(!characteristics.contains_code);
        assert!(!characteristics.contains_math);
        assert_eq!(characteristics.language, "en");
    }
    
    #[test]
    fn test_code_detection() {
        let analyzer = TextAnalyzer::new();
        let text = "Here's a function example: function calculate(x) { return x * 2; }";
        let characteristics = analyzer.analyze(text);
        
        assert!(characteristics.contains_code);
        assert_eq!(characteristics.domain, TextDomain::Code);
    }
    
    #[test]
    fn test_math_detection() {
        let analyzer = TextAnalyzer::new();
        let text = "The equation is: f(x) = x² + 2x + 1, where x ∈ ℝ";
        let characteristics = analyzer.analyze(text);
        
        assert!(characteristics.contains_math);
    }
    
    #[test]
    fn test_domain_detection() {
        let analyzer = TextAnalyzer::new();
        
        let medical_text = "The patient presented with symptoms of acute bronchitis. Treatment includes antibiotics.";
        let medical_chars = analyzer.analyze(medical_text);
        assert_eq!(medical_chars.domain, TextDomain::Medical);
        
        let business_text = "Our Q3 revenue exceeded projections. Sales strategy needs adjustment for market conditions.";
        let business_chars = analyzer.analyze(business_text);
        assert_eq!(business_chars.domain, TextDomain::Business);
    }
    
    #[test]
    fn test_complexity_calculation() {
        let analyzer = TextAnalyzer::new();
        
        let simple_text = "I like cats. Cats are nice. They are fun.";
        let simple_chars = analyzer.analyze(simple_text);
        
        let complex_text = "The implementation of quantum computing algorithms necessitates a comprehensive understanding of superposition and entanglement phenomena.";
        let complex_chars = analyzer.analyze(complex_text);
        
        assert!(simple_chars.complexity_score < complex_chars.complexity_score);
        assert!(simple_chars.reading_level < complex_chars.reading_level);
    }
}