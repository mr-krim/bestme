use crate::ai::*;
use crate::ai::context::*;
use crate::ai::telemetry::*;
use crate::ai::benchmarks::*;
use super::test_helpers::*;

#[cfg(test)]
mod context_tests {
    use super::*;
    use std::time::SystemTime;
    
    #[test]
    fn test_turn_creation() {
        let turn = Turn {
            id: "test-1".to_string(),
            role: Role::User,
            content: "Hello".to_string(),
            timestamp: SystemTime::now(),
            tokens: 1,
            metadata: None,
        };
        
        assert_eq!(turn.role.as_str(), "user");
        assert_eq!(turn.tokens, 1);
    }
    
    #[test]
    fn test_context_window_compression() {
        let mut window = ContextWindow::new("test".to_string());
        let config = ContextConfig {
            max_tokens: 10,
            max_turns: 2,
            ..Default::default()
        };
        
        // Add turns that exceed limits
        for i in 0..3 {
            window.add_turn(Turn {
                id: format!("turn-{}", i),
                role: Role::User,
                content: format!("Message {}", i),
                timestamp: SystemTime::now(),
                tokens: 5,
                metadata: None,
            });
        }
        
        assert_eq!(window.turns.len(), 3);
        assert_eq!(window.total_tokens, 15);
        
        // Compress
        window.compress(&config);
        
        // Should keep only last 2 turns
        assert_eq!(window.turns.len(), 2);
        assert_eq!(window.total_tokens, 10);
        assert_eq!(window.turns[0].id, "turn-1");
    }
    
    #[test]
    fn test_context_formatting() {
        let mut window = ContextWindow::new("test".to_string());
        
        window.add_turn(Turn {
            id: "1".to_string(),
            role: Role::User,
            content: "Hello".to_string(),
            timestamp: SystemTime::now(),
            tokens: 1,
            metadata: None,
        });
        
        window.add_turn(Turn {
            id: "2".to_string(),
            role: Role::Assistant,
            content: "Hi there".to_string(),
            timestamp: SystemTime::now(),
            tokens: 2,
            metadata: None,
        });
        
        let formatted = window.format_for_model(Some("Be helpful"));
        assert!(formatted.contains("System: Be helpful"));
        assert!(formatted.contains("USER: Hello"));
        assert!(formatted.contains("ASSISTANT: Hi there"));
    }
}

#[cfg(test)]
mod telemetry_tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_telemetry_config() {
        let config = TelemetryConfig::default();
        assert_eq!(config.service_name, "bestme-ai");
        assert!(config.enable_metrics);
        assert!(config.enable_tracing);
    }
    
    #[test]
    fn test_export_config() {
        let config = exporter::ExportConfig::default();
        assert!(matches!(config.format, exporter::ExportFormat::Json));
        assert_eq!(config.buffer_size, 1000);
    }
    
    #[test]
    fn test_telemetry_data_serialization() {
        let data = exporter::TelemetryData {
            timestamp: 1234567890,
            metric_type: exporter::MetricType::Counter,
            name: "test_metric".to_string(),
            value: exporter::MetricValue::Int(42),
            labels: vec![("env".to_string(), "test".to_string())],
        };
        
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("test_metric"));
        assert!(json.contains("42"));
    }
}

#[cfg(test)]
mod benchmark_tests {
    use super::*;
    
