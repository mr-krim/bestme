//! Custom vocabulary management for enhanced transcription accuracy

use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;

/// Vocabulary entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyEntry {
    /// The word or phrase
    pub term: String,
    
    /// Boost factor (1.0 = normal, 2.0 = double likelihood)
    pub boost: f32,
    
    /// Category or domain
    pub category: Option<String>,
    
    /// Alternative spellings or variations
    pub variants: Vec<String>,
    
    /// Context hints where this term is likely to appear
    pub context_hints: Vec<String>,
    
    /// Is this a proper noun (name, place, etc.)
    pub is_proper_noun: bool,
    
    /// Phonetic hint (if available)
    pub phonetic: Option<String>,
}

impl VocabularyEntry {
    /// Create a simple vocabulary entry
    pub fn simple(term: String, boost: f32) -> Self {
        Self {
            term,
            boost,
            category: None,
            variants: Vec::new(),
            context_hints: Vec::new(),
            is_proper_noun: false,
            phonetic: None,
        }
    }
    
    /// Create a vocabulary entry for a name
    pub fn name(name: String, boost: f32) -> Self {
        Self {
            term: name,
            boost,
            category: Some("name".to_string()),
            variants: Vec::new(),
            context_hints: Vec::new(),
            is_proper_noun: true,
            phonetic: None,
        }
    }
}

/// Vocabulary category with terms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyCategory {
    pub name: String,
    pub description: Option<String>,
    pub terms: Vec<VocabularyEntry>,
    pub enabled: bool,
}

/// Custom vocabulary manager
#[derive(Clone)]
pub struct VocabularyManager {
    /// All vocabulary entries by term
    entries: Arc<RwLock<HashMap<String, VocabularyEntry>>>,
    
    /// Categories of vocabulary
    categories: Arc<RwLock<HashMap<String, VocabularyCategory>>>,
    
    /// Path to vocabulary storage
    storage_path: PathBuf,
    
    /// Maximum number of entries
    max_entries: usize,
    
    /// Whether to auto-save changes
    auto_save: bool,
}

impl VocabularyManager {
    /// Create a new vocabulary manager
    pub fn new(storage_path: PathBuf) -> Result<Self> {
        let manager = Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            categories: Arc::new(RwLock::new(HashMap::new())),
            storage_path,
            max_entries: 10000,
            auto_save: true,
        };
        
        // Load existing vocabulary if available
        if manager.storage_path.exists() {
            manager.load()?;
        } else {
            // Initialize with default categories
            manager.init_default_categories();
        }
        
