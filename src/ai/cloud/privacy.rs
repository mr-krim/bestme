use crate::ai::{Result, AIError, PrivacyLevel};
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PrivacyManager {
    privacy_level: PrivacyLevel,
    anonymizers: Vec<Box<dyn Anonymizer>>,
}

impl PrivacyManager {
    pub fn new(privacy_level: PrivacyLevel) -> Self {
        let mut anonymizers: Vec<Box<dyn Anonymizer>> = Vec::new();
        
        match privacy_level {
            PrivacyLevel::Strict => {
                // In strict mode, we don't send any data to cloud
                anonymizers.push(Box::new(EmailAnonymizer::new()));
                anonymizers.push(Box::new(PhoneAnonymizer::new()));
                anonymizers.push(Box::new(NameAnonymizer::new()));
                anonymizers.push(Box::new(AddressAnonymizer::new()));
                anonymizers.push(Box::new(NumberAnonymizer::new()));
                anonymizers.push(Box::new(DateAnonymizer::new()));
            }
            PrivacyLevel::Balanced => {
                // In balanced mode, we anonymize sensitive data
                anonymizers.push(Box::new(EmailAnonymizer::new()));
                anonymizers.push(Box::new(PhoneAnonymizer::new()));
                anonymizers.push(Box::new(NameAnonymizer::new()));
                anonymizers.push(Box::new(AddressAnonymizer::new()));
            }
            PrivacyLevel::Permissive => {
                // In permissive mode, we only anonymize very sensitive data
                anonymizers.push(Box::new(EmailAnonymizer::new()));
                anonymizers.push(Box::new(PhoneAnonymizer::new()));
            }
        }
        
        Self {
            privacy_level,
            anonymizers,
        }
    }

    pub async fn anonymize(&self, text: &str) -> Result<(String, Option<Deanonymizer>)> {
        if self.privacy_level == PrivacyLevel::Permissive && self.anonymizers.is_empty() {
            return Ok((text.to_string(), None));
        }

        let mut result = text.to_string();
        let mut replacements = HashMap::new();
        
        for anonymizer in &self.anonymizers {
            let (anonymized, mapping) = anonymizer.anonymize(&result)?;
            result = anonymized;
            replacements.extend(mapping);
        }
        
        if replacements.is_empty() {
            Ok((result, None))
        } else {
            Ok((result, Some(Deanonymizer::new(replacements))))
        }
    }

    pub fn should_block(&self, text: &str) -> bool {
        match self.privacy_level {
            PrivacyLevel::Strict => {
                // Block if contains any sensitive patterns
                self.contains_sensitive_data(text)
            }
            _ => false,
        }
    }

    fn contains_sensitive_data(&self, text: &str) -> bool {
        // Check for patterns that indicate sensitive data
        let patterns = [
            r"\b\d{3}-\d{2}-\d{4}\b", // SSN
            r"\b\d{16}\b", // Credit card
            r"password|passwd|pwd", // Passwords
            r"api[_-]?key|secret[_-]?key", // API keys
        ];
        
        patterns.iter().any(|pattern| {
            Regex::new(pattern).unwrap().is_match(&text.to_lowercase())
        })
    }
}

pub struct Deanonymizer {
    replacements: HashMap<String, String>,
}

impl Deanonymizer {
    pub fn new(replacements: HashMap<String, String>) -> Self {
        Self { replacements }
    }

    pub fn restore(&self, text: &str) -> Result<String> {
        let mut result = text.to_string();
        
        // Sort by length descending to avoid partial replacements
        let mut sorted_replacements: Vec<_> = self.replacements.iter().collect();
        sorted_replacements.sort_by_key(|(k, _)| k.len());
        sorted_replacements.reverse();
        
        for (placeholder, original) in sorted_replacements {
            result = result.replace(placeholder, original);
        }
        
        Ok(result)
    }
}

trait Anonymizer: Send + Sync {
    fn anonymize(&self, text: &str) -> Result<(String, HashMap<String, String>)>;
}

struct EmailAnonymizer {
    pattern: Regex,
}