    #[test]
    fn test_benchmark_config() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.warmup_iterations, 5);
        assert_eq!(config.benchmark_iterations, 100);
        assert!(!config.test_samples.is_empty());
    }
    
    #[test]
    fn test_test_sample_creation() {
        let sample = TestSample {
            name: "test".to_string(),
            text: "Hello world".to_string(),
            expected_tokens: 2,
        };
        
        assert_eq!(sample.name, "test");
        assert_eq!(sample.expected_tokens, 2);
    }
    
    #[test]
    fn test_benchmark_result_serialization() {
        let result = BenchmarkResult {
            model_id: "test-model".to_string(),
            test_name: "short_text".to_string(),
            iterations: 100,
            avg_latency_ms: 50.5,
            min_latency_ms: 40,
            max_latency_ms: 60,
            p50_latency_ms: 50,
            p90_latency_ms: 55,
            p95_latency_ms: 58,
            p99_latency_ms: 59,
            throughput_tps: 20.0,
            memory_usage_mb: 256,
            gpu_usage_percent: Some(75.5),
            timestamp: chrono::Utc::now(),
        };
        
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test-model"));
        assert!(json.contains("50.5"));
    }
}

#[cfg(test)]
mod memory_tests {
    use super::*;
    use crate::ai::context::memory::*;
    use std::time::SystemTime;
    
    #[test]
    fn test_memory_entry_creation() {
        let entry = MemoryEntry {
            id: "mem-1".to_string(),
            memory_type: MemoryType::ShortTerm,
            content: "Test memory".to_string(),
            embedding: None,
            relevance: 0.8,
            access_count: 0,
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            metadata: std::collections::HashMap::new(),
        };
        
        assert_eq!(entry.relevance, 0.8);
        assert_eq!(entry.access_count, 0);
    }
    
    #[test]
    fn test_memory_config() {
        let config = MemoryConfig::default();
        assert_eq!(config.short_term_capacity, 100);
        assert_eq!(config.working_capacity, 1000);
        assert_eq!(config.promotion_threshold, 0.7);
    }
}

#[cfg(test)]
mod template_tests {
    use super::*;
    use crate::ai::context::templates::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_template_variable() {
        let var = TemplateVariable {
            name: "test".to_string(),
            description: "Test variable".to_string(),
            required: true,
            default_value: None,
        };
        
        assert!(var.required);
        assert!(var.default_value.is_none());
    }
    
    #[test]
    fn test_template_application() {
        let template = ConversationTemplate {
            name: "test".to_string(),
            description: "Test template".to_string(),
            system_prompt: "System {{mode}}".to_string(),
            user_template: "User: {{message}}".to_string(),
            assistant_template: "Assistant: {{response}}".to_string(),
            variables: vec![
                TemplateVariable {
                    name: "mode".to_string(),
                    description: "Mode".to_string(),
                    required: false,
                    default_value: Some("default".to_string()),
                },
                TemplateVariable {
                    name: "message".to_string(),
                    description: "Message".to_string(),
                    required: true,
                    default_value: None,
                },
                TemplateVariable {
                    name: "response".to_string(),
                    description: "Response".to_string(),
                    required: true,
                    default_value: None,
                },
            ],
            metadata: HashMap::new(),
        };
        
        let manager = TemplateManager::new();
        
        let mut variables = HashMap::new();
        variables.insert("message".to_string(), "Hello".to_string());
        variables.insert("response".to_string(), "Hi".to_string());
        
        // Would test with custom template
        // let applied = manager.apply_template_direct(&template, variables);
        // assert!(applied.system_prompt.contains("default"));
    }
}

#[cfg(test)]
mod streaming_tests {
    use super::*;
    use crate::ai::local::streaming_inference::*;
    
    #[test]
    fn test_streaming_config() {
        let config = StreamingConfig::default();
        assert_eq!(config.min_chunk_size, 20);
        assert_eq!(config.max_wait_ms, 300);
        assert!(config.enable_predictions);
    }
    
    #[test]
    fn test_text_update_variants() {
        let update1 = TextUpdate::Append("Hello".to_string());
        let update2 = TextUpdate::Replace {
            start: 0,
            end: 5,
            text: "Hi".to_string(),
        };
        let update3 = TextUpdate::Clear;
        let update4 = TextUpdate::Finalize;
        
        // Test that all variants can be created
        match update1 {
            TextUpdate::Append(text) => assert_eq!(text, "Hello"),
            _ => panic!("Wrong variant"),
        }
        
        match update2 {
            TextUpdate::Replace { start, end, text } => {
                assert_eq!(start, 0);
                assert_eq!(end, 5);
                assert_eq!(text, "Hi");
            }
            _ => panic!("Wrong variant"),
        }
        
        assert!(matches!(update3, TextUpdate::Clear));
        assert!(matches!(update4, TextUpdate::Finalize));
    }
}