        Ok(manager)
    }
    
    /// Initialize default vocabulary categories
    fn init_default_categories(&self) {
        let mut categories = self.categories.write();
        
        // Technical terms category
        categories.insert("technical".to_string(), VocabularyCategory {
            name: "Technical Terms".to_string(),
            description: Some("Programming and technical vocabulary".to_string()),
            terms: vec![
                VocabularyEntry::simple("API".to_string(), 2.0),
                VocabularyEntry::simple("REST".to_string(), 2.0),
                VocabularyEntry::simple("GraphQL".to_string(), 2.0),
                VocabularyEntry::simple("Kubernetes".to_string(), 2.0),
                VocabularyEntry::simple("Docker".to_string(), 2.0),
                VocabularyEntry::simple("CI/CD".to_string(), 2.0),
                VocabularyEntry::simple("DevOps".to_string(), 2.0),
                VocabularyEntry::simple("microservices".to_string(), 1.5),
                VocabularyEntry::simple("serverless".to_string(), 1.5),
                VocabularyEntry::simple("blockchain".to_string(), 1.5),
            ],
            enabled: true,
        });
        
        // Common names category
        categories.insert("names".to_string(), VocabularyCategory {
            name: "Common Names".to_string(),
            description: Some("Frequently used names".to_string()),
            terms: vec![],
            enabled: true,
        });
        
        // Medical terms category
        categories.insert("medical".to_string(), VocabularyCategory {
            name: "Medical Terms".to_string(),
            description: Some("Medical and healthcare vocabulary".to_string()),
            terms: vec![],
            enabled: false,
        });
        
        // Legal terms category
        categories.insert("legal".to_string(), VocabularyCategory {
            name: "Legal Terms".to_string(),
            description: Some("Legal and compliance vocabulary".to_string()),
            terms: vec![],
            enabled: false,
        });
    }
    
    /// Add a vocabulary entry
    pub fn add_entry(&self, entry: VocabularyEntry) -> Result<()> {
        let mut entries = self.entries.write();
        
        if entries.len() >= self.max_entries {
            return Err(anyhow::anyhow!("Vocabulary limit reached ({} entries)", self.max_entries));
        }
        
        // Add to main entries
        entries.insert(entry.term.clone(), entry.clone());
        
        // Add variants as well
        for variant in &entry.variants {
            entries.insert(variant.clone(), entry.clone());
        }
        
        // Add to category if specified
        if let Some(category) = &entry.category {
            let mut categories = self.categories.write();
            if let Some(cat) = categories.get_mut(category) {
                cat.terms.push(entry);
            }
        }
        
        drop(entries);
        
        if self.auto_save {
            self.save()?;
        }
        
        Ok(())
    }
    
    /// Add multiple entries at once
    pub fn add_entries(&self, new_entries: Vec<VocabularyEntry>) -> Result<()> {
        for entry in new_entries {
            self.add_entry(entry)?;
        }
        Ok(())
    }
    
    /// Remove a vocabulary entry
    pub fn remove_entry(&self, term: &str) -> Result<()> {
        let mut entries = self.entries.write();
        
        if let Some(entry) = entries.remove(term) {
            // Remove variants too
            for variant in &entry.variants {
                entries.remove(variant);
            }
            
            // Remove from category
            if let Some(category) = &entry.category {
                let mut categories = self.categories.write();
                if let Some(cat) = categories.get_mut(category) {
                    cat.terms.retain(|e| e.term != term);
                }
            }
            
            drop(entries);
            
            if self.auto_save {
                self.save()?;
            }
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Term '{}' not found", term))
        }
    }
    
    /// Get a vocabulary entry
    pub fn get_entry(&self, term: &str) -> Option<VocabularyEntry> {
        self.entries.read().get(term).cloned()
    }
    
    /// Search for entries matching a pattern
    pub fn search(&self, pattern: &str) -> Vec<VocabularyEntry> {
        let pattern_lower = pattern.to_lowercase();
        let entries = self.entries.read();
        
        entries.values()
            .filter(|entry| {
                entry.term.to_lowercase().contains(&pattern_lower) ||
                entry.variants.iter().any(|v| v.to_lowercase().contains(&pattern_lower))
            })
            .cloned()
            .collect()
    }
    
    /// Get all entries in a category
    pub fn get_category_entries(&self, category: &str) -> Vec<VocabularyEntry> {
        let categories = self.categories.read();
        
        if let Some(cat) = categories.get(category) {
            if cat.enabled {
                cat.terms.clone()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }
    
    /// Get all enabled vocabulary entries
    pub fn get_enabled_entries(&self) -> Vec<VocabularyEntry> {
        let categories = self.categories.read();
        let mut all_entries = Vec::new();
        
        for cat in categories.values() {
            if cat.enabled {
                all_entries.extend(cat.terms.clone());
            }
        }
        
        all_entries
    }
    
    /// Enable/disable a category
    pub fn set_category_enabled(&self, category: &str, enabled: bool) -> Result<()> {
        let mut categories = self.categories.write();
        
        if let Some(cat) = categories.get_mut(category) {
            cat.enabled = enabled;
            
            if self.auto_save {
                drop(categories);
                self.save()?;
            }
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Category '{}' not found", category))
        }
    }
    
    /// Generate prompt hints from vocabulary
    pub fn generate_prompt_hints(&self, context: Option<&str>) -> String {
        let entries = self.get_enabled_entries();
        
        if entries.is_empty() {
            return String::new();
        }
        
        let mut hints = Vec::new();
        
        // Filter by context if provided
        let filtered_entries: Vec<_> = if let Some(ctx) = context {
            entries.into_iter()
                .filter(|e| {
                    e.context_hints.is_empty() ||
                    e.context_hints.iter().any(|hint| ctx.contains(hint))
                })
                .collect()
        } else {
            entries
        };
        
        // Group by category
        let mut by_category: HashMap<String, Vec<String>> = HashMap::new();
        
        for entry in filtered_entries.iter().take(50) { // Limit to 50 terms
            let category = entry.category.as_deref().unwrap_or("general");
            by_category.entry(category.to_string())
                .or_default()
                .push(entry.term.clone());
        }
        
        // Build prompt
        for (category, terms) in by_category {
            hints.push(format!("{}: {}", category, terms.join(", ")));
        }
        
        if !hints.is_empty() {
            format!("Vocabulary hints - {}", hints.join("; "))
        } else {
            String::new()
        }
    }
    
    /// Save vocabulary to disk
    pub fn save(&self) -> Result<()> {
        let vocabulary_data = VocabularyData {
            version: 1,
            categories: self.categories.read().clone(),
        };
        
        let json = serde_json::to_string_pretty(&vocabulary_data)?;
        std::fs::write(&self.storage_path, json)
            .context("Failed to save vocabulary")?;
        
        info!("Saved vocabulary to {:?}", self.storage_path);
        Ok(())
    }
    
    /// Load vocabulary from disk
    pub fn load(&self) -> Result<()> {
        let json = std::fs::read_to_string(&self.storage_path)
            .context("Failed to read vocabulary file")?;
        
        let data: VocabularyData = serde_json::from_str(&json)
            .context("Failed to parse vocabulary data")?;
        
        // Update categories
        *self.categories.write() = data.categories;
        
        // Rebuild entries index
        let mut entries = HashMap::new();
        for category in self.categories.read().values() {
            if category.enabled {
                for entry in &category.terms {
                    entries.insert(entry.term.clone(), entry.clone());
                    for variant in &entry.variants {
                        entries.insert(variant.clone(), entry.clone());
                    }
                }
            }
        }
        
        *self.entries.write() = entries;
        
        info!("Loaded vocabulary from {:?}", self.storage_path);
        Ok(())
    }
    
    /// Import vocabulary from CSV
    pub fn import_csv(&self, csv_path: &Path) -> Result<usize> {
        let mut reader = csv::Reader::from_path(csv_path)?;
        let mut count = 0;
        
        for result in reader.records() {
            let record = result?;
            
            if record.len() >= 2 {
                let term = record[0].to_string();
                let boost: f32 = record[1].parse().unwrap_or(1.5);
                
                let mut entry = VocabularyEntry::simple(term, boost);
                
                // Optional fields
                if record.len() > 2 {
                    entry.category = Some(record[2].to_string());
                }
                
                if record.len() > 3 {
                    entry.variants = record[3].split('|').map(|s| s.to_string()).collect();
                }
                
                self.add_entry(entry)?;
                count += 1;
            }
        }
        
        Ok(count)
    }
    
    /// Export vocabulary to CSV
    pub fn export_csv(&self, csv_path: &Path) -> Result<usize> {
        let mut writer = csv::Writer::from_path(csv_path)?;
        
        // Write header
        writer.write_record(&["term", "boost", "category", "variants"])?;
        
        let entries = self.get_enabled_entries();
        let mut count = 0;
        
        for entry in entries {
            writer.write_record(&[
                &entry.term,
                &entry.boost.to_string(),
                &entry.category.unwrap_or_default(),
                &entry.variants.join("|"),
            ])?;
            count += 1;
        }
        
        writer.flush()?;
        Ok(count)
    }
}

/// Vocabulary data for serialization
#[derive(Serialize, Deserialize)]
struct VocabularyData {
    version: u32,
    categories: HashMap<String, VocabularyCategory>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_vocabulary_manager() {
        let temp_dir = TempDir::new().unwrap();
        let vocab_path = temp_dir.path().join("vocabulary.json");
        
        let manager = VocabularyManager::new(vocab_path).unwrap();
        
        // Add entry
        let entry = VocabularyEntry::simple("Rust".to_string(), 2.0);
        manager.add_entry(entry.clone()).unwrap();
        
        // Get entry
        let retrieved = manager.get_entry("Rust").unwrap();
        assert_eq!(retrieved.term, "Rust");
        assert_eq!(retrieved.boost, 2.0);
        
        // Search
        let results = manager.search("ru");
        assert_eq!(results.len(), 1);
        
        // Remove entry
        manager.remove_entry("Rust").unwrap();
        assert!(manager.get_entry("Rust").is_none());
    }
    
    #[test]
    fn test_vocabulary_categories() {
        let temp_dir = TempDir::new().unwrap();
        let vocab_path = temp_dir.path().join("vocabulary.json");
        
        let manager = VocabularyManager::new(vocab_path).unwrap();
        
        // Check default categories
        let tech_entries = manager.get_category_entries("technical");
        assert!(!tech_entries.is_empty());
        
        // Disable category
        manager.set_category_enabled("technical", false).unwrap();
        let enabled = manager.get_enabled_entries();
        assert!(enabled.is_empty());
    }
    
    #[test]
    fn test_prompt_generation() {
        let temp_dir = TempDir::new().unwrap();
        let vocab_path = temp_dir.path().join("vocabulary.json");
        
        let manager = VocabularyManager::new(vocab_path).unwrap();
        
        let prompt = manager.generate_prompt_hints(None);
        assert!(prompt.contains("Vocabulary hints"));
        assert!(prompt.contains("technical:"));
        assert!(prompt.contains("API"));
    }
}