impl EmailAnonymizer {
    fn new() -> Self {
        Self {
            pattern: Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap(),
        }
    }
}

impl Anonymizer for EmailAnonymizer {
    fn anonymize(&self, text: &str) -> Result<(String, HashMap<String, String>)> {
        let mut result = text.to_string();
        let mut replacements = HashMap::new();
        let mut counter = 0;
        
        for mat in self.pattern.find_iter(text) {
            let email = mat.as_str();
            let placeholder = format!("[EMAIL_{}]", counter);
            result = result.replace(email, &placeholder);
            replacements.insert(placeholder, email.to_string());
            counter += 1;
        }
        
        Ok((result, replacements))
    }
}

struct PhoneAnonymizer {
    patterns: Vec<Regex>,
}

impl PhoneAnonymizer {
    fn new() -> Self {
        Self {
            patterns: vec![
                Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b").unwrap(),
                Regex::new(r"\b\(\d{3}\)\s*\d{3}[-.]?\d{4}\b").unwrap(),
                Regex::new(r"\b\+?1?\s*\d{3}[-.]?\d{3}[-.]?\d{4}\b").unwrap(),
            ],
        }
    }
}

impl Anonymizer for PhoneAnonymizer {
    fn anonymize(&self, text: &str) -> Result<(String, HashMap<String, String>)> {
        let mut result = text.to_string();
        let mut replacements = HashMap::new();
        let mut counter = 0;
        
        for pattern in &self.patterns {
            for mat in pattern.find_iter(&result.clone()) {
                let phone = mat.as_str();
                let placeholder = format!("[PHONE_{}]", counter);
                result = result.replace(phone, &placeholder);
                replacements.insert(placeholder, phone.to_string());
                counter += 1;
            }
        }
        
        Ok((result, replacements))
    }
}

struct NameAnonymizer {
    common_names: Vec<String>,
}

impl NameAnonymizer {
    fn new() -> Self {
        // This is a simplified version - in production, you'd have a more comprehensive list
        let common_names = vec![
            "John", "Jane", "Bob", "Alice", "Michael", "Sarah", "David", "Emily",
            "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller",
        ].iter().map(|s| s.to_string()).collect();
        
        Self { common_names }
    }
}

impl Anonymizer for NameAnonymizer {
    fn anonymize(&self, text: &str) -> Result<(String, HashMap<String, String>)> {
        let mut result = text.to_string();
        let mut replacements = HashMap::new();
        let mut counter = 0;
        
        for name in &self.common_names {
            let pattern = Regex::new(&format!(r"\b{}\b", regex::escape(name))).unwrap();
            for mat in pattern.find_iter(&text) {
                let placeholder = format!("[NAME_{}]", counter);
                result = result.replace(mat.as_str(), &placeholder);
                replacements.insert(placeholder, mat.as_str().to_string());
                counter += 1;
            }
        }
        
        Ok((result, replacements))
    }
}

struct AddressAnonymizer {
    pattern: Regex,
}

impl AddressAnonymizer {
    fn new() -> Self {
        Self {
            // Simplified pattern for US addresses
            pattern: Regex::new(r"\b\d+\s+[A-Za-z\s]+(?:Street|St|Avenue|Ave|Road|Rd|Boulevard|Blvd|Lane|Ln|Drive|Dr)\b").unwrap(),
        }
    }
}

impl Anonymizer for AddressAnonymizer {
    fn anonymize(&self, text: &str) -> Result<(String, HashMap<String, String>)> {
        let mut result = text.to_string();
        let mut replacements = HashMap::new();
        let mut counter = 0;
        
        for mat in self.pattern.find_iter(text) {
            let address = mat.as_str();
            let placeholder = format!("[ADDRESS_{}]", counter);
            result = result.replace(address, &placeholder);
            replacements.insert(placeholder, address.to_string());
            counter += 1;
        }
        
        Ok((result, replacements))
    }
}

struct NumberAnonymizer;

impl NumberAnonymizer {
    fn new() -> Self {
        Self
    }
}