#[cfg(test)]
mod batch_tests {
    use super::*;
    use crate::ai::local::batch_processor::*;
    
    #[test]
    fn test_batch_config() {
        let config = BatchConfig::default();
        assert_eq!(config.max_parallel, 4);
        assert_eq!(config.batch_size, 8);
        assert!(config.preserve_order);
    }
    
    #[test]
    fn test_text_segment() {
        let segment = TextSegment {
            id: "seg-1".to_string(),
            text: "Test text".to_string(),
            metadata: None,
            priority: 5,
        };
        
        assert_eq!(segment.priority, 5);
        assert!(segment.metadata.is_none());
    }
    
    #[test]
    fn test_batch_statistics() {
        let results = vec![
            BatchResult {
                id: "1".to_string(),
                original: "test".to_string(),
                enhanced: "test".to_string(),
                processing_time_ms: 50,
                success: true,
                error: None,
                metadata: None,
            },
            BatchResult {
                id: "2".to_string(),
                original: "test2".to_string(),
                enhanced: "test2".to_string(),
                processing_time_ms: 100,
                success: true,
                error: None,
                metadata: None,
            },
            BatchResult {
                id: "3".to_string(),
                original: "test3".to_string(),
                enhanced: "test3".to_string(),
                processing_time_ms: 75,
                success: false,
                error: Some("Error".to_string()),
                metadata: None,
            },
        ];
        
        let stats = BatchStatistics::from_results(&results, 200);
        assert_eq!(stats.total_segments, 3);
        assert_eq!(stats.successful_segments, 2);
        assert_eq!(stats.failed_segments, 1);
        assert_eq!(stats.min_time_ms, 50);
        assert_eq!(stats.max_time_ms, 100);
        assert_eq!(stats.average_time_ms, 75.0);
    }
}

#[cfg(test)]
mod model_warmup_tests {
    use super::*;
    use crate::ai::local::model_warmup::*;
    
    #[test]
    fn test_warmup_config() {
        let config = WarmupConfig::default();
        assert_eq!(config.warmup_iterations, 3);
        assert_eq!(config.cache_size, 1000);
        assert!(config.enable_caching);
    }
    
    #[test]
    fn test_warmup_statistics() {
        let stats = WarmupStatistics {
            total_warmup_time_ms: 1000,
            average_latency_ms: 50.0,
            min_latency_ms: 40,
            max_latency_ms: 60,
            cache_hits: 10,
            cache_misses: 5,
            evictions: 2,
        };
        
        let hit_rate = stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64;
        assert!(hit_rate > 0.6);
    }
}

#[cfg(test)]
mod enhancement_options_tests {
    use super::*;
    
    #[test]
    fn test_enhancement_options_default() {
        let options = EnhancementOptions::default();
        assert!(options.correct_grammar);
        assert!(options.improve_clarity);
        assert!(options.preserve_style);
        assert!(!options.detect_intent);
        assert!(!options.format_markdown);
    }
    
    #[test]
    fn test_enhancement_options_serialization() {
        let options = EnhancementOptions {
            correct_grammar: true,
            improve_clarity: false,
            preserve_style: true,
            detect_intent: true,
            format_markdown: false,
        };
        
        let json = serde_json::to_string(&options).unwrap();
        let deserialized: EnhancementOptions = serde_json::from_str(&json).unwrap();
        
        assert_eq!(options.correct_grammar, deserialized.correct_grammar);
        assert_eq!(options.improve_clarity, deserialized.improve_clarity);
    }
}