impl Anonymizer for NumberAnonymizer {
    fn anonymize(&self, text: &str) -> Result<(String, HashMap<String, String>)> {
        let mut result = text.to_string();
        let mut replacements = HashMap::new();
        let mut counter = 0;
        
        // Replace numbers that might be sensitive (e.g., account numbers, IDs)
        let pattern = Regex::new(r"\b\d{6,}\b").unwrap();
        
        for mat in pattern.find_iter(text) {
            let number = mat.as_str();
            let placeholder = format!("[NUMBER_{}]", counter);
            result = result.replace(number, &placeholder);
            replacements.insert(placeholder, number.to_string());
            counter += 1;
        }
        
        Ok((result, replacements))
    }
}

struct DateAnonymizer {
    patterns: Vec<Regex>,
}

impl DateAnonymizer {
    fn new() -> Self {
        Self {
            patterns: vec![
                Regex::new(r"\b\d{1,2}/\d{1,2}/\d{2,4}\b").unwrap(),
                Regex::new(r"\b\d{1,2}-\d{1,2}-\d{2,4}\b").unwrap(),
                Regex::new(r"\b\d{4}-\d{2}-\d{2}\b").unwrap(),
                Regex::new(r"\b(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\s+\d{1,2},?\s+\d{4}\b").unwrap(),
            ],
        }
    }
}

impl Anonymizer for DateAnonymizer {
    fn anonymize(&self, text: &str) -> Result<(String, HashMap<String, String>)> {
        let mut result = text.to_string();
        let mut replacements = HashMap::new();
        let mut counter = 0;
        
        for pattern in &self.patterns {
            for mat in pattern.find_iter(&result.clone()) {
                let date = mat.as_str();
                let placeholder = format!("[DATE_{}]", counter);
                result = result.replace(date, &placeholder);
                replacements.insert(placeholder, date.to_string());
                counter += 1;
            }
        }
        
        Ok((result, replacements))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_email_anonymization() {
        let manager = PrivacyManager::new(PrivacyLevel::Balanced);
        let text = "Contact me at john.doe@example.com for more info";
        let (anonymized, deanonymizer) = manager.anonymize(text).await.unwrap();
        
        assert!(anonymized.contains("[EMAIL_0]"));
        assert!(!anonymized.contains("john.doe@example.com"));
        
        if let Some(deanon) = deanonymizer {
            let restored = deanon.restore(&anonymized).unwrap();
            assert_eq!(restored, text);
        }
    }

    #[tokio::test]
    async fn test_phone_anonymization() {
        let manager = PrivacyManager::new(PrivacyLevel::Balanced);
        let text = "Call me at 555-123-4567 or (555) 987-6543";
        let (anonymized, deanonymizer) = manager.anonymize(text).await.unwrap();
        
        assert!(anonymized.contains("[PHONE_"));
        assert!(!anonymized.contains("555-123-4567"));
        assert!(!anonymized.contains("(555) 987-6543"));
        
        if let Some(deanon) = deanonymizer {
            let restored = deanon.restore(&anonymized).unwrap();
            assert_eq!(restored, text);
        }
    }

    #[test]
    fn test_privacy_levels() {
        let strict = PrivacyManager::new(PrivacyLevel::Strict);
        assert_eq!(strict.anonymizers.len(), 6);
        
        let balanced = PrivacyManager::new(PrivacyLevel::Balanced);
        assert_eq!(balanced.anonymizers.len(), 4);
        
        let permissive = PrivacyManager::new(PrivacyLevel::Permissive);
        assert_eq!(permissive.anonymizers.len(), 2);
    }

    #[test]
    fn test_sensitive_data_detection() {
        let manager = PrivacyManager::new(PrivacyLevel::Strict);
        
        assert!(manager.contains_sensitive_data("My SSN is 123-45-6789"));
        assert!(manager.contains_sensitive_data("api_key: sk-1234567890"));
        assert!(manager.contains_sensitive_data("password: secret123"));
        assert!(!manager.contains_sensitive_data("Hello world"));
    }